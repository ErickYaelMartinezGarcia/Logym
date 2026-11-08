//! Único supervisor biométrico de la aplicación. Conserva la comparación 1:1
//! existente en secugen.rs. El worker no depende de la ruta Vue ni del foco.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Manager};

static PAUSADO: AtomicBool = AtomicBool::new(false);
static INICIADO: OnceLock<()> = OnceLock::new();
// Hace atómica la pausa frente al inicio de una recuperación USB.
static RECONEXION: Mutex<()> = Mutex::new(());

pub fn pausar(pausado: bool) {
    let _guard = RECONEXION.lock().unwrap_or_else(|e| e.into_inner());
    PAUSADO.store(pausado, Ordering::SeqCst);
}

pub fn inicializar_lector(bin_path: &std::path::Path) -> Result<Option<crate::secugen::SecuGenController>, String> {
    let _guard = RECONEXION.lock().unwrap_or_else(|e| e.into_inner());
    if PAUSADO.load(Ordering::SeqCst) { return Ok(None); }
    crate::secugen::SecuGenController::from_bin_path(bin_path).map(Some)
}

static THREADS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());
static CIERRE: AtomicBool = AtomicBool::new(false);

// La UI solo solicita el cierre; el coordinador espera fuera del event loop.
pub fn cerrar(app: AppHandle) {
    if CIERRE.swap(true, Ordering::SeqCst) { return; }
    crate::secugen::request_shutdown();
    std::thread::spawn(move || {
        let threads = std::mem::take(&mut *THREADS.lock().unwrap_or_else(|e| e.into_inner()));
        for thread in threads { let _ = thread.join(); }
        crate::secugen::shutdown();
        app.exit(0);
    });
}

#[derive(Default)]
struct FallosConsecutivos(u8);
impl FallosConsecutivos {
    fn registrar(&mut self, fallo: bool) -> bool {
        if !fallo { self.0 = 0; return false; }
        self.0 += 1;
        if self.0 < 3 { return false; }
        self.0 = 0;
        true
    }
}

pub fn iniciar(app: AppHandle) {
    if INICIADO.set(()).is_err() { return; }
    let monitor = crate::lector_estado::iniciar_monitor(app.clone());
    let worker = std::thread::spawn(move || {
        let mut ultimo_id: Option<i64> = None;
        let mut ultimo_instante = std::time::Instant::now() - Duration::from_secs(10);
        let mut usb_generation = crate::lector_estado::generation();
        let mut ultimo_error = String::new();
        let mut ultimo_log = std::time::Instant::now() - Duration::from_secs(10);
        let mut reintentar_desde: Option<std::time::Instant> = None;
        let mut fallos = FallosConsecutivos::default();
        while !crate::secugen::stopping() {
            if crate::licencia::verificar(&app).is_err() {
                crate::lector_estado::publicar(&app, crate::lector_estado::Estado::NoDisponible, "Activa la licencia para habilitar el lector");
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
            if PAUSADO.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
            let generation = crate::lector_estado::generation();
            if generation != usb_generation {
                usb_generation = generation;
                reintentar_desde = None;
            }
            if reintentar_desde.map_or(false, |limite| std::time::Instant::now() < limite) {
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
            if crate::lector_estado::consultar().estado == crate::lector_estado::Estado::NoDisponible {
                crate::lector_estado::publicar(&app, crate::lector_estado::Estado::Recuperando, "Comprobando el lector y el SDK");
            }
            let resultado = crate::services::buscar_cliente_por_huella_worker(&app);
            if crate::secugen::stopping() { break; }
            if PAUSADO.load(Ordering::SeqCst) { continue; }
            let fallo = resultado.as_ref().err().map_or(false, |e| e.needs_recovery() || e.persistent_fault());
            if fallo { crate::secugen::clear_capture_evidence(); }
            if fallos.registrar(fallo) {
                // También un fallo SDK desconocido puede dejar un handle
                // inutilizable. Tras tres intentos espaciados, recrearlo por RAII.
                crate::secugen::invalidate_manager();
            }
            if !fallo {
                crate::secugen::publicar_evidencia(&app);
            }
            match resultado {
                Ok(Some(cliente)) => {
                    // El registro puede haber pausado el lector durante la captura.
                    if PAUSADO.load(Ordering::SeqCst) { continue; }
                    // Evita reintentos al mantener el dedo sobre el sensor.
                    if ultimo_id == Some(cliente.id) && ultimo_instante.elapsed() < Duration::from_secs(6) {
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    ultimo_id = Some(cliente.id);
                    ultimo_instante = std::time::Instant::now();
                    let resultado = if cliente.estado_membresia == "pendiente" {
                        "membresia_pendiente".to_string()
                    } else if cliente.estado_membresia == "vencida" || cliente.estado_membresia == "sin_fecha" {
                        "membresia_vencida".to_string()
                    } else {
                        match crate::services::registrar_asistencia(&app, cliente.id) {
                            Ok(_) => "registrada".to_string(),
                            Err(e) if e.contains("Ya registró") => "duplicada".to_string(),
                            Err(e) => format!("error: {e}"),
                        }
                    };
                    // Se notifica también si la ventana está minimizada.
                    let _ = app.emit_all("logym://checkin", serde_json::json!({
                        "cliente": cliente,
                        "resultado": resultado,
                    }));
                    if resultado == "registrada" {
                        let _ = app.emit_all("logym://datos-actualizados", "asistencia");
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    if PAUSADO.load(Ordering::SeqCst) { continue; }
                    let mensaje = e.to_string();
                    if mensaje != ultimo_error || ultimo_log.elapsed() >= Duration::from_secs(60) {
                        eprintln!("[LECTOR] {mensaje}");
                        ultimo_error = mensaje.clone();
                        ultimo_log = std::time::Instant::now();
                    }
                    if e.invalid_read() {
                        let aviso = if matches!(e, crate::secugen::SecuGenError::Sdk {
                            function: "SGFPM_GetImageEx", code: 62,
                        }) { "dedo_rechazado" } else { "lectura_invalida" };
                        let _ = app.emit_all("logym://lector-aviso", aviso);
                    }
                    if e.needs_recovery() || e.persistent_fault() {
                        crate::lector_estado::publicar(&app, crate::lector_estado::Estado::NoDisponible, mensaje);
                        // El siguiente ciclo ejecuta from_bin_path: destruye el
                        // manager inválido antes de crear uno nuevo. No reabre
                        // un handle cuyo CloseDevice haya fallado.
                        reintentar_desde = Some(std::time::Instant::now() + Duration::from_secs(5));
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    });
    THREADS.lock().unwrap_or_else(|e| e.into_inner()).extend([monitor, worker]);
}

#[cfg(test)]
#[path = "tests/lector_worker.rs"]
mod tests;
