#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod biometria;
mod commands;
mod database;
mod models;
mod repositories;
mod services;

use commands::{
    buscar_cliente_por_huella, capturar_huella, comparar_huella, guardar_cliente, guardar_template,
    inicializar_db, listar_clientes, listar_clientes_membresia, obtener_metricas,
    registrar_asistencia, renovar_membresia,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
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
            guardar_template,
            comparar_huella
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
