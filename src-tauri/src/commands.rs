use crate::app_config::AppConfig;
use std::path::Path;

use crate::db::UserProfile;
use crate::errors;
use crate::image_processor::ImageProcessor;
use crate::ripple::RippleApi;
use crate::server::Server;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn exists_token(
    state_ripple: State<'_, RippleApi>,
) -> Result<bool, errors::CommandError> {
    Ok(state_ripple.initialize_token_from_db().await?)
}

#[tauri::command]
pub async fn start_server(app: AppHandle) -> Result<(), errors::CommandError> {
    let app_config = app.state::<AppConfig>();
    let state_server = app.state::<tokio::sync::Mutex<Server>>();
    let mut server = state_server.lock().await;
    server
        .start(app_config.callback_server_addr.clone(), app.clone())
        .await;
    Ok(())
}

#[tauri::command]
pub async fn stop_server(
    state_server: State<'_, tokio::sync::Mutex<Server>>,
) -> Result<(), errors::CommandError> {
    let mut server = state_server.lock().await;
    server.stop().await;
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
pub fn open_auth_url(app: AppHandle) -> Result<(), errors::CommandError> {
    let ripple = app.state::<RippleApi>();
    Ok(app
        .opener()
        .open_url(ripple.oauth_auth_url(), None::<&str>)?)
}

#[tauri::command]
pub async fn get_user_profile(
    state_ripple: State<'_, RippleApi>,
) -> Result<UserProfile, errors::CommandError> {
    let response = state_ripple.get_user_profile().await?;

    let profile = UserProfile {
        user_id: response.data.account,
        nickname: response.data.nick_name,
        avatar_path: response.data.user_portrait,
    };

    Ok(profile)
}

#[tauri::command]
pub async fn update_user_avatar(
    app: AppHandle,
    upload_filepath: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<String, errors::CommandError> {
    let ripple = app.state::<RippleApi>();
    let filepath = Path::new(&upload_filepath);
    let filename = crate::file_utils::FileUtils::get_file_name(filepath)
        .ok_or(anyhow::anyhow!("Invalid file path"))?;
    let crop_img = ImageProcessor::new().crop_image(
        filepath,
        x as u32,
        y as u32,
        width as u32,
        height as u32,
    )?;
    let res = ripple
        .upload_avatar(filename.to_string(), crop_img.0, crop_img.1)
        .await?;
    Ok(res.data.avatar_url)
}

#[tauri::command]
pub async fn update_user_nickname(
    nickname: String,
    state_ripple: State<'_, RippleApi>,
) -> Result<bool, errors::CommandError> {
    let response = state_ripple.update_nickname(nickname).await?;
    Ok(response.code == 200)
}

#[tauri::command]
pub async fn remove_user_avatar(
    state_ripple: State<'_, RippleApi>,
) -> Result<bool, errors::CommandError> {
    let response = state_ripple.delete_user_portrait().await?;
    Ok(response.code == 200)
}
