use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};
use crate::{chunk::Chunk};

pub type Hash = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

pub fn hash_bytes(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash_chunk(chunk: &Chunk) -> Hash {
    hash_bytes(&chunk.data)
}

pub fn hash_file(chunks: &[Chunk]) -> Hash {
    let mut hasher = Sha256::new();
    
    for chunk in chunks {
        hasher.update(&chunk.data);
    }
    
    hasher.finalize().into()
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

#[cfg(test)]
mod file_tests {
    use super::*;
    use crate::chunk::Chunk;

    #[test]
    fn same_file_same_hash() {
        let chunks = vec![
            Chunk { index: 0, data: b"hel".to_vec() },
            Chunk { index: 1, data: b"lo ".to_vec() },
            Chunk { index: 2, data: b"world".to_vec() },
        ];

        let h1 = hash_file(&chunks);
        let h2 = hash_file(&chunks);

        assert_eq!(h1, h2);
    }

    #[test]
    fn different_file_different_hash() {
        let chunks1 = vec![
            Chunk { index: 0, data: b"hello".to_vec() },
        ];

        let chunks2 = vec![
            Chunk { index: 0, data: b"world".to_vec() },
        ];

        let h1 = hash_file(&chunks1);
        let h2 = hash_file(&chunks2);

        assert_ne!(h1, h2);
    }

    #[test]
    fn chunk_order_affects_file_hash() {
        let chunks1 = vec![
            Chunk { index: 0, data: b"ab".to_vec() },
            Chunk { index: 1, data: b"cd".to_vec() },
        ];

        let chunks2 = vec![
            Chunk { index: 0, data: b"cd".to_vec() },
            Chunk { index: 1, data: b"ab".to_vec() },
        ];

        let h1 = hash_file(&chunks1);
        let h2 = hash_file(&chunks2);

        assert_ne!(h1, h2);
    }
}
