use crate::ripple_api::api_response::RelationChange;
use crate::ripple_syncer::event_emitter::EventEmitter;
use crate::ripple_syncer::relation_operation::{
    relation_event_action, relation_operation, RelationAction,
};
use crate::ripple_syncer::sync_handler::RippleSyncHandler;
use crate::ripple_syncer::ui_event::{BLOCKED_FLAG, FRIEND_FLAG, HIDDEN_FLAG};
use crate::ripple_syncer::DataSyncManager;
use crate::store_engine::store_engine::RippleStorage;
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
            Ok(profile_data) => {
                if let Err(e) = self.emitter.emit_user_profile_updated(profile_data) {
                    eprintln!(
                        "[IncrementalSyncManager] Failed to emit user profile updated event: {}",
                        e
                    );
                } else {
                    println!("[IncrementalSyncManager] Successfully synced and emitted user profile update");
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
        let last_version = self
            .data_sync
            .store_engine()
            .get_relation_version()
            .await
            .unwrap();

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
                        Ok(relation_users) => {
                            println!(
                                "[IncrementalSyncManager] Full sync completed: {} relations",
                                relation_users.len()
                            );

                            for user in relation_users.iter() {
                                let action = if (user.relation_flags & BLOCKED_FLAG) != 0
                                    && (user.relation_flags & HIDDEN_FLAG) == 0
                                {
                                    relation_event_action::ADD_BLOCK
                                } else if (user.relation_flags & FRIEND_FLAG) != 0 {
                                    relation_event_action::ADD_FRIEND
                                } else {
                                    // Skip users with unexpected flags during full sync
                                    eprintln!(
                                        "[IncrementalSyncManager] Warning: Skipping user {} with unexpected flags: {}",
                                        user.user_id, user.relation_flags
                                    );
                                    continue;
                                };
                                if let Err(e) = self
                                    .emitter
                                    .emit_relation_updated(action, Some(user.clone()))
                                {
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
                        let action = to_relation_action(&change);
                        let result = self
                            .data_sync
                            .store_engine()
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
                        match self
                            .data_sync
                            .store_engine()
                            .get_relation(&change.user_id)
                            .await
                        {
                            Ok(Some(user)) => {
                                let action =
                                    map_operation_to_action(change.operation, user.relation_flags);
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
                                let action = map_operation_to_action(change.operation, 0);
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
}

fn to_relation_action(change: &RelationChange) -> RelationAction {
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

fn map_operation_to_action(operation: u64, relation_flags: i32) -> i32 {
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
        _ => -1, // Unknown operation
    }
}
