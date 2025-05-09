# Chapter Five: System Implementation

## 5.1 Reviewing the Design Solution

In the design phase, we set out to enable fast, accurate semantic search and exploration of local text documents on the user’s file system. The core architecture comprises a Rust-based backend powered by Tauri, responsible for:

- **File system monitoring** via `notify` crate (watcher.rs) to detect file creations, modifications, and deletions.
- **Content extraction** (extractor.rs) and **hashing** to identify changed documents.
- **Text chunking** (chunker.rs) for semantically meaningful segments.
- **Vector embeddings** (embedder.rs) using FastEmbed (BGE-Small-EN-v1.5) and caching for performance.
- **Vector store** management with LanceDB (db.rs) to index and query embeddings.
- **Semantic search** logic (search.rs) that ranks candidates by cosine similarity.

On the front end, a React + TypeScript UI (App.tsx, components/) uses:

- **Jotai** atoms (store/atoms.ts) for state management.
- **Tailwind CSS** + Radix UI for a responsive, modern interface.
- A thin **Tauri API** layer (services/commands.ts) to invoke Rust commands asynchronously.

Overall, the implementation closely aligns with our objectives: real-time indexing, accurate semantic retrieval, and an intuitive desktop UI. The modular separation between watcher, extractor, embedder, and search maintains clarity and ease of extension.

## 5.2 Deciding on the Development Tools

To balance performance, cross-platform compatibility, and developer productivity, the following toolchain was selected:

| Layer           | Language/Tool               | Purpose                                  |
|-----------------|-----------------------------|------------------------------------------|
| Backend         | Rust + Tauri                | System daemon, file I/O, performance     |
| Vector CLI      | FastEmbed (Rust crate)      | Text embedding with BGE-Small-EN-v1.5    |
| Vector store    | LanceDB                     | On-disk vector database with Arrow schema|
| File watcher    | notify crate                | Cross-platform FS event monitoring       |
| Frontend        | React + TypeScript + Vite   | Fast Hot Reloading UI                    |
| UI components   | Radix UI + Tailwind CSS     | Accessible, themeable components         |
| State Mgmt      | Jotai                       | Simple atomic state                     |
| Packaging       | Tauri CLI                   | Desktop app bundling                      |
| Version control | Git + PNPM                  | Dependency management, reproducible builds|

**Environment setup:**

1. Install Rust (stable) and Cargo.
2. Install Node.js (>=16) and pnpm.
3. Run `pnpm install` in the project root.
4. Run `cargo install tauri-cli` or use `pnpm tauri`.
5. Configure `tauri.conf.json`, `vite.config.ts`, and `tailwind.config.js` as per defaults.

## 5.3 Developing the Solution

### Coding Practices

- **Async/Await** in Rust (Tokio) and TypeScript for non-blocking I/O.
- **Error handling** with `thiserror` and detailed logs via `tracing` in Rust.
- **Unit tests** in both Rust (`#[cfg(test)]`) and TS (Jest) to validate core logic.
- **Modularization**: each concern (watcher, db, embedder, search) lives in its own module.

### Integration Strategy

- **Tauri commands** expose backend functions to the frontend (e.g., `semantic_search_command`, `index_folder_command`).
- **IPC** via `@tauri-apps/api` to invoke Rust commands from React.
- **State synchronization**: results stored in Jotai atoms and rendered in React components.

### Major Functionality Highlights

- **Watcher + DB combo** (`setup_watcher_with_db` in watcher.rs): initializes a LanceDB table and FS watcher in one call.

```rust
let (watcher, rx, table) = setup_watcher_with_db("/Users/you/Documents").await?;
process_events(rx, table).await;
```

- **Embedding pipeline** (`embed_text` in embedder.rs): chunks input text, generates embeddings, and returns `Vec<Vec<f32>>`.

- **Semantic search** (`semantic_search_command` in search_commands.rs): accepts a query string, queries LanceDB for nearest neighbors, and returns ranked document paths.

- **Front-end query flow** (`services/commands.ts`): wraps Tauri `invoke` calls and updates Jotai atoms for UI rendering.

### Challenges and Resolutions

- **Model download latency** mitigated by lazy initialization (`once_cell::Lazy`) and caching in `.cache/`.
- **File-system events noise** filtered by supported extensions and hidden files logic in `is_relevant_file`.
- **Schema evolution** handled by compatibility checks in `schemas_compatible`.

By following this structured implementation, the system meets our design goals and provides a robust, maintainable foundation for future features (e.g., PDF support, UI theming, batch indexing).
