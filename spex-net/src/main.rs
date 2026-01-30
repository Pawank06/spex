use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

use rand::seq::SliceRandom;
use rand::thread_rng;

use spex_net::protocol::NetMessage;
use spex_core::chunk::{chunks_bytes as chunk_bytes, Chunk};
use spex_core::metadata::FileMeta;
use spex_core::reassemble::reassemble_and_verify;

async fn sender(
    socket: UdpSocket,
    receiver_addr: SocketAddr,
) {
    let data = b"hello world";
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
                println!("sender: resending chunk {index}");
                let bytes = bincode::serialize(
                    &NetMessage::Chunk(chunk.clone())
                ).unwrap();
                
                socket.send_to(&bytes, receiver_addr).await.unwrap();
            }
        }
    }
}

//
// -------------------- Receiver State --------------------
//

struct ReceiverState {
    meta: Option<FileMeta>,
    chunks: HashMap<u64, Chunk>,
    requested: HashSet<u64>,
}

//
// -------------------- Receiver --------------------
//

async fn receiver(
    socket: UdpSocket,
    sender_addr: SocketAddr
) {
    let mut state = ReceiverState {
        meta: None,
        chunks: HashMap::new(),
        requested: HashSet::new(),
    };

    let mut buf = [0u8; 2048];
    
    loop {
        let (len, _) = socket.recv_from(&mut buf).await.unwrap();
        
        let msg: NetMessage =
            bincode::deserialize(&buf[..len]).unwrap();
        
        match msg {
            NetMessage::FileMeta(meta) => {
                println!("receiver: got metadata ({} chunks)", meta.total_chunks);
            },
            
            NetMessage::Chunk(chunk) => {
                println!("receiver: got chunk {}", chunk.index);
                state.chunks.insert(chunk.index, chunk);
            }
            
            NetMessage::RequestChunk { .. } => {}
        }
        
        request_missing_chunks(&mut state, &socket, sender_addr).await;
        try_reassemble(&state);
    }
}

//
// -------------------- Helpers --------------------
//

async fn request_missing_chunks(
    state: &mut ReceiverState,
    socket: &UdpSocket,
    sender_addr: SocketAddr
) {
    let meta = match &state.meta {
        Some(m) => m,
        None => return,
    };

    for index in 0..meta.total_chunks {
        if !state.chunks.contains_key(&index)
            && !state.requested.contains(&index)
        {
            println!("receiver: requesting missing chunk {index}");
            state.requested.insert(index);

            let bytes = bincode::serialize(
                &NetMessage::RequestChunk { index }
            ).unwrap();
            
            socket.send_to(&bytes, sender_addr).await.unwrap();
        }
    }
}

fn try_reassemble(state: &ReceiverState) {
    let meta = match &state.meta {
        Some(m) => m,
        None => return,
    };

    if state.chunks.len() != meta.total_chunks as usize {
        return;
    }

    println!("receiver: all chunks received, reassembling…");

    let chunks: Vec<Chunk> = state.chunks.values().cloned().collect();

    match reassemble_and_verify(meta, chunks) {
        Ok(data) => {
            println!(
                "receiver: file verified and reconstructed ({} bytes)",
                data.len()
            );
        }
        Err(err) => {
            println!("❌ receiver: verification failed: {:?}", err);
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sender_socket = UdpSocket::bind("127.0.0.1:7001").await?;
    let receiver_socket = UdpSocket::bind("127.0.0.1:7002").await?;
    
    let sender_addr = "127.0.0.1:7002".parse().unwrap();
    let receiver_addr = "127.0.0.1:7001".parse().unwrap();

    let sender_task =
        tokio::spawn(sender(sender_socket, sender_addr));
    let receiver_task =
        tokio::spawn(receiver(receiver_socket, receiver_addr));

    let _ = tokio::join!(sender_task, receiver_task);
    
    Ok(())
}
