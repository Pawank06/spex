use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;

use tokio::net::UdpSocket;

use spex_core::chunk::Chunk;
use spex_core::io::write_file;
use spex_core::metadata::FileMeta;
use spex_core::reassemble::reassemble_and_verify;

use tracing::{debug, info, warn};

use crate::config::Config;
use crate::error::Result;
use crate::protocol::NetMessage;

#[derive(Debug, Default)]
pub struct PendingChunk {
    pub last_request: Option<Instant>,
    pub attempts: u32,
}

pub struct ReceiverState {
    pub meta: Option<FileMeta>,
    pub chunks: HashMap<u64, Chunk>,
    pub pending: HashMap<u64, PendingChunk>,
}

impl ReceiverState {
    pub fn new() -> Self {
        Self {
            meta: None,
            chunks: HashMap::new(),
            pending: HashMap::new(),
        }
    }
}

impl Default for ReceiverState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run(
    socket: UdpSocket,
    sender_addr: SocketAddr,
    out_path: PathBuf,
    cfg: Config,
) -> Result<()> {
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
                state.pending.remove(&chunk.index);
                state.chunks.insert(chunk.index, chunk);
            }

            NetMessage::RequestChunk { .. } => {}
        }

        request_missing(&mut state, &socket, sender_addr, &cfg).await?;
        if try_reassemble(&state, &out_path)? {
            return Ok(());
        }
        if all_attempts_exhausted(&state, &cfg) {
            return Err(crate::error::NetError::RetriesExhausted);
        }
    }
}

fn all_attempts_exhausted(state: &ReceiverState, cfg: &Config) -> bool {
    let total = match &state.meta {
        Some(m) => m.total_chunks,
        None => return false,
    };

    let outstanding = total as usize - state.chunks.len();
    if outstanding == 0 {
        return false;
    }

    state
        .pending
        .values()
        .filter(|p| p.attempts >= cfg.max_retries)
        .count()
        >= outstanding
}

async fn request_missing(
    state: &mut ReceiverState,
    socket: &UdpSocket,
    sender_addr: SocketAddr,
    cfg: &Config,
) -> Result<()> {
    let total_chunks = match &state.meta {
        Some(m) => m.total_chunks,
        None => return Ok(()),
    };

    let now = Instant::now();
    let retry_after = std::time::Duration::from_millis(cfg.retry_after_ms);

    for index in 0..total_chunks {
        if state.chunks.contains_key(&index) {
            continue;
        }

        let entry = state.pending.entry(index).or_default();

        let backoff = retry_after * (1 << entry.attempts.min(5));
        let due = match entry.last_request {
            Some(t) => now.duration_since(t) >= backoff,
            None => true,
        };

        if !due {
            continue;
        }

        if entry.attempts >= cfg.max_retries {
            warn!(index, "giving up on chunk");
            continue;
        }

        debug!(index, attempt = entry.attempts + 1, "requesting chunk");
        entry.last_request = Some(now);
        entry.attempts += 1;

        let bytes = bincode::serialize(&NetMessage::RequestChunk { index })?;
        socket.send_to(&bytes, sender_addr).await?;
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
