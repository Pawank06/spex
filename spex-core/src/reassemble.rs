use crate::chunk::Chunk;

pub fn reassemble_chunks(mut chunks: Vec<Chunk>) -> Vec<u8> {
    chunks.sort_by_key(|item| item.index);
    let mut result = Vec::new();
    
    for chunk in chunks {
        result.extend_from_slice(&chunk.data);
    }
    
    result
    
}