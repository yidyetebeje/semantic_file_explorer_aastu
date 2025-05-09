export interface FileItem {
  name: string;
  type: string;
  dateCreated: string;
  size?: string;
  path?: string;
}

export type ViewMode = "grid" | "list";
export interface FileInfo {
  /** The name of the file or directory (e.g., "document.txt"). */
  name: string;

  /** The full absolute path to the file or directory. */
  path: string;

  /** True if the entry is a directory, false if it's a file. */
  is_directory: boolean; // Note: snake_case matches the Rust struct field name

  /**
   * Size of the file in bytes.
   * null for directories or if the size couldn't be retrieved.
   */
  size: number | null;

  /**
   * Last modification timestamp, represented as seconds since the Unix epoch (UTC).
   * null if the timestamp couldn't be retrieved.
   * To convert to a JS Date object: `new Date(modified * 1000)` (if not null).
   */
  modified: number | null; // Note: Received as Unix timestamp (seconds) from Rust

  /**
   * Descriptive file type category (e.g., "Text", "Image", "PDF", "Directory").
   * Determined by the backend based on MIME type or extension.
   */
  file_type: string; // Note: snake_case matches the Rust struct field name

  /**
   * Optional path to a generated thumbnail in the app's cache directory.
   * Use with Tauri's `convertFileSrc` to get a usable URL.
   */
  thumbnail_path?: string | null;
}
