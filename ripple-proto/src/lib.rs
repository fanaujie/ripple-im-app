pub mod ripple_pb {
    include!(concat!(env!("OUT_DIR"), "/ripple.wsmessage.rs"));
}

#[test]
fn test_ripple_message_serialization() {
    use crate::ripple_pb;
    use prost::Message;

    let msg = ripple_pb::WsMessage {
        message_type: Some(ripple_pb::ws_message::MessageType::HeartbeatRequest(
            ripple_pb::HeartbeatRequest {
                user_id: "user123".to_string(),
                timestamp: 1625247600,
            },
        )),
    };

    // Serialize the message to a byte array
    let mut buf = Vec::new();
    msg.encode(&mut buf).unwrap();

    // Deserialize the byte array back to a RippleMessage
    let decoded_msg = ripple_pb::WsMessage::decode(&buf[..]).unwrap();

    // Verify that the original and decoded messages are the same
    assert_eq!(msg, decoded_msg);
}
