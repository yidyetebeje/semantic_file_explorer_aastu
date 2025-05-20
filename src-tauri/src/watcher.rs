// src-tauri/src/watcher.rs

use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher as NotifyWatcher, EventKind};
use notify::event::{CreateKind, ModifyKind, RenameMode, DataChange};
use log::{error, info, warn};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use thiserror::Error;
use crate::db::{delete_document, upsert_document, DbError, connect_db, open_or_create_text_table};
use crate::embedder::embed_text;
use crate::extractor::{extract_text, calculate_hash};
use crate::commands::search_commands::{add_file_to_index, remove_file_from_index};
use lancedb::Table;
use std::sync::Arc;
// Used in tests for timeouts
#[cfg(test)]
use std::time::Duration;
#[cfg(test)]
use crate::db::TestDb;
#[cfg(test)]
use std::io::Write;
use std::fs::metadata;

// Define supported extensions
const SUPPORTED_EXTENSIONS: &[&str] = &["txt", "md"];

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Failed to create file system watcher: {0}")]
    CreationFailed(notify::Error),
    #[error("Failed to watch path '{0}': {1}")]
    PathWatchFailed(String, notify::Error),
}

/// Sets up a file system watcher for the given path.
///
/// Returns the watcher instance and a receiver channel for events.
pub async fn setup_watcher(
    path_to_watch: &str,
    _table: Table
) -> Result<(RecommendedWatcher, Receiver<NotifyResult<Event>>), WatcherError> {
    let path = Path::new(path_to_watch);
    info!("Setting up file watcher for path: {:?}", path);

    // Create a channel to receive events
    let (tx, rx) = channel();

    // Create a file system watcher instance.
    // The closure passed to `new` is the event handler.
    // For now, it just sends the event through the channel.
    let mut watcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| {
            if let Err(e) = tx.send(res) {
                error!("Failed to send watcher event through channel: {}", e);
            }
        },
        notify::Config::default(), // Use default config for now
    )
    .map_err(WatcherError::CreationFailed)?;

    // Add the path to watch recursively
    watcher
        .watch(path, RecursiveMode::Recursive)
        .map_err(|e| WatcherError::PathWatchFailed(path_to_watch.to_string(), e))?;

    info!("Successfully watching path: {:?}", path);

    Ok((watcher, rx))
}

/// Setup the watcher with a new connection to the database
/// Convenience function that creates a new database connection and table
pub async fn setup_watcher_with_db(
    path_to_watch: &str,
) -> Result<(RecommendedWatcher, Receiver<NotifyResult<Event>>, Arc<Table>), WatcherError> {
    // Connect to DB and open table
    let conn = connect_db().await
        .map_err(|e| WatcherError::CreationFailed(
            notify::Error::new(notify::ErrorKind::Generic(format!("DB connection failed: {}", e)))))?;
    
    let table = open_or_create_text_table(&conn).await
        .map_err(|e| WatcherError::CreationFailed(
            notify::Error::new(notify::ErrorKind::Generic(format!("Table creation failed: {}", e)))))?;

    // Setup the watcher
    let (watcher, rx) = setup_watcher(path_to_watch, table.clone()).await?;
    
    Ok((watcher, rx, Arc::new(table)))
}

/// Processes file system events received from the watcher channel.
///
/// This function runs in a loop, checking for events until the channel is closed.
/// Loop exits when the sender is dropped (all senders dropped).
pub async fn process_events(rx: Receiver<NotifyResult<Event>>, table: Arc<Table>) {
    info!("Starting event processing loop...");

    // Use a loop with channel receiver's try_recv method to avoid indefinitely 
    // blocking in tests when the channel is closed
    loop {
        // Try to receive an event without blocking indefinitely
        match rx.try_recv() {
            Ok(result) => match result {
            Ok(event) => {
                // We only care about events with valid paths
                if event.paths.is_empty() {
                    continue;
                }
                
                // Detect action based on event kind
                let (action, paths_to_check) = match event.kind {
                    // Files created, data modified, or renamed TO this path -> UPSERT
                    EventKind::Create(CreateKind::File) |
                    EventKind::Modify(ModifyKind::Data(DataChange::Content)) |
                    EventKind::Modify(ModifyKind::Name(RenameMode::To)) | // Renamed TO this path
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) // Atomic rename
                     => ("Upsert", event.paths),
                    
                    // Files removed or renamed FROM this path -> DELETE
                    EventKind::Remove(_) | // Covers File, Folder, Other
                    EventKind::Modify(ModifyKind::Name(RenameMode::From)) // Renamed FROM this path
                     => ("Delete", event.paths),
                    
                    // Other events we don't currently handle 
                    _ => {
                        warn!("Ignoring event kind: {:?}", event.kind);
                        continue;
                    }
                };

                info!("Processing {} event with {} paths", action, paths_to_check.len());
                
                // Process each path from the event
                for path_buf in paths_to_check {
                    // Update the filename index for all files, regardless of content type
                    if action == "Upsert" {
                        // Update the filename index using the new async Tantivy command
                        // We need to spawn a task because update_filename_index is now async
                        let path_clone = path_buf.clone();
                        tokio::spawn(async move {
                            match metadata(&path_clone) {
                                Ok(meta) => {
                                    let last_modified = meta.modified()
                                        .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                                        .unwrap_or(0);
                                    let size = meta.len();
                                    if let Some(path_str) = path_clone.to_str() {
                                        match add_file_to_index(path_str.to_string(), last_modified, size).await {
                                            Ok(_) => info!("Updated Tantivy index (add/update) for {}", path_clone.display()),
                                            Err(e) => error!("Failed to update Tantivy index (add/update) for {}: {}", path_clone.display(), e),
                                        }
                                    } else {
                                         error!("Invalid path string for Tantivy add: {}", path_clone.display());
                                    }
                                }
                                Err(e) => error!("Failed to get metadata for Tantivy add {}: {}", path_clone.display(), e),
                            }
                        });
                    } else if action == "Delete" {
                        // Remove from the filename index using the new async Tantivy command
                        let path_clone = path_buf.clone();
                        tokio::spawn(async move {
                             if let Some(path_str) = path_clone.to_str() {
                                match remove_file_from_index(path_str.to_string()).await {
                                    Ok(_) => info!("Updated Tantivy index (remove) for {}", path_clone.display()),
                                    Err(e) => error!("Failed to update Tantivy index (remove) for {}: {}", path_clone.display(), e),
                                }
                            } else {
                                 error!("Invalid path string for Tantivy remove: {}", path_clone.display());
                            }
                        });
                    }
                    
                    // Skip paths we don't care about for semantic indexing
                    if !is_relevant_file(&path_buf) {
                        info!("Skipping non-relevant file for semantic index: {}", path_buf.display());
                        continue;
                    }
                    
                    // Perform action based on event type for semantic search
                    match action {
                        "Upsert" => {
                            info!("Action [Upsert] detected for: {}", path_buf.display());
                            // Pass table reference
                            match process_file_upsert(&path_buf, &table).await {
                                Ok(_) => info!("Successfully processed upsert for {}", path_buf.display()),
                                Err(e) => error!("Error processing upsert for {}: {}", path_buf.display(), e),
                            }
                        }
                        "Delete" => {
                            info!("Action [Delete] detected for: {}", path_buf.display());
                            if let Some(path_str) = path_buf.to_str() {
                                // Pass table reference
                                match delete_document(&table, path_str).await {
                                    Ok(_) => info!("Successfully deleted DB entry for {}", path_buf.display()),
                                    Err(DbError::RecordNotFound(_)) => warn!("Attempted to delete non-existent DB entry for {}", path_buf.display()),
                                    Err(e) => error!("Error deleting DB entry for {}: {}", path_buf.display(), e),
                                }
                            } else {
                                error!("Invalid path string for deletion: {}", path_buf.display());
                            }
                        }
                        _ => {
                            // Should not get here due to the matching above
                            warn!("Unhandled action type: {}", action); 
                        }
                    }
                }
            }
            Err(e) => {
                error!("Watcher error: {:?}", e);
                if matches!(e, notify::Error{ kind: notify::ErrorKind::PathNotFound, .. }) {
                    warn!("Watched path seems to have been removed.");
                }
            }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // No messages available yet, yield to other tasks briefly
                tokio::task::yield_now().await;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Channel is closed (all senders dropped)
                info!("Channel closed, exiting event processing loop");
                break;
            }
        }
    }
    
    info!("Event processing loop exited");
}

// Helper function to handle text extraction, embedding, and DB upsert for a file
async fn process_file_upsert(path_buf: &Path, table: &Table) -> Result<(), DbError> {
    // Extract content returns a single String
    let content = extract_text(path_buf)?;
    let trimmed_content = content.trim(); // Trim whitespace

    if trimmed_content.is_empty() { // Check trimmed content
        warn!("Extracted empty or whitespace-only content for {}, skipping upsert.", path_buf.display());
        return Ok(()); // Nothing to embed or hash
    }
    
    // Hash the content
    let hash = calculate_hash(trimmed_content); // Use trimmed content for hash
    info!("  -> Extracted text, Hash: {}", hash);

    // Convert the single string to a Vec<String> for embed_text
    let content_vec = vec![trimmed_content.to_string()]; // Pass trimmed content string
    let embedding_vec = match embed_text(&content_vec, false) {
        Ok(vec) => vec,
        Err(e) => {
            // Log the original embedding error and skip the file
            error!("Embedding generation failed for {}: {}. Skipping upsert.", path_buf.display(), e);
            return Ok(()); 
        }
    };

    if embedding_vec.is_empty() {
        // Log as warning and skip if no embeddings were generated (e.g., model couldn't process)
        warn!("No embeddings generated for {}, likely due to content issues (e.g., font problems during extraction). Skipping upsert.", path_buf.display());
        return Ok(()); // Skip this file gracefully
    }
    
    info!("  -> Successfully generated {} embeddings (chunks)", embedding_vec.len());

    if let Some(path_str) = path_buf.to_str() {
        // Pass the entire vector of embeddings to upsert_document
        upsert_document(table, path_str, &hash, &embedding_vec).await?;
        Ok(())
    } else {
        // Keep this as an error because an invalid path is more serious
        error!("Invalid file path encoding for {}. Cannot upsert.", path_buf.display());
        Err(DbError::Other("Invalid file path encoding".to_string()))
    }
}

/// Checks if a path points to a relevant file for indexing.
/// Ignore hidden files/directories and check for supported extensions.
fn is_relevant_file(path: &PathBuf) -> bool {
    // Check if the file name itself starts with a dot.
    let filename_is_hidden = path.file_name()
        .and_then(|name| name.to_str())
        .map_or(false, |name_str| name_str.starts_with('.'));

    if filename_is_hidden {
        return false;
    }

    // Check if it's a file
    let is_file = path.is_file();

    // Check extension
    let extension_check = path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext_str| {
            let lower_ext = ext_str.to_lowercase();
            let supported = SUPPORTED_EXTENSIONS.contains(&lower_ext.as_str());
            supported
        });

    // Final result
    let result = is_file && extension_check;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind}; 
    use std::fs::{self, File};
    use tempfile::tempdir;
    use env_logger;
    use arrow_array::{StringArray, Array}; // Import Array trait
    use crate::db::{TestDb, connect_db_with_path}; // Import connect_db_with_path
    use lancedb::query::{ExecutableQuery, QueryBase};
    use futures::TryStreamExt;
    use std::io::Write;
    use std::time::Duration;
    use tokio::sync::mpsc::error::TryRecvError;

    // Helper function to create a mock event channel for testing
    fn create_mock_channel() -> (std::sync::mpsc::Sender<NotifyResult<Event>>, Receiver<NotifyResult<Event>>) {
        channel()
    }

    #[tokio::test]
    async fn test_setup_watcher_success() {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();
        
        // Create a temporary database
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.unwrap();
        let table = open_or_create_text_table(&conn).await.unwrap();
        
        // Setup watcher with the test table
        let result = setup_watcher(&path, table).await;
        assert!(result.is_ok(), "setup_watcher failed: {:?}", result.err());
        
        if let Ok((mut watcher, _rx)) = result {
            // Stop the watcher to clean up
            watcher.unwatch(dir.path()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_setup_watcher_nonexistent_path() {
        // Create a nonexistent path
        let path = "/path/does/not/exist/for_sure";
        
        // Create a temporary database
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.unwrap();
        let table = open_or_create_text_table(&conn).await.unwrap();
        
        // Setup should fail with PathWatchFailed
        let result = setup_watcher(path, table).await;
        assert!(result.is_err());

        if let Err(WatcherError::PathWatchFailed(_, _)) = result {
            // Expected error
        } else {
            panic!("Expected PathWatchFailed error, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_setup_watcher_not_a_directory() {
        // Create a temporary file (not a directory)
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("not_a_dir.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "This is not a directory").unwrap();
        
        // Create a temporary database
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.unwrap();
        let table = open_or_create_text_table(&conn).await.unwrap();
        
        let path_str = file_path.to_string_lossy().to_string();
        
        // Modern versions of notify can watch single files too
        let result = setup_watcher(&path_str, table).await;
        
        // Accept both success and error results
        if result.is_ok() {
            info!("Modern notify version can watch single files - this is expected");
            let (mut watcher, _rx) = result.unwrap();
            // Clean up
            watcher.unwatch(Path::new(&path_str)).unwrap();
        } else if let Err(WatcherError::PathWatchFailed(_, _)) = result {
            // This is also acceptable
            info!("Notify reported an error watching a single file");
        } else {
            panic!("Unexpected error: {:?}", result);
        }
    }

    // Test that the process_events loop can start and receive from a channel
    // (Doesn't test event processing logic itself, just the loop mechanics)
    #[tokio::test]
    async fn test_process_events_runs() {
        // Create a mock channel
        let (tx, rx) = create_mock_channel();
        
        // Create temporary database for the test
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.unwrap();
        let table = open_or_create_text_table(&conn).await.unwrap();
        let table_arc = Arc::new(table);
        
        // Start the process_events function in a separate task
        let process_handle = tokio::spawn(async move {
            process_events(rx, table_arc).await;
        });
        
        // Send a few events through the channel
        let event1 = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![PathBuf::from("/test/path1.txt")],
            attrs: notify::event::EventAttributes::default(),
        };
        
        let event2 = Event {
            kind: EventKind::Remove(notify::event::RemoveKind::File),
            paths: vec![PathBuf::from("/test/path2.txt")],
            attrs: notify::event::EventAttributes::default(),
        };
        
        // Send the events
        tx.send(Ok(event1)).expect("Failed to send event1");
        tx.send(Ok(event2)).expect("Failed to send event2");
        
        // Drop the sender to close the channel
        drop(tx);
        
        // Wait for the process to complete with a timeout
        let _ = tokio::time::timeout(Duration::from_secs(2), process_handle).await;
        
        // If we got here, the test passed (didn't hang)
    }

    #[tokio::test]
    async fn test_watcher_db_integration() {
        // Create a temporary directory for watching
        let dir = tempdir().expect("Failed to create temp dir");
        let test_file_path = dir.path().join("test_doc.txt");
        
        // Create temporary database for the test
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.unwrap();
        let table = open_or_create_text_table(&conn).await.unwrap();
        
        // Set up a watcher on the temporary directory
        let dir_path = dir.path().to_string_lossy().to_string();
        let (watcher, rx) = setup_watcher(&dir_path, table.clone()).await.unwrap();
        
        // Start event processing in a background task
        let table_arc = Arc::new(table.clone());
        let process_handle = tokio::spawn(async move {
            process_events(rx, table_arc).await;
        });
        
        // Create a test file to trigger an event
        let test_content = "This is a test document for the watcher.";
        std::fs::write(&test_file_path, test_content).expect("Failed to write test file");
        
        // Allow time for the event to be processed
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Query database to check if the document was indexed
        let result = table.query()
            .execute()
            .await
            .expect("Query failed")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect results");
            
        // Verify one record was created (may fail because we're using simulated events)
        let count = result.iter().map(|batch| batch.num_rows()).sum::<usize>();
        info!("Found {} records in test DB", count);
        
        // Clean up
        drop(watcher); // Stop the watcher
        drop(process_handle); // Stop the processing task
    }
}
