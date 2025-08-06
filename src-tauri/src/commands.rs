use crate::app_config::AppConfig;
use crate::db::DB;
use crate::errors;
use crate::oauth_client::OauthClient;
use crate::server::Server;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn exists_token(
    db_state: State<'_, tokio::sync::Mutex<DB>>,
) -> Result<bool, errors::CommandError> {
    let db = db_state.lock().await;
    let result = db.exists_token().await?;
    Ok(result)
}

#[tauri::command]
pub async fn start_server(app: AppHandle) -> Result<(), errors::CommandError> {
    let app_config = app.state::<AppConfig>();
    let server = app.state::<tokio::sync::Mutex<Server>>();
    let mut s = server.lock().await;
    s.start(app_config.callback_server_addr.clone(), app.clone())
        .await;
    Ok(())
}

#[tauri::command]
pub async fn stop_server(
    server: State<'_, tokio::sync::Mutex<Server>>,
) -> Result<(), errors::CommandError> {
    server.lock().await.stop().await;
    Ok(())
}

#[tauri::command]
pub fn open_signup_url(app: AppHandle) -> Result<(), errors::CommandError> {
    let app_config = app.state::<AppConfig>();
    let _ = app
        .opener()
        .open_url(app_config.signup_url.clone(), None::<&str>);
    Ok(())
}

#[tauri::command]
pub async fn open_auth_url(app: AppHandle) -> Result<(), errors::CommandError> {
    let oauth_client = app.state::<Arc<tokio::sync::Mutex<OauthClient>>>();
    let mut guard = oauth_client.lock().await;
    let _ = app.opener().open_url(guard.auth_url(), None::<&str>);
    Ok(())
}
