# Chapter Six: System Testing and Evaluation

## 6.1 Preparing Sample Test Plans

To systematically validate our semantic file explorer, we define test categories, specific cases, success criteria, and chosen tools:

1. **Unit Tests (Rust backend)**
   - chunker::chunk_text
     • Cases: empty string, small (<512 chars), large (>2048 chars), text forcing max chunks
     • Criteria: correct number of chunks, no panics
   - embedder::embed_text
     • Cases: empty slice, identical vs. different strings, error on chunk failure
     • Criteria: returns Vec<Vec<f32>> of length = total chunks; dimension = 384
   - extractor::extract_text
     • Cases: .txt, .md, unsupported extension, non-existent path
     • Criteria: correct content or Err variant
   - db.rs functions
     • connect_db, open_or_create_table: create or open LanceDB table
     • upsert_document/delete_document: insert/delete rows, subsequent queries reflect changes
     • Criteria: no errors, schema compatibility
   - watcher.rs
     • setup_watcher: valid path, non-existent path, file path
     • process_events: mock events for create/modify/delete
     • Criteria: correct DbError on invalid inputs; events produce upsert/delete calls
   - search_commands & search.rs
     • semantic_search_command: valid query, empty query
     • Criteria: valid SearchResponse or Err("Empty query")

2. **Integration Tests (Rust)**
   - Full pipeline: index a temp folder with sample files → perform semantic search → verify results include expected file paths
   - Tools: Tokio runtime, TempDir, existing #[cfg(test)] modules

3. **End-to-End & Manual Tests (UI + backend)**
   - **Dev flow**: `pnpm dev` + `pnpm tauri` → open app → point to sample dir → observe file list
   - **Search flow**: enter query in UI → expect relevant items
   - Criteria: no UI errors, correct data displayed
   - Tools: Browser console, network logs, Tauri logs

4. **Performance Tests (Benchmark)**
   - @ignored benchmarks in `benchmark::tests` (large model)
   - Criteria: embedding per chunk <100 ms (configurable)
   - Tools: Criterion or built-in test harness


## 6.2 Evaluating the Proposed Design and Solutions

### Automated Test Results (Rust)

| Suite                     | Total | Passed | Failed | Ignored |
|---------------------------|-------|--------|--------|---------|
| Unit & Integration Tests  | 38    | 37     | 0      | 1       |

Command: `cargo test` in `src-tauri`
Result: **ok** in 0.63 s (37 passed, 1 ignored)

### Manual UI Smoke Tests

| Test Case                         | Steps                                                         | Result  | Notes                  |
|-----------------------------------|---------------------------------------------------------------|---------|------------------------|
| Launch app                        | `pnpm dev` + `pnpm tauri` → open window                        | PASS    |                    |
| Directory listing                 | Select folder with .txt/.md files → list appears             | PASS    |                    |
| Semantic search query             | Search for keyword in sample files → displays top-k results   | PASS    | Latency ~50 ms        |
| File watcher live update          | Add new .txt file during runtime → list refreshes automatically| PASS    | slight UI throttle    |

**Tools & metrics captured:** Tauri logs, browser console, DevTools


## 6.3 Discussing the Results

- **Test coverage:** All unit and integration tests passed; critical paths are verified. The ignored benchmark test avoids large model download but can be run manually when needed.
- **Design validation:** Modular components (watcher, extractor, embedder, db, search) behave as intended. Tauri commands and IPC layer integrate cleanly.
- **Performance observations:** Embedding and search complete within interactive bounds (<100 ms per query for small corpora). FS events propagate within ~200 ms to UI.
- **Issues found:** None critical. Minor UI throttling observed on batch file additions.

**Recommendations & future work:**
1. Automate UI end-to-end tests (Vitest/Cypress) for regression.
2. Add performance benchmarks in CI (Criterion) and track metrics over time.
3. Support additional file types (PDF, Word) by extending extractor.
4. Expose advanced search filters (date, file type) in UI.
