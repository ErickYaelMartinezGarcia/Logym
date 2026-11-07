use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Estado { Inicializando, Disponible, Recuperando, NoDisponible }

#[derive(Clone, Serialize)]
pub struct EstadoLector {
    pub estado: Estado,
    pub detalle: String,
    pub revision: u64,
    pub usb_presente: bool,
    pub sdk_inicializado: bool,
    pub captura_comprobada: bool,
}

static ESTADO: Mutex<EstadoLector> = Mutex::new(EstadoLector {
    estado: Estado::Inicializando, detalle: String::new(), revision: 0,
    usb_presente: false, sdk_inicializado: false, captura_comprobada: false,
});

#[derive(Default)]
struct UsbTracker {
    ids: Option<Vec<String>>,
    generation: u64,
    error: Option<String>,
}

impl UsbTracker {
    fn update(&mut self, result: Result<Vec<String>, String>) -> bool {
        let (ids, error) = match result {
            Ok(mut ids) => { ids.sort(); ids.dedup(); (Some(ids), None) }
            Err(error) => (None, Some(error)),
        };
        if self.ids == ids && self.error == error { return false; }
        self.ids = ids;
        self.error = error;
        self.generation += 1;
        true
    }

    fn ready(&self, generation: u64) -> bool {
        self.generation == generation && self.error.is_none()
            && self.ids.as_ref().map_or(false, |ids| !ids.is_empty())
    }
}

static USB: Mutex<UsbTracker> = Mutex::new(UsbTracker {
    ids: None, generation: 0, error: None,
});


pub fn consultar() -> EstadoLector {
    ESTADO.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn publicar(app: &AppHandle, estado: Estado, detalle: impl Into<String>) {
    let detalle = detalle.into();
    let mut current = ESTADO.lock().unwrap_or_else(|e| e.into_inner());
    if current.estado == estado && current.detalle == detalle { return; }
    current.estado = estado;
    current.detalle = detalle;
    current.sdk_inicializado = false;
    current.captura_comprobada = false;
    current.revision += 1;
    let _ = app.emit_all("logym://lector-estado", current.clone());
}

pub fn generation() -> u64 {
    USB.lock().unwrap_or_else(|e| e.into_inner()).generation
}

pub fn verificar(generation: u64) -> Result<(), String> {
    let usb = USB.lock().unwrap_or_else(|e| e.into_inner());
    if usb.ready(generation) { return Ok(()); }
    Err(usb.error.clone().unwrap_or_else(|| "El lector HU20 no está presente o cambió la conexión USB".into()))
}

fn aplicar_evidencia(current: &mut EstadoLector, usb: &UsbTracker, generation: u64, captura: bool) -> bool {
    if !usb.ready(generation) { return false; }
    let estado = if captura { Estado::Disponible } else { Estado::Recuperando };
    let detalle = if captura { "Captura comprobada por el SDK" }
        else { "USB presente y SDK inicializado; coloca el dedo para comprobar la captura" };
    if current.estado == estado && current.detalle == detalle && current.sdk_inicializado
        && current.captura_comprobada == captura { return false; }
    current.estado = estado;
    current.detalle = detalle.into();
    current.usb_presente = true;
    current.sdk_inicializado = true;
    current.captura_comprobada = captura;
    current.revision += 1;
    true
}

// Evidencia del manager actual bajo GLOBAL_MANAGER. Un timeout jamás eleva
// captura_comprobada; una generación anterior tampoco puede quitar la alerta.
pub fn evidencia_sdk(app: &AppHandle, generation: u64, captura: bool) {
    let usb = USB.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = ESTADO.lock().unwrap_or_else(|e| e.into_inner());
    if aplicar_evidencia(&mut current, &usb, generation, captura) {
        let _ = app.emit_all("logym://lector-estado", current.clone());
    }
}

fn aplicar_usb(current: &mut EstadoLector, usb: &UsbTracker) {
    current.usb_presente = usb.ready(usb.generation);
    current.sdk_inicializado = false;
    current.captura_comprobada = false;
    let (estado, detalle) = if let Some(error) = &usb.error {
        (Estado::NoDisponible, error.clone())
    } else if !current.usb_presente {
        (Estado::NoDisponible, "Windows no detecta un lector USB SecuGen. Revisa la conexión USB.".into())
    } else {
        (Estado::Recuperando, "Windows detecta el lector; comprobando el SDK".into())
    };
    current.estado = estado;
    current.detalle = detalle;
    current.revision += 1;
}

fn es_secugen(id: &str) -> bool {
    // VID del fabricante documentado en el INF instalado. El SDK enumera el
    // modelo: no bloquear otras revisiones HU20 por fijar un único PID.
    let id = id.to_ascii_uppercase();
    let mut parts = id.split('\\').nth(1).unwrap_or("").split('&');
    parts.next() == Some("VID_1162") && parts.next().map_or(false, |pid|
        pid.len() == 8 && pid.starts_with("PID_") && pid[4..].bytes().all(|b| b.is_ascii_hexdigit()))
}

#[cfg(windows)]
fn dispositivos_presentes() -> Result<Vec<String>, String> {
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::*;
    let filter: Vec<u16> = "USB\0".encode_utf16().collect();
    let flags = CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT;
    // La topología puede cambiar entre Size y List: reintentar CR_BUFFER_SMALL.
    for _ in 0..3 {
        let mut len = 0;
        let code = unsafe { CM_Get_Device_ID_List_SizeW(&mut len, filter.as_ptr(), flags) };
        if code != CR_SUCCESS { return Err(format!("Consulta PnP de Windows: {code}")); }
        if len > 1_048_576 { return Err("Lista PnP fuera de rango".into()); }
        let mut buffer = vec![0u16; len.max(2) as usize];
        let code = unsafe { CM_Get_Device_ID_ListW(filter.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32, flags) };
        if code == CR_BUFFER_SMALL { continue; }
        if code != CR_SUCCESS { return Err(format!("Enumeración PnP de Windows: {code}")); }
        return Ok(buffer.split(|c| *c == 0).filter(|s| !s.is_empty())
            .map(String::from_utf16_lossy).filter(|id| es_secugen(id)).collect());
    }
    Err("La topología USB sigue cambiando; se reintentará".into())
}

#[cfg(not(windows))]
fn dispositivos_presentes() -> Result<Vec<String>, String> {
    Err("El lector HU20 requiere Windows".into())
}

pub fn iniciar_monitor(app: AppHandle) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || while !crate::secugen::stopping() {
        let result = dispositivos_presentes();
        // Este hilo nunca llama al SDK ni toma GLOBAL_MANAGER: puede avisar
        // de una retirada incluso mientras GetImageEx espera en otro hilo.
        let mut usb = USB.lock().unwrap_or_else(|e| e.into_inner());
        if usb.update(result) {
            // Publicar estado y evidencia en una sola sección crítica. Un
            // snapshot nunca debe conservar Disponible tras invalidar la captura.
            let snapshot = {
                let mut current = ESTADO.lock().unwrap_or_else(|e| e.into_inner());
                aplicar_usb(&mut current, &usb);
                current.clone()
            };
            let _ = app.emit_all("logym://lector-estado", snapshot);
        }
        drop(usb);
        std::thread::sleep(std::time::Duration::from_millis(500));
    })
}

#[cfg(test)]
#[path = "tests/lector_estado.rs"]
pub(crate) mod tests;
