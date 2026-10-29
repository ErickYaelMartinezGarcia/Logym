use crate::database::{abrir_db, inicializar_esquema};
use crate::models::{
    ClienteCheckIn, ClienteMembresia, ClienteTabla, DatosCliente, DatosRenovacion, Metricas,
};
use crate::repositories;

pub fn inicializar_db(app: tauri::AppHandle) -> Result<bool, String> {
    let conexion = abrir_db(&app)?;
    inicializar_esquema(&conexion)?;
    Ok(true)
}

pub fn guardar_cliente(app: tauri::AppHandle, datos: DatosCliente) -> Result<i64, String> {
    let conexion = abrir_db(&app)?;
    repositories::guardar_cliente(&conexion, datos)
}

pub fn buscar_cliente_por_huella(
    app: tauri::AppHandle,
    template: String,
) -> Result<Option<ClienteCheckIn>, String> {
    let conexion = abrir_db(&app)?;
    repositories::buscar_cliente_por_huella(&conexion, &template)
}

pub fn registrar_asistencia(app: tauri::AppHandle, cliente_id: i64) -> Result<bool, String> {
    let conexion = abrir_db(&app)?;
    repositories::registrar_asistencia(&conexion, cliente_id)
}

pub fn renovar_membresia(
    app: tauri::AppHandle,
    datos: DatosRenovacion,
) -> Result<ClienteCheckIn, String> {
    let conexion = abrir_db(&app)?;
    repositories::renovar_membresia(&conexion, datos)
}

pub fn listar_clientes_membresia(app: tauri::AppHandle) -> Result<Vec<ClienteMembresia>, String> {
    let conexion = abrir_db(&app)?;
    repositories::listar_clientes_membresia(&conexion)
}

pub fn listar_clientes(app: tauri::AppHandle) -> Result<Vec<ClienteTabla>, String> {
    let conexion = abrir_db(&app)?;
    repositories::listar_clientes(&conexion)
}

pub fn obtener_metricas(app: tauri::AppHandle) -> Result<Metricas, String> {
    let conexion = abrir_db(&app)?;
    repositories::obtener_metricas(&conexion)
}
