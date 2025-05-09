// src-tauri/src/extractor.rs

use std::fs;
use std::path::Path;
use extractous::Extractor;
use thiserror::Error;
use log::{error, debug, info, warn};
use sha2::{Sha256, Digest};
use pdf_extract::extract_text as pdf_extract_text;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum ExtractorError {
    #[error("IO Error reading file {0}: {1}")]
    IoError(String, #[source] std::io::Error),
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    #[error("PDF extraction failed for {0}: {1}")]
    PdfExtractionFailed(String, String),
    #[error("Image file handling: {0}")]
    ImageHandling(String),
    // TODO: Add specific errors for PDF parsing if needed
}

/// Content type enum to distinguish between different file types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Unsupported,
}

/// Lists of supported file extensions
pub const SUPPORTED_TEXT_EXTENSIONS: &[&str] = &["pdf"];
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

/// Determines the content type of a file based on its extension
pub fn get_content_type(file_path: &Path) -> ContentType {
    match file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
    {
        Some(ext) if SUPPORTED_TEXT_EXTENSIONS.contains(&ext.as_str()) => ContentType::Text,
        Some(ext) if SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.as_str()) => ContentType::Image,
        _ => ContentType::Unsupported,
    }
}

/// Extracts text content from a supported file.
///
/// Currently supports `.txt`, `.md` and `.pdf` files.
///
/// # Arguments
///
/// * `file_path` - The path to the file.
///
/// # Returns
///
/// * `Ok(String)` containing the extracted text content.
/// * `Err(ExtractorError)` if the file is unsupported or cannot be read.
pub fn extract_text(file_path: &Path) -> Result<String, ExtractorError> {
    debug!("Attempting to extract text from: {}", file_path.display());

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("pdf") => {
            info!("Extracting text from PDF: {}", file_path.display());
            let extractor = Extractor::new();
            // extract file with extractor
            let (content, _metadata) = extractor.extract_file_to_string(file_path.to_str().unwrap()).unwrap();
            println!("Extracted text from PDF: {}", content);
            const MAX_TEXT_LENGTH: usize = 100000; // ~100KB limit
            if content.len() > MAX_TEXT_LENGTH {
                warn!("PDF text too large ({}), truncating to {} chars", content.len(), MAX_TEXT_LENGTH);
                Ok(content[0..MAX_TEXT_LENGTH].to_string())
            } else {
                Ok(content)
            }
        },
        Some("txt") | Some("md") => {
            let ext_str = extension.as_ref().unwrap();
            info!("Extracting text from {}: {}", ext_str, file_path.display());
            
            // Simple file read for text files
            std::fs::read_to_string(file_path).map_err(|e| {
                error!("Failed to read {} file {}: {}", ext_str, file_path.display(), e);
                ExtractorError::IoError(file_path.display().to_string(), e)
            })
        },
        Some(ext) => {
            error!("Unsupported file type attempted: {}", ext);
            Err(ExtractorError::UnsupportedFileType(ext.to_string()))
        }
        None => {
             error!("File has no extension: {}", file_path.display());
             Err(ExtractorError::UnsupportedFileType("No extension".to_string()))
        }
    }
}

/// Handles an image file by validating it exists and returning its path as a string
///
/// # Arguments
///
/// * `file_path` - The path to the image file
///
/// # Returns
///
/// * `Ok(String)` - The validated path to the image file
/// * `Err(ExtractorError)` - If the file is not a valid image or doesn't exist
pub fn process_image(file_path: &Path) -> Result<String, ExtractorError> {
    debug!("Processing image file: {}", file_path.display());
    
    // Check if file exists
    if !file_path.exists() {
        error!("Image file does not exist: {}", file_path.display());
        return Err(ExtractorError::IoError(
            file_path.display().to_string(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        ));
    }
    
    // Validate the extension
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());
        
    match extension.as_deref() {
        Some(ext) if SUPPORTED_IMAGE_EXTENSIONS.contains(&ext) => {
            // File exists and has valid extension - return absolute path
            match file_path.canonicalize() {
                Ok(abs_path) => Ok(abs_path.to_string_lossy().to_string()),
                Err(e) => {
                    error!("Failed to get absolute path for {}: {}", file_path.display(), e);
                    Err(ExtractorError::IoError(file_path.display().to_string(), e))
                }
            }
        },
        Some(ext) => {
            error!("Not a supported image type: {}", ext);
            Err(ExtractorError::UnsupportedFileType(ext.to_string()))
        },
        None => {
            error!("Image file has no extension: {}", file_path.display());
            Err(ExtractorError::UnsupportedFileType("No extension".to_string()))
        }
    }
}

/// Calculates the SHA256 hash of the given content.
///
/// # Arguments
///
/// * `content` - The string content to hash.
///
/// # Returns
///
/// * A hex-encoded string representing the SHA256 hash.
pub fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // Format the hash bytes as a hex string
    format!("{:x}", result)
}

/// Calculates the SHA256 hash of a file
///
/// # Arguments
///
/// * `file_path` - The path to the file
///
/// # Returns
///
/// * `Ok(String)` - A hex-encoded string representing the SHA256 hash
/// * `Err(ExtractorError)` - If the file cannot be read
pub fn calculate_file_hash(file_path: &Path) -> Result<String, ExtractorError> {
    // Read the file in binary mode
    let file_content = fs::read(file_path).map_err(|e| {
        error!("Failed to read file for hashing {}: {}", file_path.display(), e);
        ExtractorError::IoError(file_path.display().to_string(), e)
    })?;
    
    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let result = hasher.finalize();
    
    // Format as hex string
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from parent module
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_extract_txt_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        let content = "This is a test text file.";
        writeln!(file, "{}", content).unwrap();

        let extracted_text = extract_text(&file_path).unwrap();
        assert_eq!(extracted_text.trim(), content);
    }

     #[test]
    fn test_extract_md_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        let mut file = fs::File::create(&file_path).unwrap();
        let content = "# Markdown Header\n\nThis is markdown content.";
        writeln!(file, "{}", content).unwrap();

        let extracted_text = extract_text(&file_path).unwrap();
        assert_eq!(extracted_text.trim(), content);
    }

    #[test]
    fn test_extract_unsupported_type() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.jpg");
        let _ = fs::File::create(&file_path).unwrap(); // Create dummy file

        let result = extract_text(&file_path);
        assert!(matches!(result, Err(ExtractorError::UnsupportedFileType(ext)) if ext == "jpg"));
    }

    #[test]
    fn test_extract_no_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test");
         let _ = fs::File::create(&file_path).unwrap(); // Create dummy file

        let result = extract_text(&file_path);
        assert!(matches!(result, Err(ExtractorError::UnsupportedFileType(ext)) if ext == "No extension"));
    }

    #[test]
    fn test_extract_non_existent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent.txt");

        let result = extract_text(&file_path);
        assert!(matches!(result, Err(ExtractorError::IoError(_, _))));
    }

    #[test]
    fn test_calculate_hash() {
        let content1 = "Hello, world!";
        let content2 = "Hello, world!";
        let content3 = "Hello, Rust!";

        let hash1 = calculate_hash(content1);
        let hash2 = calculate_hash(content2);
        let hash3 = calculate_hash(content3);

        // Check that identical content produces the same hash
        assert_eq!(hash1, hash2);
        // Check that different content produces different hashes
        assert_ne!(hash1, hash3);

        // Check against a known SHA256 hash value for "Hello, world!"
        // You can verify this using online tools or `shasum -a 256`
        let expected_hash = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        assert_eq!(hash1, expected_hash);
    }
    
    #[test]
    fn test_process_image_valid_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.jpg");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "Mock image data").unwrap();
        
        let result = process_image(&file_path);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_process_image_nonexistent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.jpg");
        
        let result = process_image(&file_path);
        assert!(matches!(result, Err(ExtractorError::IoError(_, _))));
    }
    
    #[test]
    fn test_process_image_unsupported_type() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.xyz");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "Mock file data").unwrap();
        
        let result = process_image(&file_path);
        assert!(matches!(result, Err(ExtractorError::UnsupportedFileType(_))));
    }
    
    #[test]
    fn test_get_content_type() {
        // Text files
        assert_eq!(get_content_type(Path::new("document.pdf")), ContentType::Text);
        assert_eq!(get_content_type(Path::new("notes.txt")), ContentType::Text);
        assert_eq!(get_content_type(Path::new("readme.md")), ContentType::Text);
        
        // Image files
        assert_eq!(get_content_type(Path::new("photo.jpg")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("image.jpeg")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("icon.png")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("animation.gif")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("photo.webp")), ContentType::Image);
        assert_eq!(get_content_type(Path::new("screenshot.bmp")), ContentType::Image);
        
        // Unsupported files
        assert_eq!(get_content_type(Path::new("archive.zip")), ContentType::Unsupported);
        assert_eq!(get_content_type(Path::new("unknown")), ContentType::Unsupported);
    }
    
    #[test]
    fn test_calculate_file_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        // Create a file with known content
        let content = "Hello, world!";
        let mut file = fs::File::create(&file_path).unwrap();
        write!(file, "{}", content).unwrap();
        
        // Calculate hash from file
        let file_hash = calculate_file_hash(&file_path).unwrap();
        
        // Calculate hash directly from content for comparison
        let content_hash = calculate_hash(content);
        
        // Hashes should match
        assert_eq!(file_hash, content_hash);
    }
}
