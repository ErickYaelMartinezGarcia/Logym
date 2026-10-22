#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

static PLANTILLAS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Deserialize)]
struct DatosCliente {
    nombre: String,
    telefono: String,
    #[serde(rename = "fotoBase64")]
    foto_base64: String,
    #[serde(rename = "templateHuella")]
    template_huella: String,
}

#[derive(Serialize)]
struct ClienteTabla {
    id: i64,
    nombre: String,
    telefono: String,
    fecha_registro: String,
}

#[derive(Serialize)]
struct Metricas {
    total_clientes: i64,
    asistencias_hoy: i64,
    nuevos_semana: i64,
    huella_activas: i64,
}

/**
 * Abre la base de datos local gym.db.
 */
fn abrir_db() -> Result<Connection, String> {
    Connection::open("gym.db").map_err(|e| e.to_string())
}

/**
 * Crea tablas base: clientes y asistencias.
 */
#[tauri::command]
fn inicializar_db() -> Result<bool, String> {
    let conexion = abrir_db()?;

    conexion
        .execute(
            "CREATE TABLE IF NOT EXISTS clientes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nombre TEXT NOT NULL,
                telefono TEXT,
                foto_base64 TEXT,
                template_huella TEXT,
                fecha_registro TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

    conexion
        .execute(
            "CREATE TABLE IF NOT EXISTS asistencias (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cliente_id INTEGER NOT NULL,
                fecha_hora TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (cliente_id) REFERENCES clientes(id)
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/**
 * Guarda un cliente nuevo y regresa su identificador.
 */
#[tauri::command]
fn guardar_cliente(datos: DatosCliente) -> Result<i64, String> {
    let conexion = abrir_db()?;

    conexion
        .execute(
            "INSERT INTO clientes (nombre, telefono, foto_base64, template_huella) VALUES (?1, ?2, ?3, ?4)",
            params![datos.nombre, datos.telefono, datos.foto_base64, datos.template_huella],
        )
        .map_err(|e| e.to_string())?;

    Ok(conexion.last_insert_rowid())
}

/**
 * Regresa los clientes más recientes para mostrarlos en dashboard.
 */
#[tauri::command]
fn listar_clientes() -> Result<Vec<ClienteTabla>, String> {
    let conexion = abrir_db()?;
    let mut sentencia = conexion
        .prepare("SELECT id, nombre, COALESCE(telefono, ''), fecha_registro FROM clientes ORDER BY id DESC LIMIT 20")
        .map_err(|e| e.to_string())?;

    let filas = sentencia
        .query_map([], |fila| {
            Ok(ClienteTabla {
                id: fila.get(0)?,
                nombre: fila.get(1)?,
                telefono: fila.get(2)?,
                fecha_registro: fila.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut clientes = Vec::new();
    for fila in filas {
        clientes.push(fila.map_err(|e| e.to_string())?);
    }

    Ok(clientes)
}

/**
 * Regresa métricas rápidas para cards del dashboard.
 */
#[tauri::command]
fn obtener_metricas() -> Result<Metricas, String> {
    let conexion = abrir_db()?;

    let total_clientes: i64 = conexion
        .query_row("SELECT COUNT(*) FROM clientes", [], |fila| fila.get(0))
        .map_err(|e| e.to_string())?;

    let asistencias_hoy: i64 = conexion
        .query_row(
            "SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora) = date('now')",
            [],
            |fila| fila.get(0),
        )
        .map_err(|e| e.to_string())?;

    let nuevos_semana: i64 = conexion
        .query_row(
            "SELECT COUNT(*) FROM clientes WHERE date(fecha_registro) >= date('now', '-7 day')",
            [],
            |fila| fila.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(Metricas {
        total_clientes,
        asistencias_hoy,
        nuevos_semana,
        huella_activas: total_clientes,
    })
}

/**
 * Captura una huella de manera simulada.
 * Este mock deja listo el punto donde luego se conectará la DLL del DigitalPersona 4500.
 */
#[tauri::command]
fn capturar_huella() -> Result<String, String> {
    Ok(format!("template_fake_dp4500_{}", Utc::now().timestamp_millis()))
}

/**
 * Guarda el template de una huella simulada en memoria.
 * Se usa como puente temporal mientras se integra el SDK real del lector.
 */
#[tauri::command]
fn guardar_template(cliente_id: String, template: String) -> Result<bool, String> {
    let mut mapa = PLANTILLAS
        .lock()
        .map_err(|_| String::from("No se pudo bloquear memoria de plantillas"))?;

    mapa.insert(cliente_id, template);
    Ok(true)
}

/**
 * Compara una huella capturada contra una huella guardada.
 * Por ahora hace una comparación simple de texto como simulación.
 */
#[tauri::command]
fn comparar_huella(template_guardado: String, template_nuevo: String) -> Result<bool, String> {
    Ok(template_guardado == template_nuevo)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            inicializar_db,
            guardar_cliente,
            listar_clientes,
            obtener_metricas,
            capturar_huella,
            guardar_template,
            comparar_huella
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
