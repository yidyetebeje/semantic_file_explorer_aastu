use serde::Serialize;
use std::io;
use thiserror::Error;

// Define a serializable error type suitable for Tauri command results
#[derive(Debug, Error, Serialize)]
pub enum FileSystemError {
    #[error("Path not found: {path}")]
    NotFound { path: String },

    #[error("Permission denied for path: {path}")]
    PermissionDenied { path: String },

    #[error("Path is not a directory: {path}")]
    NotADirectory { path: String },

    #[error("Failed to read directory entries: {path}")]
    ReadDirError { path: String },

    #[error("Failed to retrieve file metadata: {path}")]
    MetadataError { path: String },

    #[error("Invalid path encoding: {path}")]
    InvalidPathEncoding { path: String },

    #[error("I/O error accessing path {path}: {kind}")]
    IoError { path: String, kind: String }, // Store IO error kind as string
}

// Helper to convert std::io::Error to our custom error, capturing the path context
pub(crate) fn map_io_error(e: io::Error, path: &str) -> FileSystemError {
    match e.kind() {
        io::ErrorKind::NotFound => FileSystemError::NotFound {
            path: path.to_string(),
        },
        io::ErrorKind::PermissionDenied => FileSystemError::PermissionDenied {
            path: path.to_string(),
        },
        // Add other specific mappings if needed
        _ => FileSystemError::IoError {
            path: path.to_string(),
            kind: e.kind().to_string(),
        },
    }
}

// Define WatcherError
#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("Failed to create file watcher: {0}")]
    CreationFailed(#[from] notify::Error),
    #[error("Failed to watch path '{path}': {source}")]
    WatchPathError {
        path: String,
        #[source]
        source: notify::Error,
    },
    // Add other watcher-specific errors if needed
}

// Define CoreError enum wrapping other errors (DbError, WatcherError, etc.)
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Database error: {0}")]
    DbError(#[from] crate::db::DbError), // Assuming db.rs is sibling module

    #[error("File system error: {0}")]
    FileSystemError(#[from] FileSystemError), // Reuse existing FileSystemError

    #[error("Watcher error: {0}")]
    WatcherError(#[from] WatcherError),

    #[error("Text extraction error: {0}")]
    ExtractorError(#[from] crate::extractor::ExtractorError), // Assuming extractor.rs exists

    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] crate::embedder::EmbeddingError), // Assuming embedder.rs exists

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("An unexpected error occurred: {0}")]
    Other(String),
}

// Implement From<io::Error> for CoreError to handle basic IO errors centrally if needed
// Example (adjust based on desired context capture):
impl From<io::Error> for CoreError {
    fn from(e: io::Error) -> Self {
        // Decide how to classify generic IO errors
        CoreError::Other(format!("IO Error: {}", e))
        // Or potentially map to FileSystemError if path context is available,
        // but that usually requires more info than just the io::Error.
    }
}
