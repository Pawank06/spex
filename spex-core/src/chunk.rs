use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub index: u64,
    pub data: Vec<u8>
}

pub fn chunks_bytes(data: &[u8], chunk_size: usize) -> Vec<Chunk> {
    data.chunks(chunk_size)
        .enumerate()
        .map(|(i, slice)| Chunk {
            index: i as u64,
            data: slice.to_vec(),
        })
        
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn splits_data_into_chunks() {
        let data = b"hello world";
        let chunks = chunks_bytes(data, 5);
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].data, b"hello");
        assert_eq!(chunks[1].data, b" worl");
        assert_eq!(chunks[2].data, b"d");
    }
}