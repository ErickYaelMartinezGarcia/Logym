use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;

fn ruta_db(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directorio_datos = app
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| "No se pudo resolver la carpeta de datos".to_string())?;

    fs::create_dir_all(&directorio_datos)
        .map_err(|e| format!("No se pudo crear {}: {}", directorio_datos.display(), e))?;

    let ruta_db = directorio_datos.join("gym.db");
    println!("[DB] Ruta: {}", ruta_db.display());

    if !ruta_db.exists() {
        let legacy = PathBuf::from("gym.db");
        if legacy.exists() {
            println!("[DB] Migrando legacy");
            let _ = fs::copy(&legacy, &ruta_db);
        }
    }
    Ok(ruta_db)
}

pub fn abrir_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let ruta = ruta_db(app)?;
     println!(">>> ABRIENDO DB EN: {:?}", ruta);
    let conn = Connection::open(&ruta)
        .map_err(|e| format!("No se pudo abrir DB en {}: {}", ruta.display(), e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

fn asegurar_columna(conexion: &Connection, col: &str, def: &str) -> Result<(), String> {
    let mut stmt = conexion.prepare("PRAGMA table_info(clientes)").map_err(|e| e.to_string())?;
    let existe = stmt.query_map([], |r| r.get::<usize, String>(1)).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).any(|n| n == col);
    if !existe {
        conexion.execute(&format!("ALTER TABLE clientes ADD COLUMN {} {}", col, def), []).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn inicializar_esquema(conexion: &Connection) -> Result<(), String> {
    conexion.execute("CREATE TABLE IF NOT EXISTS clientes (id INTEGER PRIMARY KEY AUTOINCREMENT, nombre TEXT NOT NULL, telefono TEXT, telefono_emergencia TEXT, foto_base64 TEXT, template_huella TEXT, fecha_registro TEXT DEFAULT CURRENT_TIMESTAMP, fecha_inicio_membresia TEXT DEFAULT (date('now')), fecha_fin_membresia TEXT DEFAULT (date('now', '+30 day')))", []).map_err(|e| e.to_string())?;
    asegurar_columna(conexion, "fecha_inicio_membresia", "TEXT DEFAULT (date('now'))")?;
    asegurar_columna(conexion, "fecha_fin_membresia", "TEXT DEFAULT (date('now', '+30 day'))")?;
    asegurar_columna(conexion, "telefono_emergencia", "TEXT")?;
    conexion.execute("CREATE TABLE IF NOT EXISTS asistencias (id INTEGER PRIMARY KEY AUTOINCREMENT, cliente_id INTEGER NOT NULL, fecha_hora TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE)", []).map_err(|e| e.to_string())?;
    conexion.execute("CREATE INDEX IF NOT EXISTS idx_asistencias_cliente_fecha ON asistencias(cliente_id, fecha_hora)", []).map_err(|e| e.to_string())?;
    conexion.execute("CREATE TABLE IF NOT EXISTS membresias_historial (id INTEGER PRIMARY KEY AUTOINCREMENT, cliente_id INTEGER NOT NULL, fecha_inicio TEXT NOT NULL, fecha_fin TEXT NOT NULL, fecha_pago TEXT NOT NULL, tipo TEXT DEFAULT 'renovacion', observaciones TEXT, creado_en TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE)", []).map_err(|e| e.to_string())?;
    Ok(())
}