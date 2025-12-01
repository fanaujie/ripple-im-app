use crate::ripple_ws::sync_aware_ws_message_handler::PushNotification;
use ripple_proto::ripple_pb::PushMessageRequest;

#[trait_variant::make(RippleSyncHandler: Send)]
pub trait SyncHandler: Sync + Clone + 'static {
    async fn handle_self_info_update_sync(&self, push_req: PushNotification);
    async fn handle_relations_update_sync(&self, push_req: PushNotification);
    async fn handle_conversation_update_sync(&self, push_req: PushNotification);
    async fn handle_message_update_sync(&self, push_req: PushMessageRequest);
}
