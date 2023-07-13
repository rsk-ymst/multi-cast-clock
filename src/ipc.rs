

use serde::{Serialize, Deserialize};
use serde_derive::{Deserialize, Serialize};


type TimeStamp = f64;

/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageContent {
    ACK(ACK),
    REQ(METHOD),
}

#[derive(Serialize, Deserialize, Debug)]

pub struct ACK {
    src: usize, // 送信元
    dst: usize  // 送信先
}

/* 今回、プロセスはCRUDアプリと仮定し、REQUESTの内容は以下のいずれかになる。*/
#[derive(Serialize, Deserialize, Debug)]
pub enum METHOD {
    CREATE,
    UPDATE,
    READ,
    DELETE
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
            time_stamp
        }
    }
}
