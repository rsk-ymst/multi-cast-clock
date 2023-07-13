use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::{collections::VecDeque, net::UdpSocket};

/* 各プロセスで用いるメッセージを管理するキュー */
pub type MessageQueue = VecDeque<Message>;

pub type TimeStamp = f64;

/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageContent {
    ACK(ACK),
    REQ(METHOD),
}

#[derive(Serialize, Deserialize, Debug)]

pub struct ACK {
    src: usize, // 送信元
    dst: usize, // 送信先
}

/* 今回、プロセスはCRUDアプリと仮定し、REQUESTの内容は以下のいずれかになる。*/
#[derive(Serialize, Deserialize, Debug)]
pub enum METHOD {
    CREATE,
    UPDATE,
    READ,
    DELETE,
}

// メッセージ・パッシングに用いる構造体情報
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub content: MessageContent,
    pub time_stamp: Option<f64>, // オペレータからプロセスへのメッセージにタイムスタンプは不要なのでOptionとする。
}

impl Message {
    pub fn init(content: MessageContent, time_stamp: Option<f64>) -> Self {
        Message {
            content,
            time_stamp,
        }
    }
}

pub async fn recv_message(socket: &UdpSocket) -> Message {
    let mut buffer: &mut [u8] = &mut [0u8; 2048];

    // let (n, _) = socket.recv_from(buffer).unwrap(); // &mut 引数として欲しい

    match socket.recv_from(buffer) {
        Ok((n, _)) => {
            return serde_json::from_slice(&buffer[..n]).expect("hoge");
        }
        Err(_) => {
            panic!("recv_data error...");
        }
    }

    // serde_json::from_slice(&buffer[..n]).expect("hoge") //  &が引数として欲しい --> as_refで受けわたし
}

pub struct UdpMessageHandler {
    socket: UdpSocket,
}

impl UdpMessageHandler {
    pub fn new(addr: &str) -> Self {
        // sock = UdpSocket::bind(addr).unwrap();

        UdpMessageHandler {
            socket:  UdpSocket::bind(addr).unwrap()
        }
    }

    pub async fn recv_message(&self) -> Message {
        let mut buffer: &mut [u8] = &mut [0u8; 2048];

        // let (n, _) = socket.recv_from(buffer).unwrap(); // &mut 引数として欲しい

        match self.socket.recv_from(buffer) {
            Ok((n, _)) => {
                return serde_json::from_slice(&buffer[..n]).expect("hoge");
            }
            Err(_) => {
                panic!("recv_data error...");
            }
        }

        // serde_json::from_slice(&buffer[..n]).expect("hoge") //  &が引数として欲しい --> as_refで受けわたし
    }
}
