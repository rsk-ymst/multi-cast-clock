#[cfg(test)]
mod tests {
    use std::{net::UdpSocket, thread, time::Duration};

    pub static IP_ADDRESS_A: &str = "127.0.0.1:8080"; // レシーバA
    pub static IP_ADDRESS_B: &str = "127.0.0.2:8080"; // レシーバB
    pub static IP_ADDRESS_OPE: &str = "127.0.0.3:8080";

    use crate::{ipc::{Message, MessageContent, METHOD, REQ}, clock::TICK_INTERVAL};

    #[test]
    fn it_works() {
        let socket = UdpSocket::bind(IP_ADDRESS_OPE).unwrap();

        let message_A = Message {
            content: MessageContent::OPE(REQ {
                method: METHOD::UPDATE,
                src: 1,
                ..Default::default()
            }),
            timestamp: None,
        };

        let message_B = Message {
            content: MessageContent::OPE(REQ {
                method: METHOD::UPDATE,
                src: 2,
                ..Default::default()
            }),
            timestamp: None,
        };

        let serialized_A = serde_json::to_vec(&message_A).unwrap();
        let serialized_B = serde_json::to_vec(&message_B).unwrap();

        // サーバーにメッセージを送信する
        println!("Client sending message: {:?}", serialized_A);
        socket.send_to(&serialized_A, IP_ADDRESS_A).unwrap();

        // thread::sleep(Duration::from_millis(10));

        socket.send_to(&serialized_B, IP_ADDRESS_B).unwrap();
    }
}
