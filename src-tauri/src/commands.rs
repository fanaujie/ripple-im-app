use crate::app_config::AppConfig;
use crate::file_utils::FileUtils;
use crate::ripple_api::api_response::{
    GroupMemberData, ReadMessagesData, RelationUsers, SendMessageRequest, UserProfileData,
};
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::event_emitter::UIConversations;
use crate::ripple_syncer::DataSyncManager;
use crate::server::Server;
use crate::{errors, DefaultStoreEngine, DefaultWsManager};
use anyhow::anyhow;
use serde::Serialize;
use sha2::{Digest, Sha256};
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
pub async fn resume_session(app: AppHandle) -> Result<(), errors::CommandError> {
    let data_sync = app.state::<DataSyncManager<DefaultStoreEngine>>();
    match data_sync.check_and_clear_on_user_change().await {
        Ok(cleared) => {
            if cleared {
                println!("[resume_session] User changed, all stored data has been cleared");
            }
        }
        Err(e) => {
            eprintln!("[resume_session] Failed to check user change: {}", e);
        }
    }
    data_sync.init().await?;
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let ws_manager = app_handle.state::<DefaultWsManager>();
        let config = app_handle.state::<AppConfig>();
        if let Err(e) = ws_manager.start(&config.ws_gateway_url).await {
            eprintln!("[resume_session] Failed to start WebSocket: {}", e);
        }
    });
    Ok(())
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
    let profile = data_sync.get_profile().await?;
    match profile {
        Some(profile_data) => Ok(profile_data.into()),
        None => Err(anyhow!("User profile not found").into()),
    }
}

#[tauri::command]
pub async fn upload_user_avatar_blob(
    app: AppHandle,
    image_data: String, // base64 encoded PNG
) -> Result<(), errors::CommandError> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use mime::IMAGE_PNG;

    let ripple = app.state::<RippleApi<DefaultStoreEngine>>();

    // Decode base64 to bytes
    let image_bytes = STANDARD
        .decode(&image_data)
        .map_err(|e| anyhow::anyhow!("Failed to decode base64: {}", e))?;

    let res = ripple
        .upload_avatar("avatar.png".to_string(), IMAGE_PNG, image_bytes)
        .await?;

    if res.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "upload_user_avatar_blob".to_string(),
            res.code,
            res.message,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn upload_image_blob(
    app: AppHandle,
    image_data: String, // base64 encoded PNG
) -> Result<String, errors::CommandError> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use mime::IMAGE_PNG;

    let ripple = app.state::<RippleApi<DefaultStoreEngine>>();

    // Decode base64 to bytes
    let image_bytes = STANDARD
        .decode(&image_data)
        .map_err(|e| anyhow::anyhow!("Failed to decode base64: {}", e))?;

    let res = ripple
        .upload_avatar("avatar.png".to_string(), IMAGE_PNG, image_bytes)
        .await?;

    if res.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "upload_image_blob".to_string(),
            res.code,
            res.message,
        ));
    }

    res.data.map(|d| d.url).ok_or_else(|| {
        errors::CommandError::RippleAPIError(
            "upload_image_blob".to_string(),
            500,
            "No URL returned".to_string(),
        )
    })
}

#[tauri::command]
pub async fn upload_group_avatar_blob(
    app: AppHandle,
    group_id: String,
    image_data: String, // base64 encoded PNG
) -> Result<(), errors::CommandError> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use mime::IMAGE_PNG;

    let ripple = app.state::<RippleApi<DefaultStoreEngine>>();

    // Decode base64 to bytes
    let image_bytes = STANDARD
        .decode(&image_data)
        .map_err(|e| anyhow::anyhow!("Failed to decode base64: {}", e))?;

    let res = ripple
        .upload_group_avatar(group_id, "avatar.png".to_string(), IMAGE_PNG, image_bytes)
        .await?;

    if res.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "upload_group_avatar_blob".to_string(),
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
    let response = state_ripple.update_profile(Some(nickname)).await?;
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
        .update_friend(friend_id, Some(remark_name))
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
    let response = state_ripple
        .update_blocked_user(target_user_id, Some(true))
        .await?;
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

#[tauri::command]
pub async fn get_relations(
    sync_manager: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<RelationUsers, errors::CommandError> {
    Ok(RelationUsers {
        users: sync_manager.get_relations().await?.into(),
    })
}

#[tauri::command]
pub async fn get_conversations(
    sync_manager: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<UIConversations, errors::CommandError> {
    Ok(sync_manager.get_conversations().await?.into())
}

#[tauri::command]
pub async fn send_message(
    sender_id: String,
    conversation_id: String,
    receiver_id: Option<String>,
    group_id: Option<String>,
    text: Option<String>,
    file_url: Option<String>,
    file_name: Option<String>,
    state_ripple: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<String, errors::CommandError> {
    let request = SendMessageRequest {
        sender_id,
        conversation_id,
        receiver_id,
        group_id,
        text_content: text,
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
    last_read_message_id: String,
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
) -> Result<ReadMessagesData, errors::CommandError> {
    // API has a max limit of 200 messages per request
    let capped_read_size = read_size.min(200);

    if !last_read_message_id.is_empty() {
        println!(
            "[Commands] read_latest_messages: conv={}, size={}, last_read={}",
            conversation_id, capped_read_size, last_read_message_id
        );
    } else {
        println!(
            "[Commands] read_latest_messages: conv={}, size={} (first visit)",
            conversation_id, capped_read_size
        );
    }

    let result = data_sync
        .read_latest_messages(conversation_id, capped_read_size, last_read_message_id)
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

#[tauri::command]
pub async fn create_group(
    sender_id: String,
    group_name: String,
    member_ids: Vec<String>,
    ripple_api: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<String, errors::CommandError> {
    let response = ripple_api
        .create_group(sender_id, group_name, member_ids)
        .await?;

    if response.code == 200 {
        if let Some(data) = response.data {
            Ok(data.group_id)
        } else {
            Err(errors::CommandError::RippleAPIError(
                "create_group".to_string(),
                response.code,
                "No group ID in response".to_string(),
            ))
        }
    } else {
        Err(errors::CommandError::RippleAPIError(
            "create_group".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn invite_members(
    group_id: String,
    sender_id: String,
    member_ids: Vec<String>,
    group_name: String,
    group_avatar: Option<String>,
    ripple_api: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = ripple_api
        .add_group_members(group_id, sender_id, member_ids, group_name, group_avatar)
        .await?;

    if response.code == 200 {
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "invite_members".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn get_group_members(
    group_id: String,
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
    ripple_api: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<Vec<GroupMemberData>, errors::CommandError> {
    // Check if members exist in local cache
    if data_sync.exist_group_members(&group_id).await? {
        println!(
            "[Commands] get_group_members: returning cached members for group {}",
            group_id
        );
        return Ok(data_sync.get_group_members(&group_id).await?);
    }

    // Fetch all members from API using pagination
    println!(
        "[Commands] get_group_members: fetching from API for group {}",
        group_id
    );
    let mut all_members: Vec<GroupMemberData> = Vec::new();
    let mut next_page_token: Option<String> = None;
    let page_size = 50;

    loop {
        let response = ripple_api
            .get_group_members(group_id.clone(), next_page_token.clone(), page_size)
            .await?;

        if response.code != 200 {
            return Err(errors::CommandError::RippleAPIError(
                "get_group_members".to_string(),
                response.code,
                response.message,
            ));
        }

        all_members.extend(response.data.members);

        if !response.data.has_more {
            break;
        }
        next_page_token = response.data.next_page_token;
    }

    // Store members in local cache
    data_sync
        .store_group_members(&group_id, all_members.clone())
        .await?;

    Ok(all_members)
}

#[tauri::command]
pub async fn update_group_name(
    group_id: String,
    sender_id: String,
    group_name: String,
    ripple_api: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = ripple_api
        .update_group(group_id, sender_id, Some(group_name))
        .await?;
    if response.code == 200 {
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "update_group_name".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[tauri::command]
pub async fn leave_group(
    group_id: String,
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
    ripple_api: State<'_, RippleApi<DefaultStoreEngine>>,
) -> Result<(), errors::CommandError> {
    let response = ripple_api.leave_group(group_id.clone()).await?;

    if response.code == 200 {
        // Clear group members cache since we can no longer access this group's member list
        if let Err(e) = data_sync.clear_group_members(&group_id).await {
            eprintln!(
                "[Commands] leave_group: failed to clear group members cache: {}",
                e
            );
        } else {
            println!(
                "[Commands] leave_group: cleared group members cache for group {}",
                group_id
            );
        }
        Ok(())
    } else {
        Err(errors::CommandError::RippleAPIError(
            "leave_group".to_string(),
            response.code,
            response.message,
        ))
    }
}

#[derive(Clone, Serialize)]
pub struct UploadAttachmentResponse {
    pub file_url: String,
}

#[tauri::command]
pub async fn upload_attachment(
    app: AppHandle,
    file_path: String,
) -> Result<UploadAttachmentResponse, errors::CommandError> {
    let ripple = app.state::<RippleApi<DefaultStoreEngine>>();
    let filepath = Path::new(&file_path);

    let file_data = std::fs::read(filepath).map_err(|e| anyhow!("Failed to read file: {}", e))?;
    let file_size = file_data.len() as i64;

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let file_sha256 = format!("{:x}", hasher.finalize());

    let original_filename =
        FileUtils::get_file_name(filepath).ok_or(anyhow!("Invalid file path"))?;

    let init_response = ripple
        .initiate_attachment_upload(
            file_size,
            file_sha256.clone(),
            original_filename.to_string(),
        )
        .await?;

    if init_response.code != 200 {
        return Err(errors::CommandError::RippleAPIError(
            "initiate_attachment_upload".to_string(),
            init_response.code,
            init_response.message,
        ));
    }

    let init_data = init_response
        .data
        .ok_or(anyhow!("No data in initiate response"))?;

    match init_data.upload_mode {
        // Mode 0: File already exists
        0 => {
            let file_url = init_data
                .file_url
                .ok_or(anyhow!("No file_url in mode 0 response"))?;
            Ok(UploadAttachmentResponse { file_url })
        }

        // Mode 1: Single upload (<5MB)
        1 => {
            let object_name = init_data
                .object_name
                .ok_or(anyhow!("No object_name in mode 1 response"))?;

            let upload_response = ripple
                .upload_attachment_single(
                    object_name,
                    file_sha256,
                    file_data,
                    original_filename.to_string(),
                )
                .await?;

            if upload_response.code != 200 {
                return Err(errors::CommandError::RippleAPIError(
                    "upload_attachment_single".to_string(),
                    upload_response.code,
                    upload_response.message,
                ));
            }

            let file_url = upload_response
                .data
                .ok_or(anyhow!("No data in single upload response"))?
                .file_url;
            Ok(UploadAttachmentResponse { file_url })
        }

        2 => {
            let object_name = init_data
                .object_name
                .ok_or(anyhow!("No object_name in mode 2 response"))?;
            let chunk_size = init_data
                .chunk_size
                .ok_or(anyhow!("No chunk_size in mode 2 response"))?
                as usize;
            let total_chunks = init_data
                .total_chunks
                .ok_or(anyhow!("No total_chunks in mode 2 response"))?;
            // API uses 1-based chunk numbering
            let start_chunk = match init_data.start_chunk_number {
                Some(n) => n,
                None => panic!("No start_chunk_number in mode 2 response"),
            };
            // Upload each chunk (1-based indexing: 1..=total_chunks)
            for chunk_number in start_chunk..=total_chunks {
                let chunk_start = ((chunk_number - 1) as usize) * chunk_size;
                let chunk_end = std::cmp::min(chunk_start + chunk_size, file_data.len());
                let chunk_data = file_data[chunk_start..chunk_end].to_vec();

                // Compute chunk SHA256
                let mut chunk_hasher = Sha256::new();
                chunk_hasher.update(&chunk_data);
                let chunk_sha256 = format!("{:x}", chunk_hasher.finalize());

                let chunk_response = ripple
                    .upload_attachment_chunk(
                        object_name.clone(),
                        chunk_number,
                        chunk_sha256,
                        chunk_data,
                    )
                    .await?;

                if chunk_response.code != 200 {
                    // Abort upload on failure
                    let _ = ripple.abort_attachment_upload(object_name.clone()).await;
                    return Err(errors::CommandError::RippleAPIError(
                        "upload_attachment_chunk".to_string(),
                        chunk_response.code,
                        chunk_response.message,
                    ));
                }
            }

            let complete_response = ripple
                .complete_attachment_upload(object_name, total_chunks)
                .await?;

            if complete_response.code != 200 {
                return Err(errors::CommandError::RippleAPIError(
                    "complete_attachment_upload".to_string(),
                    complete_response.code,
                    complete_response.message,
                ));
            }

            let file_url = complete_response
                .data
                .ok_or(anyhow!("No data in complete upload response"))?
                .file_url;
            Ok(UploadAttachmentResponse { file_url })
        }

        _ => Err(errors::CommandError::RippleAPIError(
            "upload_attachment".to_string(),
            400,
            format!("Unknown upload mode: {}", init_data.upload_mode),
        )),
    }
}

#[tauri::command]
pub async fn logout(
    data_sync: State<'_, DataSyncManager<DefaultStoreEngine>>,
    ws_manager: State<'_, crate::DefaultWsManager>,
) -> Result<(), errors::CommandError> {
    // Stop WebSocket connection (ignore errors if already stopped)
    let _ = ws_manager.stop().await;

    // Clear token from storage
    data_sync.clear_token().await?;

    Ok(())
}
