use crate::core::{
    error::FileSystemError,      // The error type
    file_system::list_directory, // Your core function
    models::FileInfo,            // The return data structure
};
use directories_next::UserDirs;
use hostname;
use std::{
    path::{Path, PathBuf},
};
use serde::{Serialize, Deserialize};
use tauri::Manager; // Needed for AppHandle
use tokio::fs::{read_to_string, write};
use tokio::io::ErrorKind;
use tauri::AppHandle;
use sha2::{Sha256, Digest};
use std::time::SystemTime;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashSet;
use image::{imageops::FilterType, DynamicImage, ImageFormat, ImageReader};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::Rescale;
 // For read_exact
use std::fs::File as StdFile; // Use std::fs::File for png crate decoder
use png;

// Keep track of paths currently being processed to avoid duplicate generation tasks
lazy_static! {
    static ref PROCESSING_THUMBNAILS: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
}

#[derive(Debug, serde::Serialize, thiserror::Error)]
pub enum ConfigError {
    #[error("Could not determine user home directory.")]
    HomeDirNotFound,
    #[error("Home directory path contains invalid UTF-8.")]
    InvalidPathEncoding,
}

#[derive(Debug, serde::Serialize, thiserror::Error)]
pub enum HostnameError {
    #[error("Failed to get hostname: {0}")]
    OsError(String),
}

// --- Custom Location Struct ---
#[derive(Serialize, Deserialize, Debug, Clone)] // Add Clone
pub struct CustomLocation {
    name: String,
    path: String,
}

// --- Error Types ---
#[derive(Debug, Serialize, thiserror::Error)]
pub enum LocationStorageError {
    #[error("Could not resolve app data directory: {0}")]
    AppDataDirError(String),
    #[error("Filesystem error: {0}")]
    IoError(String),
    #[error("Serialization/Deserialization error: {0}")]
    SerdeError(String),
}

// --- Helper Functions ---

// Gets the path to the storage file (e.g., app_data_dir/custom_locations.json)
async fn get_locations_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, LocationStorageError> {
    app_handle
        .path()
        .app_data_dir()
        // Map the error from app_data_dir() if it occurs
        .map_err(|e| LocationStorageError::AppDataDirError(format!("Failed to get app data dir: {}", e)))
        // Map the success case (PathBuf) to join the filename
        .map(|p| p.join("custom_locations.json"))
}

// Gets the path to the thumbnail cache directory
pub(crate) fn get_thumbnail_cache_dir(app_handle: &AppHandle) -> Result<PathBuf, LocationStorageError> {
    app_handle
        .path()
        .app_cache_dir()
        // Fix Result handling again
        .map_err(|e| LocationStorageError::AppDataDirError(format!("Failed to get app cache dir: {}", e)))
        .map(|p| p.join("thumbnails"))
}

// Creates a hash string from path and modified time
pub(crate) fn hash_path_and_mtime(path: &Path, modified: Option<SystemTime>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    if let Some(mtime) = modified {
        if let Ok(duration) = mtime.duration_since(SystemTime::UNIX_EPOCH) {
            hasher.update(duration.as_secs().to_le_bytes());
        }
    }
    format!("{:x}", hasher.finalize())
}

// Checks if a file type is potentially eligible for thumbnail generation
pub(crate) fn is_thumbnailable(file_type: &str) -> bool {
    matches!(file_type.to_lowercase().as_str(), 
        "image" | "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" |
        "video" | "mp4" | "mov" | "avi" | "mkv" | "webm"
    )
}

// --- Commands --- 

#[tauri::command]
pub async fn load_custom_locations(app_handle: tauri::AppHandle) -> Result<Vec<CustomLocation>, LocationStorageError> {
    let file_path = get_locations_file_path(&app_handle).await?;
    
    match read_to_string(&file_path).await {
        Ok(content) => {
            serde_json::from_str(&content).map_err(|e| LocationStorageError::SerdeError(e.to_string()))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Ok(Vec::new()) // Return empty list if file doesn't exist yet
        }
        Err(e) => Err(LocationStorageError::IoError(e.to_string())),
    }
}

#[tauri::command]
pub async fn save_custom_locations(
    locations: Vec<CustomLocation>,
    app_handle: tauri::AppHandle,
) -> Result<(), LocationStorageError> {
    let file_path = get_locations_file_path(&app_handle).await?;

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| LocationStorageError::IoError(e.to_string()))?;
    }

    let json_content = serde_json::to_string_pretty(&locations).map_err(|e| LocationStorageError::SerdeError(e.to_string()))?;
    write(&file_path, json_content).await.map_err(|e| LocationStorageError::IoError(e.to_string()))
}

// --- The new command ---
#[tauri::command]
pub async fn get_home_dir() -> Result<String, ConfigError> {
    tracing::info!("Attempting to get user home directory..."); // Use tracing if initialized

    // Get user-specific directories
    let user_dirs = UserDirs::new().ok_or(ConfigError::HomeDirNotFound)?;

    // Get the home directory path
    let home_dir_path = user_dirs.home_dir();

    // Convert the PathBuf to a String, handling potential encoding issues
    home_dir_path
        .to_str()
        .map(|s| {
            tracing::info!("Found home directory: {}", s);
            s.to_string()
        })
        .ok_or(ConfigError::InvalidPathEncoding)
}

#[tauri::command]
pub async fn list_directory_command(
    path: String, 
    app_handle: AppHandle
) -> Result<Vec<FileInfo>, FileSystemError> {
    // Check if this is a virtual route (starts with "/" but doesn't exist on filesystem)
    if path.starts_with('/') && !PathBuf::from(&path).exists() {
        // Check known virtual routes
        if path == "/indexing-status" {
            println!("Detected virtual route: {}", path);
            // Return empty result for virtual routes
            return Ok(Vec::new());
        }
    }

    let path_buf = PathBuf::from(path);

    println!("Listing directory: {:?}", path_buf); 
    // Pass app_handle to the core list_directory function
    match list_directory(&path_buf, app_handle).await { 
        Ok(items) => {
            println!("Successfully listed {} items.", items.len());
            Ok(items)
        }
        Err(e) => {
             eprintln!("Error listing directory {:?}: {}", path_buf, e); 
             Err(e) 
        }
    }
}

#[derive(Debug, serde::Serialize, thiserror::Error)]
pub enum OpenError {
    #[error("Failed to open path '{path}': {message}")]
    IoError { path: String, message: String },
}

/// Attempts to open the given path (file or directory) using the system's default application.
#[tauri::command]
pub async fn open_path_command(path: String) -> Result<(), OpenError> {
    tracing::info!("Attempting to open path: {}", path);
    opener::open(&path).map_err(|e| {
        tracing::error!("Failed to open path '{}': {}", path, e);
        OpenError::IoError {
            path: path.clone(),
            message: e.to_string(),
        }
    })
}

// Helper function to get a specific user directory path as String
fn get_user_dir_path<F>(dir_fn: F) -> Result<String, ConfigError>
where
    F: FnOnce(&UserDirs) -> Option<&Path>,
{
    let user_dirs = UserDirs::new().ok_or(ConfigError::HomeDirNotFound)?;
    dir_fn(&user_dirs)
        .and_then(Path::to_str)
        .map(String::from)
        .ok_or(ConfigError::InvalidPathEncoding)
}

#[tauri::command]
pub async fn get_documents_dir() -> Result<String, ConfigError> {
    get_user_dir_path(|dirs| dirs.document_dir())
}

#[tauri::command]
pub async fn get_downloads_dir() -> Result<String, ConfigError> {
    // Note: Downloads dir might require BaseDirs on some platforms if UserDirs fails
    // For simplicity, we try UserDirs first. Add BaseDirs fallback if needed.
    get_user_dir_path(|dirs| dirs.download_dir())
}

#[tauri::command]
pub async fn get_movies_dir() -> Result<String, ConfigError> {
    get_user_dir_path(|dirs| dirs.video_dir()) // Often video_dir corresponds to Movies
}

#[tauri::command]
pub async fn get_hostname_command() -> Result<String, HostnameError> {
    hostname::get()
        .map_err(|e| HostnameError::OsError(e.to_string()))
        .and_then(|os_str| {
            os_str
                .into_string()
                .map_err(|_| HostnameError::OsError("Hostname contains invalid UTF-8".to_string()))
        })
}

// --- Thumbnail Generation Task Implementation ---

const THUMBNAIL_SIZE: u32 = 128; // Target size for thumbnails (e.g., 128x128)

fn resize_and_save_image(
    img: DynamicImage,
    cache_path: &Path,
) -> Result<(), String> {
    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }

    // Resize the image
    let thumbnail_rgba = img.resize_to_fill(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);

    // Convert RGBA to RGB before saving as JPEG
    let thumbnail_rgb: DynamicImage = DynamicImage::ImageRgb8(thumbnail_rgba.to_rgb8());

    // Save as JPEG using the RGB image
    thumbnail_rgb
        .save_with_format(cache_path, ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))
}

// Updated generate_image_thumbnail using png crate for PNGs
async fn generate_image_thumbnail(original_path: &Path, cache_path: &Path) -> Result<(), String> {
    // 1) First, try the `image` crate
    match ImageReader::open(original_path) {
        Ok(reader) => {
            // If it succeeds, decode the image with proper error handling
            match reader.decode() {
                Ok(img) => return resize_and_save_image(img, cache_path),
                Err(e) => {
                    tracing::warn!(
                        "Failed to decode image {:?} with error: {}. Attempting PNG crate fallback...",
                        original_path,
                        e
                    );
                    println!("Failed to decode image {:?} with error: {}. Attempting PNG crate fallback...", original_path, e);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "image::open() failed for {:?} with error: {}. Attempting PNG crate fallback...",
                original_path,
                e
            );
            println!("image::open() failed for {:?} with error: {}. Attempting PNG crate fallback...", original_path, e);
        }
    }

    println!("Attempting PNG crate fallback...");
    // 2) If that fails, try the PNG crate *only if* the file's extension is actually `.png`.
    let is_png_ext = original_path
        .extension()
        .and_then(|s| s.to_str())
        .map_or(false, |ext| ext.eq_ignore_ascii_case("png"));

    if !is_png_ext {
        // Not even a `.png` file—no point to use the PNG crate
        return Err("Not a PNG, and the `image` crate failed to decode it.".to_string());
    }

    // Attempt manual PNG decoding with `png` crate
    tracing::debug!("Decoding PNG using 'png' crate: {:?}", original_path);
    let file = StdFile::open(original_path)
        .map_err(|e| format!("Failed to open file for png decoder: {}", e))?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("Failed to read png info: {}", e))?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("Failed to decode png frame: {}", e))?;

    let img = match info.color_type {
        png::ColorType::Rgb => DynamicImage::ImageRgb8(
            image::ImageBuffer::from_raw(info.width, info.height, buf)
                .ok_or_else(|| "Failed to create RGB buffer from PNG".to_string())?,
        ),
        png::ColorType::Rgba => DynamicImage::ImageRgba8(
            image::ImageBuffer::from_raw(info.width, info.height, buf)
                .ok_or_else(|| "Failed to create RGBA buffer from PNG".to_string())?,
        ),
        png::ColorType::Grayscale => {
            let gray_img = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
                info.width,
                info.height,
                buf,
            )
            .ok_or_else(|| "Failed to create Grayscale buffer from PNG".to_string())?;
            DynamicImage::ImageRgb8(DynamicImage::ImageLuma8(gray_img).to_rgb8())
        }
        png::ColorType::GrayscaleAlpha => {
            let gray_alpha_img = image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::from_raw(
                info.width,
                info.height,
                buf,
            )
            .ok_or_else(|| "Failed to create Gray+Alpha buffer".to_string())?;
            let rgba = DynamicImage::ImageLumaA8(gray_alpha_img).to_rgba8();
            DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(rgba).to_rgb8())
        }
        _ => {
            return Err(format!(
                "Unsupported PNG color type {:?}",
                info.color_type
            ))
        }
    };

    // 3) Resize & save
    resize_and_save_image(img, cache_path)
}

fn generate_video_thumbnail(original_path: &Path, cache_path: &Path) -> Result<(), String> {
    // tracing::debug!("Generating video thumbnail for: {:?}", original_path);
    ffmpeg::init().map_err(|e| format!("Failed to initialize ffmpeg: {}", e))?;

    let mut ictx = ffmpeg::format::input(&original_path)
        .map_err(|e| format!("Failed to open video input: {}", e))?;

    let input_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or_else(|| "No video stream found".to_string())?;
    let video_stream_index = input_stream.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())
        .map_err(|e| format!("Failed to get codec context: {}", e))?;
    let mut decoder = context_decoder.decoder().video()
        .map_err(|e| format!("Failed to get video decoder: {}", e))?;

    // Seek to ~10% into the video for the thumbnail frame
    let duration = ictx.duration();
    let timestamp = if duration > 0 { duration / 10 } else { 0 }; // Use 0 if duration is unknown
    // Need to convert timestamp to the stream's time_base
    let seek_target = (timestamp as i64).rescale(ffmpeg::rescale::TIME_BASE, input_stream.time_base());
    // Seek slightly before the target frame
    ictx.seek(seek_target - 1, ..seek_target)
        .map_err(|e| format!("Failed to seek video: {}", e))?;

    let mut scaler_context = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::RGB24, // Target format for image crate
        decoder.width(), // Use original dimensions for scaler initially
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )
    .map_err(|e| format!("Failed to create scaler context: {}", e))?;

    let mut frame_index = 0;
    let max_frames_to_check = 30; // Check a few frames after seeking

    let mut received_frame: Option<DynamicImage> = None;

    // Process packets until we get a valid frame
    'packet_loop: for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).map_err(|e| format!("Failed to send packet to decoder: {}", e))?;
            let mut decoded_frame = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                let mut rgb_frame = ffmpeg::util::frame::video::Video::empty();
                scaler_context.run(&decoded_frame, &mut rgb_frame)
                    .map_err(|e| format!("Failed to scale frame: {}", e))?;

                // Convert frame data to image::DynamicImage
                let img = DynamicImage::ImageRgb8(
                    image::ImageBuffer::from_raw(
                        rgb_frame.width(),
                        rgb_frame.height(),
                        rgb_frame.data(0).to_vec(), // Copy data
                    )
                    .ok_or_else(|| "Failed to create image buffer from frame data".to_string())?,
                );
                received_frame = Some(img);
                break 'packet_loop; // Got a frame, exit
            }
        }
        frame_index += 1;
        if frame_index > max_frames_to_check {
            return Err("Could not decode a suitable frame after seeking".to_string());
        }
    }
    
    if let Some(img) = received_frame {
        resize_and_save_image(img, cache_path)
    } else {
        Err("Failed to receive any frame from decoder".to_string())
    }
}

pub(crate) async fn generate_thumbnail_task(
    original_path: PathBuf,
    cache_path: PathBuf,
    _app_handle: AppHandle,
) {
    let added = {
        let mut processing = PROCESSING_THUMBNAILS.lock().unwrap();
        processing.insert(original_path.clone())
    };
    if !added { return; }

    let result = if let Some(ext) = original_path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            // Image types - Added svg
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" => {
                // Await the async image generation
                generate_image_thumbnail(&original_path, &cache_path).await
            }
            // Video types
            "mp4" | "mov" | "avi" | "mkv" | "webm" => {
                // Video generation might still be blocking depending on ffmpeg-next usage
                // Wrap potentially blocking call in spawn_blocking if performance becomes an issue
                generate_video_thumbnail(&original_path, &cache_path)
            }
            _ => Err(format!("Unsupported extension for thumbnail: {}", ext)),
        }
    } else {
        Err("File has no extension".to_string())
    };

    if let Err(e) = result {
        tracing::error!(
            "Failed to generate thumbnail for {:?}: {}",
            original_path, e
        );
    }

    {
        let mut processing = PROCESSING_THUMBNAILS.lock().unwrap();
        processing.remove(&original_path);
    }
}

// Add other file-system related commands here later if needed
