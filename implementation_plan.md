# Multimodal Semantic Search Implementation Plan

## Overview
Add image indexing and search capabilities to make the semantic file explorer truly multimodal, while preserving existing text functionality.

## Key Components to Modify

### Backend (Rust)
1. **Image Embedder**
   - [x] Create `image_embedder.rs` module to handle image embedding using CLIP-ViT-B-32
   - [x] Implement efficient image loading and processing
   - [x] Add error handling for various image formats

2. **Database Integration**
   - [x] Create a separate table for image embeddings in LanceDB
   - [x] Implement schema for image table (path, hash, embeddings, etc.)
   - [x] Add function to combine text and image search results

3. **File Processing**
   - [x] Update `extractor.rs` to support image file detection
   - [x] Add image file extensions to supported types
   - [x] Handle image file preprocessing

4. **Indexing Logic**
   - [x] Update indexer to handle image files
   - [x] Implement indexing statistics for images
   - [x] Ensure proper error handling for image processing

5. **Search Functionality**
   - [x] Modify search to query both text and image tables
   - [x] Implement intelligent result combination
   - [x] Add option to filter by content type (text/image/all)

### Frontend (React)
1. **Indexing Status UI**
   - [x] Add image indexing stats display
   - [x] Update progress indicators for image indexing
   - [x] Show image file counts and processing status

2. **Search Results UI**
   - [ ] Add proper display for image results
   - [ ] Implement image thumbnails in results
   - [ ] Add UI controls to filter by content type

## Tasks Breakdown

### Phase 1: Core Image Embedding ✅
- [x] Create image_embedder.rs module
- [x] Implement ImageEmbedding with CLIP-ViT-B-32
- [x] Add unit tests for image embedding
- [x] Test with various image formats

### Phase 2: Database & Storage ✅
- [x] Design image table schema
- [x] Create table initialization and access functions
- [x] Implement functions to store image embeddings
- [x] Add unit tests for database operations

### Phase 3: Indexing Integration ✅
- [x] Update extractor.rs to handle image files
- [x] Modify indexer.rs to process image files
- [x] Update indexing stats to include image-specific metrics
- [x] Test indexing with mixed content directories

### Phase 4: Search Integration ✅
- [x] Implement combined search function
- [x] Add relevance scoring for cross-modal results
- [x] Create result merging algorithm
- [x] Test search with various queries

### Phase 5: Frontend Updates ⏳
- [x] Update IndexingStatus.tsx to show image stats
- [x] Update progress indicators for image indexing
- [x] Show image file counts and processing status
- [ ] Add proper display for image results in search UI
- [ ] Implement image thumbnails in results
- [ ] Add UI controls to filter by content type

## Guidelines
- Maintain backward compatibility
- Follow existing code patterns and error handling
- Ensure comprehensive testing
- Use efficient image loading to minimize memory usage
- Keep UI updates consistent with existing design 