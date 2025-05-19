// src-tauri/src/core/indexer.rs

use std::path::Path;
use log::{info, warn, error, debug};
use crate::db::{
    connect_db, 
    open_or_create_text_table, 
    open_or_create_image_table,
    upsert_document, 
    upsert_image,
    TEXT_TABLE_NAME, 
    IMAGE_TABLE_NAME
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
    SUPPORTED_TEXT_EXTENSIONS,
    SUPPORTED_IMAGE_EXTENSIONS
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
#[derive(Debug, Clone)]
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
                        // text_files_processed += 1;
                        // println!("text files {}", text_files_processed);
                        // println!("path {:?}", path);
                        
                        // // Process text file
                        // if let Err(e) = process_text_file(path, &text_table).await {
                        //     error!("Error processing text file {}: {}", path.display(), e);
                        //     files_failed += 1;
                        //     text_files_failed += 1;
                        //     failed_files.push(path.to_string_lossy().to_string());
                        // } else {
                        //     info!("Indexed text file: {}", path.display());
                        //     db_inserts += 1;
                        //     text_files_indexed += 1;
                        //     indexed_files.push(path.to_string_lossy().to_string());
                        // }
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
async fn process_text_file(file_path: &Path, table: &lancedb::Table) -> Result<(), String> {
    // Extract text content from the file
    let content = extract_text(file_path).map_err(|e| {
        warn!("Extraction error for {}: {}", file_path.display(), e);
        format!("Text extraction failed: {}", e)
    })?;
    
    // Calculate content hash
    let content_hash = calculate_hash(&content);
    
    // Get embeddings for the content
    let content_vec = vec![content.clone()];
    // add a prefix passage: before every content
   
    
    let embeddings = embed_text(&content_vec, false).map_err(|e| {
        error!("Embedding error for {}: {}", file_path.display(), e);
        format!("Embedding generation failed: {}", e)
    })?;
    
    if embeddings.is_empty() {
        return Err(format!("No embeddings generated for {}", file_path.display()));
    }
    
    // Store in the database - now passing all embeddings
    let file_path_str = file_path.to_string_lossy().to_string();
    upsert_document(table, &file_path_str, &content_hash, &embeddings).await.map_err(|e| {
        error!("Database error for {}: {}", file_path.display(), e);
        format!("Database upsert failed: {}", e)
    })?;
    
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
async fn handle_text_indexing(
    text_files: Vec<String>,
    table: Arc<lancedb::Table>
) -> HashMap<String, Result<(), String>> {
    let mut results = HashMap::new();
    
    // Process files in batches to avoid overwhelming the system
    let chunks = text_files.chunks(10);
    for chunk in chunks {
        // Create futures for each file in the chunk
        let futures = chunk.iter().map(|file_path_str| {
            let path = Path::new(file_path_str);
            let table_clone = Arc::clone(&table);
            
            async move {
                let result = async {
                    // Extract text content from the file
                    let content = extract_text(path).map_err(|e| {
                        warn!("Extraction error for {}: {}", path.display(), e);
                        format!("Text extraction failed: {}", e)
                    })?;
                    
                    // Calculate content hash
                    let content_hash = calculate_hash(&content);
                    
                    // Get embeddings for the content
                    let content_vec = vec![content.clone()];
                    let embeddings = embed_text(&content_vec, false).map_err(|e| {
                        error!("Embedding error for {}: {}", path.display(), e);
                        format!("Embedding generation failed: {}", e)
                    })?;
                    
                    if embeddings.is_empty() {
                        return Err(format!("No embeddings generated for {}", path.display()));
                    }
                    
                    // Store in the database
                    upsert_document(&table_clone, file_path_str, &content_hash, &embeddings).await.map_err(|e| {
                        error!("Database error for {}: {}", path.display(), e);
                        format!("Database upsert failed: {}", e)
                    })?;
                    
                    Ok(())
                }.await;
                
                (file_path_str.clone(), result)
            }
        }).collect::<Vec<_>>();
        
        // Wait for all futures in this chunk to complete
        let chunk_results = join_all(futures).await;
        for (path, result) in chunk_results {
            results.insert(path, result);
        }
    }
    
    results
}

/// Handle image file indexing with a batch of files in a separate thread
async fn handle_image_indexing(
    image_files: Vec<String>,
    table: Arc<lancedb::Table>
) -> HashMap<String, Result<(), String>> {
    let mut results = HashMap::new();
    
    // Process files in batches to avoid overwhelming the system
    // let chunks = image_files.chunks(10);
    // for chunk in chunks {
    //     // Create futures for each file in the chunk
    //     let futures = chunk.iter().map(|file_path_str| {
    //         let path = Path::new(file_path_str);
    //         let table_clone = Arc::clone(&table);
            
    //         async move {
    //             let result = async {
    //                 // Process the image
    //                 let image_path = process_image(path).map_err(|e| {
    //                     warn!("Image processing error for {}: {}", path.display(), e);
    //                     format!("Image processing failed: {}", e)
    //                 })?;
                    
    //                 // Calculate file hash for the image
    //                 let file_hash = calculate_file_hash(path).map_err(|e| {
    //                     error!("Hashing error for {}: {}", path.display(), e);
    //                     format!("File hash calculation failed: {}", e)
    //                 })?;
                    
    //                 // Generate embedding for the image
    //                 let embedding = embed_image(&image_path).map_err(|e| {
    //                     error!("Image embedding error for {}: {}", path.display(), e);
    //                     format!("Image embedding generation failed: {}", e)
    //                 })?;
                    
    //                 // For now, we don't have image dimensions or thumbnails
    //                 let width: Option<i32> = None;
    //                 let height: Option<i32> = None;
    //                 let thumbnail_path: Option<&str> = None;
                    
    //                 // Store in the database
    //                 upsert_image(
    //                     &table_clone, 
    //                     file_path_str, 
    //                     &file_hash, 
    //                     &embedding, 
    //                     width, 
    //                     height, 
    //                     thumbnail_path
    //                 ).await.map_err(|e| {
    //                     error!("Database error for {}: {}", path.display(), e);
    //                     format!("Database upsert failed: {}", e)
    //                 })?;
                    
    //                 Ok(())
    //             }.await;
                
    //             (file_path_str.clone(), result)
    //         }
    //     }).collect::<Vec<_>>();
        
    //     // Wait for all futures in this chunk to complete
    //     let chunk_results = join_all(futures).await;
    //     for (path, result) in chunk_results {
    //         results.insert(path, result);
    //     }
    // }
    
    results
}

/// Check if a file is supported for indexing based on content type
fn is_supported_file(path: &Path) -> bool {
    // Skip hidden files (macOS dot files)
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with('.') {
            return false;
        }
    } else {
        return false;
    }
    
    // Check content type
    match get_content_type(path) {
        ContentType::Text | ContentType::Image => true,
        ContentType::Unsupported => false,
    }
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
    let mut text_files = Vec::new();
    let mut image_files = Vec::new();
    let mut files_skipped = 0;
    
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
    
    // Wrap tables in Arc to make them thread-safe
    let text_table_arc = Arc::new(text_table);
    let image_table_arc = Arc::new(image_table);
    
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
                        text_files.push(path.to_string_lossy().to_string());
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
    info!("Found {} text files and {} image files to process", text_files.len(), image_files.len());
    
    // Second pass: process files in parallel using separate threads
    info!("Starting parallel processing of files...");
    
    // Start text file processing in one task
    let text_processing = task::spawn(handle_text_indexing(text_files.clone(), text_table_arc));
    
    // Start image file processing in another task
    let image_processing = task::spawn(handle_image_indexing(image_files.clone(), image_table_arc));
    
    // Wait for both tasks to complete
    let (text_results, image_results) = tokio::join!(
        async { text_processing.await.unwrap_or_else(|e| {
            error!("Text processing task panicked: {}", e);
            HashMap::new()
        })},
        async { image_processing.await.unwrap_or_else(|e| {
            error!("Image processing task panicked: {}", e);
            HashMap::new()
        })}
    );
    
    // Process results and collect statistics
    let mut indexed_files = Vec::new();
    let mut failed_files = Vec::new();
    
    // Text-specific counters
    let text_files_processed = text_files.len() as u32;
    let mut text_files_indexed = 0;
    let mut text_files_failed = 0;
    
    // Image-specific counters
    let image_files_processed = image_files.len() as u32;
    let mut image_files_indexed = 0;
    let mut image_files_failed = 0;
    
    // Process text results
    for (file_path, result) in text_results {
        match result {
            Ok(_) => {
                text_files_indexed += 1;
                indexed_files.push(file_path);
            },
            Err(e) => {
                text_files_failed += 1;
                failed_files.push(file_path);
                error!("Failed to index text file: {}", e);
            }
        }
    }
    
    // Process image results
    for (file_path, result) in image_results {
        match result {
            Ok(_) => {
                image_files_indexed += 1;
                indexed_files.push(file_path);
            },
            Err(e) => {
                image_files_failed += 1;
                failed_files.push(file_path);
                error!("Failed to index image file: {}", e);
            }
        }
    }
    
    // Calculate total statistics
    let files_processed = text_files_processed + image_files_processed;
    let files_failed = text_files_failed + image_files_failed;
    let db_inserts = text_files_indexed + image_files_indexed;
    
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
        "Completed parallel indexing in {}.{:03} seconds: {} files processed, {} failures, {} skipped, {} database inserts",
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
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
        
        assert!(is_supported_file(&pdf_path));
        assert!(is_supported_file(&txt_path));
        assert!(is_supported_file(&md_path));
        assert!(is_supported_file(&jpg_path));
        assert!(is_supported_file(&png_path));
        
        assert!(!is_supported_file(&no_ext_path));
        assert!(!is_supported_file(&hidden_path));
        assert!(!is_supported_file(&zip_path));
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
