use crate::chunk::Chunk;

pub enum ReassemblerError {
    MissingChunk { expected: u64, found: u64},
    DuplicateChunk { index: u64 }
}

pub fn reassemble_chunks(mut chunks: Vec<Chunk>) -> Result<Vec<u8>, ReassemblerError> {
    chunks.sort_by_key(|item| item.index);
    
    for (i, chunk)in chunks.iter().enumerate() {
        let expected_index = i as u64;
        
        if chunk.index != expected_index {
            return Err(ReassemblerError::MissingChunk { 
                expected: expected_index, 
                found: chunk.index
            });
        }
    }
    
    let mut result = Vec::new();
    
    for chunk in chunks {
        result.extend_from_slice(&chunk.data);
    }
    
    Ok(result)
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