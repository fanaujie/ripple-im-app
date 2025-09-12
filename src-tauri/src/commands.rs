use crate::app_config::AppConfig;
use std::path::Path;

use crate::file_utils::FileUtils;
use crate::image_processor::ImageProcessor;
use crate::ripple::api_response::UserProfileData;
use crate::ripple::RippleApi;
use crate::server::Server;
use crate::store_engine::StoreEngine;
use crate::{errors, DefaultStoreEngine};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn exists_token(
    state_ripple: State<'_, DefaultStoreEngine>,
) -> Result<bool, errors::CommandError> {
    Ok(state_ripple.exists_token().await?)
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
    let ripple = app.state::<RippleApi<DefaultStoreEngine>>();
    Ok(app
        .opener()
        .open_url(ripple.oauth_auth_url(), None::<&str>)?)
}

#[tauri::command]
pub async fn get_user_profile(
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<UserProfileData, errors::CommandError> {
    let response = state_ripple.get_user_profile().await?;
    if response.code == 200 {
        Ok(response.data)
    } else {
        Err(errors::CommandError::RippleAPIError(
            "get_user_profile".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn update_user_avatar(
    app: AppHandle,
    upload_filepath: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), errors::CommandError> {
    let ripple = app.state::<RippleApi<DefaultStoreEngine>>();
    let filepath = Path::new(&upload_filepath);
    let filename =
        FileUtils::get_file_name(filepath).ok_or(anyhow::anyhow!("Invalid file path"))?;
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
    if res.code == 200 {
        // Emit updated user profile
        let profile_response = ripple.get_user_profile().await?;
        let _ = app.emit("user-profile-updated", &profile_response.data);
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "upload_avatar".to_string(),
            res.code,
            res.message,
        ))
    }
}

#[tauri::command]
pub async fn update_user_nickname(
    app: AppHandle,
    nickname: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.update_nickname(nickname).await?;
    if response.code == 200 {
        // Emit updated user profile
        let profile_response = state_ripple.get_user_profile().await?;
        let _ = app.emit("user-profile-updated", &profile_response.data);
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "update_nickname".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn remove_user_avatar(
    app: AppHandle,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.delete_user_portrait().await?;
    if response.code == 200 {
        // Emit updated user profile
        let profile_response = state_ripple.get_user_profile().await?;
        let _ = app.emit("user-profile-updated", &profile_response.data);
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "delete_user_portrait".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn send_friend_request(
    account: String,
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<bool, errors::CommandError> {
    // Mock implementation - always return success
    println!("Mock: Send friend request to {}", account);
    Ok(true)
}

#[tauri::command]
pub async fn handle_friend_request(
    request_id: String,
    accept: bool,
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<bool, errors::CommandError> {
    // Mock implementation - always return success
    println!(
        "Mock: Handle friend request {} - accept: {}",
        request_id, accept
    );
    Ok(true)
}

#[tauri::command]
pub async fn get_friend_requests(
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<serde_json::Value, errors::CommandError> {
    // Mock friend requests data
    let mock_requests = serde_json::json!([
        {
            "id": "req1",
            "fromAccount": "dave@example.com",
            "toAccount": "me@example.com",
            "fromNickName": "Dave",
            "fromAvatar": "https://via.placeholder.com/64",
            "status": "pending",
            "createdAt": "2024-01-15T10:30:00Z"
        },
        {
            "id": "req2",
            "fromAccount": "eve@example.com",
            "toAccount": "me@example.com",
            "fromNickName": "Eve",
            "fromAvatar": null,
            "status": "pending",
            "createdAt": "2024-01-14T15:20:00Z"
        }
    ]);
    Ok(mock_requests)
}

#[tauri::command]
pub async fn get_sent_requests(
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<serde_json::Value, errors::CommandError> {
    // Mock sent requests data
    let mock_sent_requests = serde_json::json!([
        {
            "id": "sent1",
            "fromAccount": "me@example.com",
            "toAccount": "frank@example.com",
            "fromNickName": "Me",
            "fromAvatar": "https://via.placeholder.com/64",
            "status": "pending",
            "createdAt": "2024-01-16T09:00:00Z"
        }
    ]);
    Ok(mock_sent_requests)
}

#[tauri::command]
pub async fn get_friends_list(
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<serde_json::Value, errors::CommandError> {
    // Mock friends list data
    let mock_friends = serde_json::json!([
        {
            "account": "alice@example.com",
            "nickName": "Alice",
            "avatar": "https://via.placeholder.com/64"
        },
        {
            "account": "bob@example.com",
            "nickName": "Bob",
            "avatar": null
        },
        {
            "account": "charlie@example.com",
            "nickName": "Charlie",
            "avatar": "https://via.placeholder.com/64"
        }
    ]);
    Ok(mock_friends)
}

#[tauri::command]
pub async fn remove_friend(
    friend_account: String,
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<bool, errors::CommandError> {
    // Mock implementation - always return success
    println!("Mock: Remove friend {}", friend_account);
    Ok(true)
}

#[tauri::command]
pub async fn search_friends(
    keyword: String,
    _state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<serde_json::Value, errors::CommandError> {
    // Mock search implementation - filter friends by keyword
    let all_friends = serde_json::json!([
        {
            "account": "alice@example.com",
            "nickName": "Alice",
            "avatar": "https://via.placeholder.com/64"
        },
        {
            "account": "bob@example.com",
            "nickName": "Bob",
            "avatar": null
        },
        {
            "account": "charlie@example.com",
            "nickName": "Charlie",
            "avatar": "https://via.placeholder.com/64"
        }
    ]);

    // Simple filtering based on keyword
    let filtered_friends: Vec<serde_json::Value> = all_friends
        .as_array()
        .unwrap()
        .iter()
        .filter(|friend| {
            friend["nickName"]
                .as_str()
                .unwrap()
                .to_lowercase()
                .contains(&keyword.to_lowercase())
        })
        .cloned()
        .collect();

    Ok(serde_json::json!(filtered_friends))
}
