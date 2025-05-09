use std::path::Path;
use serde::{Deserialize, Serialize};
use log::info;
use crate::benchmark::{run_model_comparison, BenchmarkResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkRequest {
    /// Directory containing sample files to use for benchmarking
    pub sample_dir: String,
    /// Optional limit for number of files to process
    pub file_limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResponse {
    /// Map of model names to their benchmark results
    pub results: Vec<ModelBenchmarkResult>,
    /// Any messages or errors that occurred during benchmarking
    pub messages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelBenchmarkResult {
    /// Name of the embedding model
    pub model_name: String,
    /// Time taken to initialize the model (ms)
    pub initialization_time_ms: u64,
    /// Average time to embed a single file (ms)
    pub average_embedding_time_ms: f64,
    /// Total time spent generating embeddings (ms)
    pub total_embedding_time_ms: u64,
    /// Number of files processed during benchmarking
    pub files_processed: usize,
    /// Approximate number of tokens processed
    pub total_tokens_processed: usize,
    /// Dimension of the generated embedding vectors
    pub embedding_dimension: usize,
}

impl From<BenchmarkResult> for ModelBenchmarkResult {
    fn from(result: BenchmarkResult) -> Self {
        ModelBenchmarkResult {
            model_name: result.model_name,
            initialization_time_ms: result.initialization_time_ms,
            average_embedding_time_ms: result.average_embedding_time_ms,
            total_embedding_time_ms: result.total_embedding_time_ms,
            files_processed: result.files_processed,
            total_tokens_processed: result.total_tokens_processed,
            embedding_dimension: result.embedding_dimension,
        }
    }
}

/// Run benchmarks comparing different embedding models
#[tauri::command]
pub async fn run_benchmarks(request: BenchmarkRequest) -> Result<BenchmarkResponse, String> {
    info!("Starting benchmarks with sample dir: {}", request.sample_dir);
    
    let sample_dir = Path::new(&request.sample_dir);
    
    if !sample_dir.exists() || !sample_dir.is_dir() {
        return Err(format!("Sample directory does not exist or is not a directory: {}", request.sample_dir));
    }
    
    // Run the benchmarks
    let benchmark_results = run_model_comparison(sample_dir, request.file_limit);
    
    // Convert to response format
    let results: Vec<ModelBenchmarkResult> = benchmark_results
        .into_iter()
        .map(|(_, result)| result.into())
        .collect();
    
    // Create messages based on benchmark results
    let mut messages = Vec::new();
    
    if results.is_empty() {
        messages.push("No benchmark results were generated. Check logs for errors.".to_string());
    } else {
        messages.push(format!("Successfully benchmarked {} models.", results.len()));
        
        // Provide summary of which model performed better
        if results.len() >= 2 {
            // Find model with lowest average embedding time
            let fastest_model = results.iter()
                .min_by(|a, b| a.average_embedding_time_ms.partial_cmp(&b.average_embedding_time_ms).unwrap())
                .unwrap();
            
            messages.push(format!(
                "Fastest model for embeddings: {} (avg. {} ms per file)",
                fastest_model.model_name,
                fastest_model.average_embedding_time_ms
            ));
            
            // Compare embedding dimensions
            let dimensions: Vec<(String, usize)> = results.iter()
                .map(|r| (r.model_name.clone(), r.embedding_dimension))
                .collect();
                
            for (model, dim) in dimensions {
                messages.push(format!("Model {} produces {}-dimensional embeddings", model, dim));
            }
        }
    }
    
    Ok(BenchmarkResponse {
        results,
        messages,
    })
}
