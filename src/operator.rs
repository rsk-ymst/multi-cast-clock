use std::{default, io, net::UdpSocket};

use ipc::*;
mod ipc;
mod static_info;
pub fn main() -> io::Result<()> {
    let socket = UdpSocket::bind(static_info::IP_ADDRESS_OPE).unwrap();

    let hoge = Message {
        content: MessageContent::REQ(REQ {
            method: METHOD::UPDATE,
            ..Default::default()
        }),
        timestamp: None,
    };
    let serialized = serde_json::to_vec(&hoge).unwrap();

    // サーバーにメッセージを送信する
    println!("Client sending message: {:?}", serialized);
    socket
        .send_to(&serialized, static_info::IP_ADDRESS_A)
        .unwrap();

    Ok(())
}
