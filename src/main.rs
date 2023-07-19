mod clock;
mod ipc;
mod operator;
mod utils;

use clock::*;
use ipc::display_log;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::net::{Ipv4Addr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::clock::TICK_INTERVAL;
use crate::ipc::{
    check_and_execute_task, sort_message_queue, Message, MessageContent, MessageQueue, ReceiverId,
    REQ,
};

#[macro_use]
extern crate lazy_static;

/* 実行時の引数をもとに静的なグローバル変数を初期化 */
lazy_static! {
    static ref MY_RECEIVER_ID: usize = utils::args::get_as_usize(1);
    static ref TARGET_RECEIVER_ID: usize = utils::args::get_as_usize(2);
    static ref MY_ADDRESS: String = utils::args::get_as_String(3);
    static ref TARGET_ADDRESS: String = utils::args::get_as_String(4);
}

const MULTICAST_ADDR: &str = "239.0.0.1";
const PORT: u16 = 8080;

// mod operator;
#[tokio::main]
async fn main() -> io::Result<()> {
    println!(
        "assigned info: {}, {}",
        MY_ADDRESS.to_string(),
        *MY_RECEIVER_ID
    );

    /* 論理クロックはスレッド間で共有するので、Arcとして宣言 */
    let logic_clock = Arc::new(Mutex::new(clock::LogicClock::default()));

    /* tickスレッド開始 */
    clock::start_clock_tick(&logic_clock);

    let mut queue = MessageQueue::new();

    let socket = UdpSocket::bind(&*MY_ADDRESS).expect("Failed to bind socket");
    socket
        .join_multicast_v4(
            &Ipv4Addr::from_str(MULTICAST_ADDR).unwrap(),
            &Ipv4Addr::LOCALHOST,
        )
        .expect("Failed to join multicast group");

    loop {
        let buffer: &mut [u8] = &mut [0u8; 2048];

        /* REQ | ACK の受信 */
        let (n, _) = socket.recv_from(buffer).unwrap();
        let message: Message = serde_json::from_slice(&buffer[..n]).expect("hoge");

        match message.content {
            // =======================================
            //              ACKの受信
            // =======================================
            MessageContent::ACK(_ack) => {
                println!("------<ACK>------");
                adjust_time_clock(&logic_clock, message.timestamp.unwrap());
                /* トランザクション */
                queue.push_front(message.clone());
            }

            // =======================================
            //         オペレータからのリクエスト
            // =======================================
            MessageContent::OPE(request) => {
                println!("------<OPE>------");

                /* レシーバ間のタイムスタンプに若干の差分を生じさせるために、スリープさせる */
                clock::sleep_random_interval(1);

                let timestamped_message = Message {
                    content: ipc::MessageContent::REQ(request.clone()),
                    timestamp: clock::get_current_timestamp(&logic_clock),
                };

                queue.push_front(timestamped_message.clone());

                /* REQを他プロセスに送信(REQの複製) */
                let send_socket = socket.try_clone().expect("Failed to clone socket");
                let send_message = timestamped_message.clone();
                thread::spawn(move || {
                    send_socket
                        .send_to(
                            &serde_json::to_vec(&send_message).unwrap(),
                            &format!("{}:{}", &*MULTICAST_ADDR, PORT),
                        )
                        .expect("Failed to send multicast message");
                });

                /***************** tick *******************/
                clock::sleep_random_interval(1);

                // ackの生成
                if let MessageContent::REQ(req) = timestamped_message.content {
                    let ack_message =
                        req.gen_ack(*MY_RECEIVER_ID, clock::get_current_timestamp(&logic_clock));

                    let serialized = serde_json::to_vec(&ack_message).unwrap();

                    clock::sleep_random_interval(1);

                    /* 非同期的マルチキャスト */
                    let send_socket = socket.try_clone().expect("Failed to clone socket");
                    thread::spawn(move || {
                        send_socket
                            .send_to(&serialized, &format!("{}:{}", MULTICAST_ADDR, PORT))
                            .expect("Failed to send multicast message");
                    });
                }
            }

            // =======================================
            //       プロセスからのリクエスト受信
            // =======================================
            MessageContent::REQ(req) => {

                /* 再帰的なREQは無視する */
                if req.src.cmp(&MY_RECEIVER_ID) == Ordering::Equal {
                    // println!("its mine");
                    continue;
                }

                println!("------<REQ>------");
                adjust_time_clock(&logic_clock, message.timestamp.unwrap());

                queue.push_front(message.clone());

                /***************** tick *******************/
                thread::sleep(TICK_INTERVAL);
                // clock::sleep_random_interval(1);

                let ack_message =
                    req.gen_ack(*MY_RECEIVER_ID, clock::get_current_timestamp(&logic_clock));
                let serialized = serde_json::to_vec(&ack_message).unwrap();

                // ackの送信
                let send_socket = socket.try_clone().expect("Failed to clone socket");
                thread::spawn(move || {
                    send_socket
                        .send_to(&serialized, &format!("{}:{}", MULTICAST_ADDR, PORT))
                        .expect("Failed to send multicast message");
                });
            }
        }
        queue = sort_message_queue(&queue);
        queue.iter().for_each(|x| display_log(x));
        // println!("------------- <sorted>");
        // queue = VecDeque::from(a);
        // queue.iter().for_each(|x| display_log(x));

        // キューのチェック；もしACKが揃っていればタスク実行＆ACK削除．
        check_and_execute_task(&mut queue);
    }
}
