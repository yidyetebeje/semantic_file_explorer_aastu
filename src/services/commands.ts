// src/services/commands.ts
import { invoke } from "@tauri-apps/api/core";
import { SearchRequest, SearchResponse, FilenameSearchRequest, FilenameSearchResponse } from "../types/search";
import { IndexingStats } from "../store/atoms";

/**
 * Calls the backend semantic_search_command.
 *
 * @param request - The search request parameters.
 * @returns A promise that resolves with the search response.
 */
export async function semanticSearch(request: SearchRequest): Promise<SearchResponse> {
  try {
    console.log("Invoking semantic_search_command with:", request);
    const response = await invoke<SearchResponse>("semantic_search_command", { request });
    console.log("Received from semantic_search_command:", response);
    return response;
  } catch (error) {
    console.error("Error invoking semantic_search_command:", error);
    // Re-throw the error so the caller (e.g., the atom) can handle it
    throw error;
  }
}

/**
 * Initiates indexing of the Downloads folder.
 * 
 * @returns A promise that resolves with the indexing statistics.
 */
export async function indexDownloads(): Promise<IndexingStats> {
  try {
    console.log("Invoking index_downloads_command");
    const response = await invoke<IndexingStats>("index_downloads_command");
    console.log("Received from index_downloads_command:", response);
    return response;
  } catch (error) {
    console.error("Error invoking index_downloads_command:", error);
    throw error;
  }
}

/**
 * Gets the current indexing statistics.
 * 
 * @returns A promise that resolves with the current indexing statistics.
 */
export async function getIndexingStats(): Promise<IndexingStats> {
  try {
    console.log("Invoking get_indexing_stats_command");
    const response = await invoke<IndexingStats>("get_indexing_stats_command");
    console.log("Received from get_indexing_stats_command:", response);
    return response;
  } catch (error) {
    console.error("Error invoking get_indexing_stats_command:", error);
    throw error;
  }
}

/**
 * Clears all indexed data from the database.
 * 
 * @returns A promise that resolves with the operation result.
 */
export async function clearIndexData(): Promise<{success: boolean, message: string}> {
  try {
    console.log("Invoking clear_index_command");
    const response = await invoke<{success: boolean, message: string}>("clear_index_command");
    console.log("Received from clear_index_command:", response);
    return response;
  } catch (error) {
    console.error("Error invoking clear_index_command:", error);
    throw error;
  }
}

/**
 * Initiates indexing of a specific folder.
 * 
 * @param folderPath - The path of the folder to index.
 * @returns A promise that resolves with the indexing statistics.
 */
export async function indexFolder(folderPath: string): Promise<IndexingStats> {
  try {
    console.log(`Invoking index_folder_command for: ${folderPath}`);
    const response = await invoke<IndexingStats>("index_folder_command", { folderPath });
    console.log("Received from index_folder_command:", response);
    return response;
  } catch (error) {
    console.error(`Error invoking index_folder_command for ${folderPath}:`, error);
    throw error;
  }
}

/**
 * Calls the backend filename_search_command.
 *
 * @param request - The filename search request parameters.
 * @returns A promise that resolves with the filename search response.
 */
export async function filenameSearch(request: FilenameSearchRequest): Promise<FilenameSearchResponse> {
  try {
    console.log("Invoking filename_search_command with:", request);
    const response = await invoke<FilenameSearchResponse>("filename_search_command", { request });
    console.log("Received from filename_search_command:", response);
    return response;
  } catch (error) {
    console.error("Error invoking filename_search_command:", error);
    // Re-throw the error so the caller (e.g., the atom) can handle it
    throw error;
  }
}

/**
 * Gets the current filename index statistics.
 * 
 * @returns A promise that resolves with the current filename index statistics.
 */
export async function getFilenameIndexStats(): Promise<{file_count: number}> {
  try {
    console.log("Invoking get_filename_index_stats");
    const response = await invoke<{file_count: number}>("get_filename_index_stats");
    console.log("Received from get_filename_index_stats:", response);
    return response;
  } catch (error) {
    console.error("Error invoking get_filename_index_stats:", error);
    throw error;
  }
}

/**
 * Clears all data from the filename index.
 * 
 * @returns A promise that resolves when the operation completes.
 */
export async function clearFilenameIndex(): Promise<void> {
  try {
    console.log("Invoking clear_filename_index");
    await invoke("clear_filename_index");
    console.log("Successfully cleared filename index");
  } catch (error) {
    console.error("Error invoking clear_filename_index:", error);
    throw error;
  }
}

/**
 * Scans a directory and adds all files to the filename index.
 * 
 * @param dirPath - The path to the directory to scan.
 * @returns A promise that resolves with scanning statistics.
 */
export async function scanDirectoryForFilenameIndex(dirPath: string): Promise<{directory: string, files_added: number, errors: string[]}> {
  try {
    console.log("Invoking scan_directory_for_filename_index with:", dirPath);
    const response = await invoke<{directory: string, files_added: number, errors: string[]}>("scan_directory_for_filename_index", { dirPath });
    console.log("Received from scan_directory_for_filename_index:", response);
    return response;
  } catch (error) {
    console.error("Error invoking scan_directory_for_filename_index:", error);
    throw error;
  }
}

/**
 * Initializes the filename index with common directories.
 * 
 * @returns A promise that resolves with initialization statistics.
 */
export async function initializeFilenameIndex(): Promise<{total_files_added: number, directory_results: any[]}> {
  try {
    console.log("Invoking initialize_filename_index");
    const response = await invoke<{total_files_added: number, directory_results: any[]}>("initialize_filename_index");
    console.log("Received from initialize_filename_index:", response);
    return response;
  } catch (error) {
    console.error("Error invoking initialize_filename_index:", error);
    throw error;
  }
}

/**
 * Gets the current vector database statistics including the count of documents in each table.
 * 
 * @returns A promise that resolves with vector database statistics.
 */
export async function getVectorDatabaseStats(): Promise<{
  text_documents_count: number;
  image_documents_count: number;
  total_documents_count: number;
}> {
  try {
    console.log("Invoking get_vector_db_stats_command");
    const response = await invoke<{
      text_documents_count: number;
      image_documents_count: number;
      total_documents_count: number;
    }>("get_vector_db_stats_command");
    console.log("Received from get_vector_db_stats_command:", response);
    return response;
  } catch (error) {
    console.error("Error invoking get_vector_db_stats_command:", error);
    return { text_documents_count: 0, image_documents_count: 0, total_documents_count: 0 };
  }
}
