use std::{default, io, net::UdpSocket};

use ipc::*;
mod ipc;
mod static_info;
pub fn main() -> io::Result<()> {
    let socket = UdpSocket::bind(static_info::IP_ADDRESS_OPE).unwrap();

    let message_A = Message {
        content: MessageContent::REQ(REQ {
            method: METHOD::UPDATE,
            ..Default::default()
        }),
        timestamp: None,
    };

    let message_B = Message {
        content: MessageContent::REQ(REQ {
            method: METHOD::UPDATE,
            src: Receiver::B,
            ..Default::default()
        }),
        timestamp: None,
    };

    let serialized_A = serde_json::to_vec(&message_A).unwrap();
    let serialized_B = serde_json::to_vec(&message_B).unwrap();

    // サーバーにメッセージを送信する
    println!("Client sending message: {:?}", serialized_A);
    socket
        .send_to(&serialized_A, static_info::IP_ADDRESS_A)
        .unwrap();

    socket
        .send_to(&serialized_B, static_info::IP_ADDRESS_B)
        .unwrap();

    Ok(())
}
