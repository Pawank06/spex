use std::path::PathBuf;
use std::time::Duration;

use tokio::net::UdpSocket;

use spex_net::config::Config;
use spex_net::{receiver, sender};

async fn pick_socket() -> (UdpSocket, std::net::SocketAddr) {
    let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = sock.local_addr().unwrap();
    (sock, addr)
}

fn tmp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("spex_test_{}_{}.bin", name, std::process::id()));
    p
}

#[tokio::test]
async fn happy_path_transfer() {
    let (sender_sock, sender_addr) = pick_socket().await;
    let (receiver_sock, receiver_addr) = pick_socket().await;

    let payload = b"hello spex";
    let in_path = tmp_path("in_happy");
    let out_path = tmp_path("out_happy");
    std::fs::write(&in_path, payload).unwrap();

    let cfg = Config {
        chunk_size: 3,
        send_delay_ms: 1,
        retry_after_ms: 50,
        max_retries: 5,
        idle_timeout_ms: 1_000,
    };

    let s_task = tokio::spawn(sender::run(sender_sock, receiver_addr, in_path.clone(), cfg.clone()));
    let r_task = tokio::spawn(receiver::run(receiver_sock, sender_addr, out_path.clone(), cfg));

    let _ = tokio::time::timeout(Duration::from_secs(5), r_task).await.unwrap();
    let _ = s_task.await;

    let read = std::fs::read(&out_path).unwrap();
    assert_eq!(read, payload);

    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
}

#[tokio::test]
async fn larger_payload_transfer() {
    let (sender_sock, sender_addr) = pick_socket().await;
    let (receiver_sock, receiver_addr) = pick_socket().await;

    let payload: Vec<u8> = (0u8..=255).cycle().take(8 * 1024).collect();
    let in_path = tmp_path("in_large");
    let out_path = tmp_path("out_large");
    std::fs::write(&in_path, &payload).unwrap();

    let cfg = Config {
        chunk_size: 256,
        send_delay_ms: 0,
        retry_after_ms: 50,
        max_retries: 5,
        idle_timeout_ms: 1_000,
    };

    let s_task = tokio::spawn(sender::run(sender_sock, receiver_addr, in_path.clone(), cfg.clone()));
    let r_task = tokio::spawn(receiver::run(receiver_sock, sender_addr, out_path.clone(), cfg));

    let _ = tokio::time::timeout(Duration::from_secs(10), r_task).await.unwrap();
    let _ = s_task.await;

    let read = std::fs::read(&out_path).unwrap();
    assert_eq!(read, payload);

    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
}
