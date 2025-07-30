mod app_config;
mod commands;
mod errors;
mod oauth_client;
mod server;

use app_config::AppConfig;
use oauth_client::OauthClient;
use server::Server;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut config_file_path = "resources/prod_app_config.json";
    if cfg!(debug_assertions) {
        config_file_path = "resources/dev_app_config.json";
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let resource_path = app
                .path()
                .resolve(config_file_path, BaseDirectory::Resource)?;
            let app_config = parse_app_config(resource_path);
            let oauth_client = Arc::new(tokio::sync::Mutex::new(OauthClient::new(&app_config)?));
            app.manage(oauth_client);
            app.manage(app_config); // read-only, no mutex needed
            app.manage(tokio::sync::Mutex::new(Server::new()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::is_token_valid,
            commands::start_server,
            commands::stop_server,
            commands::open_signup_url,
            commands::open_auth_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn parse_app_config(resource_path: PathBuf) -> AppConfig {
    let file_content = fs::read_to_string(resource_path).expect("Failed to read app config file");
    serde_json::from_str(&file_content).expect("Failed to parse app config JSON")
}
