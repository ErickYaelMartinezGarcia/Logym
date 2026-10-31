use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fs;
use tauri::Manager;

// Sustituir con la clave PUBLICA del emisor antes de distribuir un release.
const CLAVE_PUBLICA: &str = include_str!("license_public_key.hex");

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

#[cfg(windows)]
fn dpapi(bytes: &[u8], protect: bool) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, DATA_BLOB, CRYPTPROTECT_UI_FORBIDDEN};
    use windows_sys::Win32::System::Memory::LocalFree;
    let mut input = DATA_BLOB { cbData: bytes.len() as u32, pbData: bytes.as_ptr() as *mut u8 };
    let mut output = DATA_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    let ok = unsafe {
        if protect {
            CryptProtectData(&mut input, std::ptr::null(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut output)
        } else {
            CryptUnprotectData(&mut input, std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut output)
        }
    };
    if ok == 0 { return Err(format!("Windows no pudo proteger la identidad: {}", std::io::Error::last_os_error())); }
    let out = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe { LocalFree(output.pbData as isize); }
    Ok(out)
}

#[cfg(windows)]
pub fn solicitud(app: &tauri::AppHandle) -> Result<String, String> {
    let p = carpeta(app)?.join("installation.dpapi");
    let token = if p.exists() {
        let saved = fs::read(&p).map_err(|e| e.to_string())?;
        let raw = dpapi(&saved, false)?;
        if raw.len() != 32 { return Err("Identidad de instalación dañada".into()); }
        raw
    } else {
        let mut raw = vec![0u8; 32];
        getrandom::getrandom(&mut raw).map_err(|e| e.to_string())?;
        let protected = dpapi(&raw, true)?;
        let tmp = p.with_extension("tmp");
        fs::write(&tmp, protected).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &p).map_err(|e| e.to_string())?;
        raw
    };
    Ok(format!("LOGYM1-{}", hex(&Sha256::digest(&token))))
}
#[cfg(not(windows))]
pub fn solicitud(_: &tauri::AppHandle) -> Result<String, String> { Err("Licencia disponible solo en Windows".into()) }

pub fn verificar(app: &tauri::AppHandle) -> Result<(), String> {
    // Solo builds de desarrollo pueden operar sin licencia.
    #[cfg(debug_assertions)]
    { let _ = app; return Ok(()); }
    #[cfg(not(debug_assertions))]
    {
        let archivo = carpeta(app)?.join("license.txt");
        let licencia = fs::read_to_string(archivo).map_err(|_| "Logym no está activado".to_string())?;
        validar_texto(app, &licencia)
    }
}

pub fn validar_texto(app: &tauri::AppHandle, licencia: &str) -> Result<(), String> {
    let (id, firma_hex) = licencia.trim().split_once(':').ok_or("Licencia inválida")?;
    if id != solicitud(app)? { return Err("Licencia de otra instalación".into()); }
    let key: [u8; 32] = decode(CLAVE_PUBLICA.trim())?.try_into().map_err(|_| "Clave pública no configurada")?;
    let key = VerifyingKey::from_bytes(&key).map_err(|_| "Clave pública inválida")?;
    let sig: [u8; 64] = decode(firma_hex)?.try_into().map_err(|_| "Firma inválida")?;
    key.verify(id.as_bytes(), &Signature::from_bytes(&sig)).map_err(|_| "Firma de licencia inválida".into())
}

pub fn activar(app: &tauri::AppHandle, licencia: String) -> Result<(), String> {
    validar_texto(app, &licencia)?;
    let ruta = carpeta(app)?.join("license.txt");
    let tmp = ruta.with_extension("tmp");
    fs::write(&tmp, licencia.trim()).map_err(|e| e.to_string())?;
    fs::rename(tmp, ruta).map_err(|e| e.to_string())
}
