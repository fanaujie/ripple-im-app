use crate::ripple_api::api_response::RelationUser;
use crate::ripple_syncer::conversation_operation::{
    conversation_event_action, conversation_operation,
};
use crate::ripple_syncer::data_sync_manager::{ConversationSyncResult, RelationSyncResult};
use crate::ripple_syncer::event_emitter::{EventEmitter, UIConversationItem, UIMessageItem};
use crate::ripple_syncer::relation_operation::{relation_event_action, relation_operation};
use crate::ripple_syncer::sync_handler::RippleSyncHandler;
use crate::ripple_syncer::ui_event::{BLOCKED_FLAG, FRIEND_FLAG, HIDDEN_FLAG};
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
        match self.data_sync.process_relations_sync(true).await {
            Ok(Some(RelationSyncResult::FullSync { relations })) => {
                if let Err(e) = self.emitter.emit_relations_cleared() {
                    eprintln!(
                        "[IncrementalSyncManager] Failed to emit relations cleared: {}",
                        e
                    );
                }
                for user in relations {
                    let action = if (user.relation_flags & FRIEND_FLAG) != 0
                        && (user.relation_flags & BLOCKED_FLAG) == 0
                    {
                        relation_event_action::ADD_FRIEND
                    } else if (user.relation_flags & BLOCKED_FLAG) != 0
                        && (user.relation_flags & HIDDEN_FLAG) == 0
                    {
                        relation_event_action::ADD_BLOCK
                    } else {
                        // Skip users that don't match display criteria
                        continue;
                    };

                    if let Err(e) = self.emitter.emit_relation_updated(action, Some(user)) {
                        eprintln!("[IncrementalSyncManager] Failed to emit relation: {}", e);
                    }
                }
            }
            Ok(Some(RelationSyncResult::IncrementalSync { changes })) => {
                for (operation, user_id, user_data) in changes {
                    let action = match &user_data {
                        Some(user) => {
                            map_relation_operation_to_ui_action(operation, user.relation_flags)
                        }
                        None => map_relation_operation_to_ui_action(operation, 0),
                    };
                    let emit_user = user_data.or_else(|| {
                        Some(RelationUser {
                            user_id: user_id.clone(),
                            nick_name: String::new(),
                            avatar: None,
                            remark_name: String::new(),
                            relation_flags: 0,
                        })
                    });
                    if let Err(e) = self.emitter.emit_relation_updated(action, emit_user) {
                        eprintln!(
                            "[IncrementalSyncManager] Failed to emit relation event: {}",
                            e
                        );
                    }
                }
            }
            Ok(Some(RelationSyncResult::NoChange)) => {
                println!("[IncrementalSyncManager] No relation changes");
            }
            Ok(None) => {
                eprintln!("[IncrementalSyncManager] Unexpected None result");
            }
            Err(e) => {
                eprintln!("[IncrementalSyncManager] Failed to sync relations: {}", e);
            }
        }
    }

    async fn handle_message_update_sync(&self, push_req: PushMessageRequest) {
        let storage_message: StorageMessageData = (&push_req).into();
        self.data_sync.store_message(storage_message).await.unwrap();
        let message_item: UIMessageItem = push_req.into();
        if let Err(e) = self.emitter.emit_message_updated(0, Some(message_item)) {
            eprintln!("[IncrementalSyncManager] Failed to emit message: {}", e);
        }
        match self.data_sync.process_conversations_sync(true).await {
            Ok(Some(ConversationSyncResult::FullSync { conversations })) => {
                if let Err(e) = self.emitter.emit_conversations_cleared() {
                    eprintln!(
                        "[IncrementalSyncManager] Failed to emit conversations cleared: {}",
                        e
                    );
                }
                for conversation in conversations {
                    let ui_item: UIConversationItem = conversation.into();
                    if let Err(e) = self
                        .emitter
                        .emit_conversation_updated(conversation_event_action::CREATE, Some(ui_item))
                    {
                        eprintln!(
                            "[IncrementalSyncManager] Failed to emit conversation: {}",
                            e
                        );
                    }
                }
            }
            Ok(Some(ConversationSyncResult::IncrementalSync { changes })) => {
                for (operation, conversation_id, conversation_data) in changes {
                    if let Some(conversation) = conversation_data {
                        let ui_item: UIConversationItem = conversation.into();
                        let action = map_conversation_operation_to_ui_action(operation);

                        if let Err(e) = self
                            .emitter
                            .emit_conversation_updated(action, Some(ui_item))
                        {
                            eprintln!(
                                "[IncrementalSyncManager] Failed to emit conversation event: {}",
                                e
                            );
                        }
                    } else {
                        eprintln!(
                            "[IncrementalSyncManager] Conversation not found after update: {}",
                            conversation_id
                        );
                    }
                }
            }
            Ok(Some(ConversationSyncResult::NoChange)) => {
                println!("[IncrementalSyncManager] No conversation changes");
            }
            Ok(None) => {
                eprintln!("[IncrementalSyncManager] Unexpected None result");
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
        relation_operation::BLOCK_STRANGER => relation_event_action::ADD_BLOCK,
        _ => panic!(
            "[IncrementalSyncManager] Error: Unknown relation operation: {}, defaulting to UPDATE_FRIEND",
            operation
        ),
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
