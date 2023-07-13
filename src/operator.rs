use std::{net::UdpSocket, io};

use ipc::*;


mod ipc;

pub const NETWORK_ADDRESS: &str = "127.0.0.1:80";
pub const BROAD_CAST_ADDRESS: &str = "127.255.255.255:8080";
pub fn main() -> io::Result<()> {
    let socket = UdpSocket::bind(NETWORK_ADDRESS).unwrap();

    let hoge = Message::init(MessageContent::REQ(METHOD::UPDATE), None);
    let serialized = serde_json::to_vec(&hoge).unwrap();

    // サーバーにメッセージを送信する
    println!("Client sending message: {:?}", serialized);
    socket
        .send_to(&serialized, BROAD_CAST_ADDRESS)
        .unwrap();

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use std::{io, net::UdpSocket};


//     #[test]
//     fn test_example() {
//         // ソケットを作成する
//     }
// }
