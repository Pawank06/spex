use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use spex_net::protocol::NetMessage;
use spex_core::chunk::{chunks_bytes, Chunk};
use spex_core::metadata::FileMeta;

async fn sender(mut tx: mpsc::Sender<NetMessage>) {
    let data = b"hello world";
    let chunk_size = 3;
    
    let chunks = chunks_bytes(data, chunk_size);
    let meta = FileMeta::new(data, chunk_size, &chunks);
    
    println!("sender: sending metadata");
    tx.send(NetMessage::FileMeta(meta)).await.unwrap();
    
    for chunk in chunks {
        println!("sender: sending chunk {}", chunk.index);
        tx.send(Duration::from_millis(500)).await;
    }
    
    println!("sender: done")
}