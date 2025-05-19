use fastembed::{ImageEmbedding, ImageInitOptions, ImageEmbeddingModel, Embedding};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use thiserror::Error;
use log::{error, info, debug};
use std::path::Path;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Constants for the image embedding model
const MODEL_NAME: ImageEmbeddingModel = ImageEmbeddingModel::NomicEmbedVisionV15;
const CACHE_DIR_NAME: &str = ".cache"; // Same cache directory as text model

// Dimension of the Nomic Embed Vision v1.5 embeddings
const NOMIC_EMBED_VISION_V15_DIM: usize = 768; // NomicEmbedVisionV15 has 768 dimensions

// Define potential errors during image embedding
#[derive(Error, Debug)]
pub enum ImageEmbeddingError {
    #[error("Model initialization failed: {0}")]
    InitializationError(String),
    #[error("Embedding generation failed: {0}")]
    GenerationError(String),
    #[error("Model loading error: {0}")]
    ModelLoadError(#[from] fastembed::Error),
    #[error("Image processing error: {0}")]
    ImageProcessingError(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Text embedding error: {0}")]
    TextEmbeddingError(String),
}

// Use Lazy to initialize the model once with Mutex for thread safety
static IMAGE_MODEL: Lazy<Mutex<Result<ImageEmbedding, ImageEmbeddingError>>> = Lazy::new(|| {
    info!("Initializing image embedding model (Lazy)...");
    
    // Use the builder pattern for ImageInitOptions
    let init_options = ImageInitOptions::new(MODEL_NAME)
        .with_cache_dir(std::path::PathBuf::from(CACHE_DIR_NAME))
        .with_show_download_progress(true);

    let model_result = ImageEmbedding::try_new(init_options).map_err(|e| {
        let err_msg = format!("Failed to initialize image embedding model: {}", e);
        error!("{}", err_msg);
        ImageEmbeddingError::ModelLoadError(e)
    });
    
    Mutex::new(model_result)
});

// Use lazy to initialize a special text embedding model for image searches
static TEXT_FOR_IMAGE_MODEL: Lazy<Mutex<Result<TextEmbedding, ImageEmbeddingError>>> = Lazy::new(|| {
    info!("Initializing text embedding model for image search (Lazy)...");
    let init_options = InitOptions::new(EmbeddingModel::NomicEmbedTextV15)
        .with_cache_dir(std::path::PathBuf::from(CACHE_DIR_NAME))
        .with_show_download_progress(true);

    let model_result = TextEmbedding::try_new(init_options).map_err(|e| {
        let err_msg = format!("Failed to initialize text model for image search: {}", e);
        error!("{}", err_msg);
        ImageEmbeddingError::ModelLoadError(e)
    });
    
    Mutex::new(model_result)
});

/// Generates embeddings for the given image files.
/// 
/// # Arguments
/// * `image_paths` - A slice of paths to image files
/// 
/// # Returns
/// * `Result<Vec<Embedding>, ImageEmbeddingError>` - A vector of embedding vectors or an error

pub fn embed_images(image_paths: &[&str]) -> Result<Vec<Embedding>, ImageEmbeddingError> {
    if image_paths.is_empty() {
        return Ok(Vec::new()); // Return empty vec if no paths
    }
    
    // Verify that all files exist
    for path in image_paths {
        if !Path::new(path).exists() {
            return Err(ImageEmbeddingError::FileNotFound(path.to_string()));
        }
    }

    debug!("Embedding {} images", image_paths.len());
    
    // Access the lazily initialized model and generate embeddings
    let model_guard = IMAGE_MODEL.lock().map_err(|e| {
        let err_msg = format!("Failed to acquire lock on image model: {}", e);
        error!("{}", err_msg);
        ImageEmbeddingError::InitializationError(err_msg)
    })?;
    
    match &*model_guard {
        Ok(model) => {
            // Generate embeddings for all images
            match model.embed(image_paths.to_vec(), None) {
                Ok(embeddings) => {
                    debug!("Successfully generated {} image embeddings", embeddings.len());
                    Ok(embeddings)
                }
                Err(e) => {
                    let err_msg = format!("Image embedding generation failed: {}", e);
                    error!("{}", err_msg);
                    Err(ImageEmbeddingError::GenerationError(err_msg))
                }
            }
        }
        Err(init_error) => {
            error!("Image embedding model initialization failed previously: {}", init_error);
            Err(ImageEmbeddingError::InitializationError(format!("{}", init_error)))
        }
    }
}

/// Embed a single image file and return its embedding
pub fn embed_image(image_path: &str) -> Result<Embedding, ImageEmbeddingError> {
    if !Path::new(image_path).exists() {
        return Err(ImageEmbeddingError::FileNotFound(image_path.to_string()));
    }
    
    // Call embed_images with a single path
    match embed_images(&[image_path]) {
        Ok(embeddings) => {
            if embeddings.is_empty() {
                return Err(ImageEmbeddingError::GenerationError(
                    "Empty embedding result".to_string()
                ));
            }
            Ok(embeddings[0].clone())
        },
        Err(e) => Err(e)
    }
}

pub fn embed_text_for_image_search(query_text: &str) -> Result<Embedding, ImageEmbeddingError> {
    debug!("Generating image-compatible text embedding for query: {}", query_text);
    let model_guard = TEXT_FOR_IMAGE_MODEL.lock().map_err(|e| {
        let err_msg = format!("Failed to acquire lock on text model for image search: {}", e);
        error!("{}", err_msg);
        ImageEmbeddingError::InitializationError(err_msg)
    })?;
    let model = model_guard.as_ref().map_err(|e| {
        let err_msg = format!("Text model for image search not initialized: {}", e);
        error!("{}", err_msg);
        ImageEmbeddingError::InitializationError(err_msg)
    })?;

    
    // Generate the embedding using the text model
    // The API requires a Vec<String> and an optional batch size
    match model.embed(vec![query_text.to_string()], None) {
        Ok(embeddings) => {
            if embeddings.is_empty() {
                return Err(ImageEmbeddingError::TextEmbeddingError(
                    "Empty text embedding result for image search".to_string()
                ));
            }
            Ok(embeddings[0].clone())
        },
        Err(e) => {
            let err_msg = format!("Failed to generate text embedding for image search: {}", e);
            error!("{}", err_msg);
            Err(ImageEmbeddingError::TextEmbeddingError(err_msg))
        }
    }
}

/// Generates mock embeddings for testing purposes.
#[cfg(test)]
pub fn embed_images_test(image_paths: &[&str]) -> Result<Vec<Embedding>, ImageEmbeddingError> {
    if image_paths.is_empty() {
        return Ok(Vec::new());
    }
    
    // Verify that all files exist (same as in the non-test version)
    for path in image_paths {
        if !Path::new(path).exists() {
            return Err(ImageEmbeddingError::FileNotFound(path.to_string()));
        }
    }
    
    info!("Generating MOCK image embeddings for {} images...", image_paths.len());
    const MOCK_DIMENSION: usize = 768; // NomicEmbedVisionV15 dimension

    // Create different mock embeddings for different images
    let embeddings = image_paths.iter().map(|path| {
        // Create a unique embedding for each different image
        // by using the path to seed values
        let mut vec = vec![0.1f32; MOCK_DIMENSION];
        
        // Use characters from the path to differentiate embeddings
        for (j, c) in path.chars().enumerate() {
            if j < MOCK_DIMENSION {
                // Use character code to create different values
                vec[j] = (c as u32 % 100) as f32 / 100.0;
            } else {
                break;
            }
        }
        
        // This ensures that identical paths produce identical embeddings
        // and different paths produce different embeddings
        vec
    }).collect();

    Ok(embeddings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to create a mock image file (just a text file for testing)
    fn create_mock_image(dir: &tempfile::TempDir, name: &str) -> String {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        // Write some dummy content (not actual image data, just for testing)
        write!(file, "MOCK IMAGE DATA").unwrap();
        file_path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_embed_images_empty_list() {
        let image_paths: Vec<&str> = Vec::new();
        let result = embed_images(&image_paths);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_embed_image_success() {
        let dir = tempdir().unwrap();
        let image_path = create_mock_image(&dir, "test.jpg");
        
        let result = embed_image(&image_path);
        assert!(result.is_ok());
        
        // CLIP-ViT-B-32 produces 512-dimensional embeddings
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 512);
    }

    #[test]
    fn test_embed_multiple_images() {
        let dir = tempdir().unwrap();
        let image_path1 = create_mock_image(&dir, "test1.jpg");
        let image_path2 = create_mock_image(&dir, "test2.jpg");
        
        let image_paths = vec![&image_path1[..], &image_path2[..]];
        let result = embed_images(&image_paths);
        
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 512);
        assert_eq!(embeddings[1].len(), 512);
        
        // Different images should produce different embeddings
        assert_ne!(embeddings[0], embeddings[1]);
    }

    #[test]
    fn test_embed_nonexistent_image() {
        let result = embed_image("nonexistent_image.jpg");
        assert!(result.is_err());
        match result {
            Err(ImageEmbeddingError::FileNotFound(_)) => (), // Expected error
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_embed_identical_images() {
        let dir = tempdir().unwrap();
        let image_path = create_mock_image(&dir, "test.jpg");
        
        let image_paths = vec![&image_path[..], &image_path[..]];
        let result = embed_images(&image_paths);
        
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        
        // Identical images should produce identical embeddings
        assert_eq!(embeddings[0], embeddings[1]);
    }
} 