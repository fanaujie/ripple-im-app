use crate::ripple_api::api_response::{ConversationChange, MessageItem, RelationChange};
use crate::ripple_syncer::conversation_operation::{
    conversation_event_action, conversation_operation, ConversationAction,
};
use crate::ripple_syncer::event_emitter::{EventEmitter, UIConversationItem, UIMessageItem};
use crate::ripple_syncer::relation_operation::{
    relation_event_action, relation_operation, RelationAction,
};
use crate::ripple_syncer::sync_handler::RippleSyncHandler;
use crate::ripple_syncer::ui_event::FRIEND_FLAG;
use crate::ripple_syncer::DataSyncManager;
use crate::store_engine::store_engine::{RippleStorage, StorageMessageData};
use ripple_proto::ripple_pb::PushMessageRequest;

#[derive(Clone)]
pub struct IncrementalSyncManager<S, E>
where
    S: RippleStorage,
    E: EventEmitter,
{
    data_sync: DataSyncManager<S>,
    emitter: E,
}

impl<S, E> IncrementalSyncManager<S, E>
where
    S: RippleStorage,
    E: EventEmitter,
{
    pub fn new(data_sync: DataSyncManager<S>, emitter: E) -> Self {
        IncrementalSyncManager { data_sync, emitter }
    }
}

impl<S, E> RippleSyncHandler for IncrementalSyncManager<S, E>
where
    S: RippleStorage,
    E: EventEmitter,
{
    async fn handle_self_info_update_sync(&self, push_req: PushMessageRequest) {
        println!(
            "[IncrementalSyncManager] Handling self info update sync for user ID: {}",
            push_req.send_user_id
        );
        match self.data_sync.sync_user_profile().await {
            Ok(()) => {
                if let Some(profile_data) = self.data_sync.get_profile().await.unwrap() {
                    println!(
                        "[IncrementalSyncManager] User avatar URL: {}",
                        profile_data.avatar.as_ref().unwrap()
                    );
                    if let Err(e) = self.emitter.emit_user_profile_updated(profile_data) {
                        eprintln!(
                            "[IncrementalSyncManager] Failed to emit user profile updated event: {}",
                            e
                        );
                    } else {
                        println!("[IncrementalSyncManager] Successfully synced and emitted user profile update");
                    };
                } else {
                    eprintln!(
                        "[IncrementalSyncManager] Failed to retrieve user profile after sync"
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "[IncrementalSyncManager] Failed to sync user profile: {}",
                    e
                );
            }
        }
    }

    async fn handle_relations_update_sync(&self, push_req: PushMessageRequest) {
        println!(
            "[IncrementalSyncManager] Handling relations update sync for user ID: {}",
            push_req.send_user_id
        );
        let last_version = self.data_sync.get_relation_version().await.unwrap();

        match self
            .data_sync
            .sync_relations_incremental(last_version)
            .await
        {
            Ok(sync_data) => {
                if sync_data.full_sync {
                    println!("[IncrementalSyncManager] Full sync required, fetching all relations");

                    // Clear frontend relations first
                    if let Err(e) = self.emitter.emit_relations_cleared() {
                        eprintln!(
                            "[IncrementalSyncManager] Failed to emit relations cleared event: {}",
                            e
                        );
                    }

                    match self.data_sync.sync_all_relations().await {
                        Ok(()) => {
                            let relation_users =
                                self.data_sync.get_relations().await.unwrap_or_default();
                            for user in relation_users.0.into_iter() {
                                if let Err(e) = self.emitter.emit_relation_updated(
                                    relation_event_action::ADD_FRIEND,
                                    Some(user),
                                ) {
                                    eprintln!(
                                        "[IncrementalSyncManager] Failed to emit relation add event: {}",
                                        e
                                    );
                                }
                            }
                            for user in relation_users.1.into_iter() {
                                if let Err(e) = self.emitter.emit_relation_updated(
                                    relation_event_action::ADD_BLOCK,
                                    Some(user.clone()),
                                ) {
                                    eprintln!(
                                        "[IncrementalSyncManager] Failed to emit relation add event: {}",
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[IncrementalSyncManager] Full sync failed: {}", e);
                        }
                    }
                } else {
                    println!(
                        "[IncrementalSyncManager] Processing {} incremental relation changes, changes {:#?}",
                        sync_data.changes.len(), &sync_data.changes
                    );

                    for change in sync_data.changes {
                        let action = to_relation_storage_action(&change);
                        let result = self
                            .data_sync
                            .apply_relation_action(action, change.version.clone())
                            .await;
                        if let Err(e) = result {
                            eprintln!(
                                "[IncrementalSyncManager] Error: Failed to process change for user {}: {}",
                                change.user_id, e
                            );
                            continue;
                        }
                        // Emit event to frontend with complete user data from storage
                        match self.data_sync.get_relation(&change.user_id).await {
                            Ok(Some(user)) => {
                                let action = map_relation_operation_to_ui_action(
                                    change.operation,
                                    user.relation_flags,
                                );
                                if let Err(e) = self
                                    .emitter
                                    .emit_relation_updated(action, Some(user.clone()))
                                {
                                    eprintln!(
                                        "[IncrementalSyncManager] Failed to emit relation event: {}",
                                        e
                                    );
                                }
                            }
                            Ok(None) => {
                                // User was deleted, create minimal user data with userId for frontend
                                use crate::ripple_api::api_response::RelationUser;
                                let minimal_user = RelationUser {
                                    user_id: change.user_id.clone(),
                                    nick_name: String::new(),
                                    avatar: None,
                                    remark_name: String::new(),
                                    relation_flags: 0,
                                };
                                let action =
                                    map_relation_operation_to_ui_action(change.operation, 0);
                                if let Err(e) = self
                                    .emitter
                                    .emit_relation_updated(action, Some(minimal_user))
                                {
                                    eprintln!(
                                            "[IncrementalSyncManager] Failed to emit relation delete event: {}",
                                            e
                                        );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "[IncrementalSyncManager] Failed to get relation from storage for user {}: {}",
                                    change.user_id, e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[IncrementalSyncManager] Failed to sync relations: {}", e);
            }
        }
    }

    async fn handle_message_update_sync(&self, push_req: PushMessageRequest) {
        let req = &push_req;
        let storage_message: StorageMessageData = req.into();
        self.data_sync
            .store_message(storage_message.clone())
            .await
            .unwrap();
        let message_item: UIMessageItem = push_req.into();
        println!(
            "[IncrementalSyncManager] Emitting message-updated event for msg_id={}, conv_id={}",
            message_item.message_id, message_item.conversation_id
        );

        if let Err(e) = self.emitter.emit_message_updated(0, Some(message_item)) {
            eprintln!(
                "[IncrementalSyncManager] Failed to emit message-updated event: {}",
                e
            );
        }

        // Get last known conversation version
        let last_version = self.data_sync.get_conversation_version().await.unwrap();

        match self
            .data_sync
            .sync_conversations_incremental(last_version)
            .await
        {
            Ok(sync_data) => {
                if sync_data.full_sync {
                    println!(
                        "[IncrementalSyncManager] Full conversation sync required, fetching all conversations"
                    );

                    // Clear frontend conversations first
                    if let Err(e) = self.emitter.emit_conversations_cleared() {
                        eprintln!(
                            "[IncrementalSyncManager] Failed to emit conversations cleared event: {}",
                            e
                        );
                    }

                    match self.data_sync.sync_all_conversations().await {
                        Ok(()) => {
                            let conversations =
                                self.data_sync.get_conversations().await.unwrap_or_default();

                            println!(
                                "[IncrementalSyncManager] Full conversation sync completed: {} conversations",
                                conversations.len()
                            );

                            for conversation in conversations {
                                let ui_conversation_item: UIConversationItem = conversation.into();
                                if let Err(e) = self.emitter.emit_conversation_updated(
                                    conversation_event_action::CREATE,
                                    Some(ui_conversation_item),
                                ) {
                                    eprintln!(
                                        "[IncrementalSyncManager] Failed to emit conversation create event: {}",
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[IncrementalSyncManager] Full conversation sync failed: {}",
                                e
                            );
                        }
                    }
                } else {
                    println!(
                        "[IncrementalSyncManager] Processing {} incremental conversation changes",
                        sync_data.changes.len()
                    );

                    // Get current user_id for unread count logic
                    let user_id = if let Ok(Some(profile)) = self.data_sync.get_profile().await {
                        profile.user_id.parse::<i64>().unwrap_or(0)
                    } else {
                        0
                    };

                    for change in sync_data.changes {
                        let operation = change.operation;
                        let conversation_id = change.conversation_id.clone();
                        let storage_action = to_conversation_storage_action(&change);

                        let result = self
                            .data_sync
                            .apply_conversation_action(
                                storage_action,
                                change.version.clone(),
                                user_id,
                            )
                            .await;

                        if let Err(e) = result {
                            eprintln!(
                                "[IncrementalSyncManager] Error: Failed to apply conversation change for {}: {}",
                                conversation_id, e
                            );
                            continue;
                        }
                        let ui_conversation_item: UIConversationItem = match self
                            .data_sync
                            .get_conversation(&conversation_id)
                            .await
                        {
                            Ok(Some(conv)) => conv.into(),
                            Ok(None) => {
                                eprintln!(
                                        "[IncrementalSyncManager] Conversation not found after update: {}",
                                        conversation_id
                                    );
                                continue;
                            }
                            Err(e) => {
                                eprintln!(
                                    "[IncrementalSyncManager] Error getting conversation: {}",
                                    e
                                );
                                continue;
                            }
                        };

                        let action = map_conversation_operation_to_ui_action(operation);
                        if let Err(e) = self
                            .emitter
                            .emit_conversation_updated(action, Some(ui_conversation_item))
                        {
                            eprintln!(
                                "[IncrementalSyncManager] Failed to emit conversation event: {}",
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[IncrementalSyncManager] Failed to sync conversations: {}",
                    e
                );
            }
        }
    }
}

fn to_relation_storage_action(change: &RelationChange) -> RelationAction {
    match change.operation {
        relation_operation::ADD_FRIEND => RelationAction::Upsert(change.into()),
        relation_operation::UPDATE_FRIEND_REMARK_NAME => RelationAction::UpdateRemarkName {
            user_id: change.user_id.clone(),
            remark_name: change.remark_name.clone().unwrap(),
        },
        relation_operation::DELETE_FRIEND => RelationAction::Delete {
            user_id: change.user_id.clone(),
        },
        relation_operation::ADD_BLOCK => RelationAction::UpdateFlags {
            user_id: change.user_id.clone(),
            flags: change.relation_flags,
        },
        relation_operation::DELETE_BLOCK => RelationAction::Delete {
            user_id: change.user_id.clone(),
        },
        relation_operation::UNBLOCK_RESTORE_FRIEND => RelationAction::UpdateFlags {
            user_id: change.user_id.clone(),
            flags: change.relation_flags,
        },
        relation_operation::HIDE_BLOCK => RelationAction::UpdateFlags {
            user_id: change.user_id.clone(),
            flags: change.relation_flags,
        },
        relation_operation::UPDATE_FRIEND_NICK_NAME => {
            if let Some(nick_name) = &change.nick_name {
                RelationAction::UpdateNickName {
                    user_id: change.user_id.clone(),
                    nick_name: nick_name.clone(),
                }
            } else {
                eprintln!(
                    "[IncrementalSyncManager] Error: UPDATE_FRIEND_NICK_NAME missing nick_name for user: {}, using empty string",
                    change.user_id
                );
                RelationAction::UpdateNickName {
                    user_id: change.user_id.clone(),
                    nick_name: String::new(),
                }
            }
        }
        relation_operation::UPDATE_FRIEND_AVATAR => RelationAction::UpdateAvatar {
            user_id: change.user_id.clone(),
            avatar: change.avatar.clone(),
        },
        relation_operation::ADD_STRANGER => RelationAction::Upsert(change.into()),
        relation_operation::UPDATE_FRIEND_INFO => RelationAction::UpdateNickNameAvatar {
            user_id: change.user_id.clone(),
            nick_name: change.nick_name.clone().unwrap(),
            avatar: change.avatar.clone(),
        },
        _ => {
            eprintln!(
                "[IncrementalSyncManager] Error: Unknown operation type: {}, using no-op",
                change.operation
            );
            // Return a no-op action that won't crash
            RelationAction::UpdateAvatar {
                user_id: change.user_id.clone(),
                avatar: None,
            }
        }
    }
}

fn map_relation_operation_to_ui_action(operation: u64, relation_flags: i32) -> i32 {
    let is_friend = (relation_flags & FRIEND_FLAG) != 0;

    match operation {
        relation_operation::ADD_FRIEND => relation_event_action::ADD_FRIEND,
        relation_operation::DELETE_FRIEND => relation_event_action::REMOVE_FRIEND,
        relation_operation::ADD_BLOCK => {
            if is_friend {
                relation_event_action::BLOCK_FRIEND
            } else {
                relation_event_action::ADD_BLOCK
            }
        }
        relation_operation::DELETE_BLOCK => {
            if is_friend {
                relation_event_action::UNBLOCK_TO_FRIEND
            } else {
                relation_event_action::REMOVE_BLOCK
            }
        }
        relation_operation::UNBLOCK_RESTORE_FRIEND => relation_event_action::UNBLOCK_TO_FRIEND,
        relation_operation::HIDE_BLOCK => relation_event_action::REMOVE_BLOCK,
        relation_operation::UPDATE_FRIEND_REMARK_NAME
        | relation_operation::UPDATE_FRIEND_NICK_NAME
        | relation_operation::UPDATE_FRIEND_AVATAR
        | relation_operation::UPDATE_FRIEND_INFO => relation_event_action::UPDATE_FRIEND,
        _ => panic!(
            "[IncrementalSyncManager] Error: Unknown relation operation: {}, defaulting to UPDATE_FRIEND",
            operation
        ),
    }
}

fn to_conversation_storage_action(change: &ConversationChange) -> ConversationAction {
    match change.operation {
        conversation_operation::CREATE => ConversationAction::Upsert(change.into()),
        conversation_operation::NEW_MESSAGE => ConversationAction::Upsert(change.into()),
        conversation_operation::READ_MESSAGE => ConversationAction::UpdateReadStatus {
            conversation_id: change.conversation_id.clone(),
            last_read_message_id: change
                .last_read_message_id
                .as_ref()
                .and_then(|id_str| id_str.parse::<i64>().ok()),
        },
        conversation_operation::DELETE => ConversationAction::Delete {
            conversation_id: change.conversation_id.clone(),
        },
        _ => {
            panic!(
                "[IncrementalSyncManager] Error: Unknown conversation operation: {}, defaulting to full upsert",
                change.operation
            );
        }
    }
}

fn map_conversation_operation_to_ui_action(operation: i32) -> i32 {
    match operation {
        conversation_operation::CREATE => conversation_event_action::CREATE,
        conversation_operation::NEW_MESSAGE => conversation_event_action::NEW_MESSAGE,
        conversation_operation::READ_MESSAGE => conversation_event_action::READ_MESSAGE,
        conversation_operation::DELETE => conversation_event_action::DELETE,
        _ => {
            panic!(
                "[IncrementalSyncManager] Error: Unknown conversation operation: {}, defaulting to NEW_MESSAGE",
                operation
            );
        }
    }
}
