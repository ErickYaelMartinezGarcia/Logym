#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod models;
mod repositories;
mod secugen;
mod services;

use commands::{
    buscar_cliente_por_huella, capturar_huella, guardar_cliente, inicializar_db,
    listar_clientes, listar_clientes_membresia, obtener_metricas, registrar_asistencia,
    renovar_membresia,
};
use std::path::PathBuf;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("PANIC REAL: {:?}", info);
        std::thread::sleep(std::time::Duration::from_secs(15));
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let resource_dir = app.path_resolver().resource_dir().unwrap_or_else(|| PathBuf::from("."));
            println!("[INIT] resource_dir: {}", resource_dir.display());
            
            if let Ok(bin_path) = secugen::resolve_bin_path(&resource_dir) {
                println!("[INIT] bin_path: {}", bin_path.display());
                match secugen::SecuGenController::from_bin_path(&bin_path) {
                    Ok(_) => println!("[INIT] Motor 1:1 listo"),
                    Err(e) => eprintln!("[INIT] Motor no iniciado: {}", e),
                }
            } else {
                eprintln!("[INIT] No se encontró carpeta bin");
            }

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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            inicializar_db,
            guardar_cliente,
            listar_clientes,
            listar_clientes_membresia,
            buscar_cliente_por_huella,
            registrar_asistencia,
            renovar_membresia,
            obtener_metricas,
            capturar_huella,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}