use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

use rand::seq::SliceRandom;
use rand::thread_rng;
use tracing::{debug, info};

use spex_core::chunk::{chunks_bytes as chunk_bytes, Chunk};
use spex_core::io::read_file;
use spex_core::metadata::FileMeta;

use crate::config::Config;
use crate::error::Result;
use crate::protocol::NetMessage;

pub async fn run(
    socket: UdpSocket,
    receiver_addr: SocketAddr,
    path: PathBuf,
    cfg: Config,
) -> Result<()> {
    let data = read_file(&path)?;

    let chunks = chunk_bytes(&data, cfg.chunk_size);
    let meta = FileMeta::new(&data, cfg.chunk_size, &chunks);

    let mut chunk_store: HashMap<u64, Arc<Chunk>> = HashMap::new();
    for chunk in &chunks {
        chunk_store.insert(chunk.index, Arc::new(chunk.clone()));
    }

    info!(total = meta.total_chunks, "sending metadata");
    let meta_bytes = bincode::serialize(&NetMessage::FileMeta(meta))?;
    socket.send_to(&meta_bytes, receiver_addr).await?;

    let mut order: Vec<u64> = (0..chunks.len() as u64).collect();
    order.shuffle(&mut thread_rng());

    for index in order {
        let chunk = chunk_store.get(&index).expect("chunk in store");
        debug!(index, "sending chunk");
        let bytes = bincode::serialize(&NetMessage::Chunk(chunk.as_ref().clone()))?;
        socket.send_to(&bytes, receiver_addr).await?;

        sleep(Duration::from_millis(cfg.send_delay_ms)).await;
    }

    let mut buf = [0u8; 2048];
    let idle = Duration::from_millis(cfg.idle_timeout_ms);
    loop {
        let recv = tokio::time::timeout(idle, socket.recv_from(&mut buf)).await;
        let (len, _) = match recv {
            Ok(res) => res?,
            Err(_) => {
                info!("sender idle timeout, exiting");
                return Ok(());
            }
        };

        let msg: NetMessage = bincode::deserialize(&buf[..len])?;

        if let NetMessage::RequestChunk { index } = msg {
            if let Some(chunk) = chunk_store.get(&index) {
                debug!(index, "resending chunk");
                let bytes = bincode::serialize(
                    &NetMessage::Chunk(chunk.as_ref().clone())
                )?;

                socket.send_to(&bytes, receiver_addr).await?;
            }
        }
    }
}
