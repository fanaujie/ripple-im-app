use crate::ripple_syncer::data_sync_manager::{
    ConversationSyncResult, GroupMemberSyncResult, RelationSyncResult, UserGroupSyncResult,
};
use crate::ripple_syncer::event_emitter::{EventEmitter, UIMessageItem};

use crate::ripple_syncer::sync_handler::RippleSyncHandler;

use crate::ripple_api::api_response::{MessageCommandType, MessageItem, MessageItemType};
use crate::ripple_syncer::DataSyncManager;
use crate::ripple_ws::sync_aware_ws_message_handler::PushNotification;
use crate::store_engine::store_engine::RippleStorage;
use ripple_proto::ripple_pb::{push_message_request, PushMessageRequest};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico"];

/// Check if a file is an image based on its extension
fn is_image_file(file_name: &str) -> bool {
    if let Some(ext) = file_name.rsplit('.').next() {
        IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

#[derive(Clone)]
pub struct RippleWsSyncHandler<S, E>
where
    S: RippleStorage,
    E: EventEmitter,
{
    data_sync: DataSyncManager<S>,
    emitter: E,
}

impl<S, E> RippleWsSyncHandler<S, E>
where
    S: RippleStorage,
    E: EventEmitter,
{
    pub fn new(data_sync: DataSyncManager<S>, emitter: E) -> Self {
        RippleWsSyncHandler { data_sync, emitter }
    }
}

impl<S, E> RippleSyncHandler for RippleWsSyncHandler<S, E>
where
    S: RippleStorage,
    E: EventEmitter,
{
    async fn handle_self_info_update_sync(&self, push_notification: PushNotification) {
        println!(
            "[RippleWsSyncHandler] Handling self info update sync for user ID: {}",
            push_notification.send_user_id
        );
        match self.data_sync.sync_user_profile().await {
            Ok(()) => {
                if let Some(profile_data) = self.data_sync.get_profile().await.unwrap() {
                    println!(
                        "[RippleWsSyncHandler] User avatar URL: {}",
                        profile_data.avatar.as_ref().unwrap()
                    );
                    if let Err(e) = self.emitter.emit_user_profile_updated(profile_data.into()) {
                        eprintln!(
                            "[RippleWsSyncHandler] Failed to emit user profile updated event: {}",
                            e
                        );
                    } else {
                        println!("[RippleWsSyncHandler] Successfully synced and emitted user profile update");
                    };
                } else {
                    eprintln!("[RippleWsSyncHandler] Failed to retrieve user profile after sync");
                }
            }
            Err(e) => {
                eprintln!("[RippleWsSyncHandler] Failed to sync user profile: {}", e);
            }
        }
    }

    async fn handle_relations_update_sync(&self, push_notification: PushNotification) {
        println!(
            "[RippleWsSyncHandler] Handling relation update sync for user ID: {}",
            push_notification.send_user_id
        );
        if let Ok(result) = self.data_sync.process_relations_sync(true).await {
            match result {
                Some(RelationSyncResult::FullSync { relations }) => {
                    self.emitter.emit_relations_clear_all().unwrap_or_else(|e| {
                        eprintln!(
                            "[RippleWsSyncHandler] Failed to emit relations clear all event: {}",
                            e
                        );
                    });
                    for user in relations.users {
                        self.emitter.emit_relation_insert(user).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit relation insert event: {}",
                                e
                            );
                        });
                    }
                }
                Some(RelationSyncResult::IncrementalSync {
                    insert,
                    update,
                    delete,
                }) => {
                    for user in insert {
                        self.emitter.emit_relation_insert(user).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit relation insert event: {}",
                                e
                            );
                        });
                    }
                    for user in update {
                        self.emitter.emit_relation_update(user).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit relation update event: {}",
                                e
                            );
                        });
                    }
                    for user_id in delete {
                        self.emitter
                            .emit_relation_delete(user_id)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                "[RippleWsSyncHandler] Failed to emit relation delete event: {}",
                                e
                            );
                            });
                    }
                }
                Some(RelationSyncResult::NoChange) | None => {
                    println!("[RippleWsSyncHandler] No relation changes");
                }
            }
        } else {
            eprintln!("[RippleWsSyncHandler] Failed to sync relations");
        }
    }

    async fn handle_conversation_update_sync(&self, push_notification: PushNotification) {
        println!(
            "[RippleWsSyncHandler] Handling conversation update sync for user ID: {}",
            push_notification.send_user_id
        );
        self.handle_conversation_sync().await;
    }

    async fn handle_message_update_sync(&self, push_req: PushMessageRequest) {
        let storage_message: MessageItem = (&push_req).into();

        // Extract unread_count from the push message payload
        let unread_count = match &push_req.payload {
            Some(push_message_request::Payload::MessagePayload(msg_payload)) => {
                msg_payload.unread_count
            }
            _ => 0,
        };

        println!(
            "[RippleWsSyncHandler] handle_message_update_sync: message_type={:?}, command_type={:?}, conversation_id={}, unread_count={}",
            storage_message.message_type, storage_message.command_type, storage_message.conversation_id, unread_count
        );
        match self
            .data_sync
            .conversation_exists(storage_message.conversation_id.as_str())
            .await
        {
            Ok(exists) => {
                if !exists {
                    println!(
                        "[RippleWsSyncHandler] Conversation does not exist for ID: {}, sync new conversation",
                        storage_message.conversation_id.as_str()
                    );
                    self.handle_conversation_sync().await;
                }
            }
            Err(e) => {
                eprintln!(
                    "[RippleWsSyncHandler] Failed to check conversation existence: {}",
                    e
                );
            }
        }

        // Handle GroupCommandMessageContent for group sync
        if storage_message.message_type == MessageItemType::Command {
            self.handle_group_command(&push_req, &storage_message).await;
        }

        let message = match storage_message.message_type {
            MessageItemType::Text => storage_message.text.clone().unwrap_or_default(),
            MessageItemType::Command => storage_message.command_data.clone().unwrap_or_default(),
            _ => String::new(),
        };

        if let Err(e) = self.emitter.emit_conversations_received(
            storage_message.conversation_id.clone(),
            unread_count,
            message,
            storage_message.send_timestamp.clone(),
        ) {
            eprintln!(
                "[RippleWsSyncHandler] Failed to emit conversation update: {}",
                e
            );
        }
        if let Err(e) = self.data_sync.store_message(storage_message).await {
            eprintln!(
                "[RippleWsSyncHandler] Failed to store message in cache: {}",
                e
            );
        }
        let ui_message: UIMessageItem = push_req.into();
        if let Err(e) = self.emitter.emit_message_updated(0, Some(ui_message)) {
            eprintln!(
                "[RippleWsSyncHandler] Failed to emit message updated: {}",
                e
            );
        }
    }
}

impl<S, E> RippleWsSyncHandler<S, E>
where
    E: EventEmitter,
    S: RippleStorage,
{
    async fn handle_conversation_sync(&self) {
        if let Ok(result) = self.data_sync.process_conversations_sync(true).await {
            match result {
                Some(ConversationSyncResult::FullSync { conversations }) => {
                    self.emitter.emit_conversation_delete_all().unwrap_or_else(|e| {
                        eprintln!(
                            "[RippleWsSyncHandler] Failed to emit conversation delete all event: {}",
                            e
                        );
                    });
                    for convo in conversations.conversations {
                        self.emitter.emit_conversation_insert(convo).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit conversation insert event: {}",
                                e
                            );
                        });
                    }
                }
                Some(ConversationSyncResult::IncrementalSync {
                    insert,
                    update,
                    delete,
                }) => {
                    for convo in insert.conversations {
                        self.emitter.emit_conversation_insert(convo).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit conversation insert event: {}",
                                e
                            );
                        });
                    }
                    for convo in update.conversations {
                        self.emitter.emit_conversation_update(convo).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit conversation update event: {}",
                                e
                            );
                        });
                    }
                    for convo_id in delete {
                        self.emitter.emit_conversation_delete(convo_id).unwrap_or_else(|e| {
                            eprintln!(
                                "[RippleWsSyncHandler] Failed to emit conversation delete event: {}",
                                e
                            );
                        });
                    }
                }
                Some(ConversationSyncResult::NoChange) | None => {
                    println!("[RippleWsSyncHandler] No conversation changes");
                }
            }
        } else {
            eprintln!("[RippleWsSyncHandler] Failed to sync conversations");
        }
    }

    async fn handle_group_command(
        &self,
        push_req: &PushMessageRequest,
        storage_message: &MessageItem,
    ) {
        let group_id = match &storage_message.group_id {
            Some(gid) => gid.clone(),
            None => {
                eprintln!("[RippleWsSyncHandler] GroupCommand message has no group_id");
                return;
            }
        };

        let current_user_id = push_req.receive_user_id.clone();
        let is_self_leaving = matches!(storage_message.command_type, MessageCommandType::GroupQuit)
            && storage_message.sender_id == current_user_id;

        println!(
            "[RippleWsSyncHandler] Handling group command: {:?} for group: {}, sender: {}, current_user: {}, is_self_leaving: {}",
            storage_message.command_type, &group_id, &storage_message.sender_id, &current_user_id, is_self_leaving
        );

        match storage_message.command_type {
            MessageCommandType::GroupQuit if is_self_leaving => {
                // Current user left the group: sync conversations (to remove the group conversation)
                // and clear group_members cache. Do NOT call process_group_members_sync (will return 403).
                println!("[RippleWsSyncHandler] Self left group, syncing conversations and clearing group members cache");
                self.handle_conversation_sync().await;
                if let Err(e) = self.data_sync.clear_group_members(&group_id).await {
                    eprintln!(
                        "[RippleWsSyncHandler] Failed to clear group members cache: {}",
                        e
                    );
                }
            }
            MessageCommandType::GroupJoin | MessageCommandType::GroupQuit => {
                // Other user joined or left: sync user_groups and group_members
                self.sync_user_groups_and_emit().await;
                let _ = self
                    .data_sync
                    .process_group_members_sync(&group_id, false)
                    .await;
            }
            MessageCommandType::InfoUpdate => {
                // Group info updated: sync conversation metadata and user_groups
                println!("[RippleWsSyncHandler] Group info updated, syncing conversations and user groups");
                self.handle_conversation_sync().await;
                self.sync_user_groups_and_emit().await;
                // Note: NO group_members sync - members unchanged
            }
            MessageCommandType::Empty | MessageCommandType::Unknown | _ => {
                // Unknown command types: no sync needed
                println!("[RippleWsSyncHandler] Unknown or empty command type, no sync performed");
            }
        }
    }

    async fn sync_user_groups_and_emit(&self) {
        if let Ok(result) = self.data_sync.process_user_groups_sync(true).await {
            match result {
                Some(UserGroupSyncResult::FullSync { groups }) => {
                    self.emitter
                        .emit_user_groups_clear_all()
                        .unwrap_or_else(|e| {
                            eprintln!(
                            "[RippleWsSyncHandler] Failed to emit user groups clear all event: {}",
                            e
                        );
                        });
                    for group in groups {
                        self.emitter
                            .emit_user_group_insert(group)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                "[RippleWsSyncHandler] Failed to emit user group insert event: {}",
                                e
                            );
                            });
                    }
                }
                Some(UserGroupSyncResult::IncrementalSync {
                    insert,
                    update,
                    delete,
                }) => {
                    for group in insert {
                        self.emitter
                            .emit_user_group_insert(group)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                "[RippleWsSyncHandler] Failed to emit user group insert event: {}",
                                e
                            );
                            });
                    }
                    for group in update {
                        self.emitter
                            .emit_user_group_update(group)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                "[RippleWsSyncHandler] Failed to emit user group update event: {}",
                                e
                            );
                            });
                    }
                    for group_id in delete {
                        self.emitter
                            .emit_user_group_delete(group_id)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                "[RippleWsSyncHandler] Failed to emit user group delete event: {}",
                                e
                            );
                            });
                    }
                }
                Some(UserGroupSyncResult::NoChange) | None => {
                    println!("[RippleWsSyncHandler] No user group changes");
                }
            }
        } else {
            eprintln!("[RippleWsSyncHandler] Failed to sync user groups");
        }
    }
}
