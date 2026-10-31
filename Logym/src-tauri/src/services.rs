use crate::database;
use crate::models::*;
use crate::repositories;
use crate::secugen;
use rusqlite::params;
use tauri::AppHandle;

pub fn inicializar_db(app: &AppHandle) -> Result<String, String> {
    let conn = database::abrir_db(app)?;
    database::inicializar_esquema(&conn)?;
    Ok("DB OK".into())
}

pub fn guardar_cliente(app: &AppHandle, mut datos: DatosCliente) -> Result<i64, String> {
    datos.nombre = datos.nombre.trim().to_string();
    if datos.nombre.is_empty() || datos.nombre.chars().count() > 120 || datos.nombre.chars().any(char::is_control) {
        return Err("Nombre inválido".into());
    }
    for telefono in [&datos.telefono, &datos.telefono_emergencia] {
        if telefono.len() > 25 || !telefono.chars().all(|ch| ch.is_ascii_digit() || "+- ()".contains(ch)) {
            return Err("Teléfono inválido".into());
        }
    }
    if datos.template_huella.len() != 800 || !datos.template_huella.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Plantilla biométrica inválida".into());
    }
    if datos.foto_base64.len() > 3_000_000 {
        return Err("Fotografía demasiado grande".into());
    }
    if !datos.foto_base64.is_empty() && !["data:image/jpeg;base64,", "data:image/png;base64,", "data:image/webp;base64,"].iter().any(|p| datos.foto_base64.starts_with(p)) {
        return Err("Formato de fotografía inválido".into());
    }
    if datos.fecha_registro.is_empty() { datos.fecha_registro = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(); }
    if datos.fecha_inicio_membresia.is_empty() { datos.fecha_inicio_membresia = chrono::Local::now().format("%Y-%m-%d").to_string(); }
    if datos.fecha_fin_membresia.is_empty() { datos.fecha_fin_membresia = (chrono::Local::now() + chrono::Duration::days(30)).format("%Y-%m-%d").to_string(); }
    let conn = database::abrir_db(app)?;
    repositories::guardar_cliente(&conn, datos)
}

pub fn listar_clientes(app: &AppHandle) -> Result<Vec<ClienteTabla>, String> {
    let conn = database::abrir_db(app)?; repositories::listar_clientes(&conn)
}
pub fn listar_clientes_membresia(app: &AppHandle) -> Result<Vec<ClienteMembresia>, String> {
    let conn = database::abrir_db(app)?; repositories::listar_clientes_membresia(&conn)
}

// ESTA ES LA BUENA - ya no recibe template string de Vue, lo captura ella
pub fn buscar_cliente_por_huella(app: &AppHandle) -> Result<Option<ClienteCheckIn>, String> {
    let conn = database::abrir_db(app)?;
    let resource_dir = app.path_resolver().resource_dir().unwrap_or_default();
    let bin_path = secugen::resolve_bin_path(&resource_dir).map_err(|e| e.to_string())?;
    let controller = secugen::SecuGenController::from_bin_path(&bin_path)?;

    let captured = controller.capturar_template()?;
    let captured = match captured { Some(value) => value, None => return Ok(None) };
    let id_opt = controller.identificar_template(&conn, &captured)?;

    if let Some(id) = id_opt {
        let cliente = conn.query_row("SELECT id, nombre, COALESCE(telefono, ''), COALESCE(telefono_emergencia, ''), COALESCE(foto_base64, ''), COALESCE(fecha_registro, ''), COALESCE(fecha_inicio_membresia, ''), COALESCE(fecha_fin_membresia, ''), CAST(julianday(date(fecha_fin_membresia)) - julianday(date('now','localtime')) AS INTEGER) FROM clientes WHERE id=?1 AND activo=1", params![id],
            |r| { let dias: i64 = r.get::<_, Option<i64>>(8)?.unwrap_or(-1); Ok(ClienteCheckIn{ id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?, telefono_emergencia: r.get(3)?, foto_base64: r.get(4)?, fecha_registro: r.get(5)?, fecha_inicio_membresia: r.get(6)?, fecha_fin_membresia: r.get(7)?, estado_membresia: (if dias < 0 { "vencida" } else if dias <= 7 { "por_vencer" } else { "activa" }).to_string(), dias_restantes: dias }) }
        ).map_err(|e| e.to_string())?;
        Ok(Some(cliente))
    } else {
        Ok(None)
    }
}


pub fn registrar_asistencia(app: &AppHandle, cliente_id: i64) -> Result<String, String> {
    let conn = database::abrir_db(app)?; repositories::registrar_asistencia(&conn, cliente_id)
}
pub fn renovar_membresia(app: &AppHandle, datos: DatosRenovacion) -> Result<ClienteCheckIn, String> {
    let conn = database::abrir_db(app)?;
    repositories::renovar_membresia(&conn, datos.cliente_id)?;
    obtener_cliente_checkin(&conn, datos.cliente_id)
}

/// Consulta un expediente completo desde SQLite para evitar datos obsoletos en Vue.
fn obtener_cliente_checkin(conn: &rusqlite::Connection, id: i64) -> Result<ClienteCheckIn, String> {
    conn.query_row(
        "SELECT id, nombre, COALESCE(telefono,''), COALESCE(telefono_emergencia,''), COALESCE(foto_base64,''), COALESCE(fecha_registro,''), COALESCE(fecha_inicio_membresia,''), COALESCE(fecha_fin_membresia,''), CAST(julianday(date(fecha_fin_membresia))-julianday(date('now','localtime')) AS INTEGER) FROM clientes WHERE id=?1 AND activo=1",
        params![id],
        |r| {
            let dias = r.get::<_, Option<i64>>(8)?.unwrap_or(-1);
            Ok(ClienteCheckIn {
                id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?,
                telefono_emergencia: r.get(3)?, foto_base64: r.get(4)?,
                fecha_registro: r.get(5)?, fecha_inicio_membresia: r.get(6)?,
                fecha_fin_membresia: r.get(7)?,
                estado_membresia: (if dias < 0 { "vencida" } else if dias <= 7 { "por_vencer" } else { "activa" }).to_string(),
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
    let resource_path = app.path_resolver().resource_dir().unwrap_or_default();
    let bin_path = secugen::resolve_bin_path(&resource_path)?;
    let controller = secugen::SecuGenController::from_bin_path(&bin_path)?;
    controller.capturar_template()
}

pub fn comparar_huella(app: &AppHandle, t1: &str, t2: &str) -> Result<bool, String> {
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
