use crate::models::{DatosCliente, DatosRenovacion, ActualizacionCliente};
use crate::services;
use crate::licencia;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn inicializar_db(app: AppHandle) -> Result<String, String> { licencia::verificar(&app)?; services::inicializar_db(&app) }
#[tauri::command]
pub async fn guardar_cliente(app: AppHandle, cliente: DatosCliente) -> Result<i64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        licencia::verificar(&app)?;
        let id = services::guardar_cliente(&app, cliente)?;
        let _ = app.emit_all("logym://datos-actualizados", "cliente");
        Ok(id)
    }).await.map_err(|e|format!("No se pudo completar el alta: {e}"))?
}
#[tauri::command]
pub fn listar_clientes(app: AppHandle) -> Result<Vec<crate::models::ClienteTabla>, String> { licencia::verificar(&app)?; services::listar_clientes(&app) }
#[tauri::command]
pub fn listar_clientes_membresia(app: AppHandle) -> Result<Vec<crate::models::ClienteMembresia>, String> { licencia::verificar(&app)?; services::listar_clientes_membresia(&app) }
#[tauri::command]
pub fn buscar_cliente_por_huella(app: AppHandle) -> Result<Option<crate::models::ClienteCheckIn>, String> { licencia::verificar(&app)?; services::buscar_cliente_por_huella(&app) }
#[tauri::command]
pub fn check_in(app: AppHandle) -> Result<Option<crate::models::ClienteCheckIn>, String> { licencia::verificar(&app)?; services::check_in(&app) }
#[tauri::command]
pub fn registrar_asistencia(app: AppHandle, cliente_id: i64) -> Result<String, String> { { licencia::verificar(&app)?; let result = services::registrar_asistencia(&app, cliente_id)?; let _ = app.emit_all("logym://datos-actualizados", "asistencia"); Ok(result) } }
#[tauri::command]
pub fn renovar_membresia(app: AppHandle, datos: DatosRenovacion) -> Result<crate::models::ClienteCheckIn, String> { { licencia::verificar(&app)?; let result = services::renovar_membresia(&app, datos)?; let _ = app.emit_all("logym://datos-actualizados", "membresia"); Ok(result) } }
#[tauri::command]
pub fn obtener_metricas(app: AppHandle) -> Result<crate::models::Metricas, String> { licencia::verificar(&app)?; services::obtener_metricas(&app) }
#[tauri::command]
pub fn capturar_huella(app: AppHandle) -> Result<Option<String>, String> {
    licencia::verificar(&app)?;
    services::capturar_huella(&app)
}

#[tauri::command]
pub fn comparar_huella(app: AppHandle, t1: String, t2: String) -> Result<bool, String> {
    licencia::verificar(&app)?;
    services::comparar_huella(&app, &t1, &t2)
}

#[tauri::command]
pub fn actualizar_cliente(app: AppHandle, id: i64, datos: ActualizacionCliente) -> Result<(), String> {
    { licencia::verificar(&app)?; services::actualizar_cliente(&app, id, datos)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(()) }
}
#[tauri::command]
pub fn establecer_cliente_activo(app: AppHandle, id: i64, activo: bool) -> Result<(), String> {
    { licencia::verificar(&app)?; services::establecer_cliente_activo(&app, id, activo)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(()) }
}
#[tauri::command]
pub fn eliminar_cliente_definitivamente(app: AppHandle, id: i64, confirmacion: String) -> Result<(), String> {
    { licencia::verificar(&app)?; services::eliminar_cliente_definitivamente(&app, id, confirmacion)?; let _ = app.emit_all("logym://datos-actualizados", "cliente"); Ok(()) }
}

/// Pausa temporalmente el supervisor al registrar una huella nueva.
#[tauri::command]
pub fn estado_lector() -> crate::lector_estado::EstadoLector {
    crate::lector_estado::consultar()
}

#[tauri::command]
pub fn pausar_lector_global(app: AppHandle, pausado: bool) -> Result<(), String> {
    licencia::verificar(&app)?;
    crate::lector_worker::pausar(pausado);
    Ok(())
}

#[tauri::command]
pub fn solicitud_licencia(app: AppHandle) -> Result<String, String> { licencia::solicitud(&app) }

#[tauri::command]
pub fn estado_licencia(app: AppHandle) -> bool { licencia::verificar(&app).is_ok() }

#[tauri::command]
pub fn activar_licencia(app: AppHandle, licencia_texto: String) -> Result<(), String> {
    licencia::activar(&app, licencia_texto)?;
    let conn = crate::database::abrir_db(&app)?;
    crate::database::inicializar_esquema(&conn)?;
    crate::lector_worker::iniciar(app.clone());
    Ok(())
}

#[tauri::command]
pub fn listar_tipos_membresia(app: AppHandle) -> Result<Vec<crate::membresias::TipoMembresia>, String> {
    licencia::verificar(&app)?;
    let conn = crate::database::abrir_db(&app)?;
    crate::membresias::listar(&conn)
}

#[tauri::command]
pub fn obtener_precio_inscripcion(app: AppHandle) -> Result<i64, String> {
    licencia::verificar(&app)?;
    let conn = crate::database::abrir_db(&app)?;
    crate::membresias::inscripcion(&conn)
}

#[tauri::command]
pub fn resumen_ingresos(app: AppHandle, periodo: String, password: String) -> Result<crate::membresias::ResumenIngresos, String> {
    licencia::verificar(&app)?;
    let conn = crate::database::abrir_db(&app)?;
    crate::membresias::autenticar(&conn, &password)?;
    crate::membresias::ingresos(&conn, &periodo)
}

/// Reporte financiero privado: requiere licencia y autenticación administrativa.
#[tauri::command]
pub fn reporte_financiero(app: AppHandle, fecha_inicio: String, fecha_fin: String, password: String) -> Result<crate::membresias::ReporteFinanciero, String> {
    licencia::verificar(&app)?;
    let conn = crate::database::abrir_db(&app)?;
    crate::membresias::autenticar(&conn, &password)?;
    crate::membresias::ingresos_rango(&conn, &fecha_inicio, &fecha_fin)
}

#[tauri::command]
pub fn admin_configurado(app: AppHandle) -> Result<bool,String> {
    licencia::verificar(&app)?;
    crate::membresias::tiene_administrador(&crate::database::abrir_db(&app)?)
}
#[tauri::command]
pub fn crear_password_admin(app: AppHandle, password: String) -> Result<(),String> {
    licencia::verificar(&app)?;
    crate::membresias::configurar_administrador(&crate::database::abrir_db(&app)?,&password)
}
#[tauri::command]
pub fn validar_password_admin(app: AppHandle, password: String) -> Result<(),String> {
    licencia::verificar(&app)?;
    crate::membresias::autenticar(&crate::database::abrir_db(&app)?,&password)
}
#[tauri::command]
pub fn guardar_plan_admin(app: AppHandle, password: String, id: Option<i64>, nombre: String, meses: i64, precio_centavos: i64, activo: bool) -> Result<(),String> {
    licencia::verificar(&app)?;
    crate::membresias::actualizar_plan(&crate::database::abrir_db(&app)?,&password,id,&nombre,meses,precio_centavos,activo)?;
    let _ = app.emit_all("logym://datos-actualizados","precios");
    Ok(())
}
#[tauri::command]
pub fn guardar_inscripcion_admin(app: AppHandle, password: String, precio_centavos: i64) -> Result<(),String> {
    licencia::verificar(&app)?;
    crate::membresias::actualizar_inscripcion(&crate::database::abrir_db(&app)?,&password,precio_centavos)?;
    let _ = app.emit_all("logym://datos-actualizados","precios");
    Ok(())
}
#[tauri::command]
pub fn obtener_asistencias_periodo(app: AppHandle, periodo: String) -> Result<i64,String> {
    licencia::verificar(&app)?;
    crate::membresias::asistencias_periodo(&crate::database::abrir_db(&app)?,&periodo)
}

#[tauri::command]
pub fn listar_planes_admin(app: AppHandle, password: String) -> Result<Vec<crate::membresias::TipoMembresia>,String> {
 licencia::verificar(&app)?;
 crate::membresias::listar_admin(&crate::database::abrir_db(&app)?,&password)
}

/// Conteo por hora para la vista Hoy; mantiene la verificación de licencia.
#[tauri::command]
pub fn obtener_asistencias_por_hora(app: AppHandle) -> Result<Vec<crate::membresias::AsistenciaDiaria>, String> {
    licencia::verificar(&app)?;
    crate::membresias::asistencias_por_hora_hoy(&crate::database::abrir_db(&app)?)
}

#[tauri::command]
pub fn obtener_asistencias_diarias(app: AppHandle, periodo: String) -> Result<Vec<crate::membresias::AsistenciaDiaria>, String> {
    licencia::verificar(&app)?;
    crate::membresias::asistencias_diarias(&crate::database::abrir_db(&app)?, &periodo)
}
