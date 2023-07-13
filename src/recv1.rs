mod clock;
mod ipc;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time;
use std::time::Duration;
use std::{io, thread};
use ipc::UdpMessageHandler;
use serde::*;
use serde_json::*;

use crate::ipc::{Message, MessageQueue, recv_message};

const ASSIGNED_ADDRESS: &str = "127.0.0.1:8080";

// mod operator;
#[tokio::main]
async fn main() -> io::Result<()> {
    // ソケットを作成し、バインドする
    let mut queue = MessageQueue::new();
    // let socket = UdpSocket::bind(ASSIGNED_ADDRESS)?;
    let messageHandler = UdpMessageHandler::new(ASSIGNED_ADDRESS);

    loop {
        // let mut buffer: &mut [u8] = &mut [0u8; 2048];

        // let (n, src_addr) = socket.recv_from(buffer)?; // &mut 引数として欲しい
        // let deserialized: Message = serde_json::from_slice(&buffer[..n]).expect("hoge");//  &が引数として欲しい --> as_refで受けわたし

        // let deserialized = recv_message(&socket);
        let deserialized = messageHandler.recv_message().await;
        queue.push_front(deserialized);

        println!("Server received message: {:#?}", queue);

        // メッセージをハンドルする
        // let json: Message = serde_json::from_str(&message).expect("hoge");//  &が引数として欲しい --> as_refで受けわたし
        // let json: Message = serde_json::from_slice(&buffer[..n]).expect("hoge");//  &が引数として欲しい --> as_refで受けわたし
        // let serialized = serde_json::to_string(&point).unwrap();


        // クライアントに返信する
        // let response = "Hello from server!";
        // println!("Server sending response: {:?}", json);
        // socket.send_to(response.as_bytes(), src_addr)?;
    }
}
