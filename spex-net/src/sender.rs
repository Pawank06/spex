use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

use rand::seq::SliceRandom;
use rand::thread_rng;

use spex_core::chunk::{chunks_bytes as chunk_bytes, Chunk};
use spex_core::metadata::FileMeta;

use crate::protocol::NetMessage;

pub async fn run(socket: UdpSocket, receiver_addr: SocketAddr) {
    let data = b"hello rust";
    let chunk_size = 3;

    let mut chunks = chunk_bytes(data, chunk_size);
    let meta = FileMeta::new(data, chunk_size, &chunks);

    let mut chunk_store: HashMap<u64, Chunk> = HashMap::new();
    for chunk in &chunks {
        chunk_store.insert(chunk.index, chunk.clone());
    }

    println!("sender: sending metadata");
    let meta_bytes = bincode::serialize(&NetMessage::FileMeta(meta)).unwrap();
    socket.send_to(&meta_bytes, receiver_addr).await.unwrap();

    chunks.shuffle(&mut thread_rng());

    for chunk in chunks {
        if chunk.index == 1 {
            println!("sender: intentionally dropping chunk {}", chunk.index);
            continue;
        }

        println!("sender: sending chunk {}", chunk.index);
        let bytes = bincode::serialize(&NetMessage::Chunk(chunk)).unwrap();
        socket.send_to(&bytes, receiver_addr).await.unwrap();

        sleep(Duration::from_millis(300)).await;
    }

    let mut buf = [0u8; 2048];
    loop {
        let (len, _) = socket.recv_from(&mut buf).await.unwrap();
        let msg: NetMessage = bincode::deserialize(&buf[..len]).unwrap();

        if let NetMessage::RequestChunk { index } = msg {
            if let Some(chunk) = chunk_store.get(&index) {
                println!("sender: resending chunk {}", index);
                let bytes = bincode::serialize(
                    &NetMessage::Chunk(chunk.clone())
                ).unwrap();

                socket.send_to(&bytes, receiver_addr).await.unwrap();
            }
        }
    }
}
