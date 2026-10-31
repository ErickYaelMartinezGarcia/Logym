//! Único supervisor biométrico de la aplicación. Conserva la comparación 1:1
//! existente en secugen.rs. El worker no depende de la ruta Vue ni del foco.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;
use tauri::{AppHandle, Manager};

static PAUSADO: AtomicBool = AtomicBool::new(false);
static INICIADO: OnceLock<()> = OnceLock::new();

pub fn pausar(pausado: bool) {
    PAUSADO.store(pausado, Ordering::SeqCst);
}

pub fn iniciar(app: AppHandle) {
    if INICIADO.set(()).is_err() { return; }
    std::thread::spawn(move || {
        let mut ultimo_id: Option<i64> = None;
        let mut ultimo_instante = std::time::Instant::now() - Duration::from_secs(10);
        loop {
            if PAUSADO.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
            match crate::services::buscar_cliente_por_huella(&app) {
                Ok(Some(cliente)) => {
                    // Evita reintentos al mantener el dedo sobre el sensor.
                    if ultimo_id == Some(cliente.id) && ultimo_instante.elapsed() < Duration::from_secs(6) {
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    ultimo_id = Some(cliente.id);
                    ultimo_instante = std::time::Instant::now();
                    let resultado = if cliente.estado_membresia == "vencida" || cliente.estado_membresia == "sin_fecha" {
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
                    // La ausencia de dedo es esperada, no una falla del sistema.
                    if !e.contains("SGFPM_GetImageEx: 54") {
                        eprintln!("[LECTOR] {e}");
                        std::thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    });
}
