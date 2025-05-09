# Semantic File Explorer - Project Plan

## Legend

*   [ ] To Do
*   [~] In Progress
*   [x] Done
*   [!] Blocker / Needs Attention

## Phase 1: Core Setup & Backend Foundations

*   [x] **Task 1.1: Project Initialization & Setup**
    *   [x] Ensure Tauri project structure is correctly initialized.
    *   [x] Set up Rust backend module structure (`src-tauri/src/`).
*   [x] **Task 1.2: Add & Verify Core Dependencies**
    *   [x] Add `notify` to `src-tauri/Cargo.toml`.
    *   [x] Add `pdf-extract` to `src-tauri/Cargo.toml`.
    *   [x] Add `fastembed` (ensure correct features, e.g., `quantized`) to `src-tauri/Cargo.toml`.
    *   [x] Add `lancedb` to `src-tauri/Cargo.toml`.
    *   [x] Add `tokio` (ensure `full` features) to `src-tauri/Cargo.toml`.
    *   [x] Add `rayon` to `src-tauri/Cargo.toml`.
    *   [x] Add `serde`, `serde_json` (ensure `derive` feature for `serde`) to `src-tauri/Cargo.toml`.
    *   [x] Run `cargo check` within `src-tauri` to verify dependencies resolve. *(Next Step)*
*   [x] **Task 1.3: Basic ONNX Runtime/fastembed Test (PRD Priority)**
    *   [x] Create a minimal Rust function (outside Tauri command initially) to load `fastembed` with a chosen model (e.g., `all-MiniLM-L6-v2` quantized).
    *   [x] Test embedding a sample string.
    *   [x] Write a basic unit test for this embedding function.
    *   [x] **Goal:** Verify `fastembed` and its ONNX dependency work correctly in the basic Rust environment before Tauri integration/packaging.
*   [x] **Task 1.4: Initial LanceDB Setup**
    *   [x] Define initial LanceDB table schema (e.g., `id`, `path`, `content_hash`, `embedding_vector`, `metadata`).
    *   [x] Write Rust code to connect to/create a LanceDB database (`data.lancedb`).
    *   [x] Write Rust code to create the table using the schema if it doesn't exist.
    *   [x] Write basic unit tests for connection and table creation/opening.
*   [x] **Task 1.5: Address Warnings**
    *   [x] Run `cargo check` or `cargo build` and review warnings.
    *   [x] Remove unused imports, variables, etc. (using `cargo fix`).

## Phase 2: File Watching & Indexing Pipeline

*   [x] **Task 2.1: File System Watcher**
    *   [x] Implement file watcher using `notify` to monitor specified directories for changes (create, modify, delete).
    *   [x] Handle events asynchronously using `tokio`.
    *   [x] Filter events for relevant file types (e.g., `.pdf`, `.txt`, `.md`).
    *   [x] Add basic logging for watcher events.
    *   [x] Write integration tests for file watching scenarios.
*   [ ] **Task 2.2: Text Extraction**
    *   [x] Implement function to extract text content from supported file types (start with PDF using `pdf-extract`).
    *   [x] Add error handling for corrupted or unreadable files.
    *   [x] Implement basic content hashing (e.g., SHA256) for deduplication/change detection.
    *   [x] Write unit tests for text extraction and hashing.
*   [ ] **Task 2.3: Basic Indexing Pipeline**
    *   [x] Create `extractor.rs` module.
    *   [x] Implement function to extract text content from supported file types (start with `.txt`, `.md`).
    *   [x] Add error handling for corrupted or unreadable files (basic IO error handling added).
    *   [x] Implement basic content hashing (e.g., SHA256) for deduplication/change detection.
    *   [x] Write unit tests for text extraction.
    *   [x] Connect watcher events to the text extractor and embedder (from Task 1.3).
    *   [x] Implement logic to add/update embeddings in LanceDB based on file events and content hash.
    *   [x] Implement logic to remove entries from LanceDB for deleted files.
    *   [x] Use sequential embedding initially as per TDD risk mitigation.
    *   [x] Add robust error handling throughout the pipeline.
    *   [x] Write integration tests for the end-to-end indexing flow (file change -> DB update).

## Phase 3: Search Implementation & Refinement

*   [x] **Task 3.1: Embedding Model Benchmarking (PRD Recommendation)**
    *   [x] Create benchmarking module to compare embedding models
    *   [x] Implement performance metrics (initialization time, embedding time, dimensions)
    *   [x] Create command interface to run benchmarks from the application
    *   [x] Benchmark `all-MiniLM-L6-v2` vs `bge-small-en-v1.5` (quantized)
    *   [x] Evaluate embedding quality (subjective testing on sample docs/queries) and performance (speed, resource usage).
    *   [x] Decide on the default model based on results: **`bge-small-en-v1.5` selected as default model**
*   [x] **Task 3.2: Semantic Search Endpoint**
    *   [x] Implement Rust function to take a query string, generate its embedding using the chosen model.
    *   [x] Perform vector similarity search against the LanceDB index.
    *   [x] Return relevant file paths and scores.
    *   [x] Create Tauri command interface for search functionality.
    *   [x] Write unit tests for the search function.
## Phase 4: Tauri Integration & Frontend

*   [x] **Task 4.1: Expose Backend via Tauri Commands**
    *   [x] Wrap indexing control functions (start/stop watcher, trigger re-index) as Tauri commands.
    *   [x] Wrap the search function (Task 3.2/3.3) as a Tauri command.
    *   [x] Implement commands to get indexing status/progress.
    *   [x] Handle state management and potential concurrency issues between commands.
*   [x] **Task 4.2: Basic Frontend UI**
    *   [x] Create a simple Svelte/React/Vue frontend.
    *   [x] Add a search input field.
    *   [x] Add a results display area.
    *   [x] Add basic controls/status indicators for indexing.
*   [x] **Task 4.3: Frontend-Backend Communication**
    *   [x] Call Tauri commands from the frontend to trigger searches and display results.
    *   [x] Implement frontend logic to interact with indexing controls/status.

## Phase 5: Packaging, Testing & Performance

*   [~] **Task 5.1: ONNX Runtime Bundling (PRD Priority)**
    *   [~] Configure `build.rs` and/or Tauri build settings to correctly bundle ONNX Runtime native libraries (`.dll`, `.so`, `.dylib`).
    *   [ ] Test builds on target platforms (Windows, macOS, Linux).
    *   [ ] Verify the packaged application runs and performs embeddings correctly on machines *without* pre-installed ONNX Runtime.
*   [ ] **Task 5.2: Cross-Platform & Low-Spec Testing**
    *   [ ] Perform thorough testing on Windows, macOS, and Linux.
    *   [ ] Test resource usage (RAM, CPU) during idle, indexing, and search phases, especially on low-spec machines.
    *   [ ] Identify and address performance bottlenecks.
*   [ ] **Task 5.3: Error Handling & Robustness**
    *   [ ] Review and enhance error handling across all modules (watcher, parser, indexer, search, Tauri layer).
    *   [ ] Ensure graceful recovery from common errors (file access issues, corrupted data).

## Phase 6: Refinement & Advanced Features

*   [ ] **Task 6.1: Performance Optimizations**
    *   [ ] Explore safe parallelization (e.g., using `rayon` for pre-processing).
    *   [ ] Investigate batch embedding if sequential proves too slow and `fastembed`/ONNX allows safe batching.
*   [ ] **Task 6.2: UI/UX Improvements**
    *   [ ] Add features like result highlighting, file previews, sorting/filtering.
*   [ ] **Task 6.3: Configuration**
    *   [ ] Allow users to configure monitored directories, excluded paths, etc. via UI or config file.
