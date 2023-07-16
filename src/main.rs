mod clock;
mod ipc;
mod operator;
mod utils;

use clock::LogicClock;
use ipc::{display_log, UdpMessageHandler};
use std::collections::VecDeque;
use std::env::args;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::clock::TICK_INTERVAL;
use crate::ipc::{Message, MessageContent, MessageQueue, ReceiverId};

#[macro_use]
extern crate lazy_static;

/* 実行時の引数をもとに静的なグローバル変数を初期化 */
lazy_static! {
    static ref MY_RECEIVER_ID: ReceiverId = utils::args::get_as_usize(1);
    static ref TARGET_RECEIVER_ID: ReceiverId = utils::args::get_as_usize(2);
    static ref MY_ADDRESS: String = utils::args::get_as_String(3);
    static ref TARGET_ADDRESS: String = utils::args::get_as_String(4);
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

                println!("------------- <ACK received>");
            }
            MessageContent::REQ(req) => {
                match message.timestamp {
                    /* タイムスタンプあり --> 他レシーバからの受信 */
                    Some(_timestamp) => {
                        queue.push_front(message.clone());

                        /***************** tick *******************/
                        thread::sleep(TICK_INTERVAL);
                        clock::sleep_random_interval();

                        let ack_message =
                            req.gen_ack(*MY_RECEIVER_ID, get_current_timestamp(&shared_value));
                        // display_log(&ack_message);

                        // ackをキューに入れる
                        queue.push_front(ack_message.clone());

                        // ackの送信
                        message_handler
                            .send_message(ack_message, &TARGET_ADDRESS)
                            .await;
                        println!("------------- <REQ received from receiver>");
                    }

                    /* タイムスタンプなし --> オペレータからの受信 */
                    None => {
                        /* レシーバ間のタイムスタンプに若干の差分を生じさせるために、スリープさせる */
                        clock::sleep_random_interval();

                        /* リクエストにタイムスタンプを付与し、キューに入れる */
                        message.timestamp = get_current_timestamp(&shared_value);
                        queue.push_front(message.clone());
                        // println!("pushed!: {:#?}", queue);

                        message_handler
                            .send_message(message.clone(), &TARGET_ADDRESS)
                            .await;

                        /***************** tick *******************/
                        // thread::sleep(TICK_INTERVAL);
                        clock::sleep_random_interval();

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
                        println!("------------- <REQ received from operator>");
                    }
                }
            }
        }

        // キューのソート
        // let mut a: Vec<Message> = queue.clone().into_iter().collect();

        // a.sort_by(|a, b| {
        //     a.timestamp
        //         .unwrap()
        //         .partial_cmp(&b.timestamp.unwrap())
        //         .unwrap()
        // });
        queue = sort_message_queue(&queue);

        queue.iter().for_each(|x| display_log(x));
        // println!("------------- <sorted>");
        // queue = VecDeque::from(a);
        // queue.iter().for_each(|x| display_log(x));

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
    // let traversal_buf = queue.clone();

    loop {
        /* キューの中で最もタイムスタンプが小さいREQを取得 */
        let target_req = match get_min_timestamp_req(&queue) {
            Ok(req) => req,
            Err(_) => return,
        };

        /* 削除する可能性があるメッセージを保持するベクタ */
        let mut possibly_delete_message: Vec<Message> = Vec::new();
        possibly_delete_message.push(target_req.clone());
        // println!("!!!!!!!!!\n{:#?}", target_req);

        // for (_, mes) in target_req.iter().enumerate() {
        if let MessageContent::REQ(req) = target_req.content {
            /* Ackの発行元を保持するベクタ */
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
                // println!("trav: {:#?}", traversal_buf);
                // println!("origin: {:#?}", traversal_buf);
                // traversal_buf.iter().for_each(|x| println!("{:#?}", x));
                // queue.iter().for_each(|x| println!("{:#?}", x));

                // println!("-------------------");

                /* REQと対応ACKを消去 */
                possibly_delete_message.into_iter().for_each(|mes| {
                    /* 所有権の都合上、queueのクローンで走査する */
                    let buf = queue.clone();
                    for (i, e) in buf.iter().enumerate() {
                        if mes == *e {
                            // println!("remove --> {:#?}", queue.get(i));
                            queue.remove(i);
                        }
                    }
                });

                println!("------------- <remove required REQ and ACK>");
                queue.iter().for_each(|x| display_log(x));

                // println!("---------------------------- after removed");
                // println!("{:#?}", queue);
            } else {
                /* 消費できないREQがある時点でチェックを終了 */
                return;
            }
        }
    }
    // }
}

pub fn get_min_timestamp_req(queue: &MessageQueue) -> Result<Message, ()> {
    let mut req_rev: Vec<Message> = Vec::new();

    queue.iter().for_each(|m| {
        if let MessageContent::REQ(_) = m.content {
            req_rev.push(m.clone());
        }
    });

    if req_rev.len() == 0 {
        return Err(());
    }

    Ok(req_rev.get(0).unwrap().clone())
}

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
