// src-tauri/src/embedding.rs
use fastembed::{Embedding, EmbeddingModel, InitOptions, TextEmbedding};
use thiserror::Error; // Using thiserror for cleaner error handling

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to initialize embedding model: {0}")]
    InitializationError(#[from] fastembed::Error), // Convert fastembed::Error
    #[error("Embedding process failed: {0}")]
    EmbeddingFailed(fastembed::Error), // Keep specific embedding errors separate if needed
    #[error("No embeddings were generated for the provided input")]
    NoEmbeddingsGenerated,
}

/// Initializes the TextEmbedding model.
/// Uses quantized AllMiniLML6V2 by default if the 'quantized' feature is enabled.
pub fn initialize_model() -> Result<TextEmbedding, EmbeddingError> {
    // Use the builder pattern provided by fastembed v4
    let options = InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_show_download_progress(true); // Use builder method

    let model = TextEmbedding::try_new(options)?;
    Ok(model)
}

/// Embeds a single piece of text using the provided model.
pub fn embed_text(
    model: &TextEmbedding,
    text: &str,
) -> Result<Embedding, EmbeddingError> {
    // Pass a Vec<&str> even for a single item
    let documents = vec![text];

    // Embed the document
    // Using None for batch_size lets fastembed decide
    let embeddings = model
        .embed(documents, None)
        .map_err(EmbeddingError::EmbeddingFailed)?;

    // Since we passed one document, we expect one embedding vector
    embeddings
        .into_iter()
        .next()
        .ok_or(EmbeddingError::NoEmbeddingsGenerated)
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test might download the model on first run, so it might take longer.
    #[test]
    fn test_embedding_initialization_and_use() {
        // 1. Initialize model
        let model_result = initialize_model();
        assert!(model_result.is_ok(), "Model initialization failed: {:?}", model_result.err());
        let model = model_result.unwrap();

        // 2. Embed sample text
        let sample_text = "This is a test sentence for semantic embedding.";
        let embedding_result = embed_text(&model, sample_text);
        assert!(embedding_result.is_ok(), "Embedding failed: {:?}", embedding_result.err());
        let embedding = embedding_result.unwrap();

        // 3. Verify embedding dimensions (all-MiniLM-L6-v2 has 384 dimensions)
        let expected_dimensions = 384;
        assert_eq!(
            embedding.len(),
            expected_dimensions,
            "Embedding dimensions mismatch. Expected {}, got {}",
            expected_dimensions,
            embedding.len()
        );

        println!(
            "Successfully generated embedding with {} dimensions.",
            embedding.len()
        );
    }
}
