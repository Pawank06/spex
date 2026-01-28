use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use rand::seq::SliceRandom;
use rand::thread_rng;

use spex_net::protocol::NetMessage;
use spex_core::chunk::{chunks_bytes as chunk_bytes, Chunk};
use spex_core::metadata::FileMeta;
use spex_core::reassemble::reassemble_and_verify;

//
// -------------------- Sender --------------------
//

async fn sender(
    tx: mpsc::Sender<Vec<u8>>,
    mut rx: mpsc::Receiver<Vec<u8>>,
) {
    let data = b"hello world";
    let chunk_size = 3;

    let mut chunks = chunk_bytes(data, chunk_size);
    let meta = FileMeta::new(data, chunk_size, &chunks);

    // Store chunks for resend
    let mut chunk_store: HashMap<u64, Chunk> = HashMap::new();
    for chunk in &chunks {
        chunk_store.insert(chunk.index, chunk.clone());
    }

    println!("sender: sending metadata");
    let meta_bytes = bincode::serialize(&NetMessage::FileMeta(meta)).unwrap();
    tx.send(meta_bytes).await.unwrap();

    chunks.shuffle(&mut thread_rng());

    for chunk in chunks {
        if chunk.index == 1 {
            println!("sender: intentionally dropping chunk {}", chunk.index);
            continue;
        }

        println!("sender: sending chunk {}", chunk.index);
        let bytes = bincode::serialize(&NetMessage::Chunk(chunk)).unwrap();
        tx.send(bytes).await.unwrap();

        sleep(Duration::from_millis(300)).await;
    }

    while let Some(bytes) = rx.recv().await {
        let msg: NetMessage = bincode::deserialize(&bytes).unwrap();

        if let NetMessage::RequestChunk { index } = msg {
            if let Some(chunk) = chunk_store.get(&index) {
                println!("sender: resending chunk {index}");
                let bytes =
                    bincode::serialize(&NetMessage::Chunk(chunk.clone())).unwrap();
                tx.send(bytes).await.unwrap();
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
    mut rx: mpsc::Receiver<Vec<u8>>,
    tx: mpsc::Sender<Vec<u8>>,
) {
    let mut state = ReceiverState {
        meta: None,
        chunks: HashMap::new(),
        requested: HashSet::new(),
    };

    while let Some(bytes) = rx.recv().await {
        let msg: NetMessage = bincode::deserialize(&bytes).unwrap();

        match msg {
            NetMessage::FileMeta(meta) => {
                println!(
                    "receiver: got metadata ({} chunks)",
                    meta.total_chunks
                );
                state.meta = Some(meta);
            }

            NetMessage::Chunk(chunk) => {
                println!("receiver: got chunk {}", chunk.index);
                state.chunks.insert(chunk.index, chunk);
            }

            NetMessage::RequestChunk { .. } => {}
        }

        request_missing_chunks(&mut state, &tx);
        try_reassemble(&state);
    }
}

//
// -------------------- Helpers --------------------
//

fn request_missing_chunks(
    state: &mut ReceiverState,
    tx: &mpsc::Sender<Vec<u8>>,
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

            let tx = tx.clone();
            tokio::spawn(async move {
                let bytes = bincode::serialize(
                    &NetMessage::RequestChunk { index }
                )
                .unwrap();

                tx.send(bytes).await.unwrap();
            });
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
async fn main() {
    let (tx_to_receiver, rx_from_sender) = mpsc::channel::<Vec<u8>>(16);
    let (tx_to_sender, rx_from_receiver) = mpsc::channel::<Vec<u8>>(16);

    let sender_task =
        tokio::spawn(sender(tx_to_receiver, rx_from_receiver));
    let receiver_task =
        tokio::spawn(receiver(rx_from_sender, tx_to_sender));

    let _ = tokio::join!(sender_task, receiver_task);
}
