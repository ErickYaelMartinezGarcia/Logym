use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use rusqlite::Connection;
use std::ffi::c_void;
use std::path::{Path, PathBuf};

const SG_DEV_AUTO: u32 = 0xFF;
const SG_USB_AUTO_DETECT: u32 = 955;
const SG_TEMPLATE_FORMAT_SG400: u16 = 0x0200;
const SG_SECURITY_LEVEL_NORMAL: u32 = 3;
const SG_TEMPLATE_SIZE: usize = 400;
const SGFDX_ERROR_NONE: u32 = 0;

type Hsgfpm = *mut c_void;
type Dword = u32;
type Bool = i32;

#[repr(C)] #[derive(Clone, Copy)] struct DeviceList { dev_name: Dword, dev_id: Dword, dev_type: u16, dev_sn: [u8; 16], }
#[repr(C)] #[derive(Default, Clone, Copy)] struct DeviceInfo { device_id: Dword, device_sn: [u8; 16], com_port: Dword, com_speed: Dword, image_width: Dword, image_height: Dword, contrast: Dword, brightness: Dword, gain: Dword, image_dpi: Dword, fw_version: Dword, }
#[repr(C)] #[derive(Default, Clone, Copy)] struct FingerInfo { finger_number: u16, view_number: u16, impression_type: u16, image_quality: u16, }

type Create = unsafe extern "system" fn(*mut Hsgfpm) -> Dword;
type Terminate = unsafe extern "system" fn(Hsgfpm) -> Dword;
type Init = unsafe extern "system" fn(Hsgfpm, Dword) -> Dword;
type SetTemplateFormat = unsafe extern "system" fn(Hsgfpm, u16) -> Dword;
type EnumerateDevice = unsafe extern "system" fn(Hsgfpm, *mut Dword, *mut *mut DeviceList) -> Dword;
type OpenDevice = unsafe extern "system" fn(Hsgfpm, Dword) -> Dword;
type CloseDevice = unsafe extern "system" fn(Hsgfpm) -> Dword;
type GetDeviceInfo = unsafe extern "system" fn(Hsgfpm, *mut DeviceInfo) -> Dword;
type GetImageEx = unsafe extern "system" fn(Hsgfpm, *mut u8, Dword, *mut c_void, Dword) -> Dword;
type GetMaxTemplateSize = unsafe extern "system" fn(Hsgfpm, *mut Dword) -> Dword;
type CreateTemplate = unsafe extern "system" fn(Hsgfpm, *mut c_void, *mut u8, *mut u8) -> Dword;
type MatchTemplate = unsafe extern "system" fn(Hsgfpm, *mut u8, *mut u8, Dword, *mut Bool) -> Dword;

static MANAGER: OnceCell<Hsgfpm> = OnceCell::new();
static DEVICE_OPEN: OnceCell<bool> = OnceCell::new();

static FP_LIB: OnceCell<Library> = OnceCell::new();
fn get_fp_lib(bin_path: &Path) -> Result<&'static Library, String> {
    FP_LIB.get_or_try_init(|| {
        prepend_dll_directory(bin_path)?;
        unsafe { Library::new(bin_path.join("sgfplib.dll")).map_err(|e| format!("No se pudo cargar sgfplib.dll: {}", e)) }
    }).map_err(|e| e.to_string())
}

pub struct SecuGenController { bin_path: PathBuf }

impl SecuGenController {
    pub fn from_bin_path(bin_path: &Path) -> Result<Self, String> {
        println!("[INIT] bin_path: {}", bin_path.display());
        if !bin_path.join("sgfplib.dll").is_file() { 
            println!("[INIT] FAIL no existe sgfplib.dll");
            return Err(format!("No se encontró {}", bin_path.join("sgfplib.dll").display())); 
        }
        let _ = get_fp_lib(bin_path)?;
        println!("[INIT] Motor 1:1 listo - {}", bin_path.display());
        Ok(Self { bin_path: bin_path.to_path_buf() })
    }

    pub fn capturar_template(&self) -> Result<String, String> {
        println!("[CAPTURE] Iniciando captura");
        let fp_library = get_fp_lib(&self.bin_path)?;
        unsafe {
            let create: Symbol<Create> = Self::symbol_static(fp_library, b"SGFPM_Create\0")?;
            let terminate: Symbol<Terminate> = Self::symbol_static(fp_library, b"SGFPM_Terminate\0")?;
            let init: Symbol<Init> = Self::symbol_static(fp_library, b"SGFPM_Init\0")?;
            let set_format: Symbol<SetTemplateFormat> = Self::symbol_static(fp_library, b"SGFPM_SetTemplateFormat\0")?;
            let enumerate: Symbol<EnumerateDevice> = Self::symbol_static(fp_library, b"SGFPM_EnumerateDevice\0")?;
            let open: Symbol<OpenDevice> = Self::symbol_static(fp_library, b"SGFPM_OpenDevice\0")?;
            let close: Symbol<CloseDevice> = Self::symbol_static(fp_library, b"SGFPM_CloseDevice\0")?;
            let info: Symbol<GetDeviceInfo> = Self::symbol_static(fp_library, b"SGFPM_GetDeviceInfo\0")?;
            let get_image: Symbol<GetImageEx> = Self::symbol_static(fp_library, b"SGFPM_GetImageEx\0")?;
            let max_template: Symbol<GetMaxTemplateSize> = Self::symbol_static(fp_library, b"SGFPM_GetMaxTemplateSize\0")?;
            let create_template: Symbol<CreateTemplate> = Self::symbol_static(fp_library, b"SGFPM_CreateTemplate\0")?;
            let mut manager: Hsgfpm = std::ptr::null_mut();
            check_sdk(create(&mut manager), "SGFPM_Create")?;
            let result = (|| {
                check_sdk(init(manager, SG_DEV_AUTO), "SGFPM_Init")?;
                check_sdk(set_format(manager, SG_TEMPLATE_FORMAT_SG400), "SGFPM_SetTemplateFormat")?;
                let mut count = 0; let mut devices: *mut DeviceList = std::ptr::null_mut();
                check_sdk(enumerate(manager, &mut count, &mut devices), "SGFPM_EnumerateDevice")?;
                println!("[CAPTURE] devices encontrados: {}", count);
                if count == 0 { return Err("No se detectó lector".into()); }
                let first_device = *devices;
                if open(manager, first_device.dev_id) != SGFDX_ERROR_NONE { 
                    println!("[CAPTURE] probando AUTO_DETECT");
                    check_sdk(open(manager, SG_USB_AUTO_DETECT), "SGFPM_OpenDevice")?; 
                }
                let inner = (|| {
                    let mut device_info = DeviceInfo::default();
                    check_sdk(info(manager, &mut device_info), "SGFPM_GetDeviceInfo")?;
                    println!("[CAPTURE] imagen {}x{}", device_info.image_width, device_info.image_height);
                    let mut max_size = 0; check_sdk(max_template(manager, &mut max_size), "SGFPM_GetMaxTemplateSize")?;
                    let image_len = (device_info.image_width * device_info.image_height) as usize;
                    let mut image = vec![0u8; image_len];
                    println!("[CAPTURE] esperando dedo...");
                    check_sdk(get_image(manager, image.as_mut_ptr(), 10_000, std::ptr::null_mut(), 50), "SGFPM_GetImageEx")?;
                    println!("[CAPTURE] imagen capturada");
                    let mut template = vec![0u8; SG_TEMPLATE_SIZE]; let mut finger_info = FingerInfo::default();
                    check_sdk(create_template(manager, (&mut finger_info as *mut FingerInfo).cast(), image.as_mut_ptr(), template.as_mut_ptr()), "SGFPM_CreateTemplate")?;
                    println!("[CAPTURE] template creado {} bytes", template.len());
                    Ok(hex_encode(&template))
                })();
                let _ = close(manager);
                inner
            })();
            let _ = terminate(manager);
            result
        }
    }

    pub fn comparar_templates(&self, stored_hex: &str, captured_hex: &str) -> Result<bool, String> {
        println!("[COMPARE] comparando 2 templates");
        let mut stored = hex_decode(stored_hex)?; let mut captured = hex_decode(captured_hex)?;
        if stored.len() < SG_TEMPLATE_SIZE || captured.len() < SG_TEMPLATE_SIZE { return Err("Templates no tienen 400 bytes".into()); }
        let fp_library = get_fp_lib(&self.bin_path)?;
        unsafe {
            let create: Symbol<Create> = Self::symbol_static(fp_library, b"SGFPM_Create\0")?;
            let terminate: Symbol<Terminate> = Self::symbol_static(fp_library, b"SGFPM_Terminate\0")?;
            let init: Symbol<Init> = Self::symbol_static(fp_library, b"SGFPM_Init\0")?;
            let match_template: Symbol<MatchTemplate> = Self::symbol_static(fp_library, b"SGFPM_MatchTemplate\0")?;
            let mut manager = std::ptr::null_mut(); check_sdk(create(&mut manager), "SGFPM_Create")?;
            let result = (|| {
                check_sdk(init(manager, SG_DEV_AUTO), "SGFPM_Init")?;
                let mut matched = 0;
                check_sdk(match_template(manager, stored.as_mut_ptr(), captured.as_mut_ptr(), SG_SECURITY_LEVEL_NORMAL, &mut matched), "SGFPM_MatchTemplate")?;
                println!("[COMPARE] matched={}", matched);
                Ok(matched != 0)
            })();
            let _ = terminate(manager);
            result
        }
    }

    pub fn identificar_template(&self, conexion: &Connection, template_hex: &str) -> Result<Option<i64>, String> {
        println!("[SEARCH] identificar_template bypass 1:1 en RAM");
        let fp_library = get_fp_lib(&self.bin_path)?;
        let mut captured = hex_decode(template_hex)?;
        unsafe {
            let create: Symbol<Create> = Self::symbol_static(fp_library, b"SGFPM_Create\0")?;
            let terminate: Symbol<Terminate> = Self::symbol_static(fp_library, b"SGFPM_Terminate\0")?;
            let init: Symbol<Init> = Self::symbol_static(fp_library, b"SGFPM_Init\0")?;
            let match_template: Symbol<MatchTemplate> = Self::symbol_static(fp_library, b"SGFPM_MatchTemplate\0")?;
            let mut manager = std::ptr::null_mut();
            check_sdk(create(&mut manager), "SGFPM_Create")?;
            let res = (|| {
                check_sdk(init(manager, SG_DEV_AUTO), "SGFPM_Init")?;
                let mut stmt = conexion.prepare("SELECT id, template_huella FROM clientes WHERE COALESCE(template_huella, '') <> ''").map_err(|e| e.to_string())?;
                let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))).map_err(|e| e.to_string())?;
                let mut lista: Vec<(i64, String)> = Vec::new();
                for r in rows { if let Ok(v) = r { lista.push(v); } }
                println!("[SEARCH] Templates en BD: {}", lista.len());
                let start = std::time::Instant::now();
                for (client_id, stored_hex) in lista {
                    let mut stored = match hex_decode(&stored_hex) { Ok(v) => v, Err(_) => continue, };
                    if stored.len() < SG_TEMPLATE_SIZE { continue; }
                    let mut matched = 0;
                    let _ = match_template(manager, stored.as_mut_ptr(), captured.as_mut_ptr(), SG_SECURITY_LEVEL_NORMAL, &mut matched);
                    if matched != 0 {
                        println!("[SEARCH] MATCH id={} tiempo={:?}", client_id, start.elapsed());
                        return Ok(Some(client_id));
                    }
                }
                println!("[SEARCH] No match en {:?}", start.elapsed());
                Ok(None)
            })();
            let _ = terminate(manager);
            res
        }
    }

    pub fn cargar_todos_en_ram(&self, _conexion: &Connection) -> Result<String, String> {
        println!("[RAM] cargar_todos_en_ram ya no es necesario con bypass");
        Ok("ok".into())
    }

    unsafe fn symbol_static<'a, T>(library: &'static Library, name: &[u8]) -> Result<Symbol<'a, T>, String> {
        library.get(name).map_err(|e| format!("La DLL no exporta {}: {}", String::from_utf8_lossy(name), e))
    }
}

pub fn resolve_bin_path(resource_path: &Path) -> Result<PathBuf, String> {
    println!("[INIT] resolve_bin_path: {}", resource_path.display());
    let candidates = [resource_path.join("bin"), std::env::current_exe().ok().and_then(|p| p.parent().map(|parent| parent.join("bin"))).unwrap_or_default(), PathBuf::from("src-tauri").join("bin"), PathBuf::from("bin"),];
    let found = candidates.into_iter().find(|path| {
        let exists = path.join("sgfplib.dll").is_file();
        println!("[INIT] probando {} -> {}", path.display(), exists);
        exists
    }).ok_or_else(|| "No se encontró la carpeta bin".into());
    println!("[INIT] bin_path final: {:?}", found);
    found
}
fn prepend_dll_directory(path: &Path) -> Result<(), String> {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![path.to_path_buf()]; paths.extend(std::env::split_paths(&current));
    let joined = std::env::join_paths(paths).map_err(|e| e.to_string())?; std::env::set_var("PATH", joined); Ok(())
}
fn check_sdk(code: u32, function: &str) -> Result<(), String> { if code == SGFDX_ERROR_NONE { Ok(()) } else { Err(format!("{}: código {}", function, code)) } }
fn hex_encode(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn hex_decode(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 { return Err("hex len inválida".into()); }
    (0..value.len()).step_by(2).map(|i| u8::from_str_radix(&value[i..i+2], 16).map_err(|_| "hex inválido".into())).collect()
}

