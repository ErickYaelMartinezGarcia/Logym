use ed25519_dalek::{Signature, VerifyingKey};
use std::fs;

#[path = "hardware.rs"]
mod hardware;

// Solo clave pública Ed25519. El emisor privado vive fuera del proyecto.
const CLAVE_PUBLICA: &str = include_str!("license_public_key.hex");

#[cfg(test)]
#[path = "tests/licencia.rs"]
mod pruebas_automatizadas;

fn carpeta(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let p = app.path_resolver().app_data_dir().ok_or("No se encontró carpeta de datos")?;
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 || !s.is_ascii() { return Err("Formato hexadecimal inválido".into()); }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2], 16).map_err(|_| "Formato hexadecimal inválido".into())).collect()
}

pub fn solicitud(_: &tauri::AppHandle) -> Result<String, String> { hardware::solicitud() }

pub fn verificar(app: &tauri::AppHandle) -> Result<(), String> {
    // Solo builds de desarrollo pueden operar sin licencia.
    #[cfg(debug_assertions)]
    { let _ = app; return Ok(()); }
    #[cfg(not(debug_assertions))]
    {
        let archivo = carpeta(app)?.join("license.txt");
        let licencia = fs::read_to_string(archivo).map_err(|_| "Logym no está activado".to_string())?;
        validar_firma(&solicitud(app)?, &licencia)
    }
}

fn validar_firma(solicitud: &str, licencia: &str) -> Result<(), String> {
    let (id, firma_hex) = licencia.trim().split_once(':').ok_or("Licencia inválida")?;
    if id.len()!=74 || !id.starts_with("LOGYM2-HW-") || !id[10..].bytes().all(|b|b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err("Licencia incompatible; solicita una licencia LOGYM2-HW para esta computadora".into());
    }
    if id != solicitud { return Err("Licencia de otra computadora".into()); }
    let key: [u8; 32] = decode(CLAVE_PUBLICA.trim())?.try_into().map_err(|_| "Clave pública no configurada")?;
    let key = VerifyingKey::from_bytes(&key).map_err(|_| "Clave pública inválida")?;
    let sig: [u8; 64] = decode(firma_hex)?.try_into().map_err(|_| "Firma inválida")?;
    if key.is_weak() { return Err("Clave pública insegura o no configurada".into()); }
    key.verify_strict(id.as_bytes(), &Signature::from_bytes(&sig)).map_err(|_| "Firma de licencia inválida".into())
}

fn activar_archivo(ruta: &std::path::Path, solicitud: &str, licencia: &str) -> Result<(), String> {
    validar_firma(solicitud, licencia)?;
    let tmp = ruta.with_extension("tmp");
    fs::write(&tmp, licencia.trim()).map_err(|e| e.to_string())?;
    fs::rename(tmp, ruta).map_err(|e| e.to_string())
}

pub fn activar(app: &tauri::AppHandle, licencia: String) -> Result<(), String> {
    let solicitud = solicitud(app)?;
    activar_archivo(&carpeta(app)?.join("license.txt"), &solicitud, &licencia)
}
