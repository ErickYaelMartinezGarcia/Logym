#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{Duration, Utc};
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

static PLANTILLAS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Deserialize)]
struct DatosCliente {
    nombre: String,
    telefono: String,
    #[serde(rename = "fechaRegistro")]
    fecha_registro: String,
    #[serde(rename = "fechaInicioMembresia")]
    fecha_inicio_membresia: String,
    #[serde(rename = "fechaFinMembresia")]
    fecha_fin_membresia: String,
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

#[derive(Serialize)]
struct ClienteMembresia {
    id: i64,
    nombre: String,
    telefono: String,
    fecha_registro: String,
    fecha_inicio_membresia: String,
    fecha_fin_membresia: String,
    estado_membresia: String,
    dias_restantes: i64,
}

#[derive(Serialize)]
struct ClienteCheckIn {
    id: i64,
    nombre: String,
    telefono: String,
    foto_base64: String,
    fecha_registro: String,
    fecha_inicio_membresia: String,
    fecha_fin_membresia: String,
    estado_membresia: String,
    dias_restantes: i64,
}

#[derive(Deserialize)]
struct DatosRenovacion {
    #[serde(rename = "clienteId")]
    cliente_id: i64,
    dias: i64,
    monto: f64,
}

/**
 * Abre la base de datos local gym.db.
 */
fn abrir_db() -> Result<Connection, String> {
    Connection::open("gym.db").map_err(|e| e.to_string())
}

fn asegurar_columna_clientes(conexion: &Connection, nombre_columna: &str, definicion: &str) -> Result<(), String> {
    let mut sentencia = conexion
        .prepare("PRAGMA table_info(clientes)")
        .map_err(|e| e.to_string())?;

    let columnas = sentencia
        .query_map([], |fila| fila.get::<usize, String>(1))
        .map_err(|e| e.to_string())?;

    let mut existe = false;
    for columna in columnas {
        if columna.map_err(|e| e.to_string())? == nombre_columna {
            existe = true;
            break;
        }
    }

    if !existe {
        conexion
            .execute(
                &format!("ALTER TABLE clientes ADD COLUMN {} {}", nombre_columna, definicion),
                [],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn calcular_estado_membresia(fecha_fin: &str) -> (String, i64) {
    if fecha_fin.trim().is_empty() {
        return (String::from("sin_fecha"), 9999);
    }

    let hoy = Utc::now().date_naive();
    match chrono::NaiveDate::parse_from_str(fecha_fin, "%Y-%m-%d") {
        Ok(fin) => {
            let dias = (fin - hoy).num_days();
            if dias < 0 {
                (String::from("vencida"), dias)
            } else if dias <= 7 {
                (String::from("por_vencer"), dias)
            } else {
                (String::from("activa"), dias)
            }
        }
        Err(_) => (String::from("sin_fecha"), 9999),
    }
}

fn obtener_cliente_checkin_por_template(conexion: &Connection, template: &str) -> Result<Option<ClienteCheckIn>, String> {
    let mut sentencia = conexion
        .prepare(
            "SELECT id, nombre, COALESCE(telefono, ''), COALESCE(foto_base64, ''), COALESCE(fecha_registro, ''), COALESCE(fecha_inicio_membresia, ''), COALESCE(fecha_fin_membresia, '')
             FROM clientes
             WHERE template_huella = ?1
             ORDER BY id DESC
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let resultado = sentencia
        .query_row(params![template], |fila| {
            Ok((
                fila.get::<usize, i64>(0)?,
                fila.get::<usize, String>(1)?,
                fila.get::<usize, String>(2)?,
                fila.get::<usize, String>(3)?,
                fila.get::<usize, String>(4)?,
                fila.get::<usize, String>(5)?,
                fila.get::<usize, String>(6)?,
            ))
        });

    match resultado {
        Ok((id, nombre, telefono, foto_base64, fecha_registro, fecha_inicio_membresia, fecha_fin_membresia)) => {
            let (estado_membresia, dias_restantes) = calcular_estado_membresia(&fecha_fin_membresia);
            Ok(Some(ClienteCheckIn {
                id,
                nombre,
                telefono,
                foto_base64,
                fecha_registro,
                fecha_inicio_membresia,
                fecha_fin_membresia,
                estado_membresia,
                dias_restantes,
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
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
                fecha_registro TEXT DEFAULT CURRENT_TIMESTAMP,
                fecha_inicio_membresia TEXT DEFAULT (date('now')),
                fecha_fin_membresia TEXT DEFAULT (date('now', '+30 day'))
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

    asegurar_columna_clientes(&conexion, "fecha_inicio_membresia", "TEXT DEFAULT (date('now'))")?;
    asegurar_columna_clientes(&conexion, "fecha_fin_membresia", "TEXT DEFAULT (date('now', '+30 day'))")?;

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

    conexion
        .execute(
            "CREATE TABLE IF NOT EXISTS membresias_historial (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cliente_id INTEGER NOT NULL,
                fecha_inicio TEXT NOT NULL,
                fecha_fin TEXT NOT NULL,
                fecha_pago TEXT NOT NULL,
                monto REAL DEFAULT 0,
                tipo TEXT DEFAULT 'renovacion',
                observaciones TEXT,
                creado_en TEXT DEFAULT CURRENT_TIMESTAMP,
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
            "INSERT INTO clientes (nombre, telefono, fecha_registro, fecha_inicio_membresia, fecha_fin_membresia, foto_base64, template_huella) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                datos.nombre,
                datos.telefono,
                datos.fecha_registro,
                datos.fecha_inicio_membresia,
                datos.fecha_fin_membresia,
                datos.foto_base64,
                datos.template_huella
            ],
        )
        .map_err(|e| e.to_string())?;

    let cliente_id = conexion.last_insert_rowid();

    conexion
        .execute(
            "INSERT INTO membresias_historial (cliente_id, fecha_inicio, fecha_fin, fecha_pago, monto, tipo, observaciones)
             VALUES (?1, ?2, ?3, ?4, 0, 'alta', 'Alta inicial del cliente')",
            params![
                cliente_id,
                datos.fecha_inicio_membresia,
                datos.fecha_fin_membresia,
                datos.fecha_registro
            ],
        )
        .map_err(|e| e.to_string())?;

    Ok(cliente_id)
}

#[tauri::command]
fn buscar_cliente_por_huella(template: String) -> Result<Option<ClienteCheckIn>, String> {
    let conexion = abrir_db()?;
    obtener_cliente_checkin_por_template(&conexion, &template)
}

#[tauri::command]
fn renovar_membresia(datos: DatosRenovacion) -> Result<ClienteCheckIn, String> {
    if datos.dias <= 0 {
        return Err(String::from("La duración de la membresía debe ser mayor a 0 días"));
    }

    let conexion = abrir_db()?;
    let hoy = Utc::now().date_naive();
    let fecha_inicio = hoy.format("%Y-%m-%d").to_string();
    let fecha_fin = (hoy + Duration::days(datos.dias)).format("%Y-%m-%d").to_string();

    conexion
        .execute(
            "UPDATE clientes
             SET fecha_registro = ?1,
                 fecha_inicio_membresia = ?2,
                 fecha_fin_membresia = ?3
             WHERE id = ?4",
            params![fecha_inicio, fecha_inicio, fecha_fin, datos.cliente_id],
        )
        .map_err(|e| e.to_string())?;

    if conexion.changes() == 0 {
        return Err(String::from("No se encontró el cliente para renovar"));
    }

    conexion
        .execute(
            "INSERT INTO membresias_historial (cliente_id, fecha_inicio, fecha_fin, fecha_pago, monto, tipo, observaciones)
             VALUES (?1, ?2, ?3, ?4, ?5, 'renovacion', 'Renovación desde panel de check-in')",
            params![datos.cliente_id, fecha_inicio, fecha_fin, fecha_inicio, datos.monto],
        )
        .map_err(|e| e.to_string())?;

    let mut sentencia = conexion
        .prepare("SELECT template_huella FROM clientes WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let template: String = sentencia
        .query_row(params![datos.cliente_id], |fila| fila.get(0))
        .map_err(|e| e.to_string())?;

    match obtener_cliente_checkin_por_template(&conexion, &template)? {
        Some(cliente) => Ok(cliente),
        None => Err(String::from("No fue posible cargar el cliente renovado")),
    }
}

#[tauri::command]
fn listar_clientes_membresia() -> Result<Vec<ClienteMembresia>, String> {
    let conexion = abrir_db()?;
    let mut sentencia = conexion
        .prepare(
            "SELECT
                id,
                nombre,
                COALESCE(telefono, ''),
                COALESCE(fecha_registro, ''),
                COALESCE(fecha_inicio_membresia, ''),
                COALESCE(fecha_fin_membresia, ''),
                CASE
                    WHEN COALESCE(fecha_fin_membresia, '') = '' THEN 'sin_fecha'
                    WHEN julianday(fecha_fin_membresia) < julianday(date('now')) THEN 'vencida'
                    WHEN julianday(fecha_fin_membresia) <= julianday(date('now', '+7 day')) THEN 'por_vencer'
                    ELSE 'activa'
                END AS estado_membresia,
                CASE
                    WHEN COALESCE(fecha_fin_membresia, '') = '' THEN 9999
                    ELSE CAST(julianday(fecha_fin_membresia) - julianday(date('now')) AS INTEGER)
                END AS dias_restantes
             FROM clientes
             ORDER BY
                CASE
                    WHEN COALESCE(fecha_fin_membresia, '') = '' THEN 3
                    WHEN julianday(fecha_fin_membresia) < julianday(date('now')) THEN 0
                    WHEN julianday(fecha_fin_membresia) <= julianday(date('now', '+7 day')) THEN 1
                    ELSE 2
                END,
                julianday(COALESCE(fecha_fin_membresia, date('now', '+3650 day'))) ASC,
                id DESC",
        )
        .map_err(|e| e.to_string())?;

    let filas = sentencia
        .query_map([], |fila| {
            Ok(ClienteMembresia {
                id: fila.get(0)?,
                nombre: fila.get(1)?,
                telefono: fila.get(2)?,
                fecha_registro: fila.get(3)?,
                fecha_inicio_membresia: fila.get(4)?,
                fecha_fin_membresia: fila.get(5)?,
                estado_membresia: fila.get(6)?,
                dias_restantes: fila.get(7)?,
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
    let conexion = abrir_db()?;
    let mut sentencia = conexion
        .prepare("SELECT template_huella FROM clientes WHERE COALESCE(template_huella, '') <> '' ORDER BY id DESC LIMIT 1")
        .map_err(|e| e.to_string())?;

    let template = sentencia.query_row([], |fila| fila.get::<usize, String>(0));
    match template {
        Ok(valor) => Ok(valor),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(format!("template_fake_dp4500_{}", Utc::now().timestamp_millis())),
        Err(e) => Err(e.to_string()),
    }
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
            listar_clientes_membresia,
            buscar_cliente_por_huella,
            renovar_membresia,
            obtener_metricas,
            capturar_huella,
            guardar_template,
            comparar_huella
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
