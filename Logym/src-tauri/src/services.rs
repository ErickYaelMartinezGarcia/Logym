use crate::database;
use crate::models::*;
use crate::repositories;
use crate::secugen;
use rusqlite::params;
use tauri::{AppHandle, Manager};

pub fn inicializar_db(app: &AppHandle) -> Result<String, String> {
    let conn = database::abrir_db(app)?;
    database::inicializar_esquema(&conn)?;
    Ok("DB OK".into())
}

pub fn guardar_cliente(app: &AppHandle, mut datos: DatosCliente) -> Result<i64, String> {
    let _traza = crate::lector_mensajes::Tramo::nuevo("registro_guardar");
    validar_alta(&mut datos)?;
    let conn = database::abrir_db(app)?;
    let plan_id = datos.tipo_membresia_id.ok_or("Selecciona un tipo de membresía")?;
    let (meses, dias): (i64, Option<i64>) = conn.query_row(
        "SELECT duracion_meses,duracion_dias FROM tipos_membresia WHERE id=?1 AND activo=1",
        params![plan_id], |r| Ok((r.get(0)?,r.get(1)?))
    ).map_err(|_| "La membresía seleccionada no está disponible".to_string())?;
    completar_fin_alta(&mut datos, meses, dias)?;
    if datos.sin_huella {
        return repositories::guardar_cliente_sin_duplicar(&conn, datos, |_, _| {
            Err("No debe compararse biometría en un alta sin huella".into())
        });
    }
    let recurso = app.path_resolver().resource_dir().ok_or("No se encontró el SDK del lector")?;
    let bin = secugen::resolve_bin_path(&recurso)?;
    let controller = secugen::SecuGenController::from_bin_path(&bin)?;
    repositories::guardar_cliente_sin_duplicar(&conn, datos, |a,b|controller.comparar_templates(a,b))
}

// Separación mecánica para probar las mismas validaciones sin AppHandle,
// IPC ni acceso a la carpeta de datos de una instalación real.
fn validar_alta(datos: &mut DatosCliente) -> Result<(), String> {
    datos.nombre = datos.nombre.trim().to_string();
    if datos.nombre.is_empty() || datos.nombre.chars().count() > 120 || datos.nombre.chars().any(char::is_control) {
        return Err("Nombre inválido".into());
    }
    for telefono in [&datos.telefono, &datos.telefono_emergencia] {
        if telefono.len() > 25 || !telefono.chars().all(|ch| ch.is_ascii_digit() || "+- ()".contains(ch)) {
            return Err("Teléfono inválido".into());
        }
    }
    if (datos.sin_huella && !datos.template_huella.is_empty())
        || (!datos.sin_huella && (datos.template_huella.len() != 800 || !datos.template_huella.bytes().all(|b| b.is_ascii_hexdigit()))) {
        return Err("Plantilla biométrica inválida".into());
    }
    if datos.foto_base64.len() > 3_000_000 {
        return Err("Fotografía demasiado grande".into());
    }
    if !datos.foto_base64.is_empty() && !["data:image/jpeg;base64,", "data:image/png;base64,", "data:image/webp;base64,"].iter().any(|p| datos.foto_base64.starts_with(p)) {
        return Err("Formato de fotografía inválido".into());
    }
    if !datos.fecha_registro.is_empty()
        && chrono::NaiveDate::parse_from_str(&datos.fecha_registro, "%Y-%m-%d").is_err()
        && chrono::NaiveDateTime::parse_from_str(&datos.fecha_registro, "%Y-%m-%d %H:%M:%S").is_err() {
        return Err("Fecha de registro inválida".into());
    }
    for fecha in [&datos.fecha_inicio_membresia, &datos.fecha_fin_membresia] {
        if !fecha.is_empty() && (fecha.len() != 10 || chrono::NaiveDate::parse_from_str(fecha, "%Y-%m-%d").is_err()) {
            return Err("Fecha de membresía inválida".into());
        }
    }
    if !datos.fecha_inicio_membresia.is_empty() && !datos.fecha_fin_membresia.is_empty()
        && datos.fecha_fin_membresia < datos.fecha_inicio_membresia {
        return Err("El fin de membresía no puede ser anterior al inicio".into());
    }
    if datos.fecha_registro.is_empty() { datos.fecha_registro = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(); }
    if datos.fecha_inicio_membresia.is_empty() { datos.fecha_inicio_membresia = chrono::Local::now().format("%Y-%m-%d").to_string(); }
    if datos.fecha_fin_membresia.is_empty() { datos.fecha_fin_membresia = (chrono::Local::now() + chrono::Duration::days(30)).format("%Y-%m-%d").to_string(); }
    Ok(())
}

fn completar_fin_alta(datos: &mut DatosCliente, meses: i64, dias: Option<i64>) -> Result<(), String> {
    let inicio = chrono::NaiveDate::parse_from_str(&datos.fecha_inicio_membresia, "%Y-%m-%d")
        .map_err(|_| "Fecha inicial inválida")?;
    let fin = crate::membresias::fin_periodo(inicio, meses, dias)?;
    datos.fecha_fin_membresia = fin.format("%Y-%m-%d").to_string();
    Ok(())
}

#[cfg(test)]
#[path = "tests/services.rs"]
mod pruebas_automatizadas;

pub fn listar_clientes(app: &AppHandle) -> Result<Vec<ClienteTabla>, String> {
    let conn = database::abrir_db(app)?; repositories::listar_clientes(&conn)
}
pub fn listar_clientes_membresia(app: &AppHandle) -> Result<Vec<ClienteMembresia>, String> {
    let conn = database::abrir_db(app)?; repositories::listar_clientes_membresia(&conn)
}

/// Búsqueda acotada para selección explícita; no expone plantillas biométricas.
pub fn buscar_clientes_checkin(app: &AppHandle, consulta: String) -> Result<Vec<ClienteCheckIn>, String> {
    let conn = database::abrir_db(app)?;
    buscar_clientes_checkin_en(&conn, &consulta)
}

fn buscar_clientes_checkin_en(conn: &rusqlite::Connection, consulta: &str) -> Result<Vec<ClienteCheckIn>, String> {
    let consulta = consulta.trim();
    if consulta.is_empty() { return Ok(Vec::new()); }
    if consulta.chars().count() > 120 { return Err("Búsqueda demasiado larga".into()); }
    let mut stmt = conn.prepare("SELECT id FROM clientes WHERE activo=1 AND (instr(lower(nombre),lower(?1))>0 OR instr(COALESCE(telefono,''),?1)>0 OR CAST(id AS TEXT)=?1) ORDER BY nombre,id LIMIT 30").map_err(|e| e.to_string())?;
    let ids = stmt.query_map([consulta], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    ids.into_iter().map(|id| obtener_cliente_checkin(conn, id)).collect()
}

// ESTA ES LA BUENA - ya no recibe template string de Vue, lo captura ella
pub fn buscar_cliente_por_huella(app: &AppHandle) -> Result<Option<ClienteCheckIn>, String> {
    buscar_cliente_por_huella_worker(app).map_err(Into::into)
}

// Conserva los códigos del SDK hasta el supervisor; IPC sigue devolviendo String.
pub fn buscar_cliente_por_huella_worker(app: &AppHandle) -> Result<Option<ClienteCheckIn>, secugen::SecuGenError> {
    let _traza = crate::lector_mensajes::Tramo::nuevo("checkin_global");
    let conn = database::abrir_db(app)?;
    let resource_dir = app.path_resolver().resource_dir().unwrap_or_default();
    let bin_path = secugen::resolve_bin_path(&resource_dir).map_err(secugen::SecuGenError::Initialization)?;
    let controller = match crate::lector_worker::inicializar_lector(&bin_path).map_err(secugen::SecuGenError::Initialization)? {
        Some(controller) => controller,
        None => return Ok(None),
    };
    // Informar SDK inicializado antes de esperar la captura de 4000 ms.
    // Este estado conserva la alerta hasta obtener una imagen válida.
    secugen::publicar_evidencia(app);

    let captured = controller.capturar_template()?;
    let captured = match captured { Some(value) => value, None => return Ok(None) };
    let id_opt = controller.identificar_template(&conn, &captured)?;

    if let Some(id) = id_opt {
        let cliente = conn.query_row("SELECT id, nombre, COALESCE(telefono, ''), COALESCE(telefono_emergencia, ''), COALESCE(foto_base64, ''), COALESCE(fecha_registro, ''), COALESCE(fecha_inicio_membresia, ''), COALESCE(fecha_fin_membresia, ''), CAST(julianday(date(fecha_fin_membresia)) - julianday(date('now','localtime')) AS INTEGER), COALESCE((SELECT concepto_membresia FROM pagos WHERE cliente_id=clientes.id ORDER BY id DESC LIMIT 1),'Sin plan registrado') FROM clientes WHERE id=?1 AND activo=1", params![id],
            |r| { let dias: i64 = r.get::<_, Option<i64>>(8)?.unwrap_or(-1); let fecha_inicio: String = r.get(6)?; let hoy = chrono::Local::now().format("%Y-%m-%d").to_string(); Ok(ClienteCheckIn{ tipo_membresia: r.get(9)?, id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?, telefono_emergencia: r.get(3)?, foto_base64: r.get(4)?, fecha_registro: r.get(5)?, fecha_inicio_membresia: r.get(6)?, fecha_fin_membresia: r.get(7)?, estado_membresia: (if fecha_inicio > hoy { "pendiente" } else if dias < 0 { "vencida" } else if dias <= 7 { "por_vencer" } else { "activa" }).to_string(), dias_restantes: dias }) }
        ).map_err(|e| e.to_string())?;
        Ok(Some(cliente))
    } else {
        // Una captura válida sin coincidencia no es un timeout del sensor.
        let _ = app.emit_all("logym://lector-aviso", "no_registrado");
        Ok(None)
    }
}


pub fn registrar_asistencia(app: &AppHandle, cliente_id: i64) -> Result<String, String> {
    let conn = database::abrir_db(app)?; repositories::registrar_asistencia(&conn, cliente_id)
}

/// La confirmación manual comparte las restricciones de acceso del lector.
pub fn registrar_asistencia_manual(conn: &rusqlite::Connection, cliente_id: i64) -> Result<String, String> {
    repositories::registrar_asistencia(conn, cliente_id)
}
pub fn renovar_membresia(app: &AppHandle, datos: DatosRenovacion) -> Result<ClienteCheckIn, String> {
    let conn = database::abrir_db(app)?;
    repositories::renovar_membresia(&conn, datos.cliente_id, datos.tipo_membresia_id.ok_or("Selecciona el tipo de membresía para renovar")?)?;
    obtener_cliente_checkin(&conn, datos.cliente_id)
}

/// Consulta un expediente completo desde SQLite para evitar datos obsoletos en Vue.
fn obtener_cliente_checkin(conn: &rusqlite::Connection, id: i64) -> Result<ClienteCheckIn, String> {
    conn.query_row(
        "SELECT id, nombre, COALESCE(telefono,''), COALESCE(telefono_emergencia,''), COALESCE(foto_base64,''), COALESCE(fecha_registro,''), COALESCE(fecha_inicio_membresia,''), COALESCE(fecha_fin_membresia,''), CAST(julianday(date(fecha_fin_membresia))-julianday(date('now','localtime')) AS INTEGER), COALESCE((SELECT concepto_membresia FROM pagos WHERE cliente_id=clientes.id ORDER BY id DESC LIMIT 1),'Sin plan registrado') FROM clientes WHERE id=?1 AND activo=1",
        params![id],
        |r| {
            let dias = r.get::<_, Option<i64>>(8)?.unwrap_or(-1); let fecha_inicio: String = r.get(6)?; let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
            Ok(ClienteCheckIn {
                tipo_membresia: r.get(9)?,
                id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?,
                telefono_emergencia: r.get(3)?, foto_base64: r.get(4)?,
                fecha_registro: r.get(5)?, fecha_inicio_membresia: r.get(6)?,
                fecha_fin_membresia: r.get(7)?,
                estado_membresia: (if fecha_inicio > hoy { "pendiente" } else if dias < 0 { "vencida" } else if dias <= 7 { "por_vencer" } else { "activa" }).to_string(),
                dias_restantes: dias,
            })
        },
    ).map_err(|e| e.to_string())
}
pub fn obtener_metricas(app: &AppHandle) -> Result<Metricas, String> {
    let conn = database::abrir_db(app)?; repositories::obtener_metricas(&conn)
}
pub fn check_in(app: &AppHandle) -> Result<Option<ClienteCheckIn>, String> {
    buscar_cliente_por_huella(app)
}

pub fn capturar_huella(app: &AppHandle) -> Result<Option<String>, String> {
    let _traza = crate::lector_mensajes::Tramo::nuevo("registro_capturar");
    let resource_path = app.path_resolver().resource_dir().unwrap_or_default();
    let bin_path = secugen::resolve_bin_path(&resource_path)?;
    let controller = secugen::SecuGenController::from_bin_path(&bin_path)?;
    controller.capturar_template().map_err(Into::into)
}

pub fn comparar_huella(app: &AppHandle, t1: &str, t2: &str) -> Result<bool, String> {
    let _traza = crate::lector_mensajes::Tramo::nuevo("registro_comparar");
    let resource_path = app.path_resolver().resource_dir().unwrap_or_default();
    let bin_path = secugen::resolve_bin_path(&resource_path)?;
    let controller = secugen::SecuGenController::from_bin_path(&bin_path)?;
    controller.comparar_templates(t1, t2)
}

/// Operaciones administrativas centralizadas: UI -> comando -> servicio -> repositorio.
pub fn actualizar_cliente(app: &AppHandle, id: i64, datos: ActualizacionCliente) -> Result<(), String> {
    let conn = database::abrir_db(app)?;
    repositories::actualizar_cliente(&conn, id, datos)
}
pub fn establecer_cliente_activo(app: &AppHandle, id: i64, activo: bool) -> Result<(), String> {
    let conn = database::abrir_db(app)?;
    repositories::establecer_cliente_activo(&conn, id, activo)
}
pub fn eliminar_cliente_definitivamente(app: &AppHandle, id: i64, confirmacion: String) -> Result<(), String> {
    if confirmacion != "ELIMINAR" { return Err("Confirmación de eliminación inválida".into()); }
    let mut conn = database::abrir_db(app)?;
    repositories::eliminar_cliente_definitivamente(&mut conn, id)
}
