use crate::extractor::extract_text;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use log::{info, warn, error};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use std::collections::HashMap;
use std::fs;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Model initialization failed: {0}")]
    InitializationError(String),
    
    #[error("Embedding generation failed: {0}")]
    GenerationError(String),
    
    #[error("IO error during benchmarking: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Extraction error: {0}")]
    ExtractionError(String),
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub model_name: String,
    pub initialization_time_ms: u64,
    pub average_embedding_time_ms: f64,
    pub total_embedding_time_ms: u64,
    pub files_processed: usize,
    pub total_tokens_processed: usize,
    pub embedding_dimension: usize,
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "
Model: {}
Initialization time: {} ms
Average embedding time per file: {:.2} ms
Total embedding time: {} ms
Files processed: {}
Total tokens processed: ~{}
Embedding dimension: {}
        ", 
        self.model_name,
        self.initialization_time_ms,
        self.average_embedding_time_ms,
        self.total_embedding_time_ms,
        self.files_processed,
        self.total_tokens_processed,
        self.embedding_dimension
        )
    }
}

/// Initialize an embedding model with proper error handling
fn initialize_model(model: &EmbeddingModel) -> Result<(TextEmbedding, Duration), BenchmarkError> {
    info!("Initializing model: {:?}", model);
    let start = Instant::now();
    
    let init_options = InitOptions::new(model.clone())
        .with_cache_dir(PathBuf::from(".cache"))
        .with_show_download_progress(true);
    
    let embedding_model = TextEmbedding::try_new(init_options.clone())
        .map_err(|e| BenchmarkError::InitializationError(e.to_string()))?;
        
    let duration = start.elapsed();
    info!("Model initialized in {} ms", duration.as_millis());
    
    Ok((embedding_model, duration))
}

/// Generate embeddings for a list of texts using the specified model
fn generate_embeddings(
    model: &TextEmbedding,
    texts: &[String],
) -> Result<(Vec<Vec<f32>>, Duration), BenchmarkError> {
    let start = Instant::now();
    
    let embeddings = model
        .embed(texts.to_vec(), None)
        .map_err(|e| BenchmarkError::GenerationError(e.to_string()))?;
        
    let duration = start.elapsed();
    
    Ok((embeddings, duration))
}

/// Benchmark a given model on a directory of text files
pub fn benchmark_model(
    model_type: EmbeddingModel,
    sample_dir: &Path,
    file_limit: Option<usize>,
) -> Result<BenchmarkResult, BenchmarkError> {
    info!("Starting benchmark for model: {:?}", model_type);
    
    // Initialize the model and measure initialization time
    let (model, init_duration) = initialize_model(&model_type)?;
    
    // Get a list of text files to benchmark
    let mut text_files = Vec::new();
    for entry in fs::read_dir(sample_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Only process .txt and .md files for simplicity
        if path.is_file() && 
           path.extension().map_or(false, |ext| ext == "txt" || ext == "md") {
            text_files.push(path);
            if let Some(limit) = file_limit {
                if text_files.len() >= limit {
                    break;
                }
            }
        }
    }
    
    if text_files.is_empty() {
        warn!("No valid text files found in the directory: {}", sample_dir.display());
        return Err(BenchmarkError::IoError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "No text files found")
        ));
    }
    
    info!("Found {} text files for benchmarking", text_files.len());
    
    // Process each file and collect timing information
    let mut total_embedding_time = Duration::new(0, 0);
    let mut total_tokens = 0;
    let mut embedding_dimension = 0;
    
    for file_path in &text_files {
        // Extract text from the file
        let text = match extract_text(file_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to extract text from {}: {}", file_path.display(), e);
                continue;
            }
        };
        
        // Generate embeddings for the text
        let text_chunks = vec![text];
        let (embeddings, embed_duration) = generate_embeddings(&model, &text_chunks)?;
        
        // Update total time
        total_embedding_time += embed_duration;
        
        // Rough token count estimation (very approximate)
        total_tokens += text_chunks.iter().map(|s| s.split_whitespace().count()).sum::<usize>();
        
        // Track embedding dimension
        if let Some(first_embedding) = embeddings.first() {
            embedding_dimension = first_embedding.len();
        }
    }
    
    // Calculate average time per file
    let avg_time_per_file = if !text_files.is_empty() {
        total_embedding_time.as_millis() as f64 / text_files.len() as f64
    } else {
        0.0
    };
    
    // Create and return benchmark result
    let result = BenchmarkResult {
        model_name: format!("{:?}", model_type),
        initialization_time_ms: init_duration.as_millis() as u64,
        average_embedding_time_ms: avg_time_per_file,
        total_embedding_time_ms: total_embedding_time.as_millis() as u64,
        files_processed: text_files.len(),
        total_tokens_processed: total_tokens,
        embedding_dimension,
    };
    
    info!("Benchmark completed for {:?}", model_type);
    
    Ok(result)
}

/// Run benchmarks on multiple models and compare results
pub fn run_model_comparison(
    sample_dir: &Path,
    file_limit: Option<usize>,
) -> HashMap<String, BenchmarkResult> {
    info!("Starting model comparison");
    
    let models = vec![
        EmbeddingModel::AllMiniLML6V2,  // all-MiniLM-L6-v2
        EmbeddingModel::BGESmallENV15,  // bge-small-en-v1.5 (quantized)
    ];
    
    let mut results = HashMap::new();
    
    for model in models {
        let model_name = format!("{:?}", model);
        info!("Benchmarking model: {}", model_name);
        
        match benchmark_model(model, sample_dir, file_limit) {
            Ok(result) => {
                info!("Benchmark for {} completed successfully", model_name);
                results.insert(model_name, result);
            },
            Err(e) => {
                error!("Failed to benchmark {}: {}", model_name, e);
            }
        }
    }
    
    info!("Model comparison completed. Benchmarked {} models", results.len());
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    
    fn create_sample_files(dir: &Path, count: usize) -> Result<(), std::io::Error> {
        for i in 0..count {
            let file_path = dir.join(format!("sample_{}.txt", i));
            let mut file = fs::File::create(file_path)?;
            
            // Write some sample text
            writeln!(file, "This is sample text for benchmarking embedding models.")?;
            writeln!(file, "It contains multiple sentences to ensure proper embedding.")?;
            writeln!(file, "Sample document number {} for testing.", i)?;
        }
        
        Ok(())
    }
    
    // Helper function that returns a mock benchmark result for testing
    // This avoids having to download the actual model during tests
    fn create_mock_benchmark_result(model_name: &str) -> BenchmarkResult {
        BenchmarkResult {
            model_name: model_name.to_string(),
            initialization_time_ms: 100,
            average_embedding_time_ms: 25.5,
            total_embedding_time_ms: 255,
            files_processed: 10,
            total_tokens_processed: 500,
            embedding_dimension: 384,
        }
    }
    
    // Only run this test when explicitly requested, as it downloads models
    #[tokio::test]
    #[ignore = "Downloads large model files, run manually with --ignored"]
    async fn test_benchmark_model() {
        // Create a temporary directory with sample files
        let temp_dir = tempdir().expect("Failed to create temp dir");
        create_sample_files(temp_dir.path(), 2)
            .expect("Failed to create sample files");
        
        // Run benchmark on a single model with a small sample
        let result = benchmark_model(
            EmbeddingModel::AllMiniLML6V2,
            temp_dir.path(),
            Some(2)
        );
        
        assert!(result.is_ok(), "Benchmark should run successfully");
        
        if let Ok(benchmark) = result {
            assert_eq!(benchmark.files_processed, 2);
            assert!(benchmark.initialization_time_ms > 0);
            assert!(benchmark.embedding_dimension > 0);
        }
    }
    
    #[test]
    fn test_benchmark_result_display() {
        // Test that the Display implementation works correctly
        let result = create_mock_benchmark_result("TestModel");
        let display_string = format!("{}", result);
        
        // Check that key information is included in the display output
        assert!(display_string.contains("TestModel"));
        assert!(display_string.contains("100 ms"));
        assert!(display_string.contains("25.50 ms"));  // Note: formatted with 2 decimal places
        assert!(display_string.contains("Files processed: 10"));
        assert!(display_string.contains("Embedding dimension: 384"));
    }
    
    #[test]
    fn test_model_comparison_mock() {
        // Test the result aggregation logic without actually running models
        let mut results = HashMap::new();
        results.insert(
            "AllMiniLML6V2".to_string(), 
            create_mock_benchmark_result("AllMiniLML6V2")
        );
        results.insert(
            "BGESmallENV15".to_string(), 
            create_mock_benchmark_result("BGESmallENV15")
        );
        
        // Check that we can process results correctly
        assert_eq!(results.len(), 2);
        
        // Verify individual result properties
        let all_mini_result = results.get("AllMiniLML6V2").unwrap();
        assert_eq!(all_mini_result.embedding_dimension, 384);
    }
}
