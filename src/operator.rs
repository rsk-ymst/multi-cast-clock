#[cfg(test)]
mod tests {
    use std::net::UdpSocket;

    use crate::{
        ipc::{receiver_id, Message, MessageContent, METHOD, REQ},
        static_info,
    };

    #[test]
    fn it_works() {
        let socket = UdpSocket::bind(static_info::IP_ADDRESS_OPE).unwrap();

        let message_A = Message {
            content: MessageContent::REQ(REQ {
                method: METHOD::UPDATE,
                src: 1,
                ..Default::default()
            }),
            timestamp: None,
        };

        let message_B = Message {
            content: MessageContent::REQ(REQ {
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
        socket
            .send_to(&serialized_A, static_info::IP_ADDRESS_A)
            .unwrap();

        socket
            .send_to(&serialized_B, static_info::IP_ADDRESS_B)
            .unwrap();
    }
}
