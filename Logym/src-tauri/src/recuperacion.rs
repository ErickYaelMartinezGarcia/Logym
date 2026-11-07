//! Mantenimiento offline. Respaldos portables cifrados, nunca copias del archivo WAL activo.
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

pub(crate) fn sufijo() -> Result<String,String> {
    let mut bytes = [0u8;16];
    getrandom::getrandom(&mut bytes).map_err(|e|e.to_string())?;
    Ok(bytes.iter().map(|b|format!("{b:02x}")).collect())
}

pub(crate) fn comprobar(conn: &Connection) -> Result<(),String> {
    let check: String = conn.query_row("PRAGMA integrity_check",[],|r|r.get(0)).map_err(|e|format!("No se pudo verificar la base: {e}"))?;
    if check != "ok" { return Err("La base no supera integrity_check".into()); }
    if conn.prepare("PRAGMA cipher_integrity_check").map_err(|e|e.to_string())?.exists([]).map_err(|e|e.to_string())? {
        return Err("La base no supera cipher_integrity_check".into());
    }
    if conn.prepare("PRAGMA foreign_key_check").map_err(|e|e.to_string())?.exists([]).map_err(|e|e.to_string())? {
        return Err("La base contiene referencias huérfanas; no se modificó el destino".into());
    }
    Ok(())
}

fn abrir_clave(path: &Path, clave: &str) -> Result<Connection,String> {
    // SQLCipher hereda los flags del main al ATTACH de exportación. Se necesita
    // READ_WRITE (sin CREATE), aunque no se escriba SQL sobre el respaldo main.
    // Copiar un respaldo finalizado desde medios de solo lectura a una carpeta
    // local antes de restaurar. Nunca se copia aquí una base activa.
    let conn = Connection::open_with_flags(path,OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(|e|e.to_string())?;
    conn.pragma_update(None,"key",clave).map_err(|e|e.to_string())?;
    comprobar(&conn)?;
    Ok(conn)
}

pub(crate) fn exportar(conn: &Connection, destino: &Path, clave: &str) -> Result<(),String> {
    // Reservar sin sobrescribir: un fallo nunca elimina un respaldo anterior.
    let archivo = std::fs::OpenOptions::new().write(true).create_new(true).open(destino).map_err(|e|e.to_string())?;
    drop(archivo);
    let result = (|| {
        conn.execute("ATTACH DATABASE ?1 AS respaldo KEY ?2",rusqlite::params![destino.to_str().ok_or("Ruta no Unicode")?,clave]).map_err(|e|e.to_string())?;
        let result = (|| {
            let version: i64 = conn.query_row("PRAGMA user_version",[],|r|r.get(0)).map_err(|e|e.to_string())?;
            let tx = conn.unchecked_transaction().map_err(|e|e.to_string())?;
            tx.query_row("SELECT sqlcipher_export('respaldo')",[],|_|Ok(())).map_err(|e|e.to_string())?;
            tx.execute_batch(&format!("PRAGMA respaldo.user_version={version}")).map_err(|e|e.to_string())?;
            tx.commit().map_err(|e|e.to_string())
        })();
        let detach = conn.execute_batch("DETACH DATABASE respaldo").map_err(|e|e.to_string());
        result?; detach?;
        drop(abrir_clave(destino,clave)?);
        std::fs::OpenOptions::new().write(true).open(destino).and_then(|f|f.sync_all()).map_err(|e|e.to_string())
    })();
    if result.is_err() { let _ = std::fs::remove_file(destino); }
    result
}

fn validar_password(password: &str) -> Result<(),String> {
    if password.len()<16 || password.len()>256 { return Err("Usa una contraseña de respaldo de 16 a 256 bytes".into()); }
    Ok(())
}

#[cfg(windows)]
pub fn respaldar(db: &Path, destino: &Path, password: &str, admin: &str) -> Result<(),String> {
    validar_password(password)?;
    if !db.is_file() { return Err("No existe la base de origen".into()); }
    let conn = crate::database::abrir_cifrada(db)?;
    crate::membresias::autenticar(&conn,admin)?;
    comprobar(&conn)?;
    exportar(&conn,destino,password)
}

#[cfg(windows)]
pub fn restaurar(origen: &Path, db: &Path, password: &str, admin: &str) -> Result<(),String> {
    validar_password(password)?;
    let backup = abrir_clave(origen,password)?;
    // Autenticar contra la copia permite recuperar en otro equipo/perfil.
    crate::membresias::autenticar(&backup,admin)?;
    let version: i64 = backup.query_row("PRAGMA user_version",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if version>1 { return Err("Respaldo de una versión más nueva; no se modificó el destino".into()); }
    if db.exists() {
        let actual = crate::database::abrir_cifrada(db)?;
        crate::membresias::autenticar(&actual,admin)?;
        comprobar(&actual)?;
        // Snapshot de reversión con su misma clave local, independiente del WAL.
        let previo = db.with_extension(format!("pre-restauracion-{}.sqlcipher",sufijo()?));
        exportar(&actual,&previo,&format!("x'{}'",crate::database::clave_db(db)?))?;
        let (busy,_,_): (i64,i64,i64) = actual.query_row("PRAGMA wal_checkpoint(TRUNCATE)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
        if busy!=0 { return Err("La base está en uso; cierra Logym antes de restaurar".into()); }
        actual.close().map_err(|(_,e)|e.to_string())?;
    }
    let clave = crate::database::clave_db(db)?;
    let temporal = db.with_extension(format!("restaurando-{}.sqlcipher",sufijo()?));
    exportar(&backup,&temporal,&format!("x'{clave}'"))?;
    {
        let restaurada = crate::database::abrir_con_clave_para_mantenimiento(&temporal,&clave)?;
        crate::database::inicializar_esquema(&restaurada)?;
        comprobar(&restaurada)?;
        if restaurada.prepare("PRAGMA foreign_key_check").map_err(|e|e.to_string())?.exists([]).map_err(|e|e.to_string())? {
            return Err("Respaldo con referencias huérfanas; destino intacto".into());
        }
    }
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW,MOVEFILE_REPLACE_EXISTING,MOVEFILE_WRITE_THROUGH};
    let src: Vec<_> = temporal.as_os_str().encode_wide().chain(Some(0)).collect();
    let dst: Vec<_> = db.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { MoveFileExW(src.as_ptr(),dst.as_ptr(),MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH) }==0 {
        return Err(format!("No se reemplazó la base; copia validada retenida: {}",std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(windows)]
pub fn importar_legacy(origen: &Path, destino: &Path, admin: &str) -> Result<(),String> {
    if destino.exists() { return Err("La importación solo permite un destino nuevo; conserva la base original".into()); }
    // Abrir sin CREATE evita generar una base vacía si la ruta es equivocada.
    let source = Connection::open_with_flags(origen,OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(|e|e.to_string())?;
    comprobar(&source)?;
    let clientes: i64 = source.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='clientes'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if clientes!=1 { return Err("No es una base legacy de Logym".into()); }
    let clave = crate::database::clave_db(destino)?;
    let temporal = destino.with_extension(format!("importando-{}.sqlcipher",sufijo()?));
    exportar(&source,&temporal,&format!("x'{clave}'"))?;
    {
        let conn = crate::database::abrir_con_clave_para_mantenimiento(&temporal,&clave)?;
        crate::database::inicializar_esquema(&conn)?;
        if crate::membresias::tiene_administrador(&conn)? { crate::membresias::autenticar(&conn,admin)?; }
        else { crate::membresias::configurar_administrador(&conn,admin)?; }
        comprobar(&conn)?;
    }
    std::fs::rename(temporal,destino).map_err(|e|e.to_string())
}

/// Exclusión por proceso, antes de iniciar Tauri o ejecutar mantenimiento.
#[cfg(windows)]
pub struct Instancia(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Instancia {
    pub fn adquirir() -> Result<Self,String> {
        use windows_sys::Win32::{Foundation::{GetLastError,ERROR_ALREADY_EXISTS},System::Threading::CreateMutexW};
        let nombre: Vec<_> = "Local\\VirtusGym.Logym.Production\0".encode_utf16().collect();
        let handle = unsafe { CreateMutexW(std::ptr::null(),0,nombre.as_ptr()) };
        if handle==0 { return Err(std::io::Error::last_os_error().to_string()); }
        if unsafe { GetLastError() }==ERROR_ALREADY_EXISTS {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle); }
            return Err("Logym ya está abierto; cierra la aplicación antes de continuar".into());
        }
        Ok(Self(handle))
    }
}
#[cfg(windows)]
impl Drop for Instancia { fn drop(&mut self) { unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0); } } }

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Solicitud {
    pub operacion: String,
    pub base: std::path::PathBuf,
    pub respaldo: std::path::PathBuf,
    pub password_respaldo: String,
    pub password_admin: String,
}

#[cfg(windows)]
pub fn ejecutar() -> Result<(),String> {
    use std::io::Read;
    let mut entrada = String::new();
    std::io::stdin().take(16385).read_to_string(&mut entrada).map_err(|e|e.to_string())?;
    if entrada.len()>16384 { return Err("Solicitud demasiado grande".into()); }
    let request: Solicitud = serde_json::from_str(&entrada).map_err(|_| "Solicitud de mantenimiento inválida")?;
    if !request.base.is_absolute() || !request.respaldo.is_absolute() { return Err("Se requieren rutas absolutas".into()); }
    match request.operacion.as_str() {
        "respaldar" => respaldar(&request.base,&request.respaldo,&request.password_respaldo,&request.password_admin),
        "restaurar" => restaurar(&request.respaldo,&request.base,&request.password_respaldo,&request.password_admin),
        "importar_legacy" => importar_legacy(&request.respaldo,&request.base,&request.password_admin),
        _ => Err("Operación no permitida".into()),
    }
}

#[cfg(test)]
#[path = "tests/recuperacion.rs"]
mod tests;
