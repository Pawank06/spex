use crate::chunk::Chunk;

#[derive(Debug, PartialEq)] 
pub enum ReassembleError {
    EmptyInput,
    MissingChunk { expected: u64, found: u64},
    DuplicateChunk { index: u64 }
}

pub fn reassemble_chunks(mut chunks: Vec<Chunk>) -> Result<Vec<u8>, ReassembleError> {
    if chunks.is_empty() {
        return Err(ReassembleError::EmptyInput);
    }
    chunks.sort_by_key(|item| item.index);
    
    let mut expected_index = 0;
    
    for chunk in &chunks {
        if chunk.index < expected_index {
            return Err(ReassembleError::DuplicateChunk { index: chunk.index });
        }
        
        if chunk.index != expected_index {
            return  Err(ReassembleError::MissingChunk { expected: expected_index, found: chunk.index });
        }
        
        expected_index += 1;
    }
    
    let mut result = Vec::new();
    
    for chunk in chunks {
        result.extend_from_slice(&chunk.data);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;

    #[test]
    fn joins_data_from_chunks_in_correct_order() {
        let chunks = vec![
            Chunk { index: 2, data: b"world".to_vec() },
            Chunk { index: 0, data: b"hel".to_vec() },
            Chunk { index: 1, data: b"lo ".to_vec() },
        ];

        let result = reassemble_chunks(chunks).unwrap();

        assert_eq!(result, b"hello world");
    }

    #[test]
    fn fails_on_empty_input() {
        let chunks = vec![];

        let result = reassemble_chunks(chunks);

        assert!(matches!(result, Err(ReassembleError::EmptyInput)));
    }

    #[test]
    fn fails_when_chunk_is_missing() {
        let chunks = vec![
            Chunk { index: 0, data: b"hel".to_vec() },
            Chunk { index: 2, data: b"world".to_vec() },
        ];

        let result = reassemble_chunks(chunks);

        assert!(matches!(
            result,
            Err(ReassembleError::MissingChunk { expected: 1, found: 2 })
        ));
    }

    #[test]
    fn fails_on_duplicate_chunk() {
        let chunks = vec![
            Chunk { index: 0, data: b"a".to_vec() },
            Chunk { index: 1, data: b"b".to_vec() },
            Chunk { index: 1, data: b"b".to_vec() },
        ];

        let result = reassemble_chunks(chunks);

        assert!(matches!(
            result,
            Err(ReassembleError::DuplicateChunk { index: 1 })
        ));
    }

    #[test]
    fn fails_when_first_chunk_is_not_zero() {
        let chunks = vec![
            Chunk { index: 1, data: b"lo ".to_vec() },
            Chunk { index: 2, data: b"world".to_vec() },
        ];

        let result = reassemble_chunks(chunks);

        assert!(matches!(
            result,
            Err(ReassembleError::MissingChunk { expected: 0, found: 1 })
        ));
    }
}
