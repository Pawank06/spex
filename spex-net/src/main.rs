use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use rand::seq::SliceRandom;
use rand::thread_rng;

use spex_net::protocol::NetMessage;
use spex_core::chunk::{chunks_bytes, Chunk};
use spex_core::metadata::FileMeta;
use spex_core::reassemble::reassemble_and_verify;

//
// -------------------- Sender --------------------
//


async fn sender(
    mut tx: mpsc::Sender<NetMessage>,
    mut rx: mpsc::Receiver<NetMessage>,
) {
    let data = b"hello world";
    let chunk_size = 3;

    let mut chunks = chunks_bytes(data, chunk_size);
    let meta = FileMeta::new(data, chunk_size, &chunks);

    let mut chunk_store: HashMap<u64, Chunk> = HashMap::new();
    for chunk in &chunks {
        chunk_store.insert(chunk.index, chunk.clone());
    }

    println!("sender: sending metadata");
    tx.send(NetMessage::FileMeta(meta)).await.unwrap();

    chunks.shuffle(&mut thread_rng());

    for chunk in chunks {
        if chunk.index == 1 {
            println!("sender: intentionally dropping chunk {}", chunk.index);
            continue;
        }

        println!("sender: sending chunk {}", chunk.index);
        tx.send(NetMessage::Chunk(chunk)).await.unwrap();
        sleep(Duration::from_millis(300)).await;
    }

    while let Some(msg) = rx.recv().await {
        if let NetMessage::RequestChunk { index } = msg {
            if let Some(chunk) = chunk_store.get(&index) {
                println!("sender: resending chunk {index}");
                tx.send(NetMessage::Chunk(chunk.clone())).await.unwrap();
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
    mut rx: mpsc::Receiver<NetMessage>,
    tx: mpsc::Sender<NetMessage>,
) {
    let mut state = ReceiverState {
        meta: None,
        chunks: HashMap::new(),
        requested: HashSet::new(),
    };

    let retry_tx = tx.clone();
    let retry_handle = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            // retry logic handled in main loop via shared state
            let _ = &retry_tx;
        }
    });

    while let Some(msg) = rx.recv().await {
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

        try_request_missing(&mut state, &tx);
        try_reassemble(&state);
    }

    retry_handle.abort();
}

//
// -------------------- Helpers --------------------
//

fn try_request_missing(state: &mut ReceiverState, tx: &mpsc::Sender<NetMessage>) {
    let meta = match &state.meta {
        Some(m) => m,
        None => return,
    };

    for index in 0..meta.total_chunks {
        if !state.chunks.contains_key(&index) && !state.requested.contains(&index) {
            println!("receiver: requesting missing chunk {index}");
            state.requested.insert(index);

            let tx = tx.clone();
            tokio::spawn(async move {
                tx.send(NetMessage::RequestChunk { index })
                    .await
                    .unwrap();
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

//
// -------------------- Main --------------------
//

#[tokio::main]
async fn main() {
    let (tx_to_receiver, rx_from_sender) = mpsc::channel(16);
    let (tx_to_sender, rx_from_receiver) = mpsc::channel(16);

    let sender_task = tokio::spawn(sender(tx_to_receiver, rx_from_receiver));
    let receiver_task = tokio::spawn(receiver(rx_from_sender, tx_to_sender));

    let _ = tokio::join!(sender_task, receiver_task);
}
