use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

pub fn hash_bytes(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let digest = hasher.finalize();
    let bytes: [u8; 32] = digest.into();

    Hash(bytes)
}

use crate::chunk::Chunk;

pub fn hash_file(chunks: &[Chunk]) -> Hash {
    let mut hasher = Sha256::new();

    for chunk in chunks {
        hasher.update(&chunk.data);
    }

    let digest = hasher.finalize();
    let bytes: [u8; 32] = digest.into();

    Hash(bytes)
}
