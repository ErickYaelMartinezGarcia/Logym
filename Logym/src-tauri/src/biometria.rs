use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use once_cell::sync::Lazy;
use rusqlite::Connection;

static PLANTILLAS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Punto de integración del SDK SecuGen Hamster Pro 20.
/// Mientras el SDK/DLL no esté instalado, devuelve el template demo más reciente.
pub fn capturar_huella(conexion: &Connection) -> Result<String, String> {
    let mut sentencia = conexion
        .prepare(
            "SELECT template_huella FROM clientes
             WHERE COALESCE(template_huella, '') <> ''
             ORDER BY id DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    match sentencia.query_row([], |fila| fila.get::<usize, String>(0)) {
        Ok(valor) => Ok(valor),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(format!(
            "template_fake_dp4500_{}",
            Utc::now().timestamp_millis()
        )),
        Err(e) => Err(e.to_string()),
    }
}

pub fn guardar_template(cliente_id: String, template: String) -> Result<bool, String> {
    let mut mapa = PLANTILLAS
        .lock()
        .map_err(|_| String::from("No se pudo bloquear memoria de plantillas"))?;
    mapa.insert(cliente_id, template);
    Ok(true)
}

pub fn comparar_huella(template_guardado: String, template_nuevo: String) -> Result<bool, String> {
    Ok(template_guardado == template_nuevo)
}
