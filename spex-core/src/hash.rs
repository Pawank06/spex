use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// SHA-256 digest used to identify and verify files and chunks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

/// Hashes a single byte slice with SHA-256.
pub fn hash_bytes(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let digest = hasher.finalize();
    let bytes: [u8; 32] = digest.into();

    Hash(bytes)
}

use crate::chunk::Chunk;

/// Hashes the concatenation of chunk payloads in the order they are given.
pub fn hash_file(chunks: &[Chunk]) -> Hash {
    let mut hasher = Sha256::new();

    for chunk in chunks {
        hasher.update(&chunk.data);
    }

    let digest = hasher.finalize();
    let bytes: [u8; 32] = digest.into();

    Hash(bytes)
}
