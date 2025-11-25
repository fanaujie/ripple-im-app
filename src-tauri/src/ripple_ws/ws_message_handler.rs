use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

#[trait_variant::make(RippleWsMsgHandler: Send)]
pub trait WsMessageHandler: Sync + Clone + 'static {
    async fn handle_message(
        &self,
        send_tx: &UnboundedSender<Message>,
        message: Message,
    ) -> anyhow::Result<()>;
    async fn notify_connect(&self);
    async fn notify_disconnect(&self);

    async fn notify_ws_stop(&self, err_msg: String);
}
