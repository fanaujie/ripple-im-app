use crate::ripple_api::api_response::{RelationUser, UserProfileData};
use crate::ripple_syncer::event_emitter::EventEmitter;
use crate::ripple_syncer::ui_event::{RelationsUpdateEvent, UIEvent};
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
}
