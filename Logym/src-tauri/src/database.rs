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
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN};
    use windows_sys::Win32::Foundation::LocalFree;
    let mut entrada = CRYPT_INTEGER_BLOB { cbData: datos.len() as u32, pbData: datos.as_ptr() as *mut u8 };
    let mut salida = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
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
    // Precios en centavos: nunca recalcular pagos históricos desde el catálogo.
    conexion.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS tipos_membresia (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL UNIQUE,
            duracion_meses INTEGER NOT NULL CHECK(duracion_meses > 0),
            precio_centavos INTEGER NOT NULL CHECK(precio_centavos >= 0),
            activo INTEGER NOT NULL DEFAULT 1 CHECK(activo IN (0,1)),
            solo_alta INTEGER NOT NULL DEFAULT 0 CHECK(solo_alta IN (0,1))
        );
        CREATE TABLE IF NOT EXISTS configuracion_cobros (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            inscripcion_centavos INTEGER NOT NULL DEFAULT 0 CHECK(inscripcion_centavos >= 0)
        );
        INSERT OR IGNORE INTO configuracion_cobros(id, inscripcion_centavos) VALUES(1, 15000);
        CREATE TABLE IF NOT EXISTS pagos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cliente_id INTEGER NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
            membresia_id INTEGER REFERENCES tipos_membresia(id) ON DELETE RESTRICT,
            concepto_membresia TEXT NOT NULL,
            membresia_centavos INTEGER NOT NULL CHECK(membresia_centavos >= 0),
            inscripcion_centavos INTEGER NOT NULL DEFAULT 0 CHECK(inscripcion_centavos >= 0),
            total_centavos INTEGER NOT NULL CHECK(total_centavos = membresia_centavos + inscripcion_centavos),
            fecha_pago TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            tipo TEXT NOT NULL CHECK(tipo IN ('alta','renovacion'))
        );
        CREATE INDEX IF NOT EXISTS idx_pagos_fecha ON pagos(fecha_pago);
        CREATE INDEX IF NOT EXISTS idx_pagos_cliente ON pagos(cliente_id);
    "#).map_err(|e| e.to_string())?;
    // Agregar la restricción de alta a instalaciones que ya tenían el catálogo.
    let tiene_solo_alta = {
        let mut stmt = conexion.prepare("PRAGMA table_info(tipos_membresia)").map_err(|e|e.to_string())?;
        let columnas = stmt.query_map([],|r|r.get::<_,String>(1)).map_err(|e|e.to_string())?;
        let existe = columnas.filter_map(Result::ok).any(|n|n=="solo_alta");
        existe
    };
    if !tiene_solo_alta {
        conexion.execute("ALTER TABLE tipos_membresia ADD COLUMN solo_alta INTEGER NOT NULL DEFAULT 0",[])
            .map_err(|e|e.to_string())?;
    }
    conexion.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS administrador (
            id INTEGER PRIMARY KEY CHECK(id=1),
            password_hash TEXT NOT NULL
        );
        INSERT INTO tipos_membresia(nombre,duracion_meses,precio_centavos)
        SELECT column1,column2,column3 FROM (VALUES
            ('Mensual',1,35000),
            ('Pareja',1,30000),
            ('Estudiante',1,28000),
            ('Bimestral',2,65000),
            ('Semestral',6,180000)) WHERE NOT EXISTS(SELECT 1 FROM tipos_membresia);
        UPDATE tipos_membresia SET solo_alta=1 WHERE nombre='Pareja';
    "#).map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod pruebas_migracion {
    use super::*;
    #[test]
    fn migracion_conserva_clientes_y_asistencias() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT,telefono TEXT,fecha_registro TEXT,fecha_inicio_membresia TEXT,fecha_fin_membresia TEXT);
            CREATE TABLE asistencias(id INTEGER PRIMARY KEY,cliente_id INTEGER,fecha_hora TEXT);
            INSERT INTO clientes(id,nombre) VALUES(7,'Cliente antiguo');
            INSERT INTO asistencias(id,cliente_id) VALUES(9,7);
        ").unwrap();
        inicializar_esquema(&conn).unwrap();
        inicializar_esquema(&conn).unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM clientes WHERE id=7",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM asistencias WHERE id=9",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM tipos_membresia",[],|r|r.get::<_,i64>(0)).unwrap(),5);
    }
    #[test]
    fn inscripcion_gratis_no_se_restaurara_al_reiniciar() {
        let conn = Connection::open_in_memory().unwrap();
        inicializar_esquema(&conn).unwrap();
        conn.execute("UPDATE configuracion_cobros SET inscripcion_centavos=0",[]).unwrap();
        inicializar_esquema(&conn).unwrap();
        assert_eq!(conn.query_row("SELECT inscripcion_centavos FROM configuracion_cobros WHERE id=1",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    }
}
