use crate::biometria;
use crate::database::abrir_db;
use crate::models::{
    ClienteCheckIn, ClienteMembresia, ClienteTabla, DatosCliente, DatosRenovacion, Metricas,
};
use crate::services;

#[tauri::command]
pub fn inicializar_db(app: tauri::AppHandle) -> Result<bool, String> {
    services::inicializar_db(app)
}

#[tauri::command]
pub fn guardar_cliente(app: tauri::AppHandle, datos: DatosCliente) -> Result<i64, String> {
    services::guardar_cliente(app, datos)
}

#[tauri::command]
pub fn buscar_cliente_por_huella(
    app: tauri::AppHandle,
    template: String,
) -> Result<Option<ClienteCheckIn>, String> {
    services::buscar_cliente_por_huella(app, template)
}

#[tauri::command]
pub fn registrar_asistencia(app: tauri::AppHandle, cliente_id: i64) -> Result<bool, String> {
    services::registrar_asistencia(app, cliente_id)
}

#[tauri::command]
pub fn renovar_membresia(
    app: tauri::AppHandle,
    datos: DatosRenovacion,
) -> Result<ClienteCheckIn, String> {
    services::renovar_membresia(app, datos)
}

#[tauri::command]
pub fn listar_clientes_membresia(app: tauri::AppHandle) -> Result<Vec<ClienteMembresia>, String> {
    services::listar_clientes_membresia(app)
}

#[tauri::command]
pub fn listar_clientes(app: tauri::AppHandle) -> Result<Vec<ClienteTabla>, String> {
    services::listar_clientes(app)
}

#[tauri::command]
pub fn obtener_metricas(app: tauri::AppHandle) -> Result<Metricas, String> {
    services::obtener_metricas(app)
}

#[tauri::command]
pub fn capturar_huella(app: tauri::AppHandle) -> Result<String, String> {
    let conexion = abrir_db(&app)?;
    biometria::capturar_huella(&conexion)
}

#[tauri::command]
pub fn guardar_template(cliente_id: String, template: String) -> Result<bool, String> {
    biometria::guardar_template(cliente_id, template)
}

#[tauri::command]
pub fn comparar_huella(template_guardado: String, template_nuevo: String) -> Result<bool, String> {
    biometria::comparar_huella(template_guardado, template_nuevo)
}
