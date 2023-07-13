mod clock;
mod ipc;
mod static_info;

use clock::LogicClock;
use ipc::UdpMessageHandler;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::ipc::{recv_message, Message, MessageQueue, Receiver};

// mod operator;
#[tokio::main]
async fn main() -> io::Result<()> {
    let shared_value = Arc::new(Mutex::new(clock::LogicClock::default()));

    /* クロック開始 */
    clock::start_clock_tick(&shared_value);

    let mut queue = MessageQueue::new();
    let message_handler = UdpMessageHandler::new(static_info::IP_ADDRESS_A);

    loop {
        /* REQ | ACK の受信 */
        let mut message = message_handler.recv_message().await;

        match message {
            Message::ACK(ack) => { /* トランザクション */ }
            Message::REQ(mut req) => {
                match req.timestamp {
                    /* タイムスタンプあり --> 他レシーバからの受信 */
                    Some(timestamp) => {

                    }

                    /* タイムスタンプなし --> オペレータからの受信 */
                    None => {
                        req.timestamp = get_current_timestamp(&shared_value);
                        queue.push_front(Message::REQ(req.clone()));

                        message_handler
                            .send_message(Message::REQ(req.clone()), static_info::IP_ADDRESS_B)
                            .await;

                        // ackの生成
                        let ack = req.gen_ack(Receiver::A, get_current_timestamp(&shared_value));

                        // ackをキューに入れる
                        queue.push_front(Message::ACK(ack.clone()));

                        // ackの送信
                        message_handler
                            .send_message(Message::ACK(ack), static_info::IP_ADDRESS_B)
                            .await;
                    }
                }
            }
        }

        // キューのソート
        let a: Vec<Message> = queue.clone().into_iter().collect();
        a.sort(|m|
            match m {
                Message::ACK(e) => e.timestamp,
                Message::REQ(e) => e.timestamp,
            }
        );
        // キューのチェック；もしACKが揃っていればタスク実行＆ACK削除．

        println!("Server received message: {:#?}", a);
    }
}

pub fn get_current_timestamp(value: &Arc<Mutex<LogicClock>>) -> Option<f64> {
    let current = value.lock().unwrap();
    let res = current.clock;

    drop(current); // Mutexロック解除
    Some(res)
}
