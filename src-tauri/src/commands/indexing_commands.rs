// src-tauri/src/commands/indexing_commands.rs

use crate::core::indexer::{index_downloads_folder, index_folder, IndexingStats, get_last_indexing_stats};
use crate::db::{connect_db, TABLE_NAME, clear_data};
use log::{info, error};
use serde::{Deserialize, Serialize};

/// Response model for indexing operations
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexingResponse {
    pub files_processed: u32,
    pub files_indexed: u32,
    pub files_skipped: u32,
    pub files_failed: u32,
    pub time_taken_ms: u32,
    pub success: bool,
    pub message: String,
    pub indexed_files: Vec<String>,
    pub failed_files: Vec<String>,
}

/// Generic operation response
#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResponse {
    pub success: bool,
    pub message: String,
}

/// Tauri command to manually index the Downloads folder
#[tauri::command]
pub async fn index_downloads_command() -> Result<IndexingResponse, String> {
    info!("Manual Downloads folder indexing requested");
    
    match index_downloads_folder().await {
        Ok(stats) => {
            info!("Downloads folder indexing completed successfully");
            
            Ok(IndexingResponse {
                files_processed: stats.files_processed,
                files_indexed: stats.db_inserts,
                files_skipped: stats.files_skipped,
                files_failed: stats.files_failed,
                time_taken_ms: stats.elapsed_seconds * 1000 + stats.elapsed_milliseconds as u32,
                success: true,
                message: format!(
                    "Downloads folder indexed successfully. Processed: {}, Indexed: {}, Skipped: {}, Failed: {}",
                    stats.files_processed, stats.db_inserts, stats.files_skipped, stats.files_failed
                ),
                indexed_files: stats.indexed_files,
                failed_files: stats.failed_files,
            })
        },
        Err(err) => {
            error!("Downloads folder indexing failed: {}", err);
            
            Ok(IndexingResponse {
                files_processed: 0,
                files_indexed: 0,
                files_skipped: 0,
                files_failed: 0,
                time_taken_ms: 0,
                success: false,
                message: format!("Failed to index Downloads folder: {}", err),
                indexed_files: Vec::new(),
                failed_files: Vec::new(),
            })
        }
    }
}

/// Tauri command to index a specific folder
#[tauri::command]
pub async fn index_folder_command(folder_path: String) -> Result<IndexingResponse, String> {
    info!("Manual indexing of folder requested: {}", folder_path);
    
    match index_folder(&folder_path).await {
        Ok(stats) => {
            info!("Folder indexing completed successfully: {}", folder_path);
            
            Ok(IndexingResponse {
                files_processed: stats.files_processed,
                files_indexed: stats.db_inserts,
                files_skipped: stats.files_skipped,
                files_failed: stats.files_failed,
                time_taken_ms: stats.elapsed_seconds * 1000 + stats.elapsed_milliseconds as u32,
                success: true,
                message: format!(
                    "Folder indexed successfully. Processed: {}, Indexed: {}, Skipped: {}, Failed: {}",
                    stats.files_processed, stats.db_inserts, stats.files_skipped, stats.files_failed
                ),
                indexed_files: stats.indexed_files,
                failed_files: stats.failed_files,
            })
        },
        Err(err) => {
            error!("Folder indexing failed for {}: {}", folder_path, err);
            
            Ok(IndexingResponse {
                files_processed: 0,
                files_indexed: 0,
                files_skipped: 0,
                files_failed: 0,
                time_taken_ms: 0,
                success: false,
                message: format!("Failed to index folder: {}", err),
                indexed_files: Vec::new(),
                failed_files: Vec::new(),
            })
        }
    }
}

/// Tauri command to get the last indexing statistics
#[tauri::command]
pub fn get_indexing_stats_command() -> Result<IndexingResponse, String> {
    info!("Request for indexing statistics");
    
    match get_last_indexing_stats() {
        Some(stats) => {
            Ok(IndexingResponse {
                files_processed: stats.files_processed,
                files_indexed: stats.db_inserts,
                files_skipped: stats.files_skipped,
                files_failed: stats.files_failed,
                time_taken_ms: stats.elapsed_seconds * 1000 + stats.elapsed_milliseconds as u32,
                success: true,
                message: "Retrieved last indexing statistics".to_string(),
                indexed_files: stats.indexed_files,
                failed_files: stats.failed_files,
            })
        },
        None => {
            info!("No previous indexing statistics available");
            Ok(IndexingResponse {
                files_processed: 0,
                files_indexed: 0,
                files_skipped: 0,
                files_failed: 0,
                time_taken_ms: 0,
                success: true,
                message: "No indexing has been performed yet".to_string(),
                indexed_files: Vec::new(),
                failed_files: Vec::new(),
            })
        }
    }
}

/// Tauri command to clear all indexed data
#[tauri::command]
pub async fn clear_index_command() -> Result<OperationResponse, String> {
    info!("Request to clear all indexed data");
    
    match connect_db().await {
        Ok(db) => {
            match clear_data(&db, TABLE_NAME).await {
                Ok(_) => {
                    info!("Successfully cleared all indexed data");
                    Ok(OperationResponse {
                        success: true,
                        message: "All indexed data has been cleared successfully".to_string(),
                    })
                },
                Err(e) => {
                    error!("Failed to clear indexed data: {}", e);
                    Ok(OperationResponse {
                        success: false,
                        message: format!("Failed to clear indexed data: {}", e),
                    })
                }
            }
        },
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            Ok(OperationResponse {
                success: false,
                message: format!("Failed to connect to database: {}", e),
            })
        }
    }
}

/// Response model for vector database statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorDbStatsResponse {
    pub text_documents_count: usize,
    pub image_documents_count: usize,
    pub total_documents_count: usize,
}

/// Tauri command to get vector database statistics 
#[tauri::command]
pub async fn get_vector_db_stats_command() -> Result<VectorDbStatsResponse, String> {
    info!("Request for vector database statistics");
    
    // Connect to the database
    match connect_db().await {
        Ok(conn) => {
            // Call the db function to get stats
            match crate::db::get_vector_db_stats(&conn).await {
                Ok((text_count, image_count)) => {
                    let total_count = text_count + image_count;
                    info!("Vector database stats: {} text documents, {} image documents, {} total", 
                          text_count, image_count, total_count);
                    
                    Ok(VectorDbStatsResponse {
                        text_documents_count: text_count,
                        image_documents_count: image_count,
                        total_documents_count: total_count,
                    })
                },
                Err(e) => {
                    error!("Failed to get vector database stats: {}", e);
                    Err(format!("Failed to get vector database stats: {}", e))
                }
            }
        },
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            Err(format!("Database connection error: {}", e))
        }
    }
}

/// Run Downloads folder indexing at application startup
/// This is not exposed as a Tauri command, but called internally
pub async fn run_startup_indexing() {
    info!("Starting automatic Downloads folder indexing on application startup");
    
    match index_downloads_folder().await {
        Ok(stats) => {
            info!(
                "Startup indexing completed. Processed: {}, Indexed: {}, Skipped: {}, Failed: {}, Time: {}.{:03}s",
                stats.files_processed, stats.db_inserts, stats.files_skipped, stats.files_failed,
                stats.elapsed_seconds, stats.elapsed_milliseconds
            );
        },
        Err(err) => {
            error!("Startup indexing failed: {}", err);
        }
    }
}
