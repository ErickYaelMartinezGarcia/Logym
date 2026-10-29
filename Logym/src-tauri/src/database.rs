use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;

const CLIENTE_DEMO_TEMPLATE: &str = "demo-secugen-hamster-20-pro";
const CLIENTE_DEMO_FOTO: &str = "data:image/svg+xml;charset=UTF-8,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 240 240'%3E%3Crect width='240' height='240' fill='%231a2028'/%3E%3Ccircle cx='120' cy='92' r='48' fill='%23ee2046'/%3E%3Cpath d='M42 220c8-48 38-74 78-74s70 26 78 74' fill='%23ee2046'/%3E%3Ctext x='120' y='106' text-anchor='middle' font-family='Arial' font-size='42' font-weight='700' fill='white'%3EV%3C/text%3E%3C/svg%3E";

fn ruta_db(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directorio_datos = app
        .path_resolver()
        .app_local_data_dir()
        .ok_or_else(|| String::from("No se pudo resolver la carpeta de datos de la aplicación"))?;

    fs::create_dir_all(&directorio_datos)
        .map_err(|e| format!("No se pudo crear la carpeta de datos: {}", e))?;

    let ruta_db = directorio_datos.join("gym.db");
    if ruta_db.exists() {
        return Ok(ruta_db);
    }

    let ruta_legacy = PathBuf::from("gym.db");
    if ruta_legacy.exists() {
        fs::copy(&ruta_legacy, &ruta_db)
            .map_err(|e| format!("No se pudo migrar la base de datos existente: {}", e))?;
    }

    Ok(ruta_db)
}

pub fn abrir_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let ruta = ruta_db(app)?;
    Connection::open(&ruta).map_err(|e| {
        format!(
            "No se pudo abrir la base de datos en {}: {}",
            ruta.display(),
            e
        )
    })
}

fn asegurar_columna_clientes(
    conexion: &Connection,
    nombre_columna: &str,
    definicion: &str,
) -> Result<(), String> {
    let mut sentencia = conexion
        .prepare("PRAGMA table_info(clientes)")
        .map_err(|e| e.to_string())?;
    let columnas = sentencia
        .query_map([], |fila| fila.get::<usize, String>(1))
        .map_err(|e| e.to_string())?;

    let existe = columnas
        .map(|columna| columna.map(|nombre| nombre == nombre_columna))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .any(|coincide| coincide);

    if !existe {
        conexion
            .execute(
                &format!(
                    "ALTER TABLE clientes ADD COLUMN {} {}",
                    nombre_columna, definicion
                ),
                [],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn inicializar_esquema(conexion: &Connection) -> Result<(), String> {
    conexion
        .execute(
            "CREATE TABLE IF NOT EXISTS clientes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nombre TEXT NOT NULL,
                telefono TEXT,
                telefono_emergencia TEXT,
                foto_base64 TEXT,
                template_huella TEXT,
                fecha_registro TEXT DEFAULT CURRENT_TIMESTAMP,
                fecha_inicio_membresia TEXT DEFAULT (date('now')),
                fecha_fin_membresia TEXT DEFAULT (date('now', '+30 day'))
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

    asegurar_columna_clientes(
        conexion,
        "fecha_inicio_membresia",
        "TEXT DEFAULT (date('now'))",
    )?;
    asegurar_columna_clientes(
        conexion,
        "fecha_fin_membresia",
        "TEXT DEFAULT (date('now', '+30 day'))",
    )?;
    asegurar_columna_clientes(conexion, "telefono_emergencia", "TEXT")?;

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
            "CREATE INDEX IF NOT EXISTS idx_asistencias_cliente_fecha
             ON asistencias(cliente_id, fecha_hora)",
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
                tipo TEXT DEFAULT 'renovacion',
                observaciones TEXT,
                creado_en TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (cliente_id) REFERENCES clientes(id)
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

    sembrar_cliente_demo(conexion)?;

    Ok(())
}

fn sembrar_cliente_demo(conexion: &Connection) -> Result<(), String> {
    let existe: i64 = conexion
        .query_row(
            "SELECT COUNT(*) FROM clientes WHERE template_huella = ?1",
            [CLIENTE_DEMO_TEMPLATE],
            |fila| fila.get(0),
        )
        .map_err(|e| e.to_string())?;

    if existe > 0 {
        return Ok(());
    }

    conexion
        .execute(
            "INSERT INTO clientes
             (nombre, telefono, foto_base64, template_huella, fecha_registro,
              fecha_inicio_membresia, fecha_fin_membresia)
             VALUES ('Alex Rivera', '+52 55 0000 0000', ?1, ?2, date('now'),
                     date('now'), date('now', '+30 day'))",
            [CLIENTE_DEMO_FOTO, CLIENTE_DEMO_TEMPLATE],
        )
        .map_err(|e| e.to_string())?;

    let cliente_id = conexion.last_insert_rowid();
    conexion
        .execute(
            "INSERT INTO membresias_historial
             (cliente_id, fecha_inicio, fecha_fin, fecha_pago, tipo, observaciones)
             VALUES (?1, date('now'), date('now', '+30 day'), date('now'),
                     'demo', 'Cliente de demostración local')",
            [cliente_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
