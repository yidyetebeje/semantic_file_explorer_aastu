# Multimodal Search Implementation Tasks

## Plan Overview
Add image embedding and search capabilities to the existing text-based semantic search system, making it multimodal.

## Implementation Tasks
- [x] Create an `image_embedder.rs` module that handles image embedding using CLIP-ViT-B-32
- [x] Modify `extractor.rs` to support image files (read metadata and paths)
- [x] Extend the database schema to include an images table for storing image embeddings
- [ ] Update `indexer.rs` to detect and process image files
- [ ] Modify `search.rs` to search both text and image collections
- [ ] Combine and rank results from both collections
- [ ] Add appropriate unit tests for new functionality
- [ ] Update the list of supported file extensions
- [ ] Document the multimodal search capabilities

## Progress Tracking
Task completion will be marked here as we implement each component.

### Completed
- Created `image_embedder.rs` with CLIP-ViT-B-32 model for image embedding
- Updated `extractor.rs` with support for image file metadata extraction
- Modified `db.rs` to include a separate table for image embeddings with appropriate schema
- Added unit tests for image embedding and database operations 