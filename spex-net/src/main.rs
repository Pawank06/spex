use std::any::Any;

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
        tx.send(NetMessage::Chunk(chunk)).await.unwrap();
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("sender: done")
}

async fn receiver(mut rx: mpsc::Receiver<NetMessage>) {
    while let Some(msg) = rx.recv().await {
        match msg {
            NetMessage::FileMeta(meta) => {
                println!("reciever: got metadata ({} chunks, {} bytes)", meta.total_chunks, meta.file_size);
            }
            
            NetMessage::Chunk(chunk) => {
                println!("receiver: got chunk {} ({} bytes)",
                chunk.index,
                chunk.data.len());
            }
            
            NetMessage::RequestChunk { index } => {
                println!("reciver: requested chunk {index}");
            }
        }
    }
    println!("reciver: channel closed");
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(10);
    
    let sender_task = tokio::spawn(sender(tx));
    let receiver_task = tokio::spawn(receiver(rx));
    
    let _ = tokio::join!(sender_task, receiver_task);
}