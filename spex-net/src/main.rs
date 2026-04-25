use std::path::PathBuf;

use tokio::net::UdpSocket;

use spex_net::{receiver, sender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sender_socket = UdpSocket::bind("127.0.0.1:7001").await?;
    let receiver_socket = UdpSocket::bind("127.0.0.1:7002").await?;

    let sender_addr = "127.0.0.1:7002".parse().unwrap();
    let receiver_addr = "127.0.0.1:7001".parse().unwrap();

    let in_path = PathBuf::from("input.bin");
    let out_path = PathBuf::from("output.bin");

    let sender_task = tokio::spawn(sender::run(sender_socket, sender_addr, in_path));
    let receiver_task = tokio::spawn(receiver::run(receiver_socket, receiver_addr, out_path));

    let _ = tokio::join!(sender_task, receiver_task);

    Ok(())
}
