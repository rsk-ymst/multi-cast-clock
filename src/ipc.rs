use serde_derive::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::{MY_RECEIVER_ID, TARGET_RECEIVER_ID};

/* メッセージ管理用キュー */
pub type MessageQueue = VecDeque<Message>;
pub type Timestamp = Option<f64>;

/* レシーバID */
pub type ReceiverId = usize;

/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub content: MessageContent,
    pub timestamp: Timestamp,
}

/* メッセージの内容はACKもしくはREQとなる。*/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    pub id: usize,      // queue内で識別するために必要
    pub src: usize,     // オペレーションの受付元
    pub method: METHOD, // 操作内容
}

impl REQ {
    /* Note:
        基本的にReqの生成はオペレータが行い、id, timestampの付与はレシーバが行うため、
        idとtimestampはデフォルトでNoneとなる
    */
    pub fn default() -> Self {
        REQ {
            id: 0,
            src: ReceiverId::default(),
            method: METHOD::default(),
            // timestamp: None,
        }
    }

    pub fn gen_ack(&self, publisher: ReceiverId, timestamp: Timestamp) -> Message {
        Message {
            content: MessageContent::ACK(ACK {
                req_id: Some(self.id),
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

/*
    キューの内容をソートする関数
*/
pub fn sort_message_queue(queue: &MessageQueue) -> MessageQueue {
    let mut buf_vec: Vec<Message> = queue.clone().into_iter().collect();

    buf_vec.sort_by(|a, b| {
        a.timestamp
            .unwrap()
            .partial_cmp(&b.timestamp.unwrap())
            .unwrap()
    });

    VecDeque::from(buf_vec)
}

/*
    キューの内容をチェックし、ACKが揃っているREQがあれば実行する関数
*/
pub fn check_and_execute_task(queue: &mut MessageQueue) {
    let traversal_buf = queue.clone();
    for (_, message) in traversal_buf.iter().enumerate() {
        if let MessageContent::REQ(req) = message.content {
            /* 削除する可能性があるメッセージを保持するベクタ */
            let mut possibly_delete_message: Vec<Message> = Vec::new();

            /* Ackの発行元を保持するベクタ */
            possibly_delete_message.push(message.clone());
            let mut ack_publisher_list = Vec::new();

            /* Reqに対応するAckを走査する */
            for (_, m) in queue.iter().enumerate() {
                if let MessageContent::ACK(ack) = &m.content {
                    if req.src == ack.src {
                        ack_publisher_list.push(ack.publisher);
                        possibly_delete_message.push(m.clone());
                    }
                }
            }

            /* REQに対応するACKが全て存在していたら、タスク実行し、タスクと対応Ackを消去 */
            if ack_publisher_list.contains(&MY_RECEIVER_ID)
                && ack_publisher_list.contains(&TARGET_RECEIVER_ID)
            {
                println!("------------- <exec>\n{req:#?}");

                /* REQと対応ACKを消去 */
                possibly_delete_message.into_iter().for_each(|mes| {
                    /* 所有権の都合上、queueのクローンで走査する */
                    let buf = queue.clone();
                    for (i, e) in buf.iter().enumerate() {
                        if mes == *e {
                            queue.remove(i);
                        }
                    }
                });

                println!("------------- <remove required REQ and ACK>");
                queue.iter().for_each(|x| display_log(x));
            }
        }
    }
}

/*
    メッセージの状態を表示する関数
*/
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
