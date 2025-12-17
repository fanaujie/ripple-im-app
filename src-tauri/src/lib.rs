mod app_config;
mod commands;
mod db;
mod errors;
mod file_utils;
mod image_processor;
mod ripple_ws;
mod server;

mod ripple_api;
mod ripple_syncer;
mod store_engine;

use crate::ripple_api::RippleApi;
use crate::ripple_syncer::DataSyncManager;
use crate::ripple_syncer::DefaultEventEmitter;
use crate::ripple_syncer::RippleWsSyncHandler;
use crate::ripple_ws::RippleWsManager;
use crate::ripple_ws::SyncAwareWsMessageHandler;
use app_config::AppConfig;
use oauth2::reqwest;
use ripple_api::oauth_client::OauthClient;
use server::Server;
use std::fs;
use std::path::PathBuf;
use store_engine::store_engine::MemoryStore;
use tauri::path::BaseDirectory;
use tauri::Manager;

// #[cfg(feature = "memory-store")]
type DefaultStoreEngine = MemoryStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut config_file_path = "resources/prod_app_config.json";
    if cfg!(debug_assertions) {
        config_file_path = "resources/dev_app_config.json";
    }
    tauri::Builder::default()
        .setup(move |app| {
            let app_data_dir = app.path().app_data_dir()?;
            // Ensure the app data directory exists
            let _ = fs::create_dir_all(&app_data_dir).map_err(|e| {
                format!(
                    "Failed to create app data directory '{}': {}",
                    app_data_dir.display(),
                    e
                )
            })?;
            let resource_path = app
                .path()
                .resolve(config_file_path, BaseDirectory::Resource)?;
            let app_config = parse_app_config(resource_path);
            let reqwest_client = reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .proxy(reqwest::Proxy::http("http://192.168.50.31:9999")?)
                .build()?;
            let oauth_client = OauthClient::new(&app_config, reqwest_client.clone())?;
            let store = create_store();
            let ripple_api = RippleApi::new(
                app_config.upload_gateway_url.clone(),
                app_config.api_gateway_url.clone(),
                reqwest_client,
                oauth_client,
                store.clone(),
            );
            let data_sync = DataSyncManager::new(ripple_api.clone(), store);
            let emitter = DefaultEventEmitter::new(app.handle().clone());
            let syncer = RippleWsSyncHandler::new(data_sync.clone(), emitter);
            let sync_aware_msg_handler = SyncAwareWsMessageHandler::new(syncer);
            let ws_manager =
                RippleWsManager::new(sync_aware_msg_handler.clone(), data_sync.clone());
            app.manage(ripple_api);
            app.manage(data_sync);
            app.manage(sync_aware_msg_handler);
            app.manage(ws_manager);
            app.manage(app_config); // read-only, no mutex needed
            app.manage(tokio::sync::Mutex::new(Server::new()));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::exists_token,
            commands::start_server,
            commands::stop_server,
            commands::open_signup_url,
            commands::open_auth_url,
            commands::get_user_profile,
            commands::get_user_profile_by_id,
            commands::get_relations,
            commands::get_conversations,
            commands::upload_user_avatar_blob,
            commands::upload_image_blob,
            commands::upload_group_avatar_blob,
            commands::update_user_nickname,
            commands::remove_user_avatar,
            commands::add_friend,
            commands::remove_friend,
            commands::update_friend_display_name,
            commands::block_user,
            commands::unblock_user,
            commands::hide_blocked_user,
            commands::send_message,
            commands::read_latest_messages,
            commands::read_messages_before,
            commands::mark_last_read_message_id,
            commands::create_group,
            commands::invite_members,
            commands::get_group_members,
            commands::update_group_name,
            commands::leave_group,
            commands::upload_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn parse_app_config(resource_path: PathBuf) -> AppConfig {
    let file_content = fs::read_to_string(resource_path).expect("Failed to read app config file");
    let mut app_config: AppConfig =
        serde_json::from_str(&file_content).expect("Failed to parse app config JSON");
    app_config.is_dev = cfg!(debug_assertions);
    app_config
}

// #[cfg(feature = "memory-store")]
fn create_store() -> DefaultStoreEngine {
    MemoryStore::new()
}
