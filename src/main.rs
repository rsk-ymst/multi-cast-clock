mod clock;
mod ipc;
mod operator;

use clock::LogicClock;
use ipc::{display_log, UdpMessageHandler};
use std::collections::VecDeque;
use std::env::args;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::clock::TICK_INTERVAL;
use crate::ipc::{receiver_id, Message, MessageContent, MessageQueue};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MY_RECEIVER_ID: receiver_id = args()
        .collect::<Vec<String>>()
        .get(1)
        .unwrap()
        .as_str()
        .parse()
        .unwrap();
    static ref TARGET_RECEIVER_ID: receiver_id = args()
        .collect::<Vec<String>>()
        .get(2)
        .unwrap()
        .as_str()
        .parse()
        .unwrap();
    static ref MY_ADDRESS: String = args().collect::<Vec<String>>().get(3).unwrap().to_string();
    static ref TARGET_ADDRESS: String = args().collect::<Vec<String>>().get(4).unwrap().to_string();
}

// mod operator;
#[tokio::main]
async fn main() -> io::Result<()> {
    println!("{}", MY_ADDRESS.to_string());

    let shared_value = Arc::new(Mutex::new(clock::LogicClock::default()));

    /* クロック開始 */
    clock::start_clock_tick(&shared_value);

    let mut queue = MessageQueue::new();
    let message_handler = UdpMessageHandler::new(&MY_ADDRESS);

    loop {
        /* REQ | ACK の受信 */
        let mut message = message_handler.recv_message().await;

        match message.content {
            MessageContent::ACK(_ack) => {
                /* トランザクション */
                queue.push_front(message.clone());

                // println!("****** ACK Receive ******\n{queue:#?}");
                println!("****** ACK Receive ******");
                // display_log(&message);
            }
            MessageContent::REQ(req) => {
                match message.timestamp {
                    /* タイムスタンプあり --> 他レシーバからの受信 */
                    Some(_timestamp) => {
                        queue.push_front(message.clone());

                        /***************** tick *******************/
                        thread::sleep(TICK_INTERVAL);

                        let ack_message =
                            req.gen_ack(*MY_RECEIVER_ID, get_current_timestamp(&shared_value));
                        // display_log(&ack_message);

                        // ackをキューに入れる
                        queue.push_front(ack_message.clone());

                        // ackの送信
                        message_handler
                            .send_message(ack_message, &TARGET_ADDRESS)
                            .await;
                        println!("****** req Receive from receiver ******");
                    }

                    /* タイムスタンプなし --> オペレータからの受信 */
                    None => {
                        /* リクエストにタイムスタンプを付与し、キューに入れる */
                        message.timestamp = get_current_timestamp(&shared_value);
                        queue.push_front(message.clone());
                        // println!("pushed!: {:#?}", queue);

                        message_handler
                            .send_message(message.clone(), &TARGET_ADDRESS)
                            .await;

                        /***************** tick *******************/
                        thread::sleep(TICK_INTERVAL);

                        // ackの生成
                        let ack_message =
                            req.gen_ack(*MY_RECEIVER_ID, get_current_timestamp(&shared_value));

                        // ackをキューに入れる
                        queue.push_front(ack_message.clone());
                        // println!("pushed!: {:#?}", queue);

                        // ackの送信
                        message_handler
                            .send_message(ack_message, &TARGET_ADDRESS)
                            .await;
                        println!("****** req Receive from operator ******");
                    }
                }
            }
        }

        // println!("pushed!: {:#?}", queue);

        // キューのソート
        let mut a: Vec<Message> = queue.clone().into_iter().collect();
        // println!("pushed a!: {:#?}", a);
        a.sort_by(|a, b| {
            a.timestamp
                .unwrap()
                .partial_cmp(&b.timestamp.unwrap())
                .unwrap()
        });
        // println!("pushed a!: {:#?}", a.len());
        a.iter().for_each(|x| display_log(x));
        println!("------------------");
        queue = VecDeque::from(a);

        // キューのチェック；もしACKが揃っていればタスク実行＆ACK削除．
        check_and_execute_task(&mut queue);
        queue.iter().for_each(|x| display_log(x));
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

    for (_, mes) in traversal_buf.iter().enumerate() {
        if let MessageContent::REQ(req) = mes.content {
            /* Ackの発行元を保持するベクタ */
            let mut ack_publisher_list = Vec::new();

            /* 削除する可能性があるメッセージを保持するベクタ */
            let mut possibly_delete_message: Vec<Message> = Vec::new();
            possibly_delete_message.push(mes.clone());

            /* Reqに対応するAckを走査する */
            for (j, m) in traversal_buf.iter().enumerate() {
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
                println!("EXEC: {req:#?}");
                // println!("trav: {:#?}", traversal_buf);
                // println!("origin: {:#?}", traversal_buf);
                // traversal_buf.iter().for_each(|x| println!("{:#?}", x));
                // println!("-------------------");
                // queue.iter().for_each(|x| println!("{:#?}", x));

                // println!("-------------------");

                /* REQと対応ACKを消去 */
                possibly_delete_message.into_iter().for_each(|mes| {
                    /* 所有権の都合上、queueのクローンで走査する */
                    let buf = queue.clone();
                    for (i, e) in buf.iter().enumerate() {
                        if mes == *e {
                            println!("remove --> {:#?}", queue.get(i));
                            queue.remove(i);
                        }
                    }
                });

                println!("---------------------------- after removed");
                println!("{:#?}", queue);
            }
        }
    }
}
