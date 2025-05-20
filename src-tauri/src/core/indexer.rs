// src-tauri/src/core/indexer.rs

use std::path::Path;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use crate::db::{
    connect_db, 
    open_or_create_text_table, 
    open_or_create_image_table,
    open_or_create_amharic_text_table, // Added for Amharic
    upsert_document, 
    upsert_amharic_document, // Added for Amharic
    upsert_image
};
use crate::embedder::embed_text;
use crate::image_embedder::embed_image;
use crate::extractor::{
    extract_text, 
    calculate_hash, 
    process_image, 
    calculate_file_hash, 
    get_content_type, 
    ContentType,
    DetectedLanguage
};
use walkdir::WalkDir;
use std::time::Instant;
use std::sync::{RwLock, Arc};
use tokio::task;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use futures::future::join_all;

/// Directories to exclude from indexing
pub const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "Library",
    "System",
    ".git",
    ".cache",
    ".vscode",
    ".github",
    "TMWPix"
];

/// File patterns to exclude from indexing
pub const EXCLUDED_PATTERNS: &[&str] = &[
    ".app",
    ".bundle",
    ".framework",
    ".kext",
    ".plugin"
];

/// Indexing status information with separate counters for text and image files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexingStats {
    pub elapsed_seconds: u32,
    pub elapsed_milliseconds: u32,
    pub files_processed: u32,
    pub files_failed: u32,
    pub files_skipped: u32,
    pub db_inserts: u32,
    
    // Text-specific stats
    pub text_files_processed: u32,
    pub text_files_indexed: u32,
    pub text_files_failed: u32,
    
    // Image-specific stats
    pub image_files_processed: u32,
    pub image_files_indexed: u32,
    pub image_files_failed: u32,
    
    pub indexed_files: Vec<String>,
    pub failed_files: Vec<String>,
}

// Static variable to store the last indexing statistics
static LAST_INDEXING_STATS: Lazy<RwLock<Option<IndexingStats>>> = Lazy::new(|| RwLock::new(None));

/// Get the last indexing statistics
pub fn get_last_indexing_stats() -> Option<IndexingStats> {
    LAST_INDEXING_STATS.read().unwrap().clone()
}

/// Set the last indexing statistics
fn set_last_indexing_stats(stats: IndexingStats) {
    *LAST_INDEXING_STATS.write().unwrap() = Some(stats.clone());
}

/// Index the macOS Downloads folder at application startup
pub async fn index_downloads_folder() -> Result<IndexingStats, String> {
    let start_time = Instant::now();
    
    // Get the Downloads folder path for macOS
    let home_dir = dirs::home_dir().ok_or_else(|| {
        error!("Could not find home directory");
        "Failed to find home directory".to_string()
    })?;
    
    let downloads_dir = home_dir.join("Downloads");
    println!("Starting Downloads folder indexing: {}", downloads_dir.display());
    
    info!("Starting Downloads folder indexing: {}", downloads_dir.display());
    
    // Ensure the directory exists
    if !downloads_dir.exists() || !downloads_dir.is_dir() {
        error!("Downloads directory does not exist at {}", downloads_dir.display());
        return Err("Downloads directory not found".to_string());
    }
    
    info!("Excluding system folders and application bundles from indexing");
    
    // Initialize counters for statistics
    let mut files_processed = 0;
    let mut files_failed = 0;
    let mut files_skipped = 0;
    let mut db_inserts = 0;
    
    // Text-specific counters
    let mut text_files_processed = 0;
    let mut text_files_indexed = 0;
    let mut text_files_failed = 0;
    
    // Image-specific counters
    let mut image_files_processed = 0;
    let mut image_files_indexed = 0;
    let mut image_files_failed = 0;
    
    let mut indexed_files = Vec::new();
    let mut failed_files = Vec::new();
    
    // Open connection to database
    let conn = connect_db().await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        format!("Database connection error: {}", e)
    })?;
    
    // Access or create the tables
    let text_table = open_or_create_text_table(&conn).await.map_err(|e| {
        error!("Failed to open or create text table: {}", e);
        format!("Text table error: {}", e)
    })?;
    
    let image_table = open_or_create_image_table(&conn).await.map_err(|e| {
        error!("Failed to open or create image table: {}", e);
        format!("Image table error: {}", e)
    })?;

    let amharic_text_table = open_or_create_amharic_text_table(&conn).await.map_err(|e| {
        error!("Failed to open or create Amharic text table: {}", e);
        format!("Amharic text table error: {}", e)
    })?;
    
    // Walk through the directory and process files
    for entry in WalkDir::new(&downloads_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden files and directories
            if let Some(file_name) = e.file_name().to_str() {
                if file_name.starts_with(".") {
                    return false;
                }
            }
            
            // Skip directories in the excluded list
            if e.file_type().is_dir() {
                if let Some(dir_name) = e.file_name().to_str() {
                    if EXCLUDED_DIRS.iter().any(|excluded| dir_name.contains(excluded)) {
                        debug!("Skipping excluded directory: {}", e.path().display());
                        return false;
                    }
                }
            }
            
            // Skip macOS application bundles and system extensions
            if e.path().is_dir() {
                if let Some(path_str) = e.path().to_str() {
                    if EXCLUDED_PATTERNS.iter().any(|pattern| path_str.contains(pattern)) {
                        debug!("Skipping macOS bundle: {}", e.path().display());
                        return false;
                    }
                }
            }
            
            true
        }) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                // Skip directories
                if path.is_dir() {
                    continue;
                }
                
                files_processed += 1;
                
                // Determine content type and process accordingly
                let content_type = get_content_type(path);
                match content_type {
                    ContentType::Text => {
                        text_files_processed += 1;
                        println!("text files {}", text_files_processed);
                        println!("path {:?}", path);
                        
                        // Process text file
                        if let Err(e) = process_text_file(path, &text_table, &amharic_text_table).await {
                            error!("Error processing text file {}: {}", path.display(), e);
                            files_failed += 1;
                            text_files_failed += 1;
                            failed_files.push(path.to_string_lossy().to_string());
                        } else {
                            info!("Indexed text file: {}", path.display());
                            db_inserts += 1;
                            text_files_indexed += 1;
                            indexed_files.push(path.to_string_lossy().to_string());
                        }
                    },
                    ContentType::Image => {
                        image_files_processed += 1;
                        println!("Processing image file: {}", path.display());
                       
                        println!("count: {}", image_files_processed);
                        // Process image file
                        if let Err(e) = process_image_file(path, &image_table).await {
                            error!("Error processing image file {}: {}", path.display(), e);
                            files_failed += 1;
                            image_files_failed += 1;
                            failed_files.push(path.to_string_lossy().to_string());
                        } else {
                            info!("Indexed image file: {}", path.display());
                            db_inserts += 1;
                            image_files_indexed += 1;
                            indexed_files.push(path.to_string_lossy().to_string());
                        }
                    },
                    ContentType::Unsupported => {
                        debug!("Skipping unsupported file type: {}", path.display());
                        files_skipped += 1;
                    }
                }
            },
            Err(e) => {
                error!("Error walking directory: {}", e);
            }
        }
    }
    println!("Finished indexing");
    
    // Calculate statistics
    let elapsed = start_time.elapsed();
    let stats = IndexingStats {
        elapsed_seconds: elapsed.as_secs() as u32,
        elapsed_milliseconds: elapsed.subsec_millis(),
        files_processed,
        files_failed,
        files_skipped,
        db_inserts,
        text_files_processed,
        text_files_indexed,
        text_files_failed,
        image_files_processed,
        image_files_indexed,
        image_files_failed,
        indexed_files,
        failed_files,
    };
    
    info!(
        "Completed indexing in {}.{:03} seconds: {} files processed, {} failures, {} skipped, {} database inserts",
        stats.elapsed_seconds,
        stats.elapsed_milliseconds,
        stats.files_processed,
        stats.files_failed,
        stats.files_skipped,
        stats.db_inserts
    );
    
    info!(
        "Text files: {} processed, {} indexed, {} failed | Image files: {} processed, {} indexed, {} failed",
        stats.text_files_processed,
        stats.text_files_indexed,
        stats.text_files_failed,
        stats.image_files_processed,
        stats.image_files_indexed,
        stats.image_files_failed
    );
    
    // Save the stats for later retrieval
    set_last_indexing_stats(stats.clone());
    
    Ok(stats)
}

/// Process a text file for indexing - used by the single-threaded version
async fn process_text_file(file_path: &Path, text_table: &lancedb::Table, amharic_text_table: &lancedb::Table) -> Result<(), String> {
    // Extract text content from the file
    let extraction_result = extract_text(file_path).map_err(|e| format!("Failed to extract text: {}", e))?;
    
    // Calculate content hash
    let content_hash = calculate_hash(&extraction_result.text);
    
    // Get embeddings for the content
    let content_vec = vec![extraction_result.text.clone()];
    let embeddings = embed_text(&content_vec, &extraction_result.language, false).map_err(|e| {
        error!("Embedding error for {}: {}", file_path.display(), e);
        format!("Embedding generation failed: {}", e)
    })?;
    
    if embeddings.is_empty() {
        return Err(format!("No embeddings generated for {}", file_path.display()));
    }
    
    // Store in the database - now passing all embeddings
    let file_path_str = file_path.to_string_lossy().to_string();
    match extraction_result.language {
        DetectedLanguage::English | DetectedLanguage::Other => {
            upsert_document(text_table, &file_path_str, &content_hash, &embeddings).await.map_err(|e| {
                error!("Database error (English/Other) for {}: {}", file_path.display(), e);
                format!("Database upsert failed: {}", e)
            })?;
        }
        DetectedLanguage::Amharic => {
            upsert_amharic_document(amharic_text_table, &file_path_str, &content_hash, &embeddings).await.map_err(|e| {
                error!("Database error (Amharic) for {}: {}", file_path.display(), e);
                format!("Database upsert failed: {}", e)
            })?;
        }
    }
    
    Ok(())
}

/// Process an image file for indexing - used by the single-threaded version
async fn process_image_file(file_path: &Path, table: &lancedb::Table) -> Result<(), String> {
    // Process the image and get the path as a string
    let image_path = process_image(file_path).map_err(|e| {
        warn!("Image processing error for {}: {}", file_path.display(), e);
        format!("Image processing failed: {}", e)
    })?;
    
    // Calculate file hash for the image
    let file_hash = calculate_file_hash(file_path).map_err(|e| {
        error!("Hashing error for {}: {}", file_path.display(), e);
        format!("File hash calculation failed: {}", e)
    })?;
    
    // Generate embedding for the image
    let embedding = embed_image(&image_path).map_err(|e| {
        error!("Image embedding error for {}: {}", file_path.display(), e);
        format!("Image embedding generation failed: {}", e)
    })?;
    
    // Store in the database
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // For now, we don't have image dimensions or thumbnails
    // These could be added in a future enhancement
    let width: Option<i32> = None;
    let height: Option<i32> = None;
    let thumbnail_path: Option<&str> = None;
    
    upsert_image(
        table, 
        &file_path_str, 
        &file_hash, 
        &embedding, 
        width, 
        height, 
        thumbnail_path
    ).await.map_err(|e| {
        error!("Database error for {}: {}", file_path.display(), e);
        format!("Database upsert failed: {}", e)
    })?;
    
    Ok(())
}

/// Handle text file indexing with a batch of files in a separate thread
async fn handle_specific_language_text_indexing(
    text_data_batch: Vec<(String, String, Vec<Vec<f32>>)>, // path_str, content_hash, embeddings
    table: Arc<lancedb::Table>,
    language_name_for_log: &str // e.g., "English/Other" or "Amharic"
) -> HashMap<String, Result<(), String>> {
    let mut results = HashMap::new();

    // The input `text_data_batch` is Vec<(String, String, Vec<Vec<f32>>)>
    // representing (path_str, content_hash, embeddings)

    // Process files in batches (e.g., 10 at a time) to manage concurrency for DB operations
    // Each item in text_data_batch is already processed for extraction and embedding.
    for batch_chunk in text_data_batch.chunks(10) {
        let mut mut_futures = Vec::new(); // Renamed from futures to avoid conflict if std::future::futures is in scope
        for (file_path_str, content_hash, embeddings) in batch_chunk {
            // Clone Arcs and owned Strings for the async move block
            let table_clone = Arc::clone(&table);
            let path_str_clone = file_path_str.clone();
            let hash_clone = content_hash.clone();
            let embeddings_clone = embeddings.clone(); // Vec<Vec<f32>> can be cloned
            let lang_log_clone = language_name_for_log.to_string(); // Clone for async move

            mut_futures.push(async move {
                let upsert_result = upsert_document(
                    &table_clone,
                    &path_str_clone,
                    &hash_clone,
                    &embeddings_clone,
                )
                .await
                .map_err(|e| {
                    error!(
                        "Database error for {} file {}: {}",
                        lang_log_clone, path_str_clone, e
                    );
                    format!(
                        "Database upsert failed for {} file {}: {}",
                        lang_log_clone, path_str_clone, e
                    )
                });
                (path_str_clone, upsert_result) // Return path and result for HashMap
            });
        }

        let chunk_results = join_all(mut_futures).await;
        for (path_str, result) in chunk_results {
            results.insert(path_str, result);
        }
    }
    results
}

/// Handle image file indexing with a batch of files in a separate thread
async fn handle_image_indexing(
    _image_files: Vec<String>,
    _table: Arc<lancedb::Table>
) -> HashMap<String, Result<(), String>> {
    let results = HashMap::new();
    
    // Process files in batches to avoid overwhelming the system
    // Commented out code...
    
    results
}

async fn create_empty_string_result_hashmap_async() -> HashMap<String, Result<(), String>> {
    HashMap::new()
}

/// Index a specific folder with parallel processing for text and image files
pub async fn index_folder(folder_path: &str) -> Result<IndexingStats, String> {
    let start_time = Instant::now();
    
    // Ensure the directory exists
    let path = Path::new(folder_path);
    if !path.exists() || !path.is_dir() {
        error!("Directory does not exist at {}", folder_path);
        return Err(format!("Directory not found: {}", folder_path));
    }
    
    info!("Starting folder indexing with parallel processing: {}", folder_path);
    info!("Excluding system folders and application bundles from indexing");
    
    // Initialize file lists for parallel processing
    let mut english_text_data_to_process: Vec<(String, String, Vec<Vec<f32>>)> = Vec::new(); // Path, Hash, Embeddings
    let mut amharic_text_data_to_process: Vec<(String, String, Vec<Vec<f32>>)> = Vec::new(); // Path, Hash, Embeddings
    let mut image_files: Vec<String> = Vec::new(); // Paths for images
    let mut files_skipped = 0;
    let mut files_failed_preprocessing = 0; // Added for errors during initial scan/extraction/embedding
    
    // Open connection to database
    let conn = connect_db().await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        format!("Database connection error: {}", e)
    })?;
    
    // Access or create the tables
    let text_table = open_or_create_text_table(&conn).await.map_err(|e| {
        error!("Failed to open or create text table: {}", e);
        format!("Text table error: {}", e)
    })?;
    
    let image_table = open_or_create_image_table(&conn).await.map_err(|e| {
        error!("Failed to open or create image table: {}", e);
        format!("Image table error: {}", e)
    })?;

    let amharic_text_table = open_or_create_amharic_text_table(&conn).await.map_err(|e| {
        error!("Failed to open or create Amharic text table: {}", e);
        format!("Amharic text table error: {}", e)
    })?;
    
    // Wrap tables in Arc to make them thread-safe
    let text_table_arc = Arc::new(text_table);
    let image_table_arc = Arc::new(image_table);
    let amharic_text_table_arc = Arc::new(amharic_text_table); // Added
    
    // First pass: collect files by type
    info!("Scanning directory and categorizing files...");
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden files and directories
            if let Some(file_name) = e.file_name().to_str() {
                if file_name.starts_with(".") {
                    return false;
                }
            }
            
            // Skip directories in the excluded list
            if e.file_type().is_dir() {
                if let Some(dir_name) = e.file_name().to_str() {
                    if EXCLUDED_DIRS.iter().any(|excluded| dir_name.contains(excluded)) {
                        debug!("Skipping excluded directory: {}", e.path().display());
                        return false;
                    }
                }
            }
            
            // Skip macOS application bundles and system extensions
            if e.path().is_dir() {
                if let Some(path_str) = e.path().to_str() {
                    if EXCLUDED_PATTERNS.iter().any(|pattern| path_str.contains(pattern)) {
                        debug!("Skipping macOS bundle: {}", e.path().display());
                        return false;
                    }
                }
            }
            
            true
        }) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                // Skip directories
                if path.is_dir() {
                    continue;
                }
                
                // Determine content type and add to appropriate list
                let content_type = get_content_type(path);
                match content_type {
                    ContentType::Text => {
                        let file_path_display = path.display().to_string(); // For logging
                        match extract_text(path) {
                            Ok(extraction_result) => {
                                let content_hash = calculate_hash(&extraction_result.text);
                                // embed_text expects Vec<String>, even if it's just one document
                                let content_for_embedding = vec![extraction_result.text.clone()]; 
                                match embed_text(&content_for_embedding, &extraction_result.language, false) {
                                    Ok(embeddings) => {
                                        // embed_text returns Vec<Vec<f32>>, one inner Vec per input string
                                        if embeddings.is_empty() || embeddings[0].is_empty() {
                                            error!("No embeddings generated for text file: {}", file_path_display);
                                            files_failed_preprocessing += 1;
                                        } else {
                                            // We passed one string, so we expect one Vec<f32> in the outer Vec.
                                            // The db upsert functions expect &[Vec<f32>], which is effectively Vec<Vec<f32>> for multiple chunks of ONE document.
                                            // Here, embeddings IS Vec<Vec<f32>> where the outer Vec corresponds to input strings (1 here) 
                                            // and inner Vec<f32> is the embedding for that string. 
                                            // If chunking were implemented in embed_text, 'embeddings' would be Vec<Vec<f32>> where each inner Vec is an embedding for a chunk.
                                            // For now, assume embed_text returns one embedding for the whole text if not chunked internally.
                                            // The db functions (upsert_document, upsert_amharic_document) take &[Vec<f32>] where each Vec<f32> is an embedding for a chunk.
                                            // So, 'embeddings' from embed_text (which is Vec<Vec<f32>>) fits this directly.
                                            let data_tuple = (path.to_string_lossy().to_string(), content_hash, embeddings);
                                            match extraction_result.language {
                                                DetectedLanguage::English | DetectedLanguage::Other => {
                                                    english_text_data_to_process.push(data_tuple);
                                                }
                                                DetectedLanguage::Amharic => {
                                                    amharic_text_data_to_process.push(data_tuple);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to embed text for {}: {}", file_path_display, e);
                                        files_failed_preprocessing += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to extract text from {}: {}", file_path_display, e);
                                files_failed_preprocessing += 1;
                            }
                        }
                    },
                    ContentType::Image => {
                        image_files.push(path.to_string_lossy().to_string());
                    },
                    ContentType::Unsupported => {
                        debug!("Skipping unsupported file type: {}", path.display());
                        files_skipped += 1;
                    }
                }
            },
            Err(e) => {
                error!("Error walking directory: {}", e);
            }
        }
    }
    
    // Log collection summary
    info!("Found {} English/Other text items, {} Amharic text items, and {} image files to process. {} files failed pre-processing.", 
          english_text_data_to_process.len(), amharic_text_data_to_process.len(), image_files.len(), files_failed_preprocessing);
    
    // Second pass: process files in parallel using separate threads
    info!("Starting parallel processing of files...");

    // Define futures for each type of processing.
    let english_text_task_handle;
    if !english_text_data_to_process.is_empty() {
        let table_for_task = Arc::clone(&text_table_arc);
        let data_for_task = english_text_data_to_process.clone(); // Clone data for the task
        english_text_task_handle = task::spawn(async move {
            handle_specific_language_text_indexing(data_for_task, table_for_task, "English/Other").await
        });
    } else {
        english_text_task_handle = task::spawn(async move { HashMap::new() }); // Dummy task
    }

    let amharic_text_task_handle;
    if !amharic_text_data_to_process.is_empty() {
        let table_for_task = Arc::clone(&amharic_text_table_arc);
        let data_for_task = amharic_text_data_to_process.clone(); // Clone data for the task
        amharic_text_task_handle = task::spawn(async move {
            handle_specific_language_text_indexing(data_for_task, table_for_task, "Amharic").await
        });
    } else {
        amharic_text_task_handle = task::spawn(async move { HashMap::new() }); // Dummy task
    }

    let image_task_handle;
    if !image_files.is_empty() {
        let image_table_for_task = Arc::clone(&image_table_arc);
        let image_files_for_task = image_files.clone(); // Clone data for the task
        image_task_handle = task::spawn(handle_image_indexing(image_files_for_task, image_table_for_task));
    } else {
        image_task_handle = task::spawn(create_empty_string_result_hashmap_async()); // Dummy task using async helper
    }

    // Wait for all tasks to complete
    let (
        english_text_join_result,
        amharic_text_join_result,
        image_join_result
    ) = tokio::join!(
        english_text_task_handle,
        amharic_text_task_handle,
        image_task_handle
    );

    // Aggregate results
    let mut stats = IndexingStats::default();
    stats.files_skipped = files_skipped; // From the first pass (file categorization)
    // Add failures from the pre-processing (extraction/embedding) stage to text_files_failed
    stats.text_files_failed += files_failed_preprocessing; 

    // Process English text results
    match english_text_join_result {
        Ok(map) => {
            for (_path, res) in map {
                if res.is_ok() {
                    stats.text_files_processed += 1;
                } else {
                    stats.text_files_failed += 1;
                }
            }
        }
        Err(e) => {
            error!("English text processing task failed to join: {}", e);
            // If the task itself panicked or was cancelled, count all its intended files as failed.
            stats.text_files_failed += english_text_data_to_process.len() as u32;
        }
    }

    // Process Amharic text results
    match amharic_text_join_result {
        Ok(map) => {
            for (_path, res) in map {
                if res.is_ok() {
                    stats.text_files_processed += 1; // Aggregating all text together for now
                } else {
                    stats.text_files_failed += 1;    // Aggregating all text together for now
                }
            }
        }
        Err(e) => {
            error!("Amharic text processing task failed to join: {}", e);
            stats.text_files_failed += amharic_text_data_to_process.len() as u32;
        }
    }

    // Process Image results
    match image_join_result {
        Ok(map) => {
            for (_path, res) in map {
                if res.is_ok() {
                    stats.image_files_processed += 1;
                } else {
                    stats.image_files_failed += 1;
                }
            }
        }
        Err(e) => {
            error!("Image processing task failed to join: {}", e);
            stats.image_files_failed += image_files.len() as u32;
        }
    }

    let elapsed_time = start_time.elapsed();
    let final_stats = IndexingStats {
        elapsed_seconds: elapsed_time.as_secs() as u32,
        elapsed_milliseconds: elapsed_time.subsec_millis(),
        files_processed: stats.text_files_processed + stats.image_files_processed, // Total processed
        files_failed: stats.text_files_failed + stats.image_files_failed, // Total failed
        files_skipped: stats.files_skipped,
        db_inserts: stats.text_files_processed + stats.image_files_processed, // Sum of successfully processed text and image files
        
        text_files_processed: stats.text_files_processed,
        text_files_indexed: stats.text_files_processed, // Assume processed means indexed for now
        text_files_failed: stats.text_files_failed,
        
        image_files_processed: stats.image_files_processed,
        image_files_indexed: stats.image_files_processed, // Assume processed means indexed for now
        image_files_failed: stats.image_files_failed,
        
        indexed_files: Vec::new(), // Not populated in current parallel logic
        failed_files: Vec::new(),  // Not populated in current parallel logic
    };

    set_last_indexing_stats(final_stats.clone());

    info!(
        "Indexing complete for '{}' in {}.{:03}s: {} files processed ({} text, {} images), {} DB inserts, {} skipped, {} total failed ({} text, {} images)",
        folder_path,
        final_stats.elapsed_seconds,
        final_stats.elapsed_milliseconds,
        final_stats.files_processed,
        final_stats.text_files_processed,
        final_stats.image_files_processed,
        final_stats.db_inserts,
        final_stats.files_skipped,
        final_stats.files_failed,
        final_stats.text_files_failed,
        final_stats.image_files_failed
    );

    Ok(final_stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::extractor::{get_content_type, ContentType}; // Added import
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_is_supported_file() {
        let dir = tempdir().unwrap();
        
        // Supported text files
        let pdf_path = dir.path().join("document.pdf");
        let txt_path = dir.path().join("test.txt");
        let md_path = dir.path().join("readme.md");
        
        // Supported image files
        let jpg_path = dir.path().join("image.jpg");
        let png_path = dir.path().join("icon.png");
        
        // Unsupported files
        let no_ext_path = dir.path().join("no_extension");
        let hidden_path = dir.path().join(".hidden.pdf");
        let zip_path = dir.path().join("archive.zip");
        
        assert_eq!(get_content_type(&pdf_path), ContentType::Text, "PDF should be Text"); // Adjusted assertion
        assert_eq!(get_content_type(&txt_path), ContentType::Text, "TXT should be Text");
        assert_eq!(get_content_type(&md_path), ContentType::Text, "MD should be Text");
        assert_eq!(get_content_type(&jpg_path), ContentType::Image, "JPG should be Image");
        assert_eq!(get_content_type(&png_path), ContentType::Image, "PNG should be Image");
        
        assert_eq!(get_content_type(&no_ext_path), ContentType::Unsupported, "No extension should be Unsupported");
        assert_eq!(get_content_type(&hidden_path), ContentType::Text, "'.hidden.pdf' should be Text as get_content_type checks the '.pdf' extension");
        assert_eq!(get_content_type(&zip_path), ContentType::Unsupported, "ZIP should be Unsupported");
    }
    
    #[test]
    fn test_content_type_detection() {
        // Text files
        assert_eq!(get_content_type(Path::new("document.pdf")), ContentType::Text);
        assert_eq!(get_content_type(Path::new("notes.txt")), ContentType::Text);
        assert_eq!(get_content_type(Path::new("readme.md")), ContentType::Text);
        
        // Image files
        assert_eq!(get_content_type(Path::new("photo.jpg")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("icon.png")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("animation.gif")), ContentType::Image);
        
        // Unsupported files
        assert_eq!(get_content_type(Path::new("archive.zip")), ContentType::Unsupported);
        assert_eq!(get_content_type(Path::new("unknown")), ContentType::Unsupported);
    }
    
    #[test]
    fn test_create_mock_text_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "This is test content").unwrap();
        
        assert!(file_path.exists());
        assert_eq!(get_content_type(&file_path), ContentType::Text);
    }
    
    #[test]
    fn test_create_mock_image_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.jpg");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "This is mock image data").unwrap();
        
        assert!(file_path.exists());
        assert_eq!(get_content_type(&file_path), ContentType::Image);
    }
}
