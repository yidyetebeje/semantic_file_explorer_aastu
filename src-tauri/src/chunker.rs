use text_splitter::TextSplitter;
use log::{debug, info};
use thiserror::Error;

/// Default chunk size range (in characters)
/// We use a range to allow flexibility in chunk boundaries
/// Min: 500 characters (about 100 tokens)
/// Max: 1500 characters (about 300-350 tokens)
const DEFAULT_CHUNK_SIZE_RANGE: std::ops::Range<usize> = 500..1500;

/// Maximum number of chunks we want to extract and process
/// This is to prevent excessive processing for very large documents
const MAX_CHUNKS: usize = 100;

#[derive(Error, Debug)]
pub enum ChunkerError {
    #[error("Failed to split text into chunks: {0}")]
    SplittingError(String),
}

/// Splits the given text into semantically meaningful chunks.
/// Uses the TextSplitter from the text-splitter crate.
/// 
/// # Arguments
/// * `text` - The text to split into chunks
/// 
/// # Returns
/// * `Result<Vec<String>, ChunkerError>` - A vector of text chunks or an error
pub fn chunk_text(text: &str) -> Result<Vec<String>, ChunkerError> {
    debug!("Chunking text of length {} characters", text.len());
    
    if text.is_empty() {
        debug!("Input text is empty, returning empty chunk list");
        return Ok(Vec::new());
    }
    
    // Use TextSplitter with character count for chunking
    // This uses semantic boundaries (sentences, paragraphs) when possible
    let splitter = TextSplitter::new(DEFAULT_CHUNK_SIZE_RANGE);
    
    // Collect each chunk as String
    let chunks: Vec<String> = splitter.chunks(text)
        .map(|s| s.to_string())
        .collect();
    
    // Limit the number of chunks if necessary
    let chunks = if chunks.len() > MAX_CHUNKS {
        info!("Limiting chunks from {} to {}", chunks.len(), MAX_CHUNKS);
        chunks.into_iter().take(MAX_CHUNKS).collect()
    } else {
        chunks
    };
    
    debug!("Split text into {} chunks", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        debug!("Chunk {}: {} characters", i, chunk.len());
    }
    
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_text_empty() {
        let result = chunk_text("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
    
    #[test]
    fn test_chunk_text_small() {
        let small_text = "This is a small piece of text that should fit in a single chunk.";
        let result = chunk_text(small_text).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], small_text);
    }
    
    #[test]
    fn test_chunk_text_large() {
        // Create a text that's definitely larger than our chunk size
        let large_text = "This is paragraph one.\n\n".repeat(100);
        let result = chunk_text(&large_text).unwrap();
        
        // Should be split into multiple chunks
        assert!(result.len() > 1);
        
        // Each chunk should be smaller than max chunk size
        for chunk in &result {
            assert!(chunk.len() <= DEFAULT_CHUNK_SIZE_RANGE.end);
        }
        
        // Total text in chunks should equal original (except for potential whitespace differences)
        let total_chars: usize = result.iter().map(|s| s.len()).sum();
        // Allow for some margin due to potential boundary adjustments
        assert!(total_chars > large_text.len() * 9 / 10 && total_chars <= large_text.len() * 11 / 10);
    }
    
    #[test]
    fn test_chunk_text_respects_max_chunks() {
        // Create an extremely large text to test MAX_CHUNKS limit
        let huge_text = "This is paragraph one.\n\n".repeat(1000);
        let result = chunk_text(&huge_text).unwrap();
        
        // Should not exceed MAX_CHUNKS
        assert!(result.len() <= MAX_CHUNKS);
    }
} 