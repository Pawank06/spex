use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

use tokio::net::UdpSocket;

use spex_core::chunk::Chunk;
use spex_core::metadata::FileMeta;
use spex_core::reassemble::reassemble_and_verify;

use crate::protocol::NetMessage;

pub struct State {
    pub meta: Option<FileMeta>,
    pub chunks: HashMap<u64, Chunk>,
    pub requested: HashSet<u64>,
}

pub async fn run(socket: UdpSocket, sender_addr: SocketAddr) {
    let mut state = State {
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

        request_missing(&mut state, &socket, sender_addr).await;
        try_reassemble(&state);
    }
}

async fn request_missing(
    state: &mut State,
    socket: &UdpSocket,
    sender_addr: SocketAddr,
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

fn try_reassemble(state: &State) {
    let meta = match &state.meta {
        Some(m) => m,
        None => return,
    };

    if state.chunks.len() != meta.total_chunks as usize {
        return;
    }

    println!("receiver: all chunks received, reassembling");

    let chunks: Vec<Chunk> = state.chunks.values().cloned().collect();

    match reassemble_and_verify(meta, chunks) {
        Ok(data) => {
            println!(
                "receiver: file verified and reconstructed ({} bytes)",
                data.len()
            );
        }
        Err(err) => {
            println!("receiver: verification failed: {:?}", err);
        }
    }
}
