use crate::app_config::AppConfig;
use crate::file_utils::FileUtils;
use crate::image_processor::ImageProcessor;
use crate::ripple_api::api_response::{
    ReadMessagesData, RelationUser, SendMessageRequest, UserProfileData,
};
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::event_emitter::UIConversationItem;
use crate::ripple_syncer::DataSyncManager;
use crate::server::Server;
use crate::{errors, DefaultStoreEngine};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn exists_token(
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<bool, errors::CommandError> {
    Ok(data_sync.exists_token().await?)
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
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<UserProfileData, errors::CommandError> {
    data_sync.get_profile().await?.ok_or_else(|| {
        errors::CommandError::RippleAPIError(
            "get_user_profile".to_string(),
            500,
            "User profile not initialized. Please call preload_global_data first.".to_string(),
        )
    })
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
    if res.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "upload_avatar".to_string(),
            res.code,
            res.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn update_user_nickname(
    nickname: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.update_nickname(nickname).await?;
    if response.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "update_nickname".to_string(),
            response.code,
            response.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_user_avatar(
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.delete_user_avatar().await?;
    if response.code == 200 {
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "delete_user_avatar".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn add_friend(
    target_user_id: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.add_friend(target_user_id).await?;
    if response.code == 200 {
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "add_friend".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn remove_friend(
    friend_id: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.remove_friend(friend_id).await?;
    if response.code == 200 {
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "remove_friend".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn update_friend_display_name(
    friend_id: String,
    remark_name: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple
        .update_friend_remark_name(friend_id, remark_name.clone())
        .await?;
    if response.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "update_friend_display_name".to_string(),
            response.code,
            response.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn block_user(
    target_user_id: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.block_user(target_user_id).await?;
    if response.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "block_user".to_string(),
            response.code,
            response.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn unblock_user(
    target_user_id: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.unblock_user(target_user_id).await?;
    if response.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "unblock_user".to_string(),
            response.code,
            response.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_blocked_user(
    target_user_id: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = state_ripple.hide_blocked_user(target_user_id).await?;
    if response.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "hide_blocked_user".to_string(),
            response.code,
            response.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_user_profile_by_id(
    user_id: String,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<UserProfileData, errors::CommandError> {
    let response = state_ripple
        .get_user_profile_by_id(user_id.parse().unwrap())
        .await?;
    if response.code == 200 {
        Ok(response.data)
    } else {
        Err(errors::CommandError::RippleAPIError(
            "get_user_profile_by_id".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct UIRelationData {
    pub relations: Vec<RelationUser>,
}

#[tauri::command]
pub async fn get_relations(
    sync_manager: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<UIRelationData, errors::CommandError> {
    let relations = sync_manager.get_relations().await?;
    Ok(UIRelationData { relations })
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIConversations {
    pub conversation: Vec<UIConversationItem>,
}
#[tauri::command]
pub async fn get_conversations(
    sync_manager: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<UIConversations, errors::CommandError> {
    let mut conversations = Vec::<UIConversationItem>::new();
    let cached_conversations = sync_manager.get_conversations().await?;

    match sync_manager.get_profile().await? {
        Some(_profile) => {
            for conversation in cached_conversations {
                let ui_conversation: UIConversationItem = conversation.into();
                conversations.push(ui_conversation);
            }
            Ok(UIConversations {
                conversation: conversations,
            })
        }
        None => Err(errors::CommandError::AnyhowError(anyhow!(
            "User profile not found"
        ))),
    }
}

#[tauri::command]
pub async fn send_message(
    sender_id: String,
    conversation_id: String,
    receiver_id: String,
    text_content: Option<String>,
    file_url: Option<String>,
    file_name: Option<String>,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<String, errors::CommandError> {
    let request = SendMessageRequest {
        sender_id,
        conversation_id,
        receiver_id,
        text_content,
        file_url,
        file_name,
    };

    let response = state_ripple.send_message(request).await?;

    if response.code == 200 {
        if let Some(data) = response.data {
            Ok(data.message_id)
        } else {
            Err(errors::CommandError::RippleAPIError(
                "send_message".to_string(),
                500,
                "No message ID returned".to_string(),
            ))
        }
    } else {
        Err(errors::CommandError::RippleAPIError(
            "send_message".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn read_latest_messages(
    conversation_id: String,
    read_size: u32,
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<ReadMessagesData, errors::CommandError> {
    // API has a max limit of 200 messages per request
    let capped_read_size = read_size.min(200);
    println!(
        "[Commands] read_latest_messages: capping read_size {} -> {}",
        read_size, capped_read_size
    );

    let result = data_sync
        .read_latest_messages(conversation_id, capped_read_size)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn read_messages_before(
    conversation_id: String,
    before_message_id: String,
    read_size: u32,
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<ReadMessagesData, errors::CommandError> {
    // API has a max limit of 200 messages per request
    let capped_read_size = read_size.min(200);
    println!(
        "[Commands] read_messages_before: capping read_size {} -> {}",
        read_size, capped_read_size
    );

    let result = data_sync
        .read_messages_before(conversation_id, before_message_id, capped_read_size)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn mark_last_read_message_id(
    conversation_id: String,
    message_id: String,
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = data_sync
        .mark_last_read_message_id(conversation_id, message_id)
        .await?;

    if response.code == 200 {
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "mark_last_read_message_id".to_string(),
            response.code,
            response.message,
        ))
    }
}
