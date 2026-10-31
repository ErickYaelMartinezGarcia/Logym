use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Serialize)]
pub struct TipoMembresia {
    pub id: i64,
    pub nombre: String,
    pub duracion_meses: i64,
    pub precio_centavos: i64,
}

#[derive(Serialize)]
pub struct ResumenIngresos {
    pub periodo: String,
    pub total_centavos: i64,
    pub membresias_centavos: i64,
    pub inscripciones_centavos: i64,
    pub numero_pagos: i64,
}

pub fn listar(conn: &Connection) -> Result<Vec<TipoMembresia>, String> {
    let mut stmt = conn.prepare("SELECT id,nombre,duracion_meses,precio_centavos FROM tipos_membresia WHERE activo=1 ORDER BY nombre")
        .map_err(|e| e.to_string())?;
    stmt.query_map([], |r| Ok(TipoMembresia {
        id: r.get(0)?, nombre: r.get(1)?, duracion_meses: r.get(2)?, precio_centavos: r.get(3)?,
    })).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())
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
        _ => return Err("Periodo inválido".into()),
    };
    let sql = format!("SELECT COALESCE(SUM(total_centavos),0),COALESCE(SUM(membresia_centavos),0),COALESCE(SUM(inscripcion_centavos),0),COUNT(*) FROM pagos WHERE date(fecha_pago) BETWEEN {inicio} AND date('now','localtime')");
    conn.query_row(&sql, params![], |r| Ok(ResumenIngresos {
        periodo: periodo.to_string(), total_centavos: r.get(0)?,
        membresias_centavos: r.get(1)?, inscripciones_centavos: r.get(2)?,
        numero_pagos: r.get(3)?,
    })).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_revenue_ignores_catalog_price_changes() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("
            CREATE TABLE pagos (
                total_centavos INTEGER NOT NULL,
                membresia_centavos INTEGER NOT NULL,
                inscripcion_centavos INTEGER NOT NULL,
                fecha_pago TEXT NOT NULL
            );
            CREATE TABLE tipos_membresia (id INTEGER PRIMARY KEY, precio_centavos INTEGER);
            INSERT INTO tipos_membresia VALUES(1,35000);
            INSERT INTO pagos VALUES(50000,35000,15000,datetime('now','localtime'));
            UPDATE tipos_membresia SET precio_centavos=40000 WHERE id=1;
        ").unwrap();
        let reporte = ingresos(&db,"mes").unwrap();
        assert_eq!(reporte.total_centavos,50000);
        assert_eq!(reporte.membresias_centavos,35000);
        assert_eq!(reporte.inscripciones_centavos,15000);
    }
    #[test]
    fn rejects_invalid_period() {
        let db = Connection::open_in_memory().unwrap();
        assert!(ingresos(&db,"anual").is_err());
    }
}

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

pub fn actualizar_plan(conn: &Connection, password: &str, id: Option<i64>, nombre: &str, meses: i64, precio: i64, activo: bool) -> Result<(), String> {
    autenticar(conn,password)?;
    if nombre.trim().is_empty() || nombre.chars().count() > 60 || nombre.chars().any(char::is_control)
        || !(1..=36).contains(&meses) || !(0..=100_000_000).contains(&precio) {
        return Err("Datos de membresía inválidos".into());
    }
    if let Some(id) = id {
        let n = conn.execute("UPDATE tipos_membresia SET nombre=?1,duracion_meses=?2,precio_centavos=?3,activo=?4 WHERE id=?5",
            params![nombre.trim(),meses,precio,activo as i32,id]).map_err(|e|e.to_string())?;
        if n == 0 { return Err("Plan no encontrado".into()); }
    } else {
        conn.execute("INSERT INTO tipos_membresia(nombre,duracion_meses,precio_centavos,activo) VALUES(?1,?2,?3,?4)",
            params![nombre.trim(),meses,precio,activo as i32]).map_err(|e|e.to_string())?;
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
    conn.query_row(&format!("SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora) BETWEEN {inicio} AND date('now','localtime')"),[],|r|r.get(0))
        .map_err(|e|e.to_string())
}
