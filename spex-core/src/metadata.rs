use crate::{
    chunk::Chunk,
    hash::{Hash, hash_file},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileMeta {
    pub file_hash: Hash,
    pub chunk_size: usize,
    pub total_chunks: u64,
    pub file_size: u64,
}

impl FileMeta {
    pub fn new(file_bytes: &[u8], chunk_size: usize, chunks: &[Chunk]) -> Self {
        Self {
            file_hash: hash_file(chunks),
            chunk_size,
            total_chunks: chunks.len() as u64,
            file_size: file_bytes.len() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::chunks_bytes;

    use super::*;

    #[test]
    fn metadata_matches_file() {
        let data = b"hello world";
        let chunk_size = 3;
        let chunks = chunks_bytes(data, chunk_size);

        let meta = FileMeta::new(data, chunk_size, &chunks);

        assert_eq!(meta.total_chunks, chunks.len() as u64);
        assert_eq!(meta.file_size, data.len() as u64);
    }

    #[test]
    fn metadata_changes_with_chunk_size() {
        let data = b"hello world";
        let small = FileMeta::new(data, 2, &chunks_bytes(data, 2));
        let large = FileMeta::new(data, 8, &chunks_bytes(data, 8));

        assert_ne!(small.total_chunks, large.total_chunks);
        assert_eq!(small.file_size, large.file_size);
    }
}
