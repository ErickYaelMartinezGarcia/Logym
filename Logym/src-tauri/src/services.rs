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
    if datos.nombre.trim().is_empty() { return Err("Nombre vacío".into()) }
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
    let id_opt = controller.identificar_template(&conn, &captured)?;

    if let Some(id) = id_opt {
        let cliente = conn.query_row("SELECT id, nombre, telefono FROM clientes WHERE id=?1", params![id],
            |r| Ok(ClienteCheckIn{ id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)? })
        ).map_err(|e| e.to_string())?;
        Ok(Some(cliente))
    } else {
        Ok(None)
    }
}


pub fn registrar_asistencia(app: &AppHandle, cliente_id: i64) -> Result<String, String> {
    let conn = database::abrir_db(app)?; repositories::registrar_asistencia(&conn, cliente_id)
}
pub fn renovar_membresia(app: &AppHandle, datos: DatosRenovacion) -> Result<String, String> {
    let conn = database::abrir_db(app)?; repositories::renovar_membresia(&conn, datos.cliente_id)
}
pub fn obtener_metricas(app: &AppHandle) -> Result<Metricas, String> {
    let conn = database::abrir_db(app)?; repositories::obtener_metricas(&conn)
}
pub fn check_in(app: &AppHandle) -> Result<Option<ClienteCheckIn>, String> {
    // ...
    let captured_opt = controller.capturar_template()?;
    let captured = match captured_opt {
        Some(c) => c,
        None => return Ok(None), // no hay dedo
    };
    let id_opt = controller.identificar_template(&conn, &captured)?;
    // ...
}

pub fn capturar_huella(app: &AppHandle) -> Result<Option<String>, String> {
    let resource_path = app.path().resource_dir().map_err(|e| e.to_string())?;
    let bin_path = crate::secugen::resolve_bin_path(&resource_path)?;
    let controller = SecuGenController::from_bin_path(&bin_path)?;
    controller.capturar_template() // ya regresa Option
}