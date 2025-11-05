use prost::Message;
use ripple_proto::ripple_pb;
use ripple_proto::ripple_pb::ws_message::MessageType::HeartbeatRequest;

#[derive(Clone)]
pub struct WsUtilsHeartbeatRequest {
    heartbeat_request: ripple_pb::HeartbeatRequest,
    ws_msg: ripple_pb::WsMessage,
}

impl WsUtilsHeartbeatRequest {
    pub fn new(user_id: String) -> Self {
        let heartbeat_request = ripple_pb::HeartbeatRequest {
            user_id,
            timestamp: 0,
        };
        WsUtilsHeartbeatRequest {
            heartbeat_request,
            ws_msg: ripple_pb::WsMessage { message_type: None },
        }
    }

    pub fn get_heartbeat_request_buf(&mut self) -> Vec<u8> {
        self.heartbeat_request.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut buf = Vec::new();
        self.ws_msg
            .message_type
            .replace(HeartbeatRequest(self.heartbeat_request.clone()));
        self.ws_msg.encode(&mut buf).unwrap();
        buf
    }
}
