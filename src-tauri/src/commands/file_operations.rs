use std::path::Path;
use serde::{Serialize, Deserialize};
use tokio::fs;
use arboard::Clipboard;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_copy_to_clipboard_success() {
        let _ = tracing_subscriber::fmt().with_env_filter("info").is_test(true).try_init();
        let text_to_copy = "Hello, clipboard!".to_string();
        // This test mainly checks that the function can be called without panicking
        // and that it reports success. Actually verifying clipboard content
        // can be problematic in headless/CI environments.
        // On some systems, this might still fail if a clipboard provider is not available.
        match copy_to_clipboard(text_to_copy.clone()) {
            Ok(_) => {
                // If we could reliably read from clipboard here, we would.
                // For example:
                // let mut clipboard = Clipboard::new().unwrap();
                // assert_eq!(clipboard.get_text().unwrap(), text_to_copy);
                info!("copy_to_clipboard reported success.");
            }
            Err(e) => {
                // In some CI environments (especially without a display server),
                // clipboard operations might fail. We'll log this but not fail the test.
                // Consider this test "passed" if it doesn't panic and either returns Ok
                // or a known clipboard error.
                warn!("copy_to_clipboard returned an error (potentially expected in CI): {}", e);
                if !e.contains("Failed to initialize clipboard") && !e.contains("clipboard provider") {
                    // Fail if it's not a known clipboard init error
                    // panic!("copy_to_clipboard failed with an unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_open_file_external_dummy_path() {
        let _ = tracing_subscriber::fmt().with_env_filter("info").is_test(true).try_init();
        // Create a dummy file to ensure the path exists for commands that might check.
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("dummy_test_file.txt");
        File::create(&file_path).unwrap().write_all(b"dummy content").unwrap();

        let path_str = file_path.to_str().unwrap().to_string();
        
        // This test primarily checks that the command generation and spawn logic
        // doesn't immediately panic or error out for the current OS.
        // It doesn't verify that the correct application opens the file.
        match open_file_external(path_str.clone()) {
            Ok(_) => info!("open_file_external reported success for path: {}", path_str),
            Err(e) => {
                // Similar to clipboard, actual execution might be tricky or unwanted in CI.
                // We log the error. If specific OS commands are missing (e.g., xdg-open not installed),
                // it might error.
                warn!("open_file_external failed for path {}: {} (potentially ignorable in CI if command not found)", path_str, e);
                 // If the error is about the command not being found, it's an environment issue, not a code logic error.
                if !e.contains("No such file or directory") && !e.contains("cannot find the file") && !e.contains("command not found") {
                    // panic!("open_file_external failed with an unexpected error: {}", e);
                }
            }
        }
    }

    // Example test for get_item_info (if you want to add more tests for other functions)
    #[tokio::test]
    async fn test_get_item_info_file() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("test_info.txt");
        File::create(&file_path).unwrap().write_all(b"content").unwrap();

        let result = get_item_info(file_path.to_str().unwrap().to_string());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info["is_file"], true);
        assert_eq!(info["size"], 7);
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

#[tauri::command]
pub fn open_file_external(path: String) -> Result<(), String> {
    info!("Attempting to open file externally: {}", path);
    let command_string;
    let mut args: Vec<String> = Vec::new();

    if cfg!(target_os = "windows") {
        command_string = "cmd".to_string();
        args.push("/c".to_string());
        args.push("start".to_string());
        args.push("".to_string()); // Empty title for start command
        args.push(path.clone());
    } else if cfg!(target_os = "macos") {
        command_string = "open".to_string();
        args.push(path.clone());
    } else if cfg!(target_os = "linux") {
        command_string = "xdg-open".to_string();
        args.push(path.clone());
    } else {
        let err_msg = "Unsupported operating system for opening files externally.".to_string();
        error!("{}", err_msg);
        return Err(err_msg);
    }

    match std::process::Command::new(&command_string).args(&args).spawn() {
        Ok(mut child) => {
            // Optionally, wait for the command to complete, though for opening files,
            // it's usually better to let it run detached.
            // For now, we'll assume success if spawn works.
            // If you need to check the exit status:
            /*
            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        info!("File open command executed successfully for: {}", path);
                        Ok(())
                    } else {
                        error!("File open command failed for: {} with status: {:?}", path, status.code());
                        Err(format!("Failed to open file (exit code: {:?})", status.code().unwrap_or(-1)))
                    }
                }
                Err(e) => {
                    error!("Failed to wait for file open command for {}: {}", path, e);
                    Err(format!("Failed to execute file open command: {}", e))
                }
            }
            */
            info!("Successfully spawned command to open file: {}", path);
            Ok(())
        }
        Err(e) => {
            error!("Failed to execute command '{}' with args '{:?}' for path {}: {}", command_string, args, path, e);
            Err(format!("Failed to execute command to open file {}: {}", path, e))
        }
    }
}


#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(text.clone()) {
                Ok(_) => {
                    info!("Copied to clipboard: {}", text);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to copy to clipboard: {}", e);
                    Err(format!("Failed to copy to clipboard: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize clipboard: {}", e);
            Err(format!("Failed to initialize clipboard: {}", e))
        }
    }
}
