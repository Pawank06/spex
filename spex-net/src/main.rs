use rand::seq::SliceRandom;
use rand::thread_rng;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use spex_net::protocol::NetMessage;
use spex_core::chunk::{chunks_bytes, Chunk};
use spex_core::metadata::FileMeta;
use std::collections::HashMap;

struct RecieverState {
    meta: Option<FileMeta>,
    chunks: HashMap<u64, Chunk>
}

async fn sender(mut tx: mpsc::Sender<NetMessage>,
    mut rx: mpsc::Receiver<NetMessage>,) {
    use std::collections::HashMap;
        
    let mut chunk_store: HashMap<u64, Chunk> = HashMap::new();

    let data = b"hello world";
    let chunk_size = 3;
    
    let mut chunks = chunks_bytes(data, chunk_size);
    let meta = FileMeta::new(data, chunk_size, &chunks);
    
    println!("sender: sending metadata");
    tx.send(NetMessage::FileMeta(meta)).await.unwrap();
    
    chunks.shuffle(&mut thread_rng());
    
    for chunk in chunks {
        println!("sender: sending chunk {}", chunk.index);
        tx.send(NetMessage::Chunk(chunk)).await.unwrap();
        sleep(Duration::from_millis(200)).await;
    }
    
    println!("sender: done")
}

async fn receiver(mut rx: mpsc::Receiver<NetMessage>) {
    use spex_core::reassemble::reassemble_and_verify;
    
    let mut state = RecieverState {
        meta: None,
        chunks: HashMap::new(),
    };
    
    while let Some(msg) = rx.recv().await {
        match msg {
            NetMessage::FileMeta(meta) => {
                println!("reciever: got metadata ({} chunks)", meta.total_chunks);
                
                state.meta = Some(meta);
            }
            
            NetMessage::Chunk(chunk) => {
                println!("receiver: got chunk {}",
                chunk.index);
                state.chunks.insert(chunk.index, chunk);
            }
            
            NetMessage::RequestChunk { index } => {
                println!("receiver: requested chunk {index}");
            }
        }
    }
    try_reassemble(&mut state);
}

fn try_reassemble(state: &mut RecieverState) {
    let meta = match &state.meta {
        Some(m) => m,
        None => return
    };
    
    if state.chunks.iter().len() != meta.total_chunks as usize {
        return;
    }
    
    println!("receiver: all chunks verified, reassembling...");
    
    let mut chunks: Vec<Chunk> = state.chunks.values().cloned().collect();
    
    match spex_core::reassemble::reassemble_and_verify(meta, chunks) {
        Ok(data) => {
            println!("receiver: file verified and reconstructed ({} bytes)", data.len());
        }
        Err(err) => {
            println!("reciver: verification failed: {:?}", err)
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx_to_receiver, rx_from_sender) = mpsc::channel(10);
    let (tx_to_sender, rx_from_receiver) = mpsc::channel(10);
    
    let sender_task = tokio::spawn(sender(tx_to_receiver, rx_from_receiver));
    let receiver_task = tokio::spawn(receiver(rx_from_sender, tx_to_sender));
    
    let _ = tokio::join!(sender_task, receiver_task);
}