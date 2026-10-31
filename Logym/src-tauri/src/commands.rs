use crate::models::{DatosCliente, DatosRenovacion, ActualizacionCliente};
use crate::services;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn inicializar_db(app: AppHandle) -> Result<String, String> { services::inicializar_db(&app) }
#[tauri::command]
pub fn guardar_cliente(app: AppHandle, cliente: DatosCliente) -> Result<i64, String> { { let id = services::guardar_cliente(&app, cliente)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(id) } }
#[tauri::command]
pub fn listar_clientes(app: AppHandle) -> Result<Vec<crate::models::ClienteTabla>, String> { services::listar_clientes(&app) }
#[tauri::command]
pub fn listar_clientes_membresia(app: AppHandle) -> Result<Vec<crate::models::ClienteMembresia>, String> { services::listar_clientes_membresia(&app) }
#[tauri::command]
pub fn buscar_cliente_por_huella(app: AppHandle) -> Result<Option<crate::models::ClienteCheckIn>, String> { services::buscar_cliente_por_huella(&app) }
#[tauri::command]
pub fn check_in(app: AppHandle) -> Result<Option<crate::models::ClienteCheckIn>, String> { services::check_in(&app) }
#[tauri::command]
pub fn registrar_asistencia(app: AppHandle, cliente_id: i64) -> Result<String, String> { { let result = services::registrar_asistencia(&app, cliente_id)?; let _ = app.emit_all("logym://datos-actualizados", "asistencia"); Ok(result) } }
#[tauri::command]
pub fn renovar_membresia(app: AppHandle, datos: DatosRenovacion) -> Result<crate::models::ClienteCheckIn, String> { { let result = services::renovar_membresia(&app, datos)?; let _ = app.emit_all("logym://datos-actualizados", "membresia"); Ok(result) } }
#[tauri::command]
pub fn obtener_metricas(app: AppHandle) -> Result<crate::models::Metricas, String> { services::obtener_metricas(&app) }
#[tauri::command]
pub fn capturar_huella(app: AppHandle) -> Result<Option<String>, String> {
    services::capturar_huella(&app)
}

#[tauri::command]
pub fn comparar_huella(app: AppHandle, t1: String, t2: String) -> Result<bool, String> {
    services::comparar_huella(&app, &t1, &t2)
}

#[tauri::command]
pub fn actualizar_cliente(app: AppHandle, id: i64, datos: ActualizacionCliente) -> Result<(), String> {
    { services::actualizar_cliente(&app, id, datos)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(()) }
}
#[tauri::command]
pub fn establecer_cliente_activo(app: AppHandle, id: i64, activo: bool) -> Result<(), String> {
    { services::establecer_cliente_activo(&app, id, activo)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(()) }
}
#[tauri::command]
pub fn eliminar_cliente_definitivamente(app: AppHandle, id: i64, confirmacion: String) -> Result<(), String> {
    { services::eliminar_cliente_definitivamente(&app, id, confirmacion)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(()) }
}

/// Pausa temporalmente el supervisor al registrar una huella nueva.
#[tauri::command]
pub fn pausar_lector_global(pausado: bool) {
    crate::lector_worker::pausar(pausado);
}
