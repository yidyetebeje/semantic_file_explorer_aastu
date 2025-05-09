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

// --- Tantivy Setup ---
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, TantivyError, directory::MmapDirectory};
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, BooleanQuery, TermQuery, Occur};

// Define Tantivy schema fields
struct FilenameSchema {
    schema: Schema,
    path: Field,
    name: Field,
    category: Field,
    last_modified: Field,
    size: Field,
}

impl FilenameSchema {
    fn new() -> Self {
        let mut schema_builder = Schema::builder();
        let text_indexing_options = TextFieldIndexing::default()
            .set_tokenizer("en_stem") // Use English stemmer
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing_options)
            .set_stored(); // Store the original text

        let path = schema_builder.add_text_field("path", STRING | STORED); // Path stored as string, used as ID
        let name = schema_builder.add_text_field("name", text_options.clone()); // Name indexed and stored
        let category = schema_builder.add_text_field("category", STRING | STORED); // Category stored as string
        let last_modified = schema_builder.add_u64_field("last_modified", STORED);
        let size = schema_builder.add_u64_field("size", STORED);
        
        let schema = schema_builder.build();
        
        Self {
            schema,
            path,
            name,
            category,
            last_modified,
            size,
        }
    }
}

// Global lazy static for schema fields
static TANTIVY_SCHEMA: Lazy<FilenameSchema> = Lazy::new(FilenameSchema::new);

// Global lazy static for Tantivy index
static TANTIVY_INDEX: Lazy<Result<Index, TantivyError>> = Lazy::new(|| {
    // Define index path (e.g., in app data directory)
    if let Some(proj_dirs) = ProjectDirs::from("com", "YourCompany", "SemanticFileExplorer") {
        let data_dir = proj_dirs.data_local_dir();
        let index_path = data_dir.join("filename_index");
        
        info!("Using Tantivy index path: {:?}", index_path);
        
        // Create the directory if it doesn't exist
        if !index_path.exists() {
            match create_dir_all(&index_path) {
                Ok(_) => info!("Created index directory: {:?}", index_path),
                Err(e) => {
                    error!("Failed to create index directory {:?}: {}", index_path, e);
                    // Map std::io::Error to TantivyError::IoError, wrapping in Arc
                    return Err(TantivyError::IoError(e.into())); 
                }
            }
        }
        
        // Open or create the index using MmapDirectory
        let schema = TANTIVY_SCHEMA.schema.clone();
        let dir = MmapDirectory::open(&index_path)?;
        Index::open_or_create(dir, schema)
    } else {
        error!("Could not determine application data directory for Tantivy index.");
        Err(TantivyError::SystemError("Could not determine application data directory".to_string()))
    }
});

// Helper to get the index, handling the initialization result
fn get_index() -> Result<&'static Index, String> {
    match &*TANTIVY_INDEX {
        Ok(index) => Ok(index),
        Err(e) => {
            let err_msg = format!("Failed to load or create Tantivy index: {}", e);
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}

// --- End Tantivy Setup ---

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
            // We need to collect all the batches to get the total count
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
    
    // `max_distance` is removed as it's not directly applicable to Tantivy's default search
    // pub max_distance: Option<usize>,
    
    /// Optional file categories to filter by
    pub categories: Option<Vec<FileCategory>>,
    
    /// Optional maximum number of results to return (default: 10)
    pub limit: Option<usize>,
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
    info!("Received Tantivy filename search request for query: {}", request.query);
    
    let limit = request.limit.unwrap_or(10);
    let query_text = request.query.clone();
    let categories_filter = request.categories.clone(); // Clone for the closure

    let search_result_outer = task::spawn_blocking(move || -> Result<Vec<FilenameSearchResult>, String> { // Explicit Result type
        let index = get_index()?;
        let reader = index.reader()
            .map_err(|e| format!("Failed to get Tantivy reader: {}", e))?;
        let searcher = reader.searcher();
        let schema_fields = &*TANTIVY_SCHEMA;

        // Create a query parser for the 'name' field
        // You could add more default fields here if needed: vec![schema_fields.name, schema_fields.path]
        let query_parser = QueryParser::for_index(index, vec![schema_fields.name]);
        
        // Parse the user query. This supports Tantivy's query syntax (Boolean, Phrase, Fuzzy, etc.)
        let query = query_parser.parse_query(&query_text)
             .map_err(|e| format!("Failed to parse query '{}': {}", query_text, e))?;

        // --- Build the final query including category filters --- 
        let final_query: Box<dyn tantivy::query::Query>;

        if let Some(cats) = categories_filter {
            if !cats.is_empty() {
                let category_queries: Vec<Box<dyn tantivy::query::Query>> = cats.into_iter()
                    .filter_map(|cat| { // Use filter_map to handle potential serialization errors gracefully
                        match serde_json::to_string(&cat) {
                            Ok(cat_str) => Some(Box::new(TermQuery::new(
                                Term::from_field_text(schema_fields.category, &cat_str),
                                IndexRecordOption::Basic, // Basic matching is enough for category terms
                            )) as Box<dyn tantivy::query::Query>),
                            Err(e) => {
                                error!("Failed to serialize category {:?} for query: {}", cat, e);
                                None // Skip this category if serialization fails
                            }
                        }
                    })
                    .collect();
                
                if !category_queries.is_empty() {
                    // Combine category terms with SHOULD (OR logic)
                    // Use BooleanQuery::union for OR logic between terms
                    let category_boolean_query = BooleanQuery::union(category_queries);
                    // Combine the main query and the category filter with MUST (AND logic)
                    final_query = Box::new(BooleanQuery::intersection(vec![ // Use intersection for AND
                        query,
                        Box::new(category_boolean_query)
                    ]));
                } else {
                    // If all category serializations failed, just use the original query
                    final_query = query;
                }
            } else {
                 // No categories selected, use the original query
                 final_query = query;
            }
        } else {
            // No category filter applied, use the original query
            final_query = query;
        }
        // --- End of query building ---
        
        // Execute the search
        let top_docs = searcher.search(&*final_query, &TopDocs::with_limit(limit))
            .map_err(|e| format!("Tantivy search failed: {}", e))?;
        
        // Process results
        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            // Explicitly type retrieved_doc as TantivyDocument
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)
                .map_err(|e| format!("Failed to retrieve doc {:?}: {}", doc_address, e))?;
            
            // Extract fields safely using helper closure
            let get_text = |field: Field| -> String {
                retrieved_doc.get_first(field)
                    .and_then(|v| v.as_str()) // Use as_str() for OwnedValue which returns Option<&str>
                    .unwrap_or("") // Default to empty string if None
                    .to_string() // Convert &str to String
            };
            let get_u64 = |field: Field| -> u64 {
                 retrieved_doc.get_first(field)
                    .and_then(|v| v.as_u64()) // as_u64() returns Option<u64>
                    .unwrap_or(0) // Default to 0 if None
            };

            let path_str = get_text(schema_fields.path);
            let category_str = get_text(schema_fields.category);

            // Deserialize category string back to enum, default to Other on error
            let category: FileCategory = serde_json::from_str(&category_str).unwrap_or_else(|e| {
                warn!("Failed to deserialize category '{}' for path {}: {}. Defaulting to Other.", category_str, path_str, e);
                FileCategory::Other 
            });
            
            results.push(FilenameSearchResult {
                file_path: path_str,
                name: get_text(schema_fields.name),
                category,
                last_modified: get_u64(schema_fields.last_modified),
                size: get_u64(schema_fields.size),
                score, // Use the score from Tantivy
            });
        }
        Ok(results)
    }).await.map_err(|e| format!("Blocking search task failed: {}", e))?;

    // Handle potential errors from within the blocking task
    match search_result_outer {
        Ok(search_results) => {
            let total = search_results.len();
            info!("Tantivy filename search completed with {} results for query: '{}'", total, request.query);
            Ok(FilenameSearchResponse {
                results: search_results,
                total_results: total, // Using results length, Tantivy doesn't easily give total hits for complex queries
                query: request.query,
            })
        }
        Err(e) => {
             error!("Tantivy filename search failed for query '{}': {}", request.query, e);
             Err(e) // Propagate the error string
        }
    }
}

/// Command to add a file to the filename index
#[tauri::command]
pub async fn add_file_to_index(path: String, last_modified: u64, size: u64) -> Result<(), String> {
    info!("Adding file to index: {} (Tantivy)", path);
    
    let path_clone = path.clone(); // Clone for blocking task
    let result = task::spawn_blocking(move || {
        let path_buf = PathBuf::from(path_clone);
        let file_name = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let category = categorize_file(&path_buf); 
        // Use serde_json to serialize category enum to string for storing in Tantivy
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?;

        let index = get_index()?;
        let mut writer = index.writer(50_000_000) // 50MB heap budget
             .map_err(|e| format!("Failed to get Tantivy writer: {}", e))?;
        let schema_fields = &*TANTIVY_SCHEMA;

        // First, delete any existing entry for this path to handle updates
        let path_term = Term::from_field_text(schema_fields.path, &path_buf.to_string_lossy());
        writer.delete_term(path_term);

        // Add the new document
        writer.add_document(doc!(
            schema_fields.path => path_buf.to_string_lossy().as_ref(), // Store full path as ID
            schema_fields.name => file_name,
            schema_fields.category => category_str,
            schema_fields.last_modified => last_modified,
            schema_fields.size => size
        )).map_err(|e| format!("Failed to add Tantivy document: {}", e))?;

        // Commit changes
        writer.commit().map_err(|e| format!("Tantivy commit failed: {}", e))?; 
        // writer.wait_merging_threads().map_err(|e| format!("Tantivy merge thread wait failed: {}", e))?; // Optional: Wait for merges
        Ok::<(), String>(())

    }).await.map_err(|e| format!("Blocking add task failed: {}", e))?;

    match result {
        Ok(_) => {
            info!("File {} add/update processed for Tantivy index", path);
            Ok(())
        }
        Err(e) => {
            error!("Error processing file {} for Tantivy index: {}", path, e);
            Err(e)
        }
    }
}

/// Command to remove a file from the filename index
#[tauri::command]
pub async fn remove_file_from_index(path: String) -> Result<(), String> {
    info!("Removing file from Tantivy index: {}", path);
    
    let path_clone = path.clone(); // Clone path for the closure
    let result = task::spawn_blocking(move || {
        let index = get_index()?;
        // Explicitly specify the type parameter for IndexWriter
        let mut writer: IndexWriter<TantivyDocument> = index.writer(50_000_000) // 50MB heap budget 
            .map_err(|e| format!("Failed to get Tantivy writer: {}", e))?;
        let schema_fields = &*TANTIVY_SCHEMA;
        
        // Use the cloned path inside the closure
        let path_term = Term::from_field_text(schema_fields.path, &path_clone);
        let opstamp = writer.delete_term(path_term); // opstamp indicates if delete occurred
        debug!("Delete operation for path {} returned opstamp: {}", path_clone, opstamp); 
        
        writer.commit().map_err(|e| format!("Tantivy commit failed: {}", e))?;
        // writer.wait_merging_threads().map_err(|e| format!("Tantivy merge thread wait failed: {}", e))?; // Optional
        Ok::<(), String>(())
    }).await.map_err(|e| format!("Blocking remove task failed: {}", e))?;

    match result {
        Ok(_) => {
            // Use the original path variable here (outside the closure)
            info!("File {} removed from Tantivy index (if it existed)", path);
            Ok(())
        }
        Err(e) => {
             // Use the original path variable here
            error!("Error removing file {} from Tantivy index: {}", path, e);
            Err(e)
        }
    }
}

/// Command to get the total number of files in the index (Implementing)
#[tauri::command]
pub async fn get_filename_index_stats() -> Result<serde_json::Value, String> {
    info!("Getting Tantivy index stats");

    let result = task::spawn_blocking(move || {
        let index = get_index()?;
        let reader = index.reader()
            .map_err(|e| format!("Failed to get Tantivy reader: {}", e))?;
        let searcher = reader.searcher();
        let file_count = searcher.num_docs();
        Ok::<serde_json::Value, String>(serde_json::json!({ "file_count": file_count }))
    }).await.map_err(|e| format!("Blocking stats task failed: {}", e))?;

    match result {
        Ok(stats) => {
             info!("Tantivy index contains {} documents", stats.get("file_count").unwrap_or(&serde_json::Value::Null));
             Ok(stats)
        }
        Err(e) => {
            error!("Error getting Tantivy index stats: {}", e);
            Err(e)
        }
    }
}

/// Command to clear the filename index
#[tauri::command]
pub async fn clear_filename_index() -> Result<(), String> {
    info!("Request to clear Tantivy filename index");
    
    let result = task::spawn_blocking(move || {
        let index = get_index()?;
        // Use the default TantivyDocument type for the writer
        let mut writer: IndexWriter<TantivyDocument> = index.writer(50_000_000) 
            .map_err(|e| format!("Failed to get Tantivy writer: {}", e))?;
        writer.delete_all_documents()
            .map_err(|e| format!("Tantivy delete all failed: {}", e))?;
        writer.commit()
            .map_err(|e| format!("Tantivy commit failed: {}", e))?;
        // writer.wait_merging_threads().map_err(|e| format!("Tantivy merge thread wait failed: {}", e))?; // Optional
        Ok::<(), String>(())
    }).await.map_err(|e| format!("Blocking clear task failed: {}", e))?;

    match result {
        Ok(_) => {
            info!("Tantivy filename index cleared successfully");
            Ok(())
        }
        Err(e) => {
            error!("Error clearing Tantivy index: {}", e);
            Err(e)
        }
    }
}

/// Command to scan a directory and add files to the filename index
#[tauri::command]
pub async fn scan_directory_for_filename_index(dir_path: String) -> Result<serde_json::Value, String> {
    info!("Scanning directory for Tantivy index: {}", dir_path);

    let result = task::spawn_blocking(move || {
        let index = get_index()?;
        // Increase heap budget for potentially large scan operations
        let mut writer = index.writer(100_000_000) 
            .map_err(|e| format!("Failed to get Tantivy writer: {}", e))?;
        let schema_fields = &*TANTIVY_SCHEMA;
        
        let mut files_added = 0;
        let mut errors = Vec::<String>::new();
        let mut docs_processed_since_commit = 0;
        const COMMIT_THRESHOLD: u32 = 1000; // Commit every 1000 documents

        for entry in WalkDir::new(&dir_path).into_iter().filter_map(Result::ok) {
            let path_buf = entry.path().to_path_buf();
            if path_buf.is_file() {
                match metadata(&path_buf) {
                    Ok(meta) => {
                        let last_modified = meta.modified()
                            .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                            .unwrap_or(0);
                        let size = meta.len();
                        let file_name = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                        let category = categorize_file(&path_buf);
                        let category_str = serde_json::to_string(&category)
                            .unwrap_or_else(|e| {
                                errors.push(format!("Category serialization failed for {:?}: {}", path_buf, e));
                                "\"Other\"".to_string() // Default to Other on serialization error
                            });
                        let path_str = path_buf.to_string_lossy().to_string();

                        // Delete existing entry first to handle updates within the scan
                        // writer.delete_term(Term::from_field_text(schema_fields.path, &path_str)); 
                        
                        match writer.add_document(doc!(
                            schema_fields.path => path_str.clone(),
                            schema_fields.name => file_name,
                            schema_fields.category => category_str,
                            schema_fields.last_modified => last_modified,
                            schema_fields.size => size
                        )) {
                            Ok(_) => {
                                files_added += 1;
                                docs_processed_since_commit += 1;
                            },
                            Err(e) => {
                                let error_msg = format!("Failed to add doc {:?}: {}", path_buf, e);
                                error!("{}", error_msg);
                                errors.push(error_msg);
                            }
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to read metadata for {:?}: {}", path_buf, e);
                        error!("{}", error_msg);
                        errors.push(error_msg);
                    }
                }
                
                // Commit periodically to avoid using too much memory
                if docs_processed_since_commit >= COMMIT_THRESHOLD {
                    match writer.commit() {
                        Ok(_) => {
                            info!("Periodic commit during scan of {} ({} docs)", dir_path, docs_processed_since_commit);
                            docs_processed_since_commit = 0; // Reset counter
                            // Re-acquire writer as commit consumes it
                            writer = index.writer(100_000_000) 
                                .map_err(|e| format!("Failed to re-acquire Tantivy writer after commit: {}", e))?;
                        },
                        Err(e) => {
                            let error_msg = format!("Periodic commit failed during scan: {}", e);
                            error!("{}", error_msg);
                            errors.push(error_msg);
                            // Attempt to re-acquire writer anyway to continue scanning
                             writer = index.writer(100_000_000) 
                                .map_err(|e| format!("Failed to re-acquire Tantivy writer after failed commit: {}", e))?;
                        }
                    }
                }
            }
        }

        // Final commit for any remaining documents
        writer.commit().map_err(|e| format!("Final commit failed for {}: {}", dir_path, e))?;
        // writer.wait_merging_threads().map_err(|e| format!("Final merge wait failed: {}", e))?; // Optional

        info!("Directory scan completed for {}. Added: {}, Errors: {}", dir_path, files_added, errors.len());
        Ok::<serde_json::Value, String>(serde_json::json!({
            "directory": dir_path,
            "files_added": files_added,
            "errors": errors
        }))
    }).await.map_err(|e| format!("Blocking scan task failed: {}", e))?;

    result
}

/// Initialize the filename index with common directories
#[tauri::command]
pub async fn initialize_filename_index() -> Result<serde_json::Value, String> {
    info!("Initializing Tantivy filename index with common directories");
    
    let mut total_files: u64 = 0;
    let mut results = Vec::new();
    
    // Use standard `dirs` crate functions now
    let common_dirs = vec![
        dirs::download_dir(),
        dirs::document_dir(),
        dirs::desktop_dir(),
        dirs::picture_dir(),
        dirs::video_dir(),
        dirs::audio_dir(),
    ];

    for dir_option in common_dirs {
        if let Some(dir_path) = dir_option {
            let path_str = dir_path.to_string_lossy().to_string();
            info!("Initializing - Scanning common directory: {}", path_str);
            // Call the newly implemented Tantivy scan function
            match scan_directory_for_filename_index(path_str).await { 
                Ok(result) => {
                    if let Some(files_added) = result.get("files_added").and_then(|v| v.as_u64()) {
                        total_files += files_added;
                    }
                    results.push(result);
                },
                Err(e) => {
                    error!("Failed to scan common directory {:?}: {}", dir_path, e);
                    // Store error information for this directory
                    results.push(serde_json::json!({
                        "directory": dir_path.to_string_lossy(),
                        "files_added": 0,
                        "errors": [format!("Scan failed: {}", e)]
                    }));
                }
            }
        } else {
                warn!("Could not determine path for a common directory during initialization.");
        }
    }
    
    info!("Initialized Tantivy filename index with {} total files from common dirs", total_files);
    
    Ok(serde_json::json!({
        "total_files_added": total_files,
        "directory_results": results,
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
        
        // Create test request
        let request = SearchRequest {
            query: "test query".to_string(),
            limit: Some(5),
            min_score: Some(0.7),
            db_uri: Some(db_path.clone()),
            content_type: Some("all".to_string()),
        };
        
        // Execute the command
        let response = semantic_search_command(request).await;
        
        // The test DB is empty, so we shouldn't get any results but the command should succeed
        assert!(response.is_ok(), "Command should succeed even with empty results");
        
        let result = response.unwrap();
        assert_eq!(result.query, "test query");
        assert_eq!(result.total_results, 0);
        assert!(result.results.is_empty());
    }

    #[tokio::test]
    async fn test_semantic_search_command_with_empty_query() {
        // Create request with empty query
        let request = SearchRequest {
            query: "".to_string(),
            limit: None,
            min_score: None,
            db_uri: None,
            content_type: None,
        };
        
        // Execute the command - should give an error
        let response = semantic_search_command(request).await;
        assert!(response.is_err(), "Empty query should lead to an error");
        assert!(response.unwrap_err().contains("empty"), "Error should mention empty query");
    }

    // Remove or adapt old filename search test
    /*
    #[test]
    fn test_filename_search_functionality() {
        // ... old test code ...
    }
    */
}
