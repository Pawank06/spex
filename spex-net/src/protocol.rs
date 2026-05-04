use serde::{Serialize, Deserialize};
use spex_core::metadata::FileMeta;
use spex_core::chunk::Chunk;

/// Maximum size of a serialized `NetMessage`. Matches the receive buffer
/// used in both the sender and receiver loops.
pub const MAX_DATAGRAM: usize = 4096;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetMessage {
    FileMeta(FileMeta),
    Chunk(Chunk),
    RequestChunk { index: u64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_chunk_roundtrips() {
        let msg = NetMessage::RequestChunk { index: 7 };
        let bytes = bincode::serialize(&msg).unwrap();
        let decoded: NetMessage = bincode::deserialize(&bytes).unwrap();

        match decoded {
            NetMessage::RequestChunk { index } => assert_eq!(index, 7),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn chunk_roundtrips() {
        let chunk = Chunk { index: 3, data: b"abc".to_vec() };
        let msg = NetMessage::Chunk(chunk.clone());
        let bytes = bincode::serialize(&msg).unwrap();
        let decoded: NetMessage = bincode::deserialize(&bytes).unwrap();

        match decoded {
            NetMessage::Chunk(c) => {
                assert_eq!(c.index, chunk.index);
                assert_eq!(c.data, chunk.data);
            }
            _ => panic!("wrong variant"),
        }
    }
}