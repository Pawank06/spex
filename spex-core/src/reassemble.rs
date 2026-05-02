use crate::{chunk::{Chunk, chunks_bytes}, hash::hash_file, metadata::FileMeta};

/// Reasons reassembly may fail. Used by both `reassemble_chunks`
/// and `reassemble_and_verify` so callers can react to specific failures.
#[derive(Debug, PartialEq)]
pub enum ReassembleError {
    EmptyInput,
    MissingChunk { expected: u64, found: u64 },
    DuplicateChunk { index: u64 },
    FileHashMismatch,
}

/// Joins chunks back into the original byte stream after sorting by index.
/// Returns an error if any expected index is missing or duplicated.
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

/// Reassembles chunks and re-verifies file size and hash against `meta`.
/// Use this when the chunks came from an untrusted source.
pub fn reassemble_and_verify(
    meta: &FileMeta,
    chunks: Vec<Chunk>
) -> Result<Vec<u8>, ReassembleError> {
    let data = reassemble_chunks(chunks)?;
    
    if data.len() as u64 != meta.file_size {
        return Err(ReassembleError::FileHashMismatch);
    }
    
    let reconstructed_chunks = chunks_bytes(&data, meta.chunk_size);
    
    let file_hash = hash_file(&reconstructed_chunks);
    
    if file_hash != meta.file_hash {
        return  Err(ReassembleError::FileHashMismatch);
    }
    
    Ok(data)
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

#[cfg(test)]
mod verified_tests {
    use super::*;
    use crate::chunk::chunks_bytes;
    use crate::metadata::FileMeta;

    #[test]
    fn verified_reassembly_succeeds_for_valid_file() {
        let data = b"hello world";
        let chunk_size = 3;
        let chunks = chunks_bytes(data, chunk_size);
        let meta = FileMeta::new(data, chunk_size, &chunks);

        let result = reassemble_and_verify(&meta, chunks).unwrap();

        assert_eq!(result, data);
    }

    #[test]
    fn verified_reassembly_fails_on_corruption() {
        let data = b"hello world";
        let chunk_size = 3;
        let mut chunks = chunks_bytes(data, chunk_size);

        chunks[0].data[0] = b'X';

        let meta = FileMeta::new(data, chunk_size, &chunks_bytes(data, chunk_size));

        let result = reassemble_and_verify(&meta, chunks);

        assert!(matches!(result, Err(ReassembleError::FileHashMismatch)));
    }
}
