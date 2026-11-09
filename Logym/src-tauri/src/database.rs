use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
static APERTURA: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
#[path = "tests/database.rs"]
mod pruebas_automatizadas;

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
    unsafe { LocalFree(salida.pbData as *mut std::ffi::c_void); }
    Ok(resultado)
}

#[cfg(windows)]
pub(crate) fn clave_db(ruta: &std::path::Path) -> Result<String, String> {
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
        use std::io::Write;
        let mut archivo = fs::OpenOptions::new().write(true).create_new(true).open(&temporal)
            .map_err(|e| format!("No se pudo reservar la clave temporal; no se modificó la base: {e}"))?;
        archivo.write_all(&protegida).and_then(|_| archivo.sync_all()).map_err(|e| e.to_string())?;
        drop(archivo);
        fs::rename(&temporal, &ruta_clave).map_err(|e| e.to_string())?;
        nueva
    };
    Ok(clave.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(windows)]
pub(crate) fn abrir_cifrada(ruta: &std::path::Path) -> Result<Connection, String> {
    let _guard = APERTURA.lock().map_err(|_| "No se pudo serializar la apertura de la base")?;
    let ruta_clave = ruta.with_extension("key.dpapi");
    if ruta.exists() && !ruta_clave.exists() {
        return Err("Se detectó una base de datos anterior sin cifrar. No se modificó: se requiere migración segura antes de abrirla.".into());
    }
    let clave = clave_db(ruta)?;
    let conn = Connection::open(ruta).map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(10)).map_err(|e| e.to_string())?;
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

#[cfg(windows)]
pub(crate) fn abrir_con_clave_para_mantenimiento(ruta: &std::path::Path, clave: &str) -> Result<Connection,String> {
    let conn = Connection::open(ruta).map_err(|e|e.to_string())?;
    conn.execute_batch(&format!("PRAGMA key=\"x'{clave}'\"; PRAGMA foreign_keys=ON;")).map_err(|e|e.to_string())?;
    crate::recuperacion::comprobar(&conn)?;
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

pub const VERSION_ESQUEMA: i64 = 3;

pub fn inicializar_esquema(conexion: &Connection) -> Result<(), String> {
    let version: i64 = conexion.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|e|e.to_string())?;
    if version > VERSION_ESQUEMA { return Err("La base pertenece a una versión más nueva de Logym; no se modificó".into()); }
    // Conservar un snapshot cifrado previo a migrar instalaciones existentes.
    #[cfg(windows)]
    if version < VERSION_ESQUEMA {
        if let Some(ruta) = conexion.path().map(std::path::Path::new).filter(|p| p.with_extension("key.dpapi").is_file()) {
            let tablas: i64 = conexion.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r|r.get(0)).map_err(|e|e.to_string())?;
            if tablas > 0 {
                let copia = ruta.with_extension(format!("pre-migracion-{}.sqlcipher", crate::recuperacion::sufijo()?));
                crate::recuperacion::exportar(conexion, &copia, &format!("x'{}'", clave_db(ruta)?))?;
            }
        }
    }
    let tx = conexion.unchecked_transaction().map_err(|e|e.to_string())?;
    migrar_esquema(&tx)?;
    if tx.prepare("PRAGMA foreign_key_check").map_err(|e|e.to_string())?.exists([]).map_err(|e|e.to_string())? {
        return Err("Hay referencias huérfanas; la migración se revirtió sin borrar datos".into());
    }
    tx.pragma_update(None, "user_version", VERSION_ESQUEMA).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}

fn migrar_esquema(conexion: &Connection) -> Result<(), String> {
    conexion.execute("CREATE TABLE IF NOT EXISTS clientes (id INTEGER PRIMARY KEY AUTOINCREMENT, nombre TEXT NOT NULL, telefono TEXT, telefono_emergencia TEXT, foto_base64 TEXT, template_huella TEXT, fecha_registro TEXT DEFAULT CURRENT_TIMESTAMP, fecha_inicio_membresia TEXT DEFAULT (date('now')), fecha_fin_membresia TEXT DEFAULT (date('now', '+30 day')))", []).map_err(|e| e.to_string())?;
    // No inventar vigencias históricas: NULL significa fecha no configurada.
    asegurar_columna(conexion, "fecha_inicio_membresia", "TEXT")?;
    asegurar_columna(conexion, "fecha_fin_membresia", "TEXT")?;
    for columna in ["telefono", "foto_base64", "template_huella", "fecha_registro"] {
        asegurar_columna(conexion, columna, "TEXT")?;
    }
    asegurar_columna(conexion, "telefono_emergencia", "TEXT")?;
    // Migración no destructiva: todos los clientes existentes permanecen activos.
    asegurar_columna(conexion, "activo", "INTEGER NOT NULL DEFAULT 1")?;
    conexion.execute("CREATE TABLE IF NOT EXISTS asistencias (id INTEGER PRIMARY KEY AUTOINCREMENT, cliente_id INTEGER NOT NULL, fecha_hora TEXT DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE)", []).map_err(|e| e.to_string())?;
    migrar_asistencias(conexion)?;
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
    let tiene_dias = conexion.prepare("PRAGMA table_info(tipos_membresia)").map_err(|e|e.to_string())?
        .query_map([], |r| r.get::<_,String>(1)).map_err(|e|e.to_string())?
        .collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?.iter().any(|n| n == "duracion_dias");
    if !tiene_dias {
        conexion.execute("ALTER TABLE tipos_membresia ADD COLUMN duracion_dias INTEGER CHECK(duracion_dias BETWEEN 1 AND 3660)", [])
            .map_err(|e|e.to_string())?;
    }
    // Solo los nuevos cobros reciben la duración contratada. No inferirla del
    // catálogo actual para pagos históricos cuyo plan pudo haber cambiado.
    for (columna, definicion) in [("duracion_dias", "INTEGER"), ("duracion_meses", "INTEGER")] {
        let existe = conexion.prepare("PRAGMA table_info(pagos)").map_err(|e|e.to_string())?
            .query_map([], |r| r.get::<_,String>(1)).map_err(|e|e.to_string())?
            .collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?.iter().any(|n| n == columna);
        if !existe { conexion.execute(&format!("ALTER TABLE pagos ADD COLUMN {columna} {definicion}"), []).map_err(|e|e.to_string())?; }
    }
    conexion.execute_batch("CREATE TABLE IF NOT EXISTS renovaciones_operaciones (
        id TEXT PRIMARY KEY NOT NULL,
        cliente_id INTEGER NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
        plan_id INTEGER NOT NULL REFERENCES tipos_membresia(id),
        fecha_fin TEXT NOT NULL
    );").map_err(|e|e.to_string())?;
    // Entradas ocasionales independientes: no inventar clientes ni alterar sus FK.
    conexion.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS visitas_ocasionales (
            id TEXT PRIMARY KEY NOT NULL,
            nombre TEXT,
            fecha_hora TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS pagos_visitas (
            visita_id TEXT PRIMARY KEY NOT NULL REFERENCES visitas_ocasionales(id) ON DELETE RESTRICT,
            total_centavos INTEGER NOT NULL CHECK(total_centavos=5000),
            fecha_pago TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS asistencias_visitas (
            visita_id TEXT PRIMARY KEY NOT NULL REFERENCES visitas_ocasionales(id) ON DELETE RESTRICT,
            fecha_hora TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_pagos_visitas_fecha ON pagos_visitas(fecha_pago);
        CREATE INDEX IF NOT EXISTS idx_asistencias_visitas_fecha ON asistencias_visitas(fecha_hora);
        CREATE VIEW IF NOT EXISTS ingresos_todos AS
            SELECT fecha_pago,total_centavos,membresia_centavos,inscripcion_centavos,0 AS visitas_centavos FROM pagos
            UNION ALL
            SELECT fecha_pago,total_centavos,0,0,total_centavos FROM pagos_visitas;
        CREATE VIEW IF NOT EXISTS asistencias_todas AS
            SELECT fecha_hora FROM asistencias UNION ALL SELECT fecha_hora FROM asistencias_visitas;
    "#).map_err(|e|e.to_string())?;
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

fn migrar_asistencias(conn: &Connection) -> Result<(), String> {
    let tiene_fk = conn.prepare("PRAGMA foreign_key_list(asistencias)").map_err(|e|e.to_string())?
        .query_map([], |r| Ok((r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(6)?)))
        .map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?
        .iter().any(|(tabla,columna,borrado)| tabla=="clientes" && columna=="cliente_id" && borrado=="CASCADE");
    if tiene_fk { return Ok(()); }
    let columnas = conn.prepare("PRAGMA table_info(asistencias)").map_err(|e|e.to_string())?
        .query_map([],|r|r.get::<_,String>(1)).map_err(|e|e.to_string())?
        .collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    if columnas != ["id", "cliente_id", "fecha_hora"] {
        return Err("Esquema legacy de asistencias no soportado; se conserva intacto para migración asistida".into());
    }
    let huerfanos: i64 = conn.query_row("SELECT count(*) FROM asistencias a LEFT JOIN clientes c ON c.id=a.cliente_id WHERE c.id IS NULL",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if huerfanos != 0 { return Err("Asistencias legacy huérfanas; se conserva todo sin eliminar registros".into()); }
    let tablas = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name<>'asistencias'").map_err(|e|e.to_string())?
        .query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    for tabla in tablas {
        let quoted = tabla.replace('"',"\"\"");
        let referencias = conn.prepare(&format!("PRAGMA foreign_key_list(\"{quoted}\")")).map_err(|e|e.to_string())?
            .query_map([],|r|r.get::<_,String>(2)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        if referencias.iter().any(|t|t=="asistencias") { return Err("Una tabla legacy depende de asistencias; se requiere migración asistida".into()); }
    }
    let vistas: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE type='view' AND lower(sql) LIKE '%asistencias%'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if vistas>0 { return Err("Una vista legacy depende de asistencias; se requiere migración asistida".into()); }
    let sql: String = conn.query_row("SELECT sql FROM sqlite_master WHERE name='asistencias' AND type='table'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if ["CHECK", "UNIQUE", "REFERENCES"].iter().any(|s|sql.to_ascii_uppercase().contains(s)) {
        return Err("Restricciones legacy de asistencias requieren migración asistida; no se modificaron".into());
    }
    let objetos = conn.prepare("SELECT sql FROM sqlite_master WHERE tbl_name='asistencias' AND type IN ('index','trigger') AND sql IS NOT NULL").map_err(|e|e.to_string())?
        .query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    conn.execute_batch("CREATE TABLE asistencias_migracion (id INTEGER PRIMARY KEY AUTOINCREMENT,cliente_id INTEGER NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,fecha_hora TEXT DEFAULT CURRENT_TIMESTAMP);
        INSERT INTO asistencias_migracion(id,cliente_id,fecha_hora) SELECT id,cliente_id,fecha_hora FROM asistencias;
        DROP TABLE asistencias; ALTER TABLE asistencias_migracion RENAME TO asistencias;").map_err(|e|e.to_string())?;
    for objeto in objetos { conn.execute_batch(&objeto).map_err(|e|e.to_string())?; }
    Ok(())
}
#[cfg(test)]
#[path = "tests/database_migrations.rs"]
mod pruebas_migracion;
