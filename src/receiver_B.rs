mod clock;
mod ipc;
mod static_info;

use clock::LogicClock;
use ipc::UdpMessageHandler;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::clock::TICK_INTERVAL;
use crate::ipc::{Message, MessageContent, MessageQueue, Receiver};

// mod operator;
#[tokio::main]
async fn main() -> io::Result<()> {
    let shared_value = Arc::new(Mutex::new(clock::LogicClock::default()));

    /* クロック開始 */
    clock::start_clock_tick(&shared_value);

    let mut queue = MessageQueue::new();
    let message_handler = UdpMessageHandler::new(static_info::IP_ADDRESS_B);

    loop {
        /* REQ | ACK の受信 */
        let mut message = message_handler.recv_message().await;

        match message.content {
            MessageContent::ACK(_ack) => {
                /* トランザクション */
                // ackをキューに入れる
                queue.push_front(message.clone());
                println!("****** ACK Receive ******\n{queue:#?}");
            }
            MessageContent::REQ(req) => {
                match message.timestamp {
                    /* タイムスタンプあり --> 他レシーバからの受信 */
                    Some(_timestamp) => {
                        queue.push_front(message);

                        /***************** tick *******************/
                        thread::sleep(TICK_INTERVAL);

                        let ack_message =
                            req.gen_ack(Receiver::B, get_current_timestamp(&shared_value));

                        // ackをキューに入れる
                        queue.push_front(ack_message.clone());

                        // ackの送信
                        message_handler
                            .send_message(ack_message, static_info::IP_ADDRESS_A)
                            .await;

                        println!("****** REQ Receive from Receiver ******\n{queue:#?}");
                    }

                    /* タイムスタンプなし --> オペレータからの受信 */
                    None => {
                        /* リクエストにタイムスタンプを付与し、キューに入れる */
                        message.timestamp = get_current_timestamp(&shared_value);
                        queue.push_front(message.clone());

                        message_handler
                            .send_message(message.clone(), static_info::IP_ADDRESS_A)
                            .await;

                        /***************** tick *******************/
                        thread::sleep(TICK_INTERVAL);

                        // ackの生成
                        let ack_message =
                            req.gen_ack(Receiver::B, get_current_timestamp(&shared_value));

                        // ackをキューに入れる
                        queue.push_front(ack_message.clone());

                        // ackの送信
                        message_handler
                            .send_message(ack_message, static_info::IP_ADDRESS_B)
                            .await;

                        println!("****** REQ Receive from Operator ******\n{queue:#?}");
                    }
                }
            }
        }

        // キューのソート
        let mut a: Vec<Message> = queue.clone().into_iter().collect();
        a.sort_by(|a, b| {
            a.timestamp
                .unwrap()
                .partial_cmp(&b.timestamp.unwrap())
                .unwrap()
        });
        queue = VecDeque::from(a);

        // キューのチェック；もしACKが揃っていればタスク実行＆ACK削除．
        check_and_execute_task(&mut queue);
    }
}

pub fn get_current_timestamp(value: &Arc<Mutex<LogicClock>>) -> Option<f64> {
    let current = value.lock().unwrap();
    let res = current.clock;

    drop(current); // Mutexロック解除
    Some(res)
}

pub fn check_and_execute_task(queue: &mut MessageQueue) {
    /* 所有権の都合上、走査用のクローンを用意する */
    let traversal_buf = queue.clone();

    for (i, mes) in traversal_buf.iter().enumerate() {
        if let MessageContent::REQ(req) = mes.content {
            /* Ackの発行元を保持するベクタ */
            let mut ack_publisher_list = Vec::new();

            /* 削除する可能性があるidxを保持するベクタ。タスク完了後にReqとそれに紐づくAckを消去するために必要 */
            let mut possibly_delete_idx = Vec::new();
            possibly_delete_idx.push(i);

            /* Reqに対応するAckを走査する */
            for (j, m) in traversal_buf.iter().enumerate() {
                if let MessageContent::ACK(ack) = &m.content {
                    if req.src == ack.src {
                        ack_publisher_list.push(ack.publisher);
                        possibly_delete_idx.push(j);
                    }
                }
            }

            /* REQとそれに対応するACKが存在していたら、タスク実行＆タスクと対応Ackを消去 */
            if ack_publisher_list.contains(&Receiver::A)
                && ack_publisher_list.contains(&Receiver::B)
            {
                println!("EXEC: {req:#?}");
                possibly_delete_idx.into_iter().for_each(|idx| {
                    queue.remove(idx);
                })
            }
        }
    }
}
