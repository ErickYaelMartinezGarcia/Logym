//! Frontera de presentación: los detalles nativos solo se escriben en logs.
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
thread_local! { static MOTIVO: Cell<&'static str> = const { Cell::new("sin_contexto") }; }
static OPERACION: AtomicU64 = AtomicU64::new(1);

/// Instrumentación optativa: no adquiere el lector ni cambia sus reintentos.
pub fn trazar(contexto: &str, detalle: &str) {
    if std::env::var_os("LOGYM_DIAGNOSTICO_LECTOR").as_deref() == Some(std::ffi::OsStr::new("1")) {
        registrar(contexto, detalle);
    }
}

pub struct Tramo {
    anterior: &'static str,
    id: u64,
    inicio: std::time::Instant,
    _hilo: std::marker::PhantomData<*mut ()>,
}
impl Tramo {
    pub fn nuevo(motivo: &'static str) -> Self {
        let anterior = MOTIVO.with(|valor| valor.replace(motivo));
        let id = OPERACION.fetch_add(1, Ordering::Relaxed);
        trazar("operacion_inicio", &format!("operacion_id={id}"));
        Self { anterior, id, inicio: std::time::Instant::now(), _hilo: std::marker::PhantomData }
    }
}
impl Drop for Tramo {
    fn drop(&mut self) {
        trazar("operacion_fin", &format!("operacion_id={} duracion_ms={}", self.id, self.inicio.elapsed().as_millis()));
        MOTIVO.with(|valor| valor.set(self.anterior));
    }
}
pub const INICIO: &str = "No se pudo iniciar el lector de huellas. Verifica su conexión USB.";
pub const CAPTURA: &str = "No se pudo leer la huella. Coloca el dedo completo e intenta nuevamente.";
pub const VERIFICACION: &str = "No se pudo verificar la huella. Intenta nuevamente.";

pub fn publico(error: &str) -> &'static str {
    if error.contains("SGFPM_MatchTemplate") || error.contains("SGFPM_SetTemplateFormat") {
        VERIFICACION
    } else if error.contains("SGFPM_GetImageEx") || error.contains("SGFPM_CreateTemplate") {
        CAPTURA
    } else { INICIO }
}

pub fn diagnostico(contexto: &str, error: &str) -> String {
    let estado = crate::lector_estado::consultar();
    format!("[LECTOR] fecha={} pid={} hilo={:?} motivo={} contexto={} supervisor={:?} revision={} usb_presente={} sdk_inicializado={} captura_comprobada={} generacion_usb={} detalle={}",
        chrono::Local::now().to_rfc3339(), std::process::id(), std::thread::current().id(), MOTIVO.with(Cell::get), contexto, estado.estado, estado.revision, estado.usb_presente,
        estado.sdk_inicializado, estado.captura_comprobada, crate::lector_estado::generation(), error)
}

pub fn registrar(contexto: &str, error: &str) {
    eprintln!("{}", diagnostico(contexto, error));
}

pub fn error_publico(contexto: &str, error: String) -> String {
    registrar(contexto, &error);
    publico(&error).into()
}

#[cfg(test)]
#[path = "tests/lector_mensajes.rs"]
mod tests;
