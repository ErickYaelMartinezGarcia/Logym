#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod models;
mod lector_worker;
mod lector_estado;
mod lector_mensajes;
mod licencia;
mod membresias;
mod visitas;
mod repositories;
mod secugen;
mod services;
mod recuperacion;
#[cfg(test)]
#[path = "tests/test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "tests/business.rs"]
mod business_tests;

use commands::{
    actualizar_cliente, establecer_cliente_activo, eliminar_cliente_definitivamente,
    buscar_cliente_por_huella, capturar_huella, comparar_huella, check_in, guardar_cliente, inicializar_db,
    listar_clientes, listar_clientes_membresia, obtener_metricas,
    renovar_membresia, registrar_asistencia, buscar_clientes_checkin,
};

fn main() {
    let inicio_arranque = std::time::Instant::now();
    eprintln!("[ARRANQUE] pid={} fase=proceso_iniciado tiempo_ms=0", std::process::id());
    #[cfg(windows)]
    if std::env::args().nth(1).as_deref()==Some("--diagnostico-licencia") {
        let archivo=std::env::args_os().nth(2).map(std::path::PathBuf::from);
        println!("{}",licencia::diagnostico(archivo.as_deref()));
        return;
    }
    #[cfg(windows)]
    let _instancia = match recuperacion::Instancia::adquirir() {
        Ok(instancia) => instancia,
        Err(error) => { eprintln!("{error}"); std::process::exit(1); }
    };
    #[cfg(windows)]
    if std::env::args().nth(1).as_deref()==Some("--mantenimiento") {
        let resultado = recuperacion::ejecutar();
        match resultado { Ok(()) => println!("Mantenimiento verificado"), Err(error) => { eprintln!("{error}"); std::process::exit(1); } }
        return;
    }
    std::panic::set_hook(Box::new(|info| {
        eprintln!("PANIC REAL: {:?}", info);
        std::thread::sleep(std::time::Duration::from_secs(15));
    }));

    tauri::Builder::default()
        .setup(move |app| {
            eprintln!("[ARRANQUE] pid={} fase=setup_inicio tiempo_ms={}", std::process::id(), inicio_arranque.elapsed().as_millis());
            let verificacion = licencia::verificar(&app.handle());
            eprintln!("[ARRANQUE] pid={} fase=verificacion_finalizada tiempo_ms={}", std::process::id(), inicio_arranque.elapsed().as_millis());
            if verificacion.is_err() {
                println!("[LICENCIA] Activación requerida");
                lector_worker::iniciar(app.handle());
                return Ok(());
            }
            eprintln!("[ARRANQUE] pid={} fase=almacenamiento_inicio tiempo_ms={}", std::process::id(), inicio_arranque.elapsed().as_millis());

            match database::abrir_db(&app.handle()) {
                Ok(conn) => {
                    if let Err(e) = database::inicializar_esquema(&conn) {
                        eprintln!("[DB] Error esquema: {}", e);
                    } else {
                        println!("[DB] Lista");
                    }
                }
                Err(e) => eprintln!("[DB] Error: {}", e),
            }
            eprintln!("[ARRANQUE] pid={} fase=almacenamiento_finalizado tiempo_ms={}", std::process::id(), inicio_arranque.elapsed().as_millis());
            lector_worker::iniciar(app.handle());
            eprintln!("[ARRANQUE] pid={} fase=setup_finalizado tiempo_ms={}", std::process::id(), inicio_arranque.elapsed().as_millis());
            Ok(())
        })
        .on_page_load(move |window, _| {
            eprintln!("[ARRANQUE] pid={} fase=documento_cargado tiempo_ms={}", std::process::id(), inicio_arranque.elapsed().as_millis());
            if let Err(error) = window.show() {
                eprintln!("[ARRANQUE] No se pudo mostrar la ventana: {error}");
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::registrar_visita,
            commands::listar_planes_admin, commands::admin_configurado, commands::crear_password_admin, commands::validar_password_admin,
            commands::guardar_plan_admin, commands::guardar_inscripcion_admin, commands::obtener_asistencias_periodo, commands::obtener_asistencias_diarias, commands::obtener_asistencias_por_hora,commands::obtener_asistencias_periodo, commands::obtener_asistencias_diarias,
            commands::listar_tipos_membresia, commands::obtener_precio_inscripcion, commands::resumen_ingresos, commands::reporte_financiero,
 commands::solicitud_licencia,
            commands::estado_licencia,
            commands::activar_licencia,
            commands::pausar_lector_global,
            commands::estado_lector,
            actualizar_cliente,
            establecer_cliente_activo,
            eliminar_cliente_definitivamente,
            inicializar_db,
            guardar_cliente,
            listar_clientes,
            listar_clientes_membresia,
            buscar_cliente_por_huella,
            renovar_membresia,
            // API de asistencia invocada desde el frontend; registrarla evita código inaccesible.
            registrar_asistencia,
            buscar_clientes_checkin,
            obtener_metricas,
            capturar_huella,
            comparar_huella,
            check_in,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
                lector_worker::cerrar(app.clone());
            }
        });
}
