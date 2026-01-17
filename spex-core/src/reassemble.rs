use crate::chunk::Chunk;

pub fn reassemble_chunks(mut chunks: Vec<Chunk>) -> Vec<u8> {
    chunks.sort_by_key(|item| item.index);
    
    let mut result = Vec::new();
    
    for chunk in chunks {
        result.extend_from_slice(&chunk.data);
    }
    
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn join_data_from_chunks() {
        let chunks = vec![
            Chunk {
                index: 2,
                data: b"world".to_vec(),
            },
            Chunk {
                index: 0,
                data: b"hel".to_vec(),
            },
            Chunk {
                index: 1,
                data: b"lo ".to_vec(),
            },
        ];
    
        let data = reassemble_chunks(chunks);
    
        assert_eq!(data, b"hello world");
    }
}