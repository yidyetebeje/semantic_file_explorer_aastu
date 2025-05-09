# Tantivy Filename Index Implementation Plan

This document outlines the steps to replace the custom BK-Tree based filename index with Tantivy.

## 1. Add Dependencies
- Add `tantivy = "<latest_version>"` (e.g., "0.22") to the `[dependencies]` section in `src-tauri/Cargo.toml`.
- Add `tantivy = "<latest_version>"` to `[build-dependencies]` if needed for build scripts (unlikely for this use case).
- Add `directories = "<latest_version>"` (e.g., "5.0") to get application data directories.

## 2. Define Schema and Index Setup
- **Schema:**
    - `path`: TEXT, STORED, (Use as unique identifier for deletes)
    - `name`: TEXT, STRING, (Indexed for full-text search, stored for display)
    - `category`: TEXT, STRING, (Indexed for filtering, stored for display)
    - `last_modified`: U64, STORED
    - `size`: U64, STORED
- **Index Location:** Store the Tantivy index in a subdirectory (e.g., `filename_index`) within the path provided by `dirs::app_local_data_dir()`. Ensure this directory exists.
- **Index Instance:** Use `once_cell::sync::Lazy` to create a static `tantivy::Index` instance, similar to the previous `FILENAME_INDEX`. Handle index creation/opening errors gracefully.

## 3. Refactor Indexing Commands (`src-tauri/src/commands/search_commands.rs`)
- **General:** All commands interacting with the index writer or performing significant search/scan work should remain `async` and use `tokio::task::spawn_blocking`.
- **`add_file_to_index`**:
    - Inside `spawn_blocking`:
        - Get an `IndexWriter`.
        - Create a Tantivy document using the defined schema based on input parameters.
        - Add the document using `writer.add_document()`.
        - Call `writer.commit()`. Handle potential errors.
- **`remove_file_from_index`**:
    - Inside `spawn_blocking`:
        - Get an `IndexWriter`.
        - Create a `Term` for the `path` field using the provided path string.
        - Delete the document using `writer.delete_term()`.
        - Call `writer.commit()`. Handle errors.
- **`clear_filename_index`**:
    - Inside `spawn_blocking`:
        - Get an `IndexWriter`.
        - Use `writer.delete_all_documents()` for simplicity.
        - Call `writer.commit()`. Handle errors.
        - (Alternative: Delete and recreate the index directory, might be faster for very large indices).
- **`scan_directory_for_filename_index`**:
    - Inside `spawn_blocking`:
        - Get an `IndexWriter`.
        - Walk the directory using `walkdir`.
        - For each file, create a Tantivy document.
        - Add documents in batches using `writer.add_document()`.
        - Call `writer.commit()` periodically or at the end. Return stats. Handle errors.
- **`initialize_filename_index`**:
    - Update to call the new Tantivy-based `scan_directory_for_filename_index` for common directories. Ensure `.await` is used.
- **`get_filename_index_stats`**:
    - Inside `spawn_blocking`:
        - Get a `Searcher` from the `Index`.
        - Use `searcher.num_docs()` to get the file count. Return as JSON `{"file_count": count}`.

## 4. Refactor Search Command (`src-tauri/src/commands/search_commands.rs`)
- **`filename_search_command`**:
    - Modify `FilenameSearchRequest`: Remove `max_distance`. Consider adding options for fuzzy search or query syntax if needed.
    - Modify `FilenameSearchResult`: Replace `distance: usize` with `score: f32`.
    - Inside `spawn_blocking`:
        - Get an `IndexReader` and create a `Searcher`.
        - Get schema fields for `name`, `path`, `category`, etc.
        - Create a `QueryParser` for the `name` field (and potentially others if multi-field search is desired).
        - Parse the user's query string. Allow fuzzy terms like `query~1` or `query~2` if fuzziness is desired.
        - Build the final query:
            - Start with the parsed query from `QueryParser`.
            - If categories are provided, create `TermQuery` for each category on the `category` field, combine them with `BooleanQuery` (SHOULD), and combine *this* with the main query using `BooleanQuery` (MUST).
        - Execute the search using `searcher.search(&query, &TopDocs::with_limit(limit.unwrap_or(10)))`.
        - Iterate through the results `(score, doc_address)`.
        - Retrieve the stored document using `searcher.doc(doc_address)?`.
        - Extract stored fields (`path`, `name`, `category`, `last_modified`, `size`) and construct `FilenameSearchResult` objects, using the Tantivy `score`.
        - Return the `Vec<FilenameSearchResult>`.
    - Handle Tantivy query parsing and search errors.

## 5. Remove Old Code
- Delete the file `src-tauri/src/filename_index.rs`.
- Remove the `pub static FILENAME_INDEX: Lazy<ThreadSafeIndex>` definition from `src-tauri/src/commands/search_commands.rs`.
- Remove any `use crate::filename_index::{...}` statements that are no longer needed.

## 6. Update Frontend (Review)
- Check `src/types/search.ts` (`FilenameSearchRequest`, `FilenameSearchResult`) and ensure they align with the backend changes (removal of `max_distance`, change from `distance` to `score`).
- Check `src/store/atoms.ts` and `src/services/commands.ts` for any logic dependent on the old fields. Update UI components if they display distance or allow setting max distance.

## 7. Build and Test
- Run `cargo build` frequently during implementation.
- Run `pnpm tauri dev` to test the integration.
- Add unit tests for schema creation, document mapping, and potentially query building logic if complex. 