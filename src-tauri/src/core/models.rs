use chrono::{DateTime, Utc}; // Import chrono types
use serde::{Deserialize, Serialize};

/// Represents information about a file or directory entry.
/// Derives Ord for easy sorting in tests (uses name as primary sort key).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    /// Size in bytes. None for directories or if unable to retrieve.
    pub size: Option<u64>,
    /// Last modification timestamp (UTC). None if unable to retrieve.
    #[serde(with = "chrono::serde::ts_seconds_option")] // Serialize as optional Unix timestamp
    pub modified: Option<DateTime<Utc>>,
    /// Descriptive file type (e.g., "Text", "Image", "Directory").
    pub file_type: String,
    /// Optional path to a generated thumbnail in the cache directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_path: Option<String>,
    /// Optional embedding for the file, if available and applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

// Note: The default Ord derived above will sort primarily by `name`.
// If you need different sorting later (e.g., by date), you might need a custom implementation
// or sort explicitly after fetching the data.
