use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    default,
    net::{IpAddr, UdpSocket},
};

/* メッセージ管理用キュー */
pub type MessageQueue = VecDeque<Message>;
pub type Timestamp = Option<f64>;

/* レシーバの定義。今回はAとBの */
#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy, PartialEq)]
pub enum Receiver {
    #[default]
    A,
    B,
}

/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub content: MessageContent,
    pub timestamp: Timestamp,
}

/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageContent {
    ACK(ACK),
    REQ(REQ),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ACK {
    pub req_id: Option<usize>, // REQに紐づくID. originと紐づいて初めて固有の値となる
    pub src: Receiver,         // オペレーションの受付元
    pub publisher: Receiver,   // 認証の発行元
                               // pub timestamp: Timestamp, // オペレータからプロセスへのメッセージにタイムスタンプは不要なのでOptionとする。
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct REQ {
    pub id: Option<usize>, // queue内で識別するために必要
    pub src: Receiver,     // オペレーションの受付元
    pub method: METHOD,    // 操作内容
    pub done: bool,        // 操作が完了したかどうか
                           // pub timestamp: Timestamp, // オペレータからプロセスへのメッセージにタイムスタンプは不要なのでOptionとする。
}

impl REQ {
    /* Note:
        基本的にReqの生成はオペレータが行い、id, timestampの付与はレシーバが行うため、
        idとtimestampはデフォルトでNoneとなる
    */
    pub fn default() -> Self {
        REQ {
            id: None,
            src: Receiver::default(),
            method: METHOD::default(),
            done: false,
            // timestamp: None,
        }
    }

    pub fn gen_ack(&self, publisher: Receiver, timestamp: Timestamp) -> Message {
        Message {
            content: MessageContent::ACK(ACK {
                req_id: self.id,
                src: self.src,
                publisher,
            }),
            timestamp,
        }
    }
}

/* プロセスはCRUDアプリと仮定し、REQUESTの内容は以下のいずれかになる。*/
#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub enum METHOD {
    CREATE,
    READ,
    #[default]
    UPDATE,
    DELETE,
}

pub async fn recv_message(socket: &UdpSocket) -> MessageContent {
    let mut buffer: &mut [u8] = &mut [0u8; 2048];

    match socket.recv_from(buffer) {
        Ok((n, _)) => {
            return serde_json::from_slice(&buffer[..n]).expect("hoge");
        }
        Err(_) => {
            panic!("recv_data error...");
        }
    }
}

pub struct UdpMessageHandler {
    socket: UdpSocket,
}

impl UdpMessageHandler {
    pub fn new(addr: &str) -> Self {
        UdpMessageHandler {
            socket: UdpSocket::bind(addr).unwrap(),
        }
    }

    pub async fn recv_message(&self) -> Message {
        let mut buffer: &mut [u8] = &mut [0u8; 2048];

        match self.socket.recv_from(buffer) {
            Ok((n, _)) => {
                return serde_json::from_slice(&buffer[..n]).expect("hoge");
            }
            Err(_) => {
                panic!("recv_data error...");
            }
        }
    }

    pub async fn send_message(&self, message: Message, dst_addr: &str) {
        let serialized = serde_json::to_vec(&message).unwrap();

        self.socket.send_to(&serialized, dst_addr).unwrap();
    }
}

pub fn display_log(message: &Message) {
    match message.content {
        MessageContent::ACK(ack) => {
            println!(
                "ACK: {:?}-{:?}: {:.1}",
                ack.src,
                ack.publisher,
                message.timestamp.unwrap()
            );
        }
        MessageContent::REQ(req) => {
            println!(
                "REQ-{:?}: {:?} {:.1}",
                req.src,
                req.method,
                message.timestamp.unwrap()
            );
        }
    };
}
