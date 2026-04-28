use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;

use tokio::net::UdpSocket;

use spex_core::chunk::Chunk;
use spex_core::io::write_file;
use spex_core::metadata::FileMeta;
use spex_core::reassemble::reassemble_and_verify;

use tracing::{debug, info};

use crate::error::Result;
use crate::protocol::NetMessage;

pub struct ReceiverState {
    pub meta: Option<FileMeta>,
    pub chunks: HashMap<u64, Chunk>,
    pub requested: HashSet<u64>,
}

impl ReceiverState {
    pub fn new() -> Self {
        Self {
            meta: None,
            chunks: HashMap::new(),
            requested: HashSet::new(),
        }
    }
}

impl Default for ReceiverState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run(socket: UdpSocket, sender_addr: SocketAddr, out_path: PathBuf) -> Result<()> {
    let mut state = ReceiverState::new();

    let mut buf = [0u8; 2048];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;

        let msg: NetMessage = bincode::deserialize(&buf[..len])?;

        match msg {
            NetMessage::FileMeta(meta) => {
                info!(chunks = meta.total_chunks, "got metadata");
                state.meta = Some(meta);
            }

            NetMessage::Chunk(chunk) => {
                debug!(index = chunk.index, "got chunk");
                state.chunks.insert(chunk.index, chunk);
            }

            NetMessage::RequestChunk { .. } => {}
        }

        request_missing(&mut state, &socket, sender_addr).await?;
        if try_reassemble(&state, &out_path)? {
            return Ok(());
        }
    }
}

async fn request_missing(
    state: &mut ReceiverState,
    socket: &UdpSocket,
    sender_addr: SocketAddr,
) -> Result<()> {
    let meta = match &state.meta {
        Some(m) => m,
        None => return Ok(()),
    };

    for index in 0..meta.total_chunks {
        if !state.chunks.contains_key(&index) && !state.requested.contains(&index) {
            debug!(index, "requesting missing chunk");
            state.requested.insert(index);

            let bytes = bincode::serialize(&NetMessage::RequestChunk { index })?;

            socket.send_to(&bytes, sender_addr).await?;
        }
    }

    Ok(())
}

fn try_reassemble(state: &ReceiverState, out_path: &std::path::Path) -> Result<bool> {
    let meta = match &state.meta {
        Some(m) => m,
        None => return Ok(false),
    };

    if state.chunks.len() != meta.total_chunks as usize {
        return Ok(false);
    }

    info!("all chunks received, reassembling");

    let chunks: Vec<Chunk> = state.chunks.values().cloned().collect();
    let data = reassemble_and_verify(meta, chunks)?;

    write_file(out_path, &data)?;
    info!(bytes = data.len(), path = %out_path.display(), "wrote output");

    Ok(true)
}
