// Assuming you have a types file matching your Rust struct
import { invoke } from "@tauri-apps/api/core";
import { FileInfo } from "@/types/file";
import { CustomLocation } from "@/types/location"; // Use alias path
// Define the error type shape based on your Rust FileSystemError enum
// (This helps with type checking in TypeScript)
interface FileSystemError {
  // Match the variants from your Rust enum
  NotFound?: { path: string };
  PermissionDenied?: { path: string };
  NotADirectory?: { path: string };
  ReadDirError?: { path: string };
  MetadataError?: { path: string };
  InvalidPathEncoding?: { path: string };
  IoError?: { path: string; kind: string };
}

export async function fetchDirectoryContents(
  directoryPath: string,
): Promise<FileInfo[]> {
  try {
    const items = await invoke<FileInfo[]>("list_directory_command", {
      path: directoryPath,
    });
    console.log("Files received:", items);
    return items;
  } catch (error) {
    // Tauri wraps the Rust error Enum within a general error message.
    // The actual structured error might be inside the error object,
    // often as a stringified JSON or directly accessible depending on Tauri version/setup.
    // For robust error handling, you might need to parse the error message
    // or expect a specific structure.
    console.error(`Error listing directory "${directoryPath}":`, error);

    // Example of trying to access structured error (adjust based on actual error format)
    // Usually the error thrown is a string, you might need more robust parsing
    const typedError = error as any; // Be cautious with 'any'
    // Log the specific error variant if possible
    // This parsing logic might need adjustment based on how Tauri serializes the Err variant.
    // Often it's just the string from `thiserror::Error`.
    // You might need to adjust your Rust error serialization or TS parsing.
    // For now, just re-throwing or returning empty array.

    // alert(`Error: ${error}`); // Simple user feedback
    return []; // Return empty array or throw error for caller to handle
  }
}

// Function to get the home directory
export async function getHomeDir(): Promise<string> {
  try {
    const homeDir = await invoke<string>("get_home_dir");
    console.log("Home directory:", homeDir);
    return homeDir;
  } catch (error) {
    console.error("Error getting home directory:", error);
    // You might want to throw the error or return a default/fallback path
    throw new Error(`Failed to get home directory: ${error}`);
  }
}

/**
 * Calls the Tauri backend to open a given file or directory path using the default system application.
 */
export async function openPath(path: string): Promise<void> {
  try {
    await invoke("open_path_command", { path });
    console.log(`Attempted to open path: ${path}`);
  } catch (error) {
    console.error(`Error opening path "${path}":`, error);
    // Handle or surface the error to the user (e.g., via a notification or error state)
    // Example: alert(`Failed to open: ${error}`);
    // Depending on how Tauri serializes the `OpenError`, the `error` variable
    // might contain structured information like { OpenerError: { path: ..., source: ... } }
    // or just a string representation.
    // You might want to parse it for better user feedback.
    throw new Error(`Failed to open path: ${error}`); // Re-throw for potential higher-level handling
  }
}

// Helper function to invoke a command expecting a single string path result
async function getDirPath(command: string): Promise<string> {
  try {
    const path = await invoke<string>(command);
    console.log(`Fetched path for ${command}: ${path}`);
    return path;
  } catch (error) {
    console.error(`Error fetching path for ${command}:`, error);
    throw new Error(`Failed to get path for ${command}: ${error}`);
  }
}

// Functions to get specific user directories
export const getDocumentsDir = () => getDirPath("get_documents_dir");
export const getDownloadsDir = () => getDirPath("get_downloads_dir");
export const getMoviesDir = () => getDirPath("get_movies_dir");

// Function to get the computer's hostname
export const getHostname = () => getDirPath("get_hostname_command");

// --- Custom Location Functions ---

export async function loadCustomLocations(): Promise<CustomLocation[]> {
  try {
    const locations = await invoke<CustomLocation[]>("load_custom_locations");
    console.log("Loaded custom locations:", locations);
    return locations;
  } catch (error) {
    console.error("Error loading custom locations:", error);
    // Return empty array or re-throw depending on how you want to handle errors
    return []; 
  }
}

export async function saveCustomLocations(locations: CustomLocation[]): Promise<void> {
  try {
    await invoke("save_custom_locations", { locations });
    console.log("Saved custom locations:", locations);
  } catch (error) {
    console.error("Error saving custom locations:", error);
    // Re-throw or handle the error (e.g., show notification)
    throw new Error(`Failed to save custom locations: ${error}`);
  }
}

// Example usage in a React component:
// const [files, setFiles] = useState<FileInfo[]>([]);
// useEffect(() => {
//   fetchDirectoryContents('/') // Fetch root directory on mount
//     .then(setFiles)
//     .catch(err => console.error("Failed to load initial directory"));
// }, []);
