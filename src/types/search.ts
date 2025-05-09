// src/types/search.ts

/**
 * File categories for filtering in filename search
 * Matches the Rust FileCategory enum
 */
export type FileCategory = 
  | 'Document'
  | 'Image'
  | 'Video'
  | 'Audio'
  | 'Archive'
  | 'Code'
  | 'Other';

/**
 * Represents a single semantic search result from the backend.
 * Matches the Rust SearchResult struct.
 */
export interface SearchResult {
  file_path: string;
  score: number;
  content_hash: string;
  last_modified: number; // Assuming Rust i64 (timestamp) maps to number
}

/**
 * Represents a single filename search result from the backend.
 * Matches the Rust FilenameSearchResult struct.
 */
export interface FilenameSearchResult {
  file_path: string;
  name: string;
  category: FileCategory;
  last_modified: number;
  size: number;
  score: number;
  distance: number; // Levenshtein distance
}

/**
 * Represents the request payload for the semantic search command.
 * Matches the Rust SearchRequest struct.
 */
export interface SearchRequest {
  query: string;
  limit?: number;
  min_score?: number;
  db_uri?: string;
  table_name?: string;
}

/**
 * Represents the request payload for the filename search command.
 * Matches the Rust FilenameSearchRequest struct.
 */
export interface FilenameSearchRequest {
  query: string;
  max_distance?: number;
  categories?: FileCategory[];
  limit?: number;
}

/**
 * Represents the response from the semantic search command.
 * Matches the Rust SearchResponse struct.
 */
export interface SearchResponse {
  results: SearchResult[];
  total_results: number;
  query: string;
}

/**
 * Represents the response from the filename search command.
 * Matches the Rust FilenameSearchResponse struct.
 */
export interface FilenameSearchResponse {
  results: FilenameSearchResult[];
  total_results: number;
  query: string;
}
