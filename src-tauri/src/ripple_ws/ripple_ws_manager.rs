use crate::ripple_api::auth_token_parser::AuthTokenParser;
use crate::ripple_syncer::DataSyncManager;
use crate::ripple_ws::syncer_control::SyncerControl;
use crate::ripple_ws::ws_message_handler::RippleWsMsgHandler;
use crate::ripple_ws::ws_utils::WsUtilsHeartbeatRequest;
use crate::DefaultStoreEngine;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use futures_channel::mpsc::UnboundedSender;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

const HEADER_AUTHORIZATION: &'static str = "Authorization";
const HEADER_RIPPLE_DEVICE_ID: &'static str = "Ripple-Device-ID";

pub struct RippleWsManager<R>
where
    R: RippleWsMsgHandler + SyncerControl,
{
    message_handler: R,
    sender_tx: Arc<Mutex<Option<UnboundedSender<Message>>>>,
    is_running: AtomicBool,
    data_sync: DataSyncManager<DefaultStoreEngine>,
}

impl<R> RippleWsManager<R>
where
    R: RippleWsMsgHandler + SyncerControl,
{
    pub fn new(
        msg_handler: R,
        data_sync: DataSyncManager<DefaultStoreEngine>,
    ) -> RippleWsManager<R> {
        RippleWsManager {
            message_handler: msg_handler,
            sender_tx: Arc::new(Mutex::new(None)),
            is_running: AtomicBool::new(false),
            data_sync,
        }
    }
    pub async fn start(&self, ws_url: &str) -> anyhow::Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            anyhow::bail!("WebSocket manager is running");
        }
        let token = self.data_sync.get_token().await?;
        let claims = AuthTokenParser::decode_jwt_payload(&token.access_token)?;
        let user_id = claims.get_sub();
        let device_id = self.data_sync.get_device_id().await?;
        println!(
            "Ws client run: User ID: {} Device ID: {}",
            user_id, device_id
        );
        let mut request = ws_url.into_client_request()?;
        request.headers_mut().insert(
            HEADER_AUTHORIZATION,
            format!("Bearer {}", &token.access_token).parse()?,
        );
        request
            .headers_mut()
            .insert(HEADER_RIPPLE_DEVICE_ID, device_id.to_string().parse()?);
        let sender_tx_clone = self.sender_tx.clone();
        let msg_handler_clone = self.message_handler.clone();
        self.message_handler.start_syncer().await?;
        tauri::async_runtime::spawn(async move {
            let mut backoff = ExponentialBackoff::default();
            loop {
                let result = connect_async(request.clone()).await;
                match result {
                    Ok((ws_stream, _)) => {
                        backoff.reset();
                        let (mut ws_write, mut ws_read) = ws_stream.split();
                        // release previous sender if any
                        if let Some(mut older_sender) = sender_tx_clone.lock().await.take() {
                            let _ = older_sender.send(Message::Close(None)).await;
                        }

                        let (sender_tx, mut sender_rx) =
                            futures_channel::mpsc::unbounded::<Message>();
                        sender_tx_clone.lock().await.replace(sender_tx.clone());
                        let mut heartbeat_req = WsUtilsHeartbeatRequest::new(user_id.clone());
                        tauri::async_runtime::spawn(async move {
                            let mut heartbeat_interval =
                                tokio::time::interval(tokio::time::Duration::from_secs(15));
                            loop {
                                tokio::select! {
                                    Some(message) = sender_rx.next() => {
                                        let is_close = message.is_close();
                                        if let Err(e) = ws_write.send(message).await {
                                            eprintln!("WebSocket send error: {}", e);
                                            break;
                                        }
                                        if is_close {
                                            break;
                                        }
                                    }
                                    _ = heartbeat_interval.tick() => {
                                        let buf = heartbeat_req.get_heartbeat_request_buf();
                                        if let Err(e) = ws_write.send(Message::binary(buf)).await {
                                            eprintln!("WebSocket heartbeat error: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                        msg_handler_clone.notify_connect().await;
                        while let Some(message) = ws_read.next().await {
                            if message.is_err() {
                                eprintln!("WebSocket read error: {}", message.err().unwrap());
                                break;
                            }
                            let result = msg_handler_clone
                                .handle_message(&sender_tx, message.unwrap())
                                .await;
                            if result.is_err() {
                                eprintln!(
                                    "WebSocket message handling error: {}",
                                    result.err().unwrap()
                                );
                                return;
                            }
                        }
                        msg_handler_clone.notify_disconnect().await;
                    }
                    Err(e) => {
                        eprintln!("WebSocket connection error: {}", e);
                        if let Some(duration) = backoff.next_backoff() {
                            tokio::time::sleep(duration).await;
                        } else {
                            eprintln!(
                                "WebSocket max retries reached. Stopping reconnection attempts."
                            );
                            break;
                        }
                    }
                }
            }
        });
        Ok(())
    }
    pub async fn send_message(&self, msg: Message) -> anyhow::Result<()> {
        let sender_lock = self.sender_tx.lock().await;
        let sender = sender_lock
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("WebSocket connection is not established"))?;

        sender
            .unbounded_send(msg)
            .map_err(|_| anyhow::anyhow!("Failed to send message to WebSocket"))?;
        Ok(())
    }
    pub async fn stop(&self) -> anyhow::Result<()> {
        let syncer_result = self.message_handler.stop_syncer().await;
        let stop_result = self.send_message(Message::Close(None)).await;
        match (syncer_result, stop_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(e1), Ok(())) => Err(e1),
            (Ok(()), Err(e2)) => Err(e2),
            (Err(e1), Err(e2)) => Err(anyhow::anyhow!("syncer error: {}, stop error: {}", e1, e2)),
        }
    }
}
