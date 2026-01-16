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
        let mut chunks: Vec<Chunk> = vec![{1; 1}];
        
        let data = reassemble_chunks(chunks);
        
        
    }
}