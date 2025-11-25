use crate::ripple_api::api_response::{RelationUser, UserProfileData};
use crate::ripple_syncer::event_emitter::{EventEmitter, UIConversationItem, UIMessageItem};
use crate::ripple_syncer::ui_event::{
    ConversationUpdateEvent, MessageUpdateEvent, RelationsUpdateEvent, UIEvent,
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

    fn emit_relation_updated(&self, action: i32, user: Option<RelationUser>) -> anyhow::Result<()> {
        println!("Emitting relation updated event: action={}", action);
        let event = RelationsUpdateEvent {
            action,
            user_profile: user,
        };
        self.app_handle
            .emit(UIEvent::RelationUpdated.to_string().as_str(), &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit relation updated event: {}", e))
    }

    fn emit_relations_cleared(&self) -> anyhow::Result<()> {
        println!("Emitting relations cleared event");
        let event = RelationsUpdateEvent {
            action: -1, // CLEAR action
            user_profile: None,
        };
        self.app_handle
            .emit(UIEvent::RelationUpdated.to_string().as_str(), &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit relations cleared event: {}", e))
    }

    fn emit_conversation_updated(
        &self,
        action: i32,
        conversation: Option<UIConversationItem>,
    ) -> anyhow::Result<()> {
        println!("Emitting conversation updated event: action={}", action);
        let event = ConversationUpdateEvent {
            action,
            conversation,
        };
        self.app_handle
            .emit(UIEvent::ConversationUpdated.to_string().as_str(), &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit conversation updated event: {}", e))
    }

    fn emit_conversations_cleared(&self) -> anyhow::Result<()> {
        println!("Emitting conversations cleared event");
        let event = ConversationUpdateEvent {
            action: -1, // CLEAR action
            conversation: None,
        };
        self.app_handle
            .emit(UIEvent::ConversationUpdated.to_string().as_str(), &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit conversations cleared event: {}", e))
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
}
