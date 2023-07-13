mod clock;
mod ipc;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time;
use std::time::Duration;
use std::{io, thread};
use serde::*;
use serde_json::*;

use crate::ipc::Message;

const ASSIGNED_ADDRESS: &str = "127.0.0.1:8080";

// mod operator;
fn main() -> io::Result<()> {
    // ソケットを作成し、バインドする
    let socket = UdpSocket::bind(ASSIGNED_ADDRESS)?;

    loop {
        let mut buffer: &mut [u8] = &mut [0u8; 2048];
        let (n, src_addr) = socket.recv_from(buffer)?; // &mut 引数として欲しい

        let message = String::from_utf8_lossy(&buffer[..n]);
        let json: Message = serde_json::from_slice(&buffer[..n]).expect("hoge");//  &が引数として欲しい --> as_refで受けわたし

        println!("Server received message: {}", message);

        // メッセージをハンドルする
        // let json: Message = serde_json::from_str(&message).expect("hoge");//  &が引数として欲しい --> as_refで受けわたし
        // let json: Message = serde_json::from_slice(&buffer[..n]).expect("hoge");//  &が引数として欲しい --> as_refで受けわたし
        // let serialized = serde_json::to_string(&point).unwrap();


        // クライアントに返信する
        let response = "Hello from server!";
        println!("Server sending response: {:?}", json);
        // socket.send_to(response.as_bytes(), src_addr)?;
    }

    // let shared_value = Arc::new(Mutex::new(clock::LogicClock::default()));
    //
    // let thread = clock::start_clock_tick(&shared_value);

    // // サーバーアドレスを設定する
    // let server_address = "127.0.0.1:8080";

    // // ソケットを作成する
    // let socket = UdpSocket::bind("0.0.0.0:0")?;

    // // サーバーにメッセージを送信する
    // let message = "Hello from client!";
    // println!("Client sending message: {}", message);
    // socket.send_to(message.as_bytes(), server_address)?;

    // // サーバーからの返信を受信する
    // let mut buffer = [0u8; 1024];
    // let (n, _) = socket.recv_from(&mut buffer)?;
    // let response = String::from_utf8_lossy(&buffer[..n]);
    // println!("Client received response: {}", response);

    // clock::LogicClock::default();

    // // スレッド終了後の値を参照する
    // let value = shared_value.lock().unwrap();
    // println!("Main thread: Value is {}", value.clock);
    // drop(value);

    // // thread.join();
    // loop {

    // }

    // Ok(())
}
