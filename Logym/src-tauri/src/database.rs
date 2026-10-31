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

    // No copiar automáticamente SQLite legacy: podría omitir datos WAL.
    Ok(ruta_db)
}

// La clave SQLCipher nunca se expone a Vue ni se almacena en texto plano.
#[cfg(windows)]
fn dpapi(datos: &[u8], proteger: bool) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, DATA_BLOB, CRYPTPROTECT_UI_FORBIDDEN};
    use windows_sys::Win32::System::Memory::LocalFree;
    let mut entrada = DATA_BLOB { cbData: datos.len() as u32, pbData: datos.as_ptr() as *mut u8 };
    let mut salida = DATA_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    let ok = unsafe {
        if proteger {
            CryptProtectData(&mut entrada, std::ptr::null(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut salida)
        } else {
            CryptUnprotectData(&mut entrada, std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut salida)
        }
    };
    if ok == 0 { return Err(format!("Windows DPAPI: {}", std::io::Error::last_os_error())); }
    let resultado = unsafe { std::slice::from_raw_parts(salida.pbData, salida.cbData as usize).to_vec() };
    unsafe { LocalFree(salida.pbData as isize); }
    Ok(resultado)
}

#[cfg(windows)]
fn clave_db(ruta: &std::path::Path) -> Result<String, String> {
    let ruta_clave = ruta.with_extension("key.dpapi");
    let clave = if ruta_clave.exists() {
        let protegida = fs::read(&ruta_clave).map_err(|e| e.to_string())?;
        let bytes = dpapi(&protegida, false)?;
        if bytes.len() != 32 { return Err("Clave cifrada inválida".into()); }
        bytes
    } else {
        // No regenerar la clave de una base existente: eso impediría recuperar los datos.
        let mut nueva = vec![0u8; 32];
        getrandom::getrandom(&mut nueva).map_err(|e| e.to_string())?;
        let protegida = dpapi(&nueva, true)?;
        let temporal = ruta_clave.with_extension("key.tmp");
        fs::write(&temporal, protegida).map_err(|e| e.to_string())?;
        fs::rename(&temporal, &ruta_clave).map_err(|e| e.to_string())?;
        nueva
    };
    Ok(clave.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(windows)]
fn abrir_cifrada(ruta: &std::path::Path) -> Result<Connection, String> {
    let ruta_clave = ruta.with_extension("key.dpapi");
    if ruta.exists() && !ruta_clave.exists() {
        return Err("Se detectó una base de datos anterior sin cifrar. No se modificó: se requiere migración segura antes de abrirla.".into());
    }
    let clave = clave_db(ruta)?;
    let conn = Connection::open(ruta).map_err(|e| e.to_string())?;
    conn.execute_batch(&format!("PRAGMA key = \"x'{clave}'\";")).map_err(|e| e.to_string())?;
    // Detecta una clave incorrecta antes de intentar migraciones o escrituras.
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))
        .map_err(|e| format!("No se pudo descifrar la base: {e}"))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

pub fn abrir_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let ruta = ruta_db(app)?;
    #[cfg(windows)]
    { abrir_cifrada(&ruta) }
    #[cfg(not(windows))]
    {
        let _ = ruta;
        Err("El almacenamiento cifrado de Logym requiere Windows DPAPI".into())
    }
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
    // Migración no destructiva: todos los clientes existentes permanecen activos.
    asegurar_columna(conexion, "activo", "INTEGER NOT NULL DEFAULT 1")?;
    conexion.execute("CREATE TABLE IF NOT EXISTS asistencias (id INTEGER PRIMARY KEY AUTOINCREMENT, cliente_id INTEGER NOT NULL, fecha_hora TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE)", []).map_err(|e| e.to_string())?;
    conexion.execute("CREATE INDEX IF NOT EXISTS idx_asistencias_cliente_fecha ON asistencias(cliente_id, fecha_hora)", []).map_err(|e| e.to_string())?;
    conexion.execute("CREATE TABLE IF NOT EXISTS membresias_historial (id INTEGER PRIMARY KEY AUTOINCREMENT, cliente_id INTEGER NOT NULL, fecha_inicio TEXT NOT NULL, fecha_fin TEXT NOT NULL, fecha_pago TEXT NOT NULL, tipo TEXT DEFAULT 'renovacion', observaciones TEXT, creado_en TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE)", []).map_err(|e| e.to_string())?;
    Ok(())
}