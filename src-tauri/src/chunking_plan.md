# Document Chunking Implementation Plan

## Goals
- Use text-splitter crate to split documents into semantic chunks
- Store chunks in the database with reference to source file
- When querying, return the highest scoring chunk per file
- Ensure backward compatibility with existing code

## Implementation Steps

1. Add text-splitter to dependencies
2. Create a new module for chunking functionality
   - Implement chunk_text function that takes text and returns chunks
   - Choose optimal chunking method and size
3. Modify embedder.rs to:
   - Process and embed chunks instead of whole documents
   - Return vector of embeddings for all chunks
4. Update upsert_document in db.rs to handle multiple chunks per file
5. Update query logic to select highest scoring chunk per file
6. Update tests to verify chunking behavior

## Considerations
- Best chunk size for semantic search effectiveness (research suggests ~512 tokens)
- Test with various document types and sizes
- Preserve document metadata with each chunk (file path, position in document)
- Update schema to handle multiple chunks per document

## Dependencies
- text-splitter: For semantic text chunking
- Current embedding model: BGESmallENV15 (384 dimensions) 