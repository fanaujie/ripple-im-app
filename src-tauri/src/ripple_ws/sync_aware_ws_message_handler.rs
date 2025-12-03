use crate::ripple_syncer::sync_handler::RippleSyncHandler;
use crate::ripple_ws::syncer_control::SyncerControl;
use crate::ripple_ws::ws_message_handler::RippleWsMsgHandler;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use prost::Message as ProstMessage;
use ripple_proto::ripple_pb;
use ripple_proto::ripple_pb::ws_message::MessageType;
use ripple_proto::ripple_pb::PushMessageType;
use std::sync::Arc;
use tokio::sync::watch::Sender;
use tokio::sync::{watch, Mutex};
use tokio_tungstenite::tungstenite::Message;

pub struct PushNotification {
    pub push_message_type: PushMessageType,
    pub send_user_id: i64,
    pub receive_user_id: i64,
    pub request_device_id: String,
}

impl PushNotification {
    fn new(msg_type: i32, push_req: ripple_pb::PushMessageRequest) -> Self {
        PushNotification {
            push_message_type: PushMessageType::try_from(msg_type).unwrap(),
            send_user_id: push_req.send_user_id.parse().unwrap_or(0),
            receive_user_id: push_req.receive_user_id.parse().unwrap_or(0),
            request_device_id: push_req.request_device_id,
        }
    }
}

struct SyncAwareWsMessageHandlerInner {
    self_update_sender: Option<UnboundedSender<PushNotification>>,
    relation_update_sender: Option<UnboundedSender<PushNotification>>,
    conversation_update_sender: Option<UnboundedSender<PushNotification>>,
    message_update_sender: Option<UnboundedSender<ripple_pb::PushMessageRequest>>,
    watch_tx: Option<Sender<bool>>,
}

#[derive(Clone)]
pub struct SyncAwareWsMessageHandler<S>
where
    S: RippleSyncHandler,
{
    inner: Arc<Mutex<SyncAwareWsMessageHandlerInner>>,
    syncer: S,
}

impl<S> SyncAwareWsMessageHandler<S>
where
    S: RippleSyncHandler,
{
    pub fn new(syncer: S) -> SyncAwareWsMessageHandler<S> {
        SyncAwareWsMessageHandler {
            inner: Arc::new(Mutex::new(SyncAwareWsMessageHandlerInner {
                self_update_sender: None,
                relation_update_sender: None,
                conversation_update_sender: None,
                message_update_sender: None,
                watch_tx: None,
            })),
            syncer,
        }
    }

    fn spawn_self_update_handler(
        syncer: S,
        mut receiver: futures_channel::mpsc::UnboundedReceiver<PushNotification>,
        mut watch_rx: watch::Receiver<bool>,
    ) {
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::select! {
                    Some(push_req) = receiver.next() => {
                        syncer.handle_self_info_update_sync(push_req).await;
                    }
                    _ = watch_rx.changed() => {
                        if *watch_rx.borrow() == true {
                            break;
                        }
                    }
                }
            }
        });
    }

    fn spawn_relation_update_handler(
        syncer: S,
        mut receiver: futures_channel::mpsc::UnboundedReceiver<PushNotification>,
        mut watch_rx: watch::Receiver<bool>,
    ) {
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::select! {
                    Some(push_req) = receiver.next() => {
                        syncer.handle_relations_update_sync(push_req).await;
                    }
                    _ = watch_rx.changed() => {
                        if *watch_rx.borrow() == true {
                            break;
                        }
                    }
                }
            }
        });
    }

    fn spawn_conversation_update_handler(
        syncer: S,
        mut receiver: futures_channel::mpsc::UnboundedReceiver<PushNotification>,
        mut watch_rx: watch::Receiver<bool>,
    ) {
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::select! {
                    Some(push_req) = receiver.next() => {
                        syncer.handle_conversation_update_sync(push_req).await;
                    }
                    _ = watch_rx.changed() => {
                        if *watch_rx.borrow() == true {
                            break;
                        }
                    }
                }
            }
        });
    }

    fn spawn_message_update_handler(
        syncer: S,
        mut receiver: futures_channel::mpsc::UnboundedReceiver<ripple_pb::PushMessageRequest>,
        mut watch_rx: watch::Receiver<bool>,
    ) {
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::select! {
                    Some(push_req) = receiver.next() => {
                        syncer.handle_message_update_sync(push_req).await;
                    }
                    _ = watch_rx.changed() => {
                        if *watch_rx.borrow() == true {
                            break;
                        }
                    }
                }
            }
        });
    }
}

impl<S> SyncerControl for SyncAwareWsMessageHandler<S>
where
    S: RippleSyncHandler,
{
    async fn start_syncer(&self) -> anyhow::Result<()> {
        let (watch_tx, watch_rx) = watch::channel(false);
        let (self_update_sender, self_update_receiver) =
            futures_channel::mpsc::unbounded::<PushNotification>();
        let (relation_update_sender, relation_update_receiver) =
            futures_channel::mpsc::unbounded::<PushNotification>();
        let (conversation_update_sender, _conversation_update_receiver) =
            futures_channel::mpsc::unbounded::<PushNotification>();
        let (message_update_sender, message_update_receiver) =
            futures_channel::mpsc::unbounded::<ripple_pb::PushMessageRequest>();

        let mut inner = self.inner.lock().await;
        inner.watch_tx.replace(watch_tx);
        inner.self_update_sender.replace(self_update_sender);
        inner.relation_update_sender.replace(relation_update_sender);
        inner
            .conversation_update_sender
            .replace(conversation_update_sender);
        inner.message_update_sender.replace(message_update_sender);
        drop(inner);
        // Spawn handlers
        Self::spawn_self_update_handler(
            self.syncer.clone(),
            self_update_receiver,
            watch_rx.clone(),
        );
        Self::spawn_relation_update_handler(
            self.syncer.clone(),
            relation_update_receiver,
            watch_rx.clone(),
        );
        Self::spawn_conversation_update_handler(
            self.syncer.clone(),
            _conversation_update_receiver,
            watch_rx.clone(),
        );
        Self::spawn_message_update_handler(self.syncer.clone(), message_update_receiver, watch_rx);
        Ok(())
    }

    async fn stop_syncer(&self) -> anyhow::Result<()> {
        if let Some(tx) = &self.inner.lock().await.watch_tx {
            let _ = tx.send(true);
        }
        Ok(())
    }
}

impl<S> RippleWsMsgHandler for SyncAwareWsMessageHandler<S>
where
    S: RippleSyncHandler,
{
    async fn handle_message(
        &self,
        send_tx: &UnboundedSender<Message>,
        message: Message,
    ) -> anyhow::Result<()> {
        if message.is_binary() {
            if let Ok(ws) = ripple_pb::WsMessage::decode(message.into_data()) {
                match ws.message_type {
                    Some(MessageType::PushMessageRequest(push_message)) => {
                        for msg_type in &push_message.push_message_type {
                            println!("PushMessageType: {:?}", msg_type);
                            match ripple_pb::PushMessageType::try_from(*msg_type) {
                                Ok(ripple_pb::PushMessageType::SelfInfoUpdate) => {
                                    if let Some(sender) =
                                        &self.inner.lock().await.self_update_sender
                                    {
                                        let _ = sender.unbounded_send(PushNotification::new(
                                            *msg_type,
                                            push_message.clone(),
                                        ));
                                    }
                                }
                                Ok(ripple_pb::PushMessageType::RelationInfoUpdate) => {
                                    if let Some(sender) =
                                        &self.inner.lock().await.relation_update_sender
                                    {
                                        let _ = sender.unbounded_send(PushNotification::new(
                                            *msg_type,
                                            push_message.clone(),
                                        ));
                                    }
                                }
                                Ok(ripple_pb::PushMessageType::ConversationInfoUpdate) => {
                                    if let Some(sender) =
                                        &self.inner.lock().await.conversation_update_sender
                                    {
                                        let _ = sender.unbounded_send(PushNotification::new(
                                            *msg_type,
                                            push_message.clone(),
                                        ));
                                    }
                                }
                                Ok(ripple_pb::PushMessageType::SingleMessage) => {
                                    if let Some(sender) =
                                        &self.inner.lock().await.message_update_sender
                                    {
                                        let _ = sender.unbounded_send(push_message.clone());
                                    }
                                }
                                _ => {
                                    eprintln!("Unexpected PushMessageType: {}", *msg_type);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                println!("Failed to decode WsMessage");
            }
        } else if message.is_text() {
            let text = message.into_text().unwrap();
            println!("Received text message: {}", text);
        } else if message.is_close() {
            println!("Received close message");
        } else if message.is_ping() {
            println!("Received ping message");
        } else if message.is_pong() {
            println!("Received pong message");
        } else {
            println!("Received other type of message");
        }
        Ok(())
    }

    async fn notify_connect(&self) {
        println!("WebSocket connected.");
    }

    async fn notify_disconnect(&self) {
        println!("WebSocket disconnected.");
    }

    async fn notify_ws_stop(&self, err_msg: String) {
        eprintln!("WebSocket stopped: {}", err_msg);
    }
}
