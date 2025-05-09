use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use thiserror::Error;
use log::{error, info, debug};
use std::path::PathBuf;
use once_cell::sync::Lazy;
use crate::chunker::{chunk_text, ChunkerError};

// Constants for the embedding model
const MODEL_NAME: EmbeddingModel = EmbeddingModel::BGESmallENV15;
const CACHE_DIR_NAME: &str = ".cache"; // Directory to store cached models

// Define potential errors during embedding
#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Model initialization failed: {0}")]
    InitializationError(String),
    #[error("Embedding generation failed: {0}")]
    GenerationError(String),
    #[error("Model loading error: {0}")]
    ModelLoadError(String),
    #[error("Text chunking error: {0}")]
    ChunkingError(#[from] ChunkerError),
}

// Use Lazy to initialize the model once
// NOTE: The real model initialization still happens even during tests,
// potentially downloading files, due to Lazy evaluation. But the mock
// embed_text function below will prevent it from being *used* during tests.
static MODEL: Lazy<Result<TextEmbedding, EmbeddingError>> = Lazy::new(|| {
    info!("Initializing embedding model (Lazy)...");
    let model_name = EmbeddingModel::BGESmallENV15;

    // Use the builder pattern for InitOptions
    let init_options = InitOptions::new(model_name)
        .with_cache_dir(PathBuf::from(".cache"))
        .with_show_download_progress(true);

    TextEmbedding::try_new(init_options).map_err(|e| {
        let err_msg = format!("Failed to initialize embedding model: {}", e);
        error!("{}", err_msg);
        EmbeddingError::ModelLoadError(err_msg)
    })
});

/// Generates embeddings for the given text content.
/// First chunks the text into semantically meaningful portions,
/// then embeds each chunk separately.
/// 
/// # Arguments
/// * `content` - A slice of strings to embed
/// 
/// # Returns
/// * `Result<Vec<Vec<f32>>, EmbeddingError>` - A vector of embedding vectors or an error
#[cfg(not(test))]
pub fn embed_text(content: &[String], query: bool) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    if !query {
        if content.is_empty() {
            return Ok(Vec::new()); // Return empty vec if no content
        }
    
        // Chunk each text in content and collect all chunks
        let mut all_chunks: Vec<String> = Vec::new();
        let mut chunk_map: Vec<(usize, usize)> = Vec::new(); // Maps original index -> (chunk_start, chunk_count)
        
        for (_i, text) in content.iter().enumerate() {
            let chunks = chunk_text(text)?;
            let chunk_start = all_chunks.len();
            let chunk_count = chunks.len();
            // add passage: prefix on all chunks
            let chunks: Vec<String> = chunks.iter().map(|chunk| format!("{}", chunk)).collect();
            
            chunk_map.push((chunk_start, chunk_count));
            all_chunks.extend(chunks);
        }
        
        info!("Chunked {} texts into {} total chunks", content.len(), all_chunks.len());
        
        // If no chunks were generated, return empty results
        if all_chunks.is_empty() {
            return Ok(Vec::new());
        }
    
        // Access the lazily initialized model and embed all chunks
        match &*MODEL {
            Ok(model) => {
                // Generate embeddings for all chunks
                let all_embeddings = match model.embed(all_chunks, None) {
                    Ok(embeddings) => {
                        debug!("Successfully generated {} embeddings", embeddings.len());
                        embeddings
                    }
                    Err(e) => {
                        let err_msg = format!("Embedding generation failed: {}", e);
                        error!("{}", err_msg);
                        return Err(EmbeddingError::GenerationError(err_msg));
                    }
                };
                
                // Return all embeddings 
                Ok(all_embeddings)
            }
            Err(init_error) => {
                // If initialization failed, return the stored error
                error!("Embedding model initialization failed previously: {}", init_error);
                Err(EmbeddingError::InitializationError(format!("{}", init_error)))
            }
        }
    } else {
        println!("Embedding query: {:?}", content.clone());
        match &*MODEL {
            Ok(model) => {
                // Generate embeddings for all chunks
                let all_embeddings = match model.embed(content.to_vec(), None) {
                    Ok(embeddings) => {
                        debug!("Successfully generated {} embeddings", embeddings.len());
                        embeddings
                    }
                    Err(e) => {
                        let err_msg = format!("Embedding generation failed: {}", e);
                        error!("{}", err_msg);
                        return Err(EmbeddingError::GenerationError(err_msg));
                    }
                };
                
                // Return all embeddings 
                Ok(all_embeddings)
            }
            Err(init_error) => {
                // If initialization failed, return the stored error
                error!("Embedding model initialization failed previously: {}", init_error);
                Err(EmbeddingError::InitializationError(format!("{}", init_error)))
            }
        }
    }
}

/// Generates mock embeddings for testing purposes.
/// (Mock implementation for test builds)
#[cfg(test)]
pub fn embed_text(content: &[String], query:bool) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    if content.is_empty() {
        return Ok(Vec::new());
    }

    // Process each text to get all chunks
    let mut all_chunks: Vec<String> = Vec::new();
    
    for text in content {
        match chunk_text(text) {
            Ok(chunks) => all_chunks.extend(chunks),
            Err(e) => return Err(EmbeddingError::from(e)),
        }
    }
    
    info!("Generating MOCK embeddings for {} text chunks...", all_chunks.len());
    const MOCK_DIMENSION: usize = 384; // Match BGE-Small-EN-v1.5 dimension

    // Create different mock embeddings for different text
    let embeddings = all_chunks.iter().map(|text| {
        // Create a unique embedding for each different text
        // by using the text content to seed values
        let mut vec = vec![0.1f32; MOCK_DIMENSION];
        
        // Use characters from the text to differentiate embeddings
        for (j, c) in text.chars().enumerate() {
            if j < MOCK_DIMENSION {
                // Use character code to create different values
                vec[j] = (c as u32 % 100) as f32 / 100.0;
            } else {
                break;
            }
        }
        
        // This ensures that identical text produces identical embeddings
        // and different text produces different embeddings
        vec
    }).collect();

    Ok(embeddings)
}

// Helper function to get the chunk count for a piece of text
// This is useful for database operations to know how many records to expect
pub fn get_chunk_count(text: &str) -> Result<usize, EmbeddingError> {
    let chunks = chunk_text(text)?;
    Ok(chunks.len())
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from parent module

    // Test successful embedding with chunking
    #[test]
    fn test_embed_text_success() {
        // This should create multiple chunks
        let texts = vec!["Hello world. This is a test. ".repeat(50)];
        let result = embed_text(&texts, false);

        match result {
            Ok(embeddings) => {
                // Should have produced multiple embeddings due to chunking
                assert!(embeddings.len() > 1);
                // BGE-Small-EN-v1.5 has dimension 384
                assert_eq!(embeddings[0].len(), 384);
                // Check embeddings are not identical
                if embeddings.len() > 1 {
                    assert_ne!(embeddings[0], embeddings[1]);
                }
            }
            Err(e) => {
                panic!("Embedding failed when it should have succeeded: {}", e);
            }
        }
    }

    #[test]
    fn test_embed_empty_list() {
        let texts: Vec<String> = Vec::new();
        let result = embed_text(&texts, false);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // Test embedding identical texts (should produce identical vectors)
    #[test]
    fn test_embed_identical_text() {
        let texts = vec!["Same text".to_string(), "Same text".to_string()];
        let result = embed_text(&texts, false);
        match result {
            Ok(embeddings) => {
                // Should have at least 2 embeddings (one per input text)
                assert!(embeddings.len() >= 2);
                assert_eq!(embeddings[0].len(), 384);
                // Since we're chunking, we need to compare the first chunk of each text
                // Identical text should produce identical embeddings for corresponding chunks
                assert_eq!(embeddings[0], embeddings[1]);
            }
            Err(e) => {
                panic!("Embedding failed for identical text: {}", e);
            }
        }
    }

    // Test embedding different texts (should produce different vectors)
    #[test]
    fn test_embed_different_text() {
        let texts = vec!["Text one".to_string(), "Text two".to_string()];
        let result = embed_text(&texts, false);
        match result {
            Ok(embeddings) => {
                // Should have at least 2 embeddings (one per input text)
                assert!(embeddings.len() >= 2);
                assert_eq!(embeddings[0].len(), 384);
                // Different text should produce different embeddings
                assert_ne!(embeddings[0], embeddings[1]);
            }
            Err(e) => {
                panic!("Embedding failed for different text: {}", e);
            }
        }
    }
}
