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
pub type ReceiverId = usize;

#[derive(PartialEq)]
/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub content: MessageContent,
    pub timestamp: Timestamp,
}

#[derive(PartialEq)]
/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageContent {
    ACK(ACK), // リクエストに対する認証
    REQ(REQ), // レシーバが発行するリクエスト
    OPE(REQ), // オペレータが発行するリクエスト
}
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ACK {
    pub req_id: Option<usize>, // REQに紐づくID. originと紐づいて初めて固有の値となる
    pub src: ReceiverId,       // オペレーションの受付元
    pub publisher: ReceiverId, // 認証の発行元
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct REQ {
    pub id: Option<usize>, // queue内で識別するために必要
    pub src: usize,   // オペレーションの受付元
    pub method: METHOD,    // 操作内容
    pub done: bool,        // 操作が完了したかどうか
}

impl REQ {
    /* Note:
        基本的にReqの生成はオペレータが行い、id, timestampの付与はレシーバが行うため、
        idとtimestampはデフォルトでNoneとなる
    */
    pub fn default() -> Self {
        REQ {
            id: None,
            src: ReceiverId::default(),
            method: METHOD::default(),
            done: false,
            // timestamp: None,
        }
    }

    pub fn gen_ack(&self, publisher: ReceiverId, timestamp: Timestamp) -> Message {
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
#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy, PartialEq)]
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
    // println!("display....");

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
                match message.timestamp {
                    Some(val) => val,
                    _ => -1.0,
                }
            );
        }
        _ => {
            return;
        }
    };

    // println!("display....done");
}
