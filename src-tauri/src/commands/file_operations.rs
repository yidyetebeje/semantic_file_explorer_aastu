use std::path::Path;
use serde::{Serialize, Deserialize};
use tokio::fs;
use std::fs::metadata;
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum FileOperationError {
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("File already exists: {0}")]
    AlreadyExists(String),
    
    #[error("I/O error: {0}")]
    IoError(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Source and destination are the same: {0}")]
    SameSourceAndDestination(String),
    
    #[error("Cannot move/copy into subfolder of itself: {0}")]
    RecursiveOperation(String),
}

/// Converts a generic I/O error into a FileOperationError
fn io_to_error(error: std::io::Error, path: &str) -> FileOperationError {
    match error.kind() {
        std::io::ErrorKind::NotFound => FileOperationError::NotFound(path.to_string()),
        std::io::ErrorKind::PermissionDenied => FileOperationError::PermissionDenied(path.to_string()),
        std::io::ErrorKind::AlreadyExists => FileOperationError::AlreadyExists(path.to_string()),
        _ => FileOperationError::IoError(error.to_string()),
    }
}

/// Check if source is parent of destination (to prevent recursive operations)
fn is_parent_of(source: &Path, destination: &Path) -> bool {
    if let Ok(relative) = destination.strip_prefix(source) {
        !relative.as_os_str().is_empty()
    } else {
        false
    }
}

/// Helper function to copy a single file
async fn copy_file(src: &Path, dst: &Path) -> Result<(), FileOperationError> {
    // Make sure parent directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).await
            .map_err(|e| io_to_error(e, parent.to_str().unwrap_or("")))?;
    }
    
    // Copy file
    fs::copy(src, dst).await
        .map_err(|e| io_to_error(e, dst.to_str().unwrap_or("")))?;
    
    Ok(())
}

/// Helper function to copy a directory without recursion
async fn copy_directory(src_dir: &Path, dst_dir: &Path) -> Result<(), FileOperationError> {
    // Create the target directory
    fs::create_dir_all(dst_dir).await
        .map_err(|e| io_to_error(e, dst_dir.to_str().unwrap_or("")))?;
    
    // Collect all files and directories first to avoid recursion issues
    let mut dirs_to_process = vec![(src_dir.to_path_buf(), dst_dir.to_path_buf())];
    
    // Process each directory and its contents
    while let Some((src, dst)) = dirs_to_process.pop() {
        // Read directory entries
        let mut read_dir = fs::read_dir(&src).await
            .map_err(|e| io_to_error(e, src.to_str().unwrap_or("")))?;
        
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(file_name);
            
            // Handle directory or file
            if entry_path.is_dir() {
                // Create the directory
                fs::create_dir_all(&dst_path).await
                    .map_err(|e| io_to_error(e, dst_path.to_str().unwrap_or("")))?;
                
                // Add to processing queue
                dirs_to_process.push((entry_path, dst_path));
            } else {
                // Copy the file
                copy_file(&entry_path, &dst_path).await?;
            }
        }
    }
    
    Ok(())
}

/// Copy a file or directory to a new location
#[tauri::command]
pub async fn copy_item(source: String, destination: String) -> Result<(), FileOperationError> {
    info!("Copying from '{}' to '{}'", source, destination);
    
    let source_path = Path::new(&source);
    let destination_path = Path::new(&destination);
    
    // Validate input
    if !source_path.exists() {
        return Err(FileOperationError::NotFound(source));
    }
    
    if source_path == destination_path {
        return Err(FileOperationError::SameSourceAndDestination(source));
    }
    
    // Check if trying to copy into subfolder of itself
    if source_path.is_dir() && is_parent_of(source_path, destination_path) {
        return Err(FileOperationError::RecursiveOperation(source));
    }
    
    // Perform the copy based on whether it's a file or directory
    if source_path.is_dir() {
        copy_directory(source_path, destination_path).await?
    } else {
        copy_file(source_path, destination_path).await?
    }
    
    Ok(())
}


/// Move a file or directory to a new location
#[tauri::command]
pub async fn move_item(source: String, destination: String) -> Result<(), FileOperationError> {
    info!("Moving from '{}' to '{}'", source, destination);
    
    let source_path = Path::new(&source);
    let destination_path = Path::new(&destination);
    
    // Validate input
    if !source_path.exists() {
        return Err(FileOperationError::NotFound(source));
    }
    
    if source_path == destination_path {
        return Err(FileOperationError::SameSourceAndDestination(source));
    }
    
    // Check if trying to move into subfolder of itself
    if source_path.is_dir() && is_parent_of(source_path, destination_path) {
        return Err(FileOperationError::RecursiveOperation(source));
    }
    
    // Make sure parent directory exists
    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent).await
            .map_err(|e| io_to_error(e, parent.to_str().unwrap_or("")))?;
    }
    
    // Perform the move operation
    fs::rename(source_path, destination_path).await
        .map_err(|e| io_to_error(e, &destination))?;
    
    Ok(())
}

/// Delete a file or directory
#[tauri::command]
pub async fn delete_item(path: String) -> Result<(), FileOperationError> {
    info!("Deleting '{}'", path);
    
    let path_obj = Path::new(&path);
    
    // Validate input
    if !path_obj.exists() {
        return Err(FileOperationError::NotFound(path));
    }
    
    // Perform the delete operation
    if path_obj.is_dir() {
        fs::remove_dir_all(path_obj).await
            .map_err(|e| io_to_error(e, &path))?;
    } else {
        fs::remove_file(path_obj).await
            .map_err(|e| io_to_error(e, &path))?;
    }
    
    Ok(())
}

/// Rename a file or directory
#[tauri::command]
pub async fn rename_item(path: String, new_name: String) -> Result<(), FileOperationError> {
    info!("Renaming '{}' to '{}'", path, new_name);
    
    let path_obj = Path::new(&path);
    
    // Validate input
    if !path_obj.exists() {
        return Err(FileOperationError::NotFound(path.clone()));
    }
    
    // Calculate the new path
    let parent = path_obj.parent().ok_or_else(|| 
        FileOperationError::InvalidPath(format!("Cannot determine parent directory of {}", path))
    )?;
    
    let new_path = parent.join(new_name);
    
    // Check if the destination already exists
    if new_path.exists() {
        return Err(FileOperationError::AlreadyExists(new_path.to_string_lossy().to_string()));
    }
    
    // Perform the rename operation
    fs::rename(path_obj, &new_path).await
        .map_err(|e| io_to_error(e, &path))?;
    
    Ok(())
}

/// Create a new directory
#[tauri::command]
pub async fn create_directory(path: String) -> Result<(), FileOperationError> {
    info!("Creating directory '{}'", path);
    
    let path_obj = Path::new(&path);
    
    // Check if the directory already exists
    if path_obj.exists() {
        return Err(FileOperationError::AlreadyExists(path));
    }
    
    // Create the directory
    fs::create_dir_all(path_obj).await
        .map_err(|e| io_to_error(e, &path))?;
    
    Ok(())
}

/// Get information about a file or directory
#[tauri::command]
pub fn get_item_info(path: String) -> Result<serde_json::Value, FileOperationError> {
    let path_obj = Path::new(&path);
    
    // Validate input
    if !path_obj.exists() {
        return Err(FileOperationError::NotFound(path));
    }
    
    let metadata = metadata(&path).map_err(|e| io_to_error(e, &path))?;
    
    let info = serde_json::json!({
        "path": path,
        "is_file": metadata.is_file(),
        "is_dir": metadata.is_dir(),
        "size": metadata.len(),
        "modified": metadata.modified().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())),
        "created": metadata.created().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())),
        "readonly": metadata.permissions().readonly(),
    });
    
    Ok(info)
}
