use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

pub fn hash_bytes(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
}