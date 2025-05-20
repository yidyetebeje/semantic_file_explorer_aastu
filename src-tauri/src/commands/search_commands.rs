use crate::db::{connect_db, connect_db_with_path, open_or_create_text_table};
use crate::search::{multimodal_search, SearchResult, SearchContentType};
use crate::extractor::ContentType;
// Remove old FilenameIndex imports
// use crate::filename_index::{ThreadSafeIndex, FilenameSearchResult, FileCategory, FilenameIndexError};
use log::{info, error, warn, debug};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use lancedb::query::ExecutableQuery;
use futures_util::stream::TryStreamExt;
// HashSet removed - not used
use std::path::{PathBuf};
use once_cell::sync::Lazy;
use std::fs::{self, metadata, create_dir_all};
use walkdir::WalkDir;
use tokio::task;
use directories::ProjectDirs; // Use the new 'directories' crate
use dirs; // Add the dirs crate for home_dir()

// Using rust_search for filename search. Tantivy imports removed.
use rust_search::SearchBuilder;
use std::path::Path; // Only import Path, not PathBuf again
use shellexpand; // For tilde path expansion
// Removed duplicate import of metadata
use directories; // For user directories (already a dependency, ensure consistent use)
// Tantivy-specific structs (FilenameSchema), statics (TANTIVY_SCHEMA, TANTIVY_INDEX),
// and helper functions (get_index) have been removed as rust_search operates on the live filesystem.

// Remove old static FILENAME_INDEX
// pub static FILENAME_INDEX: Lazy<ThreadSafeIndex> = Lazy::new(|| FilenameIndex::new_thread_safe());

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    /// The search query text
    pub query: String,
    
    /// Optional maximum number of results to return
    pub limit: Option<usize>,
    
    /// Optional minimum score threshold (0.0 to 1.0)
    pub min_score: Option<f32>,
    
    /// Optional database URI (defaults to DB_URI)
    pub db_uri: Option<String>,
    
    /// Optional content type filter (defaults to All)
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results sorted by relevance (highest score first)
    pub results: Vec<SearchResult>,
    
    /// Total number of results found
    pub total_results: usize,
    
    /// Original query that was searched for
    pub query: String,
}

/// Command to perform a semantic search across both text and image content
#[tauri::command]
pub async fn semantic_search_command(request: SearchRequest) -> Result<SearchResponse, String> {
    println!("Received search request for query: {}", request.query);
    info!("Received search request for query: {}", request.query);
    
    // Validate the query is not empty
    if request.query.trim().is_empty() {
        return Err("Query is empty".to_string());
    }
    
    // Parse content type filter if provided
    let content_type = match request.content_type.as_deref() {
        Some("text") => Some(SearchContentType::TextOnly),
        Some("image") => Some(SearchContentType::ImageOnly),
        Some("all") | None => Some(SearchContentType::All),
        Some(unknown) => {
            warn!("Unknown content type filter: {}", unknown);
            Some(SearchContentType::All)
        }
    };

    // Use custom DB URI if provided, otherwise use default
    let conn = match if let Some(db_uri) = request.db_uri {
        println!("Connecting to custom database: {}", db_uri);
        connect_db_with_path(&db_uri).await
    } else {
        println!("Connecting to default database");
        connect_db().await
    } {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(format!("Failed to connect to database: {}", e));
        }
    };
    
    println!("Performing multimodal search");
    // Perform the multimodal search (text and images)
    match multimodal_search(&conn, &request.query, request.limit, request.min_score, content_type).await {
        Ok(results) => {
            let total = results.len();
            let text_count = results.iter().filter(|r| r.content_type == ContentType::Text).count();
            let image_count = results.iter().filter(|r| r.content_type == ContentType::Image).count();         
            info!("Search completed with {} results ({} text, {} images)", total, text_count, image_count);
            println!("Search completed with {} results ({} text, {} images)", total, text_count, image_count);
            Ok(SearchResponse {
                results,
                total_results: total,
                query: request.query,
            })
        },
        Err(e) => {
            println!("Search failed: {}", e);
            error!("Search failed: {}", e);
            Err(format!("Search failed: {}", e))
        }
    }
}

/// Command to get the total number of documents in the database
#[tauri::command]
pub async fn get_document_count() -> Result<usize, String> {
    // Connect to the database
    let conn = match connect_db().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(format!("Failed to connect to database: {}", e));
        }
    };
    
    let table = match open_or_create_text_table(&conn).await {
        Ok(table) => table,
        Err(e) => {
            error!("Failed to open table: {}", e);
            return Err(format!("Failed to open table: {}", e));
        }
    };
    
    // Get the count of documents by executing a simple query that returns all records
    match table.query().execute().await {
        Ok(batches) => {
            let batch_count = batches
                .try_collect::<Vec<_>>()
                .await
                .map(|batches| batches.iter().map(|batch| batch.num_rows()).sum::<usize>())
                .unwrap_or(0);
                
            info!("Database contains {} documents", batch_count);
            Ok(batch_count)
        },
        Err(e) => {
            error!("Failed to count documents: {}", e);
            Err(format!("Failed to count documents: {}", e))
        }
    }
}

// --- Filename Search Types (Adjusted) ---
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)] // Ensure Clone, etc. are present if needed
pub enum FileCategory {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Other
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilenameSearchRequest {
    /// The search query text
    pub query: String,
    
    /// Optional file categories to filter by
    pub categories: Option<Vec<FileCategory>>,
    
    /// Optional maximum number of results to return (default: 10)
    pub limit: Option<usize>,
    
    /// Optional path to filter results by
    pub path_filter: Option<String>,
    
    /// Optional category filter
    pub category_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilenameSearchResult {
    pub file_path: String,
    pub name: String,
    pub category: FileCategory, // Keep using the enum
    pub last_modified: u64,
    pub size: u64,
    pub score: f32, // Changed from distance: usize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilenameSearchResponse {
    /// Search results sorted by relevance
    pub results: Vec<FilenameSearchResult>,
    
    /// Total number of results found (from searcher or results.len())
    pub total_results: usize,
    
    /// Original query that was searched for
    pub query: String,
}

// --- Filename Commands (Implementing) ---

// Helper to determine file category (You might want to move this to a shared module)
fn categorize_file(path: &PathBuf) -> FileCategory {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        match extension.to_lowercase().as_str() {
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "md" | "csv" | "xls" | "xlsx" | "ppt" | "pptx" => FileCategory::Document,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "svg" | "ico" | "heic" => FileCategory::Image,
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" => FileCategory::Video,
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" | "wma" => FileCategory::Audio,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => FileCategory::Archive,
            "py" | "js" | "jsx" | "ts" | "tsx" | "html" | "css" | "scss" | "json" | "rs" | "go" | "java" | "cpp" | "c" | "h" | "cs" | "php" | "rb" => FileCategory::Code,
            _ => FileCategory::Other,
        }
    } else {
        FileCategory::Other
    }
}

/// Command to perform a filename search using Tantivy
#[tauri::command]
pub async fn filename_search_command(request: FilenameSearchRequest) -> Result<FilenameSearchResponse, String> {
    info!("Filename search request with rust_search: {:?}", request);

    let search_query = request.query.trim();
    if search_query.is_empty() {
        return Err("Filename search query cannot be empty.".to_string());
    }

    let mut search_builder = SearchBuilder::default()
        .search_input(search_query)
        .ignore_case()
        .hidden(); // Consider making .hidden() configurable

    // Apply limit if provided
    if let Some(limit) = request.limit {
        search_builder = search_builder.limit(limit);
    }

    // Determine search locations
    let mut search_locations: Vec<String> = Vec::new();
    if let Some(path_filter) = &request.path_filter {
        let expanded_path_str = shellexpand::tilde(path_filter).into_owned();
        match Path::new(&expanded_path_str).try_exists() {
            Ok(true) => {
                search_locations.push(expanded_path_str);
            },
            Ok(false) => {
                warn!("Path filter doesn't exist: {}", path_filter);
                return Err(format!("Path doesn't exist: {}", path_filter));
            },
            Err(e) => {
                error!("Error checking path filter: {}", e);
                return Err(format!("Error checking path: {}", e));
            }
        }
    } else {
        // Default to home directory if no path filter provided
        if let Some(home_dir) = dirs::home_dir() {
            let home_dir_str = home_dir.to_string_lossy().to_string();
            search_locations.push(home_dir_str);
        } else {
            return Err("Could not determine home directory".to_string());
        }
    }

    // Apply search locations to the builder
    if let Some(first_location) = search_locations.first() {
        search_builder = search_builder.location(first_location);
        if search_locations.len() > 1 {
             search_builder = search_builder.more_locations(search_locations.iter().skip(1).map(|s| s.as_str()).collect());
        }
    } else {
        // This case should ideally be handled by the empty check above, but as a safeguard:
        return Err("No search locations specified or determined.".to_string());
    }

    // Perform the search using rust_search
    let found_paths_str: Vec<String> = search_builder.build().collect();
    debug!("rust_search found {} paths before category filtering.", found_paths_str.len());

    let mut results: Vec<FilenameSearchResult> = Vec::new();
    for path_str in found_paths_str {
        let path_buf = PathBuf::from(&path_str);

        // Apply category filter (post-search filtering)
        if let Some(category_filter) = &request.category_filter {
            let file_cat = categorize_file(&path_buf);
            
            // Convert category_filter string to FileCategory for comparison
            let category_to_match = match category_filter.to_lowercase().as_str() {
                "document" => FileCategory::Document,
                "image" => FileCategory::Image,
                "video" => FileCategory::Video,
                "audio" => FileCategory::Audio,
                "archive" => FileCategory::Archive,
                "code" => FileCategory::Code,
                "other" => FileCategory::Other,
                _ => {
                    warn!("Unknown category filter: {}", category_filter);
                    continue; // Skip this file if category is unknown
                }
            };
            
            if file_cat != category_to_match {
                continue; // Skip if category doesn't match
            }
        }

        let name = path_buf.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let category = categorize_file(&path_buf);
        
        let mut last_modified_ms: Option<u64> = None;
        let mut size_bytes: Option<u64> = None;
        if let Ok(md) = metadata(&path_buf) {
            size_bytes = Some(md.len());
            if let Ok(modified_time) = md.modified() {
                if let Ok(duration_since_epoch) = modified_time.duration_since(std::time::UNIX_EPOCH) {
                    last_modified_ms = Some(duration_since_epoch.as_millis() as u64);
                }
            }
        }

        results.push(FilenameSearchResult {
            file_path: path_str,
            name,
            category,
            score: 1.0, // Default score for a filename match
            last_modified: last_modified_ms.unwrap_or(0),
            size: size_bytes.unwrap_or(0),
        });
    }

    // If a limit was specified, rust_search should handle it. If not, and we need to apply it post-category-filtering:
    // if let Some(limit) = request.limit {
    //     results.truncate(limit);
    // }
    // `rust_search`'s `.limit()` applies to its direct output. If category filtering significantly reduces items,
    // the number of results might be less than the requested limit.
    // This behavior is acceptable for now.

    let total_results = results.len();
    
    Ok(FilenameSearchResponse {
        results,
        total_results,
        query: request.query,
    })
}

/// Command to add a file to the filename index (No-op with rust_search)
#[tauri::command]
pub async fn add_file_to_index(path: String, last_modified: u64, size: u64) -> Result<(), String> {
    info!("'add_file_to_index' called for path: {}. Args (last_modified: {}, size: {}). This is a no-op as filename search uses the live filesystem via rust_search.", path, last_modified, size);
    Ok(())
}

/// Command to remove a file from the filename index (No-op with rust_search)
#[tauri::command]
pub async fn remove_file_from_index(path: String) -> Result<(), String> {
    info!("'remove_file_from_index' called for path: {}. This is a no-op as filename search uses the live filesystem via rust_search.", path);
    Ok(())
}

/// Command to get stats about the filename "index" (Informational with rust_search)
#[tauri::command]
pub async fn get_filename_index_stats() -> Result<serde_json::Value, String> {
    info!("'get_filename_index_stats' called. Filename search uses the live filesystem via rust_search, so no persistent index is maintained.");
    let stats = serde_json::json!({
        "status": "Filename search operates on the live filesystem using rust_search.",
        "indexed_files_count": 0, // Reflects no separate persistent index
        "index_type": "rust_search (live filesystem)"
    });
    Ok(stats)
}

/// Command to clear the filename index (No-op with rust_search)
#[tauri::command]
pub async fn clear_filename_index() -> Result<(), String> {
    info!("'clear_filename_index' called. This is a no-op as filename search uses the live filesystem via rust_search and does not maintain a persistent index to clear.");
    Ok(())
}

/// Command to scan a directory and add files to the filename index (No-op with rust_search)
#[tauri::command]
pub async fn scan_directory_for_filename_index(dir_path: String) -> Result<serde_json::Value, String> {
    info!("'scan_directory_for_filename_index' called for path: {}. This is a no-op as filename search uses the live filesystem via rust_search.", dir_path);
    Ok(serde_json::json!({
        "status": format!("Directory scan for a persistent index is not applicable with rust_search. Search is live for directory: {}.", dir_path),
        "files_added_or_updated": 0,
        "errors_encountered": 0
    }))
}

/// Initialize the filename index with common directories (No-op with rust_search)
#[tauri::command]
pub async fn initialize_filename_index() -> Result<serde_json::Value, String> {
    info!("'initialize_filename_index' called. This is a no-op as filename search uses the live filesystem via rust_search and does not require explicit initialization of common directories in this manner.");
    Ok(serde_json::json!({
        "status": "Filename index initialization is not applicable with rust_search. Search is live.",
        "total_files_added_or_updated": 0,
        "total_errors_encountered": 0,
        "scanned_paths": []
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::TestDb;
    use std::fs::File;
    use tempfile::tempdir;

    // Helper to create a test database
    async fn setup_test_db() -> (TestDb, String) {
        let test_db = TestDb::new();
        let path = test_db.path.clone();
        (test_db, path)
    }

    #[tokio::test]
    async fn test_semantic_search_command_success() {
        // Setup test database
        let (_test_db, db_path) = setup_test_db().await;
        
        let request = SearchRequest {
            query: "test query".to_string(),
            limit: Some(5),
            min_score: Some(0.7),
            db_uri: Some(db_path.clone()),
            content_type: Some("all".to_string()),
        };
        
        let response = semantic_search_command(request).await;
        
        assert!(response.is_ok(), "Command should succeed even with empty results");
        
        let result = response.unwrap();
        assert_eq!(result.query, "test query");
        assert_eq!(result.total_results, 0);
        assert!(result.results.is_empty());
    }

    #[tokio::test]
    async fn test_semantic_search_command_with_empty_query() {
        let request = SearchRequest {
            query: "".to_string(),
            limit: None,
            min_score: None,
            db_uri: None,
            content_type: Some("all".to_string()), // Ensuring this matches original intent
        };
        
        let response = semantic_search_command(request).await;
        assert!(response.is_err(), "Empty query should lead to an error");
        assert!(response.unwrap_err().to_lowercase().contains("empty"), "Error should mention empty query");
    }

    // Old filename search tests related to Tantivy are removed or commented out.
    // New tests for rust_search based live filesystem search would require
    // mocking the filesystem or `rust_search` interactions, which is complex for this scope.
    // For now, manual testing or integration tests would be more appropriate for `filename_search_command`.
}

