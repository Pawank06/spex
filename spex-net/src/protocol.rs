use serde::{Serialize, Deserialize};
use spex_core::metadata::FileMeta;
use spex_core::chunk::Chunk;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetMessage {
    FileMeta(FileMeta),
    Chunk(Chunk),
    RequestChunk { index: u64 }
}