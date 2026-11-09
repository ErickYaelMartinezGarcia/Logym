use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Serialize)]
pub struct TipoMembresia {
    pub id: i64,
    pub nombre: String,
    pub duracion_meses: i64,
    pub duracion_dias: Option<i64>,
    pub precio_centavos: i64,
    pub activo: bool,
    pub solo_alta: bool,
}

/// Fin inclusivo para días; los meses conservan la semántica histórica.
pub fn fin_periodo(inicio: chrono::NaiveDate, meses: i64, dias: Option<i64>) -> Result<chrono::NaiveDate, String> {
    let fin = match dias {
        Some(d) if (1..=3660).contains(&d) => inicio.checked_add_signed(chrono::Duration::days(d - 1)),
        Some(_) => return Err("Duración inválida".into()),
        None if (1..=36).contains(&meses) => inicio.checked_add_months(chrono::Months::new(meses as u32)),
        None => return Err("Duración inválida".into()),
    };
    fin.ok_or_else(|| "Duración inválida".into())
}

#[derive(Serialize)]
pub struct ResumenIngresos {
    pub periodo: String,
    pub total_centavos: i64,
    pub membresias_centavos: i64,
    pub inscripciones_centavos: i64,
    pub visitas_centavos: i64,
    pub numero_pagos: i64,
}

/// Punto de la serie financiera; los importes provienen de pagos históricos.
#[derive(Serialize)]
pub struct IngresoDiario {
    pub fecha: String,
    pub total_centavos: i64,
}

/// Resumen y evolución para un rango inclusivo de fechas locales.
#[derive(Serialize)]
pub struct ReporteFinanciero {
    pub categoria: String,
    pub total_periodo_centavos: i64,
    pub movimientos: Vec<MovimientoIngreso>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub total_centavos: i64,
    pub membresias_centavos: i64,
    pub inscripciones_centavos: i64,
    pub visitas_centavos: i64,
    pub numero_pagos: i64,
    pub serie: Vec<IngresoDiario>,
}

#[derive(Serialize)]
pub struct MovimientoIngreso {
    pub id: String,
    pub fecha: String,
    pub cliente: String,
    pub cliente_id: Option<i64>,
    pub plan: String,
    pub tipo: String,
    pub duracion_dias: Option<i64>,
    pub total_centavos: i64,
    pub membresia_centavos: i64,
    pub inscripcion_centavos: i64,
    pub visitas_centavos: i64,
}

#[cfg(test)]
pub fn ingresos_rango(conn: &Connection, inicio: &str, fin: &str) -> Result<ReporteFinanciero, String> {
    ingresos_rango_categoria(conn, inicio, fin, "todos")
}

pub fn ingresos_rango_categoria(conn: &Connection, inicio: &str, fin: &str, categoria: &str) -> Result<ReporteFinanciero, String> {
    if !["todos", "altas", "renovaciones", "visitas", "semanales"].contains(&categoria) { return Err("Categoría de ingresos inválida".into()); }
    // No interpolar fechas en SQL: validar calendario y enlazar parámetros.
    fn fecha_valida(s: &str) -> bool {
        if s.len() != 10 || !s.is_ascii() || !s.as_bytes().iter().enumerate().all(|(i,b)| if i == 4 || i == 7 { *b == b'-' } else { b.is_ascii_digit() }) { return false; }
        let (Ok(y), Ok(m), Ok(d)) = (s[0..4].parse::<i32>(), s[5..7].parse::<u32>(), s[8..10].parse::<u32>()) else { return false };
        let dias = match m { 1|3|5|7|8|10|12 => 31, 4|6|9|11 => 30, 2 if y % 400 == 0 || (y % 4 == 0 && y % 100 != 0) => 29, 2 => 28, _ => return false };
        (1900..=9999).contains(&y) && d >= 1 && d <= dias
    }
    if !fecha_valida(inicio) || !fecha_valida(fin) || inicio > fin {
        return Err("Selecciona un rango de fechas válido (inicio no posterior al fin)".into());
    }
    let mut stmt = conn.prepare("SELECT 'pago-'||p.id,p.fecha_pago,c.nombre,p.cliente_id,p.concepto_membresia,p.tipo,p.duracion_dias,p.total_centavos,p.membresia_centavos,p.inscripcion_centavos,0
        FROM pagos p JOIN clientes c ON c.id=p.cliente_id WHERE date(p.fecha_pago) BETWEEN ?1 AND ?2
        UNION ALL
        SELECT 'visita-'||v.id,p.fecha_pago,COALESCE(v.nombre,'Visita sin nombre'),NULL,'Visita ocasional','visita',NULL,p.total_centavos,0,0,p.total_centavos
        FROM pagos_visitas p JOIN visitas_ocasionales v ON v.id=p.visita_id WHERE date(p.fecha_pago) BETWEEN ?1 AND ?2
        ORDER BY 2 DESC,1").map_err(|e|e.to_string())?;
    let todos = stmt.query_map(params![inicio,fin], |r| Ok(MovimientoIngreso {id:r.get(0)?,fecha:r.get(1)?,cliente:r.get(2)?,cliente_id:r.get(3)?,plan:r.get(4)?,tipo:r.get(5)?,duracion_dias:r.get(6)?,total_centavos:r.get(7)?,membresia_centavos:r.get(8)?,inscripcion_centavos:r.get(9)?,visitas_centavos:r.get(10)?}))
        .map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let total_periodo_centavos = todos.iter().map(|m|m.total_centavos).sum();
    let movimientos: Vec<_> = todos.into_iter().filter(|m| match categoria { "altas" => m.tipo=="alta", "renovaciones" => m.tipo=="renovacion", "visitas" => m.tipo=="visita", "semanales" => m.duracion_dias==Some(7), _ => true }).collect();
    let mut diarios = std::collections::BTreeMap::new();
    for m in &movimientos { *diarios.entry(m.fecha.chars().take(10).collect::<String>()).or_insert(0) += m.total_centavos; }
    let serie = diarios.into_iter().map(|(fecha,total_centavos)|IngresoDiario{fecha,total_centavos}).collect();
    Ok(ReporteFinanciero { categoria:categoria.into(), total_periodo_centavos,
        fecha_inicio:inicio.into(), fecha_fin:fin.into(),
        total_centavos:movimientos.iter().map(|m|m.total_centavos).sum(),
        membresias_centavos:movimientos.iter().map(|m|m.membresia_centavos).sum(),
        inscripciones_centavos:movimientos.iter().map(|m|m.inscripcion_centavos).sum(),
        visitas_centavos:movimientos.iter().map(|m|m.visitas_centavos).sum(), numero_pagos:movimientos.len() as i64, serie, movimientos })
}

pub fn listar(conn: &Connection) -> Result<Vec<TipoMembresia>, String> {
    let mut stmt = conn.prepare("SELECT id,nombre,duracion_meses,precio_centavos,activo,solo_alta,duracion_dias FROM tipos_membresia WHERE activo=1 ORDER BY nombre")
        .map_err(|e| e.to_string())?;
    let resultado = stmt.query_map([], |r| Ok(TipoMembresia {
        id: r.get(0)?, nombre: r.get(1)?, duracion_meses: r.get(2)?, precio_centavos: r.get(3)?, activo: r.get::<_,i64>(4)? != 0, solo_alta: r.get::<_,i64>(5)? != 0, duracion_dias: r.get(6)?,
    })).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string());
    resultado
}

pub fn listar_admin(conn: &Connection, password: &str) -> Result<Vec<TipoMembresia>,String> {
    autenticar(conn,password)?;
    let mut stmt = conn.prepare("SELECT id,nombre,duracion_meses,precio_centavos,activo,solo_alta,duracion_dias FROM tipos_membresia ORDER BY nombre").map_err(|e|e.to_string())?;
    let resultado = stmt.query_map([],|r| Ok(TipoMembresia {
        id:r.get(0)?,nombre:r.get(1)?,duracion_meses:r.get(2)?,precio_centavos:r.get(3)?,activo:r.get::<_,i64>(4)? != 0,solo_alta:r.get::<_,i64>(5)? != 0,duracion_dias:r.get(6)?
    })).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string());
    resultado
}

pub fn inscripcion(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT inscripcion_centavos FROM configuracion_cobros WHERE id=1", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn ingresos(conn: &Connection, periodo: &str) -> Result<ResumenIngresos, String> {
    let inicio = match periodo {
        "dia" => "date('now','localtime')",
        "semana" => "date('now','localtime','-6 days')",
        "mes" => "date('now','localtime','start of month')",
        "anual" => "date('now','localtime','start of year')",
        _ => return Err("Periodo inválido".into()),
    };
    let sql = format!("SELECT COALESCE(SUM(total_centavos),0),COALESCE(SUM(membresia_centavos),0),COALESCE(SUM(inscripcion_centavos),0),COUNT(*),COALESCE(SUM(visitas_centavos),0) FROM ingresos_todos WHERE date(fecha_pago) BETWEEN {inicio} AND date('now','localtime')");
    conn.query_row(&sql, params![], |r| Ok(ResumenIngresos {
        periodo: periodo.to_string(), total_centavos: r.get(0)?,
        membresias_centavos: r.get(1)?, inscripciones_centavos: r.get(2)?,
        numero_pagos: r.get(3)?, visitas_centavos: r.get(4)?,
    })).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "tests/membresias.rs"]
mod tests;

fn validar_password(password: &str) -> Result<(), String> {
    if password.len() < 12 || password.len() > 128 { return Err("La contraseña debe tener entre 12 y 128 caracteres".into()); }
    Ok(())
}

pub fn tiene_administrador(conn: &Connection) -> Result<bool, String> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM administrador WHERE id=1)", [], |r| r.get::<_,i64>(0))
        .map(|n| n == 1).map_err(|e| e.to_string())
}

pub fn configurar_administrador(conn: &Connection, password: &str) -> Result<(), String> {
    use argon2::password_hash::{PasswordHasher, SaltString};
    validar_password(password)?;
    let mut salt_bytes = [0u8;16];
    getrandom::getrandom(&mut salt_bytes).map_err(|e| e.to_string())?;
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|e| e.to_string())?;
    let hash = argon2::Argon2::default().hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?.to_string();
    let changed = conn.execute("INSERT OR IGNORE INTO administrador(id,password_hash) VALUES(1,?1)",params![hash])
        .map_err(|e| e.to_string())?;
    if changed == 0 { return Err("Ya existe una contraseña administrativa".into()); }
    Ok(())
}

pub fn autenticar(conn: &Connection, password: &str) -> Result<(), String> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    let hash: String = conn.query_row("SELECT password_hash FROM administrador WHERE id=1",[],|r|r.get(0))
        .map_err(|_| "Configura primero la contraseña administrativa".to_string())?;
    let parsed = PasswordHash::new(&hash).map_err(|e| e.to_string())?;
    argon2::Argon2::default().verify_password(password.as_bytes(),&parsed)
        .map_err(|_| "Contraseña incorrecta".to_string())
}

pub fn actualizar_plan(conn: &Connection, password: &str, id: Option<i64>, nombre: &str, meses: i64, precio: i64, activo: bool, dias: Option<i64>) -> Result<(), String> {
    autenticar(conn,password)?;
    if nombre.trim().is_empty() || nombre.chars().count() > 60 || nombre.chars().any(char::is_control)
        || !(1..=36).contains(&meses) || dias.map_or(false, |d| !(1..=3660).contains(&d) || meses != 1) || !(0..=100_000_000).contains(&precio) {
        return Err("Datos de membresía inválidos".into());
    }
    if let Some(id) = id {
        let n = conn.execute("UPDATE tipos_membresia SET nombre=?1,duracion_meses=?2,precio_centavos=?3,activo=?4,duracion_dias=?6 WHERE id=?5",
            params![nombre.trim(),meses,precio,activo as i32,id,dias]).map_err(|e|e.to_string())?;
        if n == 0 { return Err("Plan no encontrado".into()); }
    } else {
        // La columna histórica sigue siendo positiva por compatibilidad. En
        // planes en días, duracion_dias tiene precedencia y meses vale 1.
        conn.execute("INSERT INTO tipos_membresia(nombre,duracion_meses,precio_centavos,activo,duracion_dias) VALUES(?1,?2,?3,?4,?5)",
            params![nombre.trim(),meses,precio,activo as i32,dias]).map_err(|e|e.to_string())?;
    }
    Ok(())
}

pub fn actualizar_inscripcion(conn: &Connection, password: &str, precio: i64) -> Result<(), String> {
    autenticar(conn,password)?;
    if !(0..=100_000_000).contains(&precio) { return Err("Precio inválido".into()); }
    conn.execute("UPDATE configuracion_cobros SET inscripcion_centavos=?1 WHERE id=1",params![precio])
        .map_err(|e|e.to_string())?;
    Ok(())
}

pub fn asistencias_periodo(conn: &Connection, periodo: &str) -> Result<i64,String> {
    let inicio = match periodo {
        "dia" => "date('now','localtime')",
        "semana" => "date('now','localtime','-6 days')",
        "mes" => "date('now','localtime','start of month')",
        _ => return Err("Periodo inválido".into()),
    };
    conn.query_row(&format!("SELECT COUNT(*) FROM asistencias_todas WHERE date(fecha_hora) BETWEEN {inicio} AND date('now','localtime')"),[],|r|r.get(0))
        .map_err(|e|e.to_string())
}

#[derive(serde::Serialize)]
pub struct AsistenciaDiaria {
    pub fecha: String,
    pub total: i64,
}

/// Conteo de entradas por hora del día local actual (00:00–23:59).
/// Se agregan asistencias, sin exponer datos personales.
pub fn asistencias_por_hora_hoy(conn: &Connection) -> Result<Vec<AsistenciaDiaria>, String> {
    let mut stmt = conn.prepare(
        "SELECT strftime('%H', fecha_hora), COUNT(*) FROM asistencias_todas WHERE date(fecha_hora) = date('now','localtime') GROUP BY strftime('%H', fecha_hora) ORDER BY 1"
    ).map_err(|e|e.to_string())?;
    let filas = stmt.query_map([], |r| Ok(AsistenciaDiaria { fecha: r.get(0)?, total: r.get(1)? }))
        .map_err(|e|e.to_string())?;
    filas.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

/// Resumen diario agregado, sin identificar a ningún cliente.
pub fn asistencias_diarias(conn: &Connection, periodo: &str) -> Result<Vec<AsistenciaDiaria>, String> {
    let (inicio, agrupacion) = match periodo {
        "dia" => ("date(\'now\',\'localtime\')", "date(fecha_hora)"),
        "semana" => ("date(\'now\',\'localtime\',\'-6 days\')", "date(fecha_hora)"),
        "mes" => ("date(\'now\',\'localtime\',\'start of year\')", "strftime(\'%Y-%m\',fecha_hora)"),
        _ => return Err("Periodo inválido".into()),
    };
    let consulta = format!("SELECT {agrupacion}, COUNT(*) FROM asistencias_todas WHERE date(fecha_hora) BETWEEN {inicio} AND date(\'now\',\'localtime\') GROUP BY {agrupacion} ORDER BY {agrupacion}");
    let mut sentencia = conn.prepare(&consulta).map_err(|e| e.to_string())?;
    let filas = sentencia.query_map([], |fila| Ok(AsistenciaDiaria { fecha: fila.get(0)?, total: fila.get(1)? })).map_err(|e| e.to_string())?;
    let mut datos = Vec::new();
    for fila in filas { datos.push(fila.map_err(|e| e.to_string())?); }
    if periodo == "semana" {
        let hoy = chrono::Local::now().date_naive();
        let conteos: std::collections::BTreeMap<_,_> = datos.into_iter().map(|d|(d.fecha,d.total)).collect();
        datos = (0..7).map(|i| {
            let fecha = (hoy - chrono::Duration::days(6-i)).format("%Y-%m-%d").to_string();
            let total = *conteos.get(&fecha).unwrap_or(&0);
            AsistenciaDiaria { fecha,total }
        }).collect();
    }
    Ok(datos)
}
