use libloading::Library;
use std::sync::atomic::{AtomicBool, Ordering};
use rusqlite::Connection;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::{OnceLock, Mutex};

// ==========================================
// CONSTANTES OFICIALES SECUGEN
// ==========================================
pub const SG_USB_AUTO_DETECT: u32 = 0x3BC + 1; // sgfplib.h: 957, no 955
pub const SG_TEMPLATE_FORMAT_SG400: u16 = 0x0200;
pub const SG_SECURITY_LEVEL_NORMAL: u32 = 3;
pub const SG_TEMPLATE_SIZE: usize = 400;
pub const SGFDX_ERROR_NONE: u32 = 0;

#[derive(Debug)]
pub enum SecuGenError {
    Sdk { function: &'static str, code: DWORD },
    Disconnected,
    Initialization(String),
    Other(String),
}

impl SecuGenError {
    pub fn needs_recovery(&self) -> bool {
        match self {
            Self::Disconnected | Self::Initialization(_) => true,
            Self::Sdk { function: "SGFPM_GetDeviceInfo", code } => matches!(code, 2 | 51 | 52 | 55 | 56 | 58 | 59 | 60 | 63),
            Self::Sdk { function: "SGFPM_GetImageEx", code } => {
                // Fallos de dispositivo/driver que justifican reabrir el SDK.
                // Solo 55 significa DEVICE_NOT_FOUND; no implica USB desconectado
                // para los demás códigos (p. ej. 58: ancho de banda insuficiente).
                matches!(code, 51 | 52 | 55 | 56 | 58 | 59 | 60 | 63)
            }
            _ => false,
        }
    }

    pub fn invalid_read(&self) -> bool {
        matches!(self, Self::Sdk { function: "SGFPM_CreateTemplate", code: 2 | 101 | 105 }
            | Self::Sdk { function: "SGFPM_GetImageEx", code: 53 | 57 | 62 })
    }

    pub fn persistent_fault(&self) -> bool {
        // Parámetros, licencia, funciones no soportadas y códigos desconocidos:
        // informar y espaciar intentos, sin suponer desconexión ni reabrir USB.
        matches!(self, Self::Sdk { .. }) && !self.needs_recovery()
            && !self.invalid_read()
            && !matches!(self, Self::Sdk { function: "SGFPM_GetImageEx", code: 54 })
    }
}

impl std::fmt::Display for SecuGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sdk { function: "SGFPM_Init", code: 6 } => f.write_str("SGFPM_Init: 6 (DLLLOAD_FAILED_DRV: el SDK no pudo cargar el driver del modelo enumerado; comprobar instalación, arquitectura y dependencias. El código no identifica la DLL ni la causa exacta)"),
            Self::Sdk { function, code } => write!(f, "{function}: {code}"),
            Self::Disconnected => f.write_str("Lector SecuGen no disponible"),
            Self::Initialization(message) | Self::Other(message) => f.write_str(message),
        }
    }
}

impl From<String> for SecuGenError {
    fn from(message: String) -> Self { Self::Other(message) }
}
impl From<&str> for SecuGenError {
    fn from(message: &str) -> Self { Self::Other(message.into()) }
}
impl From<SecuGenError> for String {
    fn from(error: SecuGenError) -> Self { error.to_string() }
}

// ==========================================
// TIPOS FFI
// ==========================================
pub type HSGFPM = *mut c_void;
pub type DWORD = u32;
pub type BOOL = i32;

#[repr(C)] #[derive(Clone, Copy)] pub struct DeviceList { pub dev_name: DWORD, pub dev_id: DWORD, pub dev_type: u16, pub dev_sn: [u8; 16], }
#[repr(C)] #[derive(Default, Clone, Copy)] pub struct DeviceInfo { pub device_id: DWORD, pub device_sn: [u8; 16], pub com_port: DWORD, pub com_speed: DWORD, pub image_width: DWORD, pub image_height: DWORD, pub contrast: DWORD, pub brightness: DWORD, pub gain: DWORD, pub image_dpi: DWORD, pub fw_version: DWORD, }
#[repr(C)] #[derive(Default, Clone, Copy)] pub struct FingerInfo { pub finger_number: u16, pub view_number: u16, pub impression_type: u16, pub image_quality: u16, }

struct NativeManager {
    handle: NonNull<c_void>,
    api: &'static SgfpmApi,
    info: Option<DeviceInfo>,
    needs_close: bool,
    connected: bool,
    usb_generation: u64,
    capture_verified: bool,
}

// El propietario solo se accede bajo GLOBAL_MANAGER, incluso al destruirlo.
unsafe impl Send for NativeManager {}

impl NativeManager {
    fn record_capture(&mut self, code: DWORD) {
        if code == SGFDX_ERROR_NONE { self.capture_verified = true; return; }
        let error = SecuGenError::Sdk { function: "SGFPM_GetImageEx", code };
        if error.needs_recovery() || error.persistent_fault() { self.capture_verified = false; }
    }

    fn connected_handle(&self) -> Result<HSGFPM, SecuGenError> {
        check_running()?;
        if !self.connected || self.info.is_none() {
            return Err(SecuGenError::Disconnected);
        }
        crate::lector_estado::verificar(self.usb_generation).map_err(|_| SecuGenError::Disconnected)?;
        Ok(self.handle.as_ptr())
    }

    fn refresh_info(&mut self) -> Result<(), SecuGenError> {
        if !self.needs_close { return Err(SecuGenError::Disconnected); }
        let mut info = DeviceInfo::default();
        unsafe { check_sdk((self.api.get_device_info)(self.handle.as_ptr(), &mut info), "SGFPM_GetDeviceInfo")?; }
        image_len(&info).map_err(SecuGenError::Initialization)?;
        self.info = Some(info);
        self.connected = true;
        Ok(())
    }
}

impl Drop for NativeManager {
    fn drop(&mut self) {
        unsafe {
            if self.needs_close {
                let code = (self.api.close_device)(self.handle.as_ptr());
                if code != 0 { eprintln!("[LECTOR] SGFPM_CloseDevice al liberar: {code}; se terminará el manager"); }
            }
            let code = (self.api.terminate)(self.handle.as_ptr());
            if code != 0 { eprintln!("[LECTOR] SGFPM_Terminate: {code}"); }
        }
    }
}

fn image_len(info: &DeviceInfo) -> Result<usize, String> {
    let width = usize::try_from(info.image_width).map_err(|_| "Ancho de imagen inválido")?;
    let height = usize::try_from(info.image_height).map_err(|_| "Alto de imagen inválido")?;
    if !(100..=2000).contains(&width) || !(100..=2000).contains(&height) {
        return Err("Dimensiones del lector fuera de rango seguro".into());
    }
    width.checked_mul(height).ok_or_else(|| "Desbordamiento del buffer de imagen".into())
}

type FnCreate = unsafe extern "system" fn(*mut HSGFPM) -> DWORD;
type FnTerminate = unsafe extern "system" fn(HSGFPM) -> DWORD;
type FnInit = unsafe extern "system" fn(HSGFPM, DWORD) -> DWORD;
type FnEnumerateDevice = unsafe extern "system" fn(HSGFPM, *mut DWORD, *mut *mut DeviceList) -> DWORD;
type FnSetTemplateFormat = unsafe extern "system" fn(HSGFPM, u16) -> DWORD;
type FnOpenDevice = unsafe extern "system" fn(HSGFPM, DWORD) -> DWORD;
type FnGetDeviceInfo = unsafe extern "system" fn(HSGFPM, *mut DeviceInfo) -> DWORD;
type FnGetImageEx = unsafe extern "system" fn(HSGFPM, *mut u8, DWORD, *mut c_void, DWORD) -> DWORD;
type FnCreateTemplate = unsafe extern "system" fn(HSGFPM, *mut c_void, *mut u8, *mut u8) -> DWORD;
type FnCloseDevice = unsafe extern "system" fn(HSGFPM) -> DWORD;
type FnMatchTemplate = unsafe extern "system" fn(HSGFPM, *mut u8, *mut u8, DWORD, *mut BOOL) -> DWORD;

pub struct SgfpmApi {
    pub create: FnCreate,
    pub terminate: FnTerminate,
    pub init: FnInit,
    pub enumerate_device: FnEnumerateDevice,
    pub set_template_format: FnSetTemplateFormat,
    pub open_device: FnOpenDevice,
    pub get_device_info: FnGetDeviceInfo,
    pub get_image_ex: FnGetImageEx,
    pub create_template: FnCreateTemplate,
    pub match_template: FnMatchTemplate,
    pub close_device: FnCloseDevice,
}

static STOPPING: AtomicBool = AtomicBool::new(false);
static ALGORITHM_LIB: OnceLock<Library> = OnceLock::new();
static FP_LIB: OnceLock<Library> = OnceLock::new();
static API: OnceLock<SgfpmApi> = OnceLock::new();
static GLOBAL_MANAGER: Mutex<Option<NativeManager>> = Mutex::new(None);

fn create_manager(api: &'static SgfpmApi, usb_generation: u64) -> Result<NativeManager, String> {
    unsafe {
        let mut raw = std::ptr::null_mut();
        let code = (api.create)(&mut raw);
        let mut owner = NonNull::new(raw).map(|handle| NativeManager {
            handle, api, info: None, needs_close: false, connected: false, usb_generation,
            capture_verified: false,
        });
        check_sdk(code, "SGFPM_Create")?;
        let manager = owner.as_mut().ok_or("SGFPM_Create devolvió manager nulo")?;
        let handle = manager.handle.as_ptr();
        // Guía USB SG1-0030D-001, pp. 2-4; DeviceTest/sgdvcDlg.cpp.
        // La lista pertenece al SDK: copiar la entrada, no liberarla con Rust.
        let mut count = 0;
        let mut devices = std::ptr::null_mut();
        check_sdk((api.enumerate_device)(handle, &mut count, &mut devices), "SGFPM_EnumerateDevice")?;
        if count == 0 { return Err("El SDK no enumeró lectores SecuGen".into()); }
        if devices.is_null() { return Err("SGFPM_EnumerateDevice devolvió devices nulo".into()); }
        // Un solo lector, como en la guía. No elegir arbitrariamente entre
        // dispositivos distintos y luego abrir otro con USB_AUTO_DETECT.
        if count != 1 { return Err(format!("El SDK detectó {count} lectores; conecta un solo lector para evitar una selección ambigua")); }
        let device = *devices;
        let code = (api.init)(handle, device.dev_name);
        check_sdk(code, "SGFPM_Init").map_err(|error| format!("{error}; modelo enumerado={}, proceso={} bits", device.dev_name, usize::BITS))?;
        check_sdk((api.set_template_format)(handle, SG_TEMPLATE_FORMAT_SG400), "SGFPM_SetTemplateFormat")?;
        // Patrón del ejemplo oficial Matching: Init(modelo), Open(USB_AUTO_DETECT).
        check_sdk((api.open_device)(handle, SG_USB_AUTO_DETECT), "SGFPM_OpenDevice")?;
        manager.needs_close = true;
        manager.refresh_info().map_err(String::from)?;
        owner.ok_or_else(|| "Manager nulo".into())
    }
}

fn check_running() -> Result<(), SecuGenError> {
    if stopping() { Err(SecuGenError::Other("El lector se está cerrando".into())) } else { Ok(()) }
}

pub fn stopping() -> bool { STOPPING.load(Ordering::SeqCst) }
pub fn request_shutdown() { STOPPING.store(true, Ordering::SeqCst); }

// Llamar desde un hilo de cierre, nunca desde el hilo de eventos de Tauri.
pub fn shutdown() {
    request_shutdown();
    let mut global = GLOBAL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    drop(global.take());
}

pub fn invalidate_manager() {
    let mut global = GLOBAL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(manager) = global.as_mut() {
        manager.connected = false;
        manager.capture_verified = false;
        manager.info = None;
    }
}

pub fn clear_capture_evidence() {
    let mut global = GLOBAL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(manager) = global.as_mut() { manager.capture_verified = false; }
}

pub fn publicar_evidencia(app: &tauri::AppHandle) {
    let global = GLOBAL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(manager) = global.as_ref().filter(|m| m.connected && !stopping()) {
        crate::lector_estado::evidencia_sdk(app, manager.usb_generation, manager.capture_verified);
    }
}

fn ensure_manager(global: &mut Option<NativeManager>, api: &'static SgfpmApi, generation: u64) -> Result<(), String> {
    if global.as_ref().map_or(false, |manager| !manager.connected || manager.usb_generation != generation) {
        drop(global.take());
    }
    if global.is_none() { *global = Some(create_manager(api, generation)?); }
    Ok(())
}

pub struct SecuGenController { pub bin_path: PathBuf }

impl SecuGenController {
    pub fn from_bin_path(bin_path: &Path) -> Result<Self, String> {
        let mut global = GLOBAL_MANAGER.lock().map_err(|e| e.to_string())?;
        check_running().map_err(String::from)?;
        let api = get_api(bin_path)?;
        let generation = crate::lector_estado::generation();
        if let Err(error) = crate::lector_estado::verificar(generation) {
            drop(global.take());
            return Err(error);
        }
        ensure_manager(&mut global, api, generation).map_err(|error| format!("{error}; SDK={}", bin_path.join("sgfplib.dll").display()))?;
        Ok(Self { bin_path: bin_path.to_path_buf() })
    }

    pub fn capturar_template(&self) -> Result<Option<String>, SecuGenError> {
        let api = get_api(&self.bin_path)?;
        let mut guard = GLOBAL_MANAGER.lock().map_err(|e| e.to_string())?;
        let state = guard.as_mut().ok_or(SecuGenError::Disconnected)?;
        let manager = state.connected_handle()?;
        let generation = state.usb_generation;
        if crate::lector_estado::verificar(generation).is_err() {
            state.connected = false;
            return Err(SecuGenError::Disconnected);
        }
        // Consultar dimensiones actuales antes de reservar el buffer.
        if let Err(error) = state.refresh_info() {
            state.connected = false;
            return Err(error);
        }
        let info = state.info.ok_or(SecuGenError::Disconnected)?;
        unsafe {
            let image_len = image_len(&info)?;
            let mut image = vec![0u8; image_len];
            let code = (api.get_image_ex)(manager, image.as_mut_ptr(), 4000, std::ptr::null_mut(), 40);
            // Un timeout no prueba presencia. Comprobar PnP y SDK también después
            // de la espera para no anunciar una recuperación con un handle viejo.
            if crate::lector_estado::verificar(generation).is_err() {
                state.connected = false;
                return Err(SecuGenError::Disconnected);
            }
            if let Err(error) = state.refresh_info() {
                state.connected = false;
                return Err(error);
            }
            let current = state.info.ok_or(SecuGenError::Disconnected)?;
            if current.image_width != info.image_width || current.image_height != info.image_height {
                state.connected = false;
                return Err(SecuGenError::Initialization("Las dimensiones del lector cambiaron durante la captura".into()));
            }
            state.record_capture(code);
            // GetDeviceInfo puede devolver datos del driver. TIMEOUT no es
            // prueba de captura ni habilita el indicador de disponibilidad.
            if code == 54 { return Ok(None); }
            if code != SGFDX_ERROR_NONE {
                let error = SecuGenError::Sdk { function: "SGFPM_GetImageEx", code };
                if error.needs_recovery() { state.connected = false; }
                return Err(error);
            }
            let mut template = [0u8; SG_TEMPLATE_SIZE];
            let mut finger = FingerInfo::default();
            if let Err(error) = check_sdk((api.create_template)(manager, (&mut finger as *mut FingerInfo).cast(), image.as_mut_ptr(), template.as_mut_ptr()), "SGFPM_CreateTemplate") {
                if error.persistent_fault() { state.capture_verified = false; }
                return Err(error);
            }
            Ok(Some(hex_encode(&template)))
        }
    }

    pub fn comparar_templates(&self, stored_hex: &str, captured_hex: &str) -> Result<bool, String> {
        let mut stored = hex_decode(stored_hex)?;
        let mut captured = hex_decode(captured_hex)?;
        if stored.len() != SG_TEMPLATE_SIZE || captured.len() != SG_TEMPLATE_SIZE {
            return Err("Plantilla biométrica inválida: se requieren 400 bytes".into());
        }
        let api = get_api(&self.bin_path)?;
        let guard = GLOBAL_MANAGER.lock().map_err(|e| e.to_string())?;
        let manager = guard.as_ref().ok_or("SecuGen desconectado")?.connected_handle()?;
        unsafe {
            let mut matched: BOOL = 0;
            check_sdk((api.match_template)(manager, stored.as_mut_ptr(), captured.as_mut_ptr(), SG_SECURITY_LEVEL_NORMAL, &mut matched), "SGFPM_MatchTemplate")?;
            Ok(matched != 0)
        }
    }

    pub fn identificar_template(&self, conexion: &Connection, template_hex: &str) -> Result<Option<i64>, SecuGenError> {
        let mut captured = hex_decode(template_hex)?;
        if captured.len() != SG_TEMPLATE_SIZE {
            return Err("Plantilla capturada inválida: se requieren 400 bytes".into());
        }
        let api = get_api(&self.bin_path)?;
        let guard = GLOBAL_MANAGER.lock().map_err(|e| e.to_string())?;
        let manager = guard.as_ref().ok_or(SecuGenError::Disconnected)?.connected_handle()?;
        unsafe {
            let mut stmt = conexion.prepare("SELECT id, template_huella FROM clientes WHERE activo = 1 AND COALESCE(template_huella, '') <> ''").map_err(|e| e.to_string())?;
            let lista: Vec<(i64, String)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
            for (id, hex) in lista {
                let mut stored = match hex_decode(&hex) { Ok(v) => v, Err(_) => continue };
                if stored.len() != SG_TEMPLATE_SIZE { continue; }
                let mut matched: BOOL = 0;
                check_sdk((api.match_template)(manager, stored.as_mut_ptr(), captured.as_mut_ptr(), SG_SECURITY_LEVEL_NORMAL, &mut matched), "SGFPM_MatchTemplate")?;
                if matched != 0 { return Ok(Some(id)); }
            }
            Ok(None)
        }
    }
}

pub fn resolve_bin_path(resource_path: &Path) -> Result<PathBuf, String> {
    let candidates = [resource_path.join("bin"), std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.join("bin"))).unwrap_or_default(), PathBuf::from("src-tauri").join("bin"), PathBuf::from("bin"),];
    let found = candidates.into_iter().find(|p| p.join("sgfplib.dll").is_file()).ok_or_else(|| "No se encontró bin".to_string())?;
    Ok(found)
}
#[cfg(windows)]
fn load_sdk_library(path: &Path) -> Result<Library, String> {
    use libloading::os::windows::Library as WinLibrary;
    use windows_sys::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32};
    unsafe { WinLibrary::load_with_flags(path, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32) }
        .map(Into::into).map_err(|e| format!("DLL {}: {e}", path.display()))
}

#[cfg(not(windows))]
fn load_sdk_library(path: &Path) -> Result<Library, String> {
    unsafe { Library::new(path) }.map_err(|e| e.to_string())
}

fn get_fp_lib(bin_path: &Path) -> Result<&'static Library, String> {
    if let Some(lib) = FP_LIB.get() { return Ok(lib); }
    let bin_path = bin_path.canonicalize().map_err(|e| e.to_string())?;
    // Manual nativo, componentes runtime: sgfpamx.dll es el algoritmo oficial.
    // El binario llama LoadLibraryA("sgfpaMX.dll") al inicializarlo. Retener
    // esta DLL conocida por ruta absoluta evita depender del directorio actual
    // sin cambiar globalmente la búsqueda. No precargar drivers de un modelo.
    let algorithm = load_sdk_library(&bin_path.join("sgfpamx.dll"))?;
    let lib = load_sdk_library(&bin_path.join("sgfplib.dll"))?;
    let _ = ALGORITHM_LIB.set(algorithm);
    Ok(FP_LIB.get_or_init(|| lib))
}
fn get_api(bin_path: &Path) -> Result<&'static SgfpmApi, String> {
    if let Some(api) = API.get() { return Ok(api); }
    let lib = get_fp_lib(bin_path)?;
    let api = unsafe {
        SgfpmApi {
            create: *lib.get(b"SGFPM_Create\0").map_err(|e| e.to_string())?,
            terminate: *lib.get(b"SGFPM_Terminate\0").map_err(|e| e.to_string())?,
            init: *lib.get(b"SGFPM_Init\0").map_err(|e| e.to_string())?,
            enumerate_device: *lib.get(b"SGFPM_EnumerateDevice\0").map_err(|e| e.to_string())?,
            set_template_format: *lib.get(b"SGFPM_SetTemplateFormat\0").map_err(|e| e.to_string())?,
            open_device: *lib.get(b"SGFPM_OpenDevice\0").map_err(|e| e.to_string())?,
            get_device_info: *lib.get(b"SGFPM_GetDeviceInfo\0").map_err(|e| e.to_string())?,
            get_image_ex: *lib.get(b"SGFPM_GetImageEx\0").map_err(|e| e.to_string())?,
            create_template: *lib.get(b"SGFPM_CreateTemplate\0").map_err(|e| e.to_string())?,
            match_template: *lib.get(b"SGFPM_MatchTemplate\0").map_err(|e| e.to_string())?,
            close_device: *lib.get(b"SGFPM_CloseDevice\0").map_err(|e| e.to_string())?,
        }
    };
    Ok(API.get_or_init(|| api))
}
pub fn check_sdk(code: DWORD, function: &'static str) -> Result<(), SecuGenError> {
    if code == SGFDX_ERROR_NONE { Ok(()) } else { Err(SecuGenError::Sdk { function, code }) }
}
pub fn hex_encode(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
pub fn hex_decode(value: &str) -> Result<Vec<u8>, String> {
    // No cortar UTF-8 en offsets de bytes: una plantilla dañada no debe
    // provocar panic y envenenar el mutex del manager.
    if !value.is_ascii() { return Err("hex invalido".into()); }
    if value.len() % 2 != 0 { return Err("hex len invalida".into()); }
    (0..value.len()).step_by(2).map(|i| u8::from_str_radix(&value[i..i+2], 16).map_err(|_| "hex invalido".into())).collect()
}

#[cfg(test)]
#[path = "tests/secugen.rs"]
mod tests;
