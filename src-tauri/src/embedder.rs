// src-tauri/src/embedder.rs

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use thiserror::Error;
use log::{error, info, debug};
use std::path::PathBuf;
use once_cell::sync::Lazy;
use crate::chunker::{chunk_text, ChunkerError};

const DEFAULT_MODEL_NAME: EmbeddingModel = EmbeddingModel::BGESmallENV15;
const AMHARIC_MODEL_NAME: EmbeddingModel = EmbeddingModel::MultilingualE5Small;
const CACHE_DIR_NAME: &str = ".cache";

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

static DEFAULT_MODEL: Lazy<Result<TextEmbedding, EmbeddingError>> = Lazy::new(|| {
    info!("Initializing default embedding model (Lazy)...");
    let init_options = InitOptions::new(DEFAULT_MODEL_NAME)
        .with_cache_dir(PathBuf::from(CACHE_DIR_NAME))
        .with_show_download_progress(true);
    TextEmbedding::try_new(init_options).map_err(|e| {
        EmbeddingError::ModelLoadError(format!("Failed to initialize default embedding model: {}", e))
    })
});

static AMHARIC_MODEL: Lazy<Result<TextEmbedding, EmbeddingError>> = Lazy::new(|| {
    info!("Initializing Amharic embedding model (MultilingualE5Small) (Lazy)...");
    let init_options = InitOptions::new(AMHARIC_MODEL_NAME)
        .with_cache_dir(PathBuf::from(CACHE_DIR_NAME))
        .with_show_download_progress(true);
    TextEmbedding::try_new(init_options).map_err(|e| {
        EmbeddingError::ModelLoadError(format!("Failed to initialize Amharic embedding model: {}", e))
    })
});

fn embed_with_model(
    model_instance: &Result<TextEmbedding, EmbeddingError>,
    content: &[String],
    query: bool,
    lang_prefix: Option<&str> // e.g., "query" or "passage" for E5
) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    let processed_content: Vec<String> = content.iter().map(|s| {
        if let Some(prefix) = lang_prefix {
            format!("{}: {}", prefix, s)
        } else {
            s.to_string()
        }
    }).collect();

    if processed_content.is_empty() { return Ok(Vec::new()); }

    let mut final_chunks_to_embed: Vec<String> = Vec::new();
    if !query { // Only chunk passages
        if processed_content.is_empty() { 
             return Ok(Vec::new());
        }
        for text_content in processed_content.iter() {
            if text_content.trim().is_empty() {
                continue; 
            }
            let chunks = chunk_text(text_content)?;
            final_chunks_to_embed.extend(chunks);
        }
        if final_chunks_to_embed.is_empty() { return Ok(Vec::new()); }
    } else {
        final_chunks_to_embed = processed_content;
        final_chunks_to_embed.retain(|s| !s.trim().is_empty());
        if final_chunks_to_embed.is_empty() {
            return Ok(Vec::new());
        }
    }
    
    debug!("Embedding {} final chunks.", final_chunks_to_embed.len());

    match model_instance {
        Ok(model) => model.embed(final_chunks_to_embed, None).map_err(|e| {
            error!("Embedding generation failed: {}", e);
            EmbeddingError::GenerationError(format!("Embedding generation failed: {}", e))
        }),
        Err(init_error) => {
            error!("Model not initialized, cannot embed: {}", init_error);
            Err(EmbeddingError::InitializationError(format!("Model not initialized: {}", init_error)))
        }
    }
}

pub fn embed_text(content: &[String], query: bool) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    let prefix = if query { Some("query") } else { None }; 
    embed_with_model(&DEFAULT_MODEL, content, query, prefix)
}

pub fn embed_amharic_text(content: &[String], query: bool) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    let prefix = if query { "query" } else { "passage" };
    embed_with_model(&AMHARIC_MODEL, content, query, Some(prefix))
}


#[cfg(test)]
fn embed_text_test(content: &[String], _query: bool) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    use crate::db::TEXT_EMBEDDING_DIM;
    if content.is_empty() { return Ok(Vec::new()); }
    Ok(content.iter().map(|_| vec![0.1f32; TEXT_EMBEDDING_DIM as usize]).collect())
}

#[cfg(test)]
fn embed_amharic_text_test(content: &[String], _query: bool) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    use crate::db::AMHARIC_EMBEDDING_DIM;
    if content.is_empty() { return Ok(Vec::new()); }
    Ok(content.iter().map(|_| vec![0.2f32; AMHARIC_EMBEDDING_DIM as usize]).collect())
}

pub fn get_chunk_count(text: &str) -> Result<usize, EmbeddingError> {
    if text.trim().is_empty() {
        return Ok(0); 
    }
    Ok(chunk_text(text)?.len())
}


#[cfg(test)]
mod tests {
    use super::*; // Imports the test-specific functions too
    use crate::db::{TEXT_EMBEDDING_DIM, AMHARIC_EMBEDDING_DIM};

    #[test]
    fn mock_embed_text_test_basic() {
        let texts = vec!["Hello world".to_string()];
        let result = embed_text_test(&texts, false); 
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), TEXT_EMBEDDING_DIM as usize);
        assert_eq!(embeddings[0][0], 0.1f32); 
    }

    #[test]
    fn mock_embed_amharic_text_test_basic() {
        let texts = vec!["ሰላም አለም".to_string()];
        let result = embed_amharic_text_test(&texts, false); 
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), AMHARIC_EMBEDDING_DIM as usize);
        assert_eq!(embeddings[0][0], 0.2f32); 
    }

    #[test]
    fn mock_embed_text_empty() {
        let texts: Vec<String> = Vec::new();
        let result = embed_text_test(&texts, false);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
    
    #[test]
    fn mock_embed_amharic_text_empty() {
        let texts: Vec<String> = Vec::new();
        let result = embed_amharic_text_test(&texts, false);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
