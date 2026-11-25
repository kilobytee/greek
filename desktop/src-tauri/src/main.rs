#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use greek_core::models::ConnectionInfo;
use greek_core::server;
use tracing_subscriber::EnvFilter;

struct AppState {
    info: ConnectionInfo,
}

#[tauri::command]
fn get_connection_info(state: tauri::State<'_, AppState>) -> ConnectionInfo {
    state.info.clone()
}

fn main() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("greek=info,greek_core=info,tower_http=warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();

    let info = match server::spawn_server() {
        Ok(info) => info,
        Err(error) => {
            eprintln!("No se pudo iniciar el núcleo de Greek: {error:#}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(AppState { info })
        .invoke_handler(tauri::generate_handler![get_connection_info])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar Greek");
}
