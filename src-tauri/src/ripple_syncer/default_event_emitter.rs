use crate::ripple_api::api_response::{RelationUser, UserGroupData, UserProfileData};
use crate::ripple_syncer::event_emitter::{EventEmitter, UIConversationItem, UIMessageItem};
use crate::ripple_syncer::ui_event::{
    ConversationReceivedMessageEvent, MessageUpdateEvent, UIEvent,
};
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
pub struct DefaultEventEmitter {
    app_handle: AppHandle,
}

impl DefaultEventEmitter {
    pub fn new(app_handle: AppHandle) -> Self {
        DefaultEventEmitter { app_handle }
    }
}

impl EventEmitter for DefaultEventEmitter {
    fn emit_user_profile_updated(&self, profile: UserProfileData) -> anyhow::Result<()> {
        println!("Emitting user profile updated event");
        self.app_handle
            .emit(UIEvent::UserProfileUpdated.to_string().as_str(), &profile)
            .map_err(|e| anyhow::anyhow!("Failed to emit user profile updated event: {}", e))
    }

    fn emit_relation_insert(&self, user: RelationUser) -> anyhow::Result<()> {
        println!("Emitting relation insert event");
        self.app_handle
            .emit(UIEvent::RelationInserted.to_string().as_str(), &user)
            .map_err(|e| anyhow::anyhow!("Failed to emit relation insert event: {}", e))
    }

    fn emit_relation_update(&self, user: RelationUser) -> anyhow::Result<()> {
        println!("Emitting relation updated event");
        self.app_handle
            .emit(UIEvent::RelationUpdated.to_string().as_str(), &user)
            .map_err(|e| anyhow::anyhow!("Failed to emit relation updated event: {}", e))
    }

    fn emit_relation_delete(&self, user_id: String) -> anyhow::Result<()> {
        println!("Emitting relation deleted event");
        self.app_handle
            .emit(UIEvent::RelationDeleted.to_string().as_str(), &user_id)
            .map_err(|e| anyhow::anyhow!("Failed to emit relation deleted event: {}", e))
    }

    fn emit_relations_clear_all(&self) -> anyhow::Result<()> {
        println!("Emitting relations cleared event");
        self.app_handle
            .emit(UIEvent::RelationClearedAll.to_string().as_str(), &())
            .map_err(|e| anyhow::anyhow!("Failed to emit relations cleared event: {}", e))
    }

    fn emit_conversation_insert(&self, conversation: UIConversationItem) -> anyhow::Result<()> {
        println!("Emitting conversation insert event");
        self.app_handle
            .emit(
                UIEvent::ConversationInserted.to_string().as_str(),
                &conversation,
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit conversation insert event: {}", e))
    }

    fn emit_conversation_update(&self, conversation: UIConversationItem) -> anyhow::Result<()> {
        println!("Emitting conversation update event");
        self.app_handle
            .emit(
                UIEvent::ConversationUpdated.to_string().as_str(),
                conversation,
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit conversation update event: {}", e))
    }

    fn emit_conversation_delete(&self, conversation_id: String) -> anyhow::Result<()> {
        println!("Emitting conversation delete event");
        self.app_handle
            .emit(
                UIEvent::ConversationsDeleted.to_string().as_str(),
                &conversation_id,
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit conversation delete event: {}", e))
    }

    fn emit_conversation_delete_all(&self) -> anyhow::Result<()> {
        println!("Emitting conversation delete all event");
        self.app_handle
            .emit(UIEvent::ConversationsClearedAll.to_string().as_str(), &())
            .map_err(|e| anyhow::anyhow!("Failed to emit conversation delete all event: {}", e))
    }

    fn emit_conversations_received(
        &self,
        conversation_id: String,
        unread_count: i32,
        message: String,
        timestamp: String,
    ) -> anyhow::Result<()> {
        println!("Emitting conversations received event");
        let event = ConversationReceivedMessageEvent {
            conversation_id,
            unread_count,
            message,
            timestamp,
        };
        self.app_handle
            .emit(
                UIEvent::ConversationReceivedNewMessage.to_string().as_str(),
                &event,
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit conversations received event: {}", e))
    }

    fn emit_message_updated(
        &self,
        action: i32,
        message: Option<UIMessageItem>,
    ) -> anyhow::Result<()> {
        println!("Emitting message updated event: action={}", action);
        let event = MessageUpdateEvent { action, message };
        self.app_handle
            .emit(UIEvent::MessageUpdated.to_string().as_str(), &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit message updated event: {}", e))
    }

    fn emit_messages_cleared(&self) -> anyhow::Result<()> {
        println!("Emitting messages cleared event");
        let event = MessageUpdateEvent {
            action: -1, // CLEAR action
            message: None,
        };
        self.app_handle
            .emit(UIEvent::MessageUpdated.to_string().as_str(), &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit messages cleared event: {}", e))
    }

    fn emit_user_group_insert(&self, group: UserGroupData) -> anyhow::Result<()> {
        println!("Emitting user group insert event");
        self.app_handle
            .emit(UIEvent::UserGroupInserted.to_string().as_str(), &group)
            .map_err(|e| anyhow::anyhow!("Failed to emit user group insert event: {}", e))
    }

    fn emit_user_group_update(&self, group: UserGroupData) -> anyhow::Result<()> {
        println!("Emitting user group update event");
        self.app_handle
            .emit(UIEvent::UserGroupUpdated.to_string().as_str(), &group)
            .map_err(|e| anyhow::anyhow!("Failed to emit user group update event: {}", e))
    }

    fn emit_user_group_delete(&self, group_id: String) -> anyhow::Result<()> {
        println!("Emitting user group delete event");
        self.app_handle
            .emit(UIEvent::UserGroupDeleted.to_string().as_str(), &group_id)
            .map_err(|e| anyhow::anyhow!("Failed to emit user group delete event: {}", e))
    }

    fn emit_user_groups_clear_all(&self) -> anyhow::Result<()> {
        println!("Emitting user groups cleared all event");
        self.app_handle
            .emit(UIEvent::UserGroupsClearedAll.to_string().as_str(), &())
            .map_err(|e| anyhow::anyhow!("Failed to emit user groups cleared all event: {}", e))
    }
}
