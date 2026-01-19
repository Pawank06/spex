use sha2::{Digest, Sha256};

use crate::chunk::Chunk;

pub type Hash = [u8; 32];

pub fn hash_bytes(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash_chunk(chunk: &Chunk) -> Hash {
    hash_bytes(&chunk.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;
    
    #[test]
    fn same_bytes_produce_same_hash() {
        let h1 = hash_bytes(b"hello world");
        let h2 = hash_bytes(b"hello world");
        
        assert_eq!(h1, h2);
    }
    
    #[test]
    fn different_bytes_produce_different_hash() {
        let h1 = hash_bytes(b"hello");
        let h2 = hash_bytes(b"world");
        
        assert_ne!(h1, h2);
    }
    
    #[test]
    fn same_chunk_data_same_hash() {
        let c1 = Chunk {
            index: 0,
            data: b"hello".to_vec(),
        };
        
        let c2 = Chunk {
            index: 1,
            data: b"hello".to_vec()
        };
        
        let hash1 = hash_chunk(&c1);
        let hash2 = hash_chunk(&c2);
        
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn different_chunk_data_different_hash() {
        let c1 = Chunk {
            index: 0,
            data: b"hello".to_vec(),
        };
        
        let c2 = Chunk {
            index: 1,
            data: b"world".to_vec()
        };
        
        let hash1 = hash_chunk(&c1);
        let hash2 = hash_chunk(&c2);
        
        assert_ne!(hash1, hash2);
    }
    
}