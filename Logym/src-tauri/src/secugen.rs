use libloading::{Library, Symbol};
use rusqlite::Connection;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, Mutex};

// ==========================================
// CONSTANTES OFICIALES SECUGEN
// ==========================================
pub const SG_DEV_AUTO: u32 = 0xFF;
pub const SG_USB_AUTO_DETECT: u32 = 955;
pub const SG_TEMPLATE_FORMAT_SG400: u16 = 0x0200;
pub const SG_SECURITY_LEVEL_NORMAL: u32 = 3;
pub const SG_TEMPLATE_SIZE: usize = 400;
pub const SGFDX_ERROR_NONE: u32 = 0;
pub const WM_APP_SGAUTOONEVENT: u32 = 0x8100;
pub const SGDEVEVNET_FINGER_OFF: u16 = 0;
pub const SGDEVEVNET_FINGER_ON: u16 = 1;

// ==========================================
// TIPOS FFI
// ==========================================
pub type HSGFPM = *mut c_void;
pub type DWORD = u32;
pub type BOOL = i32;
pub type HWND = *mut c_void;

#[repr(C)] #[derive(Clone, Copy)] pub struct DeviceList { pub dev_name: DWORD, pub dev_id: DWORD, pub dev_type: u16, pub dev_sn: [u8; 16], }
#[repr(C)] #[derive(Default, Clone, Copy)] pub struct DeviceInfo { pub device_id: DWORD, pub device_sn: [u8; 16], pub com_port: DWORD, pub com_speed: DWORD, pub image_width: DWORD, pub image_height: DWORD, pub contrast: DWORD, pub brightness: DWORD, pub gain: DWORD, pub image_dpi: DWORD, pub fw_version: DWORD, }
#[repr(C)] #[derive(Default, Clone, Copy)] pub struct FingerInfo { pub finger_number: u16, pub view_number: u16, pub impression_type: u16, pub image_quality: u16, }

#[derive(Debug)]
pub struct SafeHsgfpm(pub HSGFPM);
unsafe impl Send for SafeHsgfpm {}
unsafe impl Sync for SafeHsgfpm {}

type FnCreate = unsafe extern "system" fn(*mut HSGFPM) -> DWORD;
type FnTerminate = unsafe extern "system" fn(HSGFPM) -> DWORD;
type FnInit = unsafe extern "system" fn(HSGFPM, DWORD) -> DWORD;
type FnSetTemplateFormat = unsafe extern "system" fn(HSGFPM, u16) -> DWORD;
type FnEnumerateDevice = unsafe extern "system" fn(HSGFPM, *mut DWORD, *mut *mut DeviceList) -> DWORD;
type FnOpenDevice = unsafe extern "system" fn(HSGFPM, DWORD) -> DWORD;
type FnCloseDevice = unsafe extern "system" fn(HSGFPM) -> DWORD;
type FnGetDeviceInfo = unsafe extern "system" fn(HSGFPM, *mut DeviceInfo) -> DWORD;
type FnGetImageEx = unsafe extern "system" fn(HSGFPM, *mut u8, DWORD, *mut c_void, DWORD) -> DWORD;
type FnGetMaxTemplateSize = unsafe extern "system" fn(HSGFPM, *mut DWORD) -> DWORD;
type FnCreateTemplate = unsafe extern "system" fn(HSGFPM, *mut c_void, *mut u8, *mut u8) -> DWORD;
type FnMatchTemplate = unsafe extern "system" fn(HSGFPM, *mut u8, *mut u8, DWORD, *mut BOOL) -> DWORD;
type FnEnableAutoOnEvent = unsafe extern "system" fn(HSGFPM, BOOL, HWND, *mut c_void) -> DWORD;

pub struct SgfpmApi {
    pub create: Symbol<'static, FnCreate>,
    pub terminate: Symbol<'static, FnTerminate>,
    pub init: Symbol<'static, FnInit>,
    pub set_template_format: Symbol<'static, FnSetTemplateFormat>,
    pub enumerate_device: Symbol<'static, FnEnumerateDevice>,
    pub open_device: Symbol<'static, FnOpenDevice>,
    pub close_device: Symbol<'static, FnCloseDevice>,
    pub get_device_info: Symbol<'static, FnGetDeviceInfo>,
    pub get_image_ex: Symbol<'static, FnGetImageEx>,
    pub get_max_template_size: Symbol<'static, FnGetMaxTemplateSize>,
    pub create_template: Symbol<'static, FnCreateTemplate>,
    pub match_template: Symbol<'static, FnMatchTemplate>,
    pub enable_auto_on_event: Symbol<'static, FnEnableAutoOnEvent>,
}

static FP_LIB: OnceLock<Library> = OnceLock::new();
static API: OnceLock<SgfpmApi> = OnceLock::new();
static GLOBAL_MANAGER: OnceLock<Mutex<SafeHsgfpm>> = OnceLock::new();
static GLOBAL_INFO: OnceLock<DeviceInfo> = OnceLock::new();

pub struct SecuGenController { pub bin_path: PathBuf }

impl SecuGenController {
    pub fn from_bin_path(bin_path: &Path) -> Result<Self, String> {
        if GLOBAL_MANAGER.get().is_none() {
            println!("[INIT] Cargando motor SecuGen desde: {}", bin_path.display());
        }
        let api = get_api(bin_path)?;
        if GLOBAL_MANAGER.get().is_none() {
            unsafe {
                let mut manager: HSGFPM = std::ptr::null_mut();
                check_sdk((api.create)(&mut manager), "SGFPM_Create")?;
                check_sdk((api.init)(manager, SG_DEV_AUTO), "SGFPM_Init")?;
                check_sdk((api.set_template_format)(manager, SG_TEMPLATE_FORMAT_SG400), "SGFPM_SetTemplateFormat")?;
                let mut count: DWORD = 0;
                let mut devices: *mut DeviceList = std::ptr::null_mut();
                check_sdk((api.enumerate_device)(manager, &mut count, &mut devices), "SGFPM_EnumerateDevice")?;
                if count == 0 { return Err("No se detectó lector".to_string()); }
                let first = *devices;
                let mut code = (api.open_device)(manager, first.dev_id);
                if code != SGFDX_ERROR_NONE { code = (api.open_device)(manager, SG_USB_AUTO_DETECT); }
                check_sdk(code, "SGFPM_OpenDevice")?;
                let mut info = DeviceInfo::default();
                check_sdk((api.get_device_info)(manager, &mut info), "SGFPM_GetDeviceInfo")?;
                GLOBAL_INFO.set(info).ok();
                GLOBAL_MANAGER.set(Mutex::new(SafeHsgfpm(manager))).ok();
                println!("[INIT] Motor global listo {}x{} - UNA SOLA VEZ", info.image_width, info.image_height);
            }
        }
        Ok(Self { bin_path: bin_path.to_path_buf() })
    }

    pub fn habilitar_auto_on(&self, hwnd: HWND, activar: bool) -> Result<(), String> {
        let api = get_api(&self.bin_path)?;
        let guard = GLOBAL_MANAGER.get().ok_or("Manager no inicializado")?.lock().map_err(|e| e.to_string())?;
        unsafe {
            let flag: BOOL = if activar { 1 } else { 0 };
            check_sdk((api.enable_auto_on_event)(guard.0, flag, hwnd, std::ptr::null_mut()), "SGFPM_EnableAutoOnEvent")?;
        }
        println!("[AUTO-ON] {} HWND: {:?}", if activar { "Activado" } else { "Desactivado" }, hwnd);
        Ok(())
    }

    // AHORA REGRESA Option: None = no hay dedo (era error 54)
    pub fn capturar_template(&self) -> Result<Option<String>, String> {
        let api = get_api(&self.bin_path)?;
        let guard = GLOBAL_MANAGER.get().ok_or("Manager no inicializado")?.lock().map_err(|e| e.to_string())?;
        let manager = guard.0;
        let info = GLOBAL_INFO.get().ok_or("Info no inicializada")?;
        unsafe {
            let image_len = (info.image_width * info.image_height) as usize;
            let mut image = vec![0u8; image_len];
            let code = (api.get_image_ex)(manager, image.as_mut_ptr(), 2000, std::ptr::null_mut(), 40);
            if code == 54 { // TIMEOUT - no hay dedo
               return Err("SGFPM_GetImageEx: 54".to_string()); // como antes, para que no te rompa todo lo demás
            }
            check_sdk(code, "SGFPM_GetImageEx")?;
            let mut template = [0u8; SG_TEMPLATE_SIZE];
            let mut finger = FingerInfo::default();
            check_sdk((api.create_template)(manager, (&mut finger as *mut FingerInfo).cast(), image.as_mut_ptr(), template.as_mut_ptr()), "SGFPM_CreateTemplate")?;
            Ok(Some(hex_encode(&template)))
        }
    }

    pub fn comparar_templates(&self, stored_hex: &str, captured_hex: &str) -> Result<bool, String> {
        let mut stored = hex_decode(stored_hex)?;
        let mut captured = hex_decode(captured_hex)?;
        let api = get_api(&self.bin_path)?;
        let guard = GLOBAL_MANAGER.get().ok_or("No init")?.lock().map_err(|e| e.to_string())?;
        unsafe {
            let mut matched: BOOL = 0;
            check_sdk((api.match_template)(guard.0, stored.as_mut_ptr(), captured.as_mut_ptr(), SG_SECURITY_LEVEL_NORMAL, &mut matched), "SGFPM_MatchTemplate")?;
            Ok(matched != 0)
        }
    }

    pub fn identificar_template(&self, conexion: &Connection, template_hex: &str) -> Result<Option<i64>, String> {
        let mut captured = hex_decode(template_hex)?;
        let api = get_api(&self.bin_path)?;
        let guard = GLOBAL_MANAGER.get().ok_or("No init")?.lock().map_err(|e| e.to_string())?;
        unsafe {
            let mut stmt = conexion.prepare("SELECT id, template_huella FROM clientes WHERE activo = 1 AND COALESCE(template_huella, '') <> ''").map_err(|e| e.to_string())?;
            let lista: Vec<(i64, String)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
            for (id, hex) in lista {
                let mut stored = match hex_decode(&hex) { Ok(v) => v, Err(_) => continue };
                if stored.len() < SG_TEMPLATE_SIZE { continue; }
                let mut matched: BOOL = 0;
                let _ = (api.match_template)(guard.0, stored.as_mut_ptr(), captured.as_mut_ptr(), SG_SECURITY_LEVEL_NORMAL, &mut matched);
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
fn prepend_dll_directory(path: &Path) -> Result<(), String> {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![path.to_path_buf()]; paths.extend(std::env::split_paths(&current));
    let joined = std::env::join_paths(paths).map_err(|e| e.to_string())?; std::env::set_var("PATH", joined); Ok(())
}
fn get_fp_lib(bin_path: &Path) -> Result<&'static Library, String> {
    if let Some(lib) = FP_LIB.get() { return Ok(lib); }
    let _ = prepend_dll_directory(bin_path);
    let lib = unsafe { Library::new(bin_path.join("sgfplib.dll")).map_err(|e| format!("Fallo DLL {}", e)) }?;
    Ok(FP_LIB.get_or_init(|| lib))
}
fn get_api(bin_path: &Path) -> Result<&'static SgfpmApi, String> {
    if let Some(api) = API.get() { return Ok(api); }
    let lib = get_fp_lib(bin_path)?;
    let api = unsafe {
        SgfpmApi {
            create: lib.get(b"SGFPM_Create\0").map_err(|e| e.to_string())?,
            terminate: lib.get(b"SGFPM_Terminate\0").map_err(|e| e.to_string())?,
            init: lib.get(b"SGFPM_Init\0").map_err(|e| e.to_string())?,
            set_template_format: lib.get(b"SGFPM_SetTemplateFormat\0").map_err(|e| e.to_string())?,
            enumerate_device: lib.get(b"SGFPM_EnumerateDevice\0").map_err(|e| e.to_string())?,
            open_device: lib.get(b"SGFPM_OpenDevice\0").map_err(|e| e.to_string())?,
            close_device: lib.get(b"SGFPM_CloseDevice\0").map_err(|e| e.to_string())?,
            get_device_info: lib.get(b"SGFPM_GetDeviceInfo\0").map_err(|e| e.to_string())?,
            get_image_ex: lib.get(b"SGFPM_GetImageEx\0").map_err(|e| e.to_string())?,
            get_max_template_size: lib.get(b"SGFPM_GetMaxTemplateSize\0").map_err(|e| e.to_string())?,
            create_template: lib.get(b"SGFPM_CreateTemplate\0").map_err(|e| e.to_string())?,
            match_template: lib.get(b"SGFPM_MatchTemplate\0").map_err(|e| e.to_string())?,
            enable_auto_on_event: lib.get(b"SGFPM_EnableAutoOnEvent\0").map_err(|e| format!("No se encontró EnableAutoOnEvent: {}", e))?,
        }
    };
    Ok(API.get_or_init(|| api))
}
pub fn check_sdk(code: DWORD, function: &str) -> Result<(), String> { if code == SGFDX_ERROR_NONE { Ok(()) } else { Err(format!("{}: {}", function, code)) } }
pub fn hex_encode(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
pub fn hex_decode(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 { return Err("hex len invalida".into()); }
    (0..value.len()).step_by(2).map(|i| u8::from_str_radix(&value[i..i+2], 16).map_err(|_| "hex invalido".into())).collect()
}