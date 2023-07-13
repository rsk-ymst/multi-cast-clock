use std::io;
use std::net::UdpSocket;
mod ipc;


fn main() -> io::Result<()> {
    // サーバーアドレスを設定する
    let server_address = "127.0.0.2:8080";

    // ソケットを作成し、バインドする
    let socket = UdpSocket::bind(server_address)?;

    // クライアントからのメッセージを受信する
    let mut buffer = [0u8; 1024];
    let (n, src_addr) = socket.recv_from(&mut buffer)?;

    let message = String::from_utf8_lossy(&buffer[..n]);
    println!("Server received message: {}", message);

    // クライアントに返信する
    let response = "Hello from server!";
    println!("Server sending response: {}", response);
    socket.send_to(response.as_bytes(), src_addr)?;

    Ok(())
}
