# Filename Search Implementation Plan

## Architecture Overview

We've implemented a filename search module that allows users to search for files by name with tolerance for spelling errors, plus advanced file type filtering options.

### Key Components

1. **Rust Backend (Tauri)**
   - Filename indexer (BK-Tree for fuzzy matching)
   - File type detection and categorization
   - Tauri commands for search operations

2. **TypeScript Frontend**
   - Extended search UI with toggle between semantic/filename search
   - File type filter components
   - State management for the new search options

## Implementation Tasks

### 1. Backend: Filename Index Module ✅

- [x] Create `filename_index.rs` to implement a BK-Tree for fuzzy filename search
- [x] Implement methods for adding, removing, and searching filenames
- [x] Add file type detection and categorization
- [x] Write unit tests for the module

### 2. Backend: Integration with File Watcher ✅

- [x] Modify `watcher.rs` to update the filename index on file system changes
- [x] Ensure proper synchronization between file system events and the index
- [x] Test the integration

### 3. Backend: Tauri Commands ✅

- [x] Create `filename_search_command` in `search_commands.rs`
- [x] Add file type filtering parameters to the command
- [x] Implement filtering logic for different file types
- [x] Write unit tests for the commands

### 4. Frontend: State Management ✅

- [x] Update `atoms.ts` to include state for:
  - Search mode (semantic vs. filename)
  - File type filters
  - Filename search results
- [x] Create actions to trigger filename search

### 5. Frontend: UI Components ✅

- [x] Modify the search bar in `Navbar.tsx` to add toggle between search modes
- [x] Create file type filter components
- [x] Update `SearchResultsDisplay.tsx` to handle both search types
- [x] Style new components to match existing design

### 6. Testing and Integration ⚠️

- [x] Write unit tests for backend components
- [ ] Test the integration thoroughly
- [ ] Test performance with large directories

## Implementation Details

### BK-Tree Implementation

We implemented a BK-Tree data structure for efficient fuzzy filename matching. The BK-Tree allows us to:

- Find files with similar names using Levenshtein distance as the metric
- Efficiently search for matches within a specified edit distance
- Support filtering by file type categories

### File Type Categorization

Files are automatically categorized based on their extensions into these groups:

- 📄 Documents (pdf, doc, txt, etc.)
- 🖼️ Images (jpg, png, gif, etc.)
- 🎬 Videos (mp4, mkv, avi, etc.)
- 🎵 Audio (mp3, wav, ogg, etc.)
- 📦 Archives (zip, rar, 7z, etc.)
- 💻 Code (rs, js, py, etc.)
- ❓ Other (anything else)

### User Interface

The UI includes:

- A toggle to switch between semantic and filename search modes
- File type filters for narrowing down filename search results
- Fuzzy match control slider to adjust the tolerance for spelling errors
- Enhanced search results display showing file metadata and match details

## Next Steps

1. Thoroughly test the integration between frontend and backend
2. Test performance with large directories
3. Consider adding persistence for the filename index to improve startup performance
4. Optimize the BK-Tree for larger datasets if needed

## Technical Decisions

### 1. Fuzzy Search Implementation

We'll use a BK-Tree data structure for fuzzy filename matching, which offers:
- O(log n) search complexity
- Efficient edit distance calculations
- Memory efficiency compared to other approaches

### 2. File Type Detection

We'll use file extensions for basic categorization along with:
- MIME type detection for more accurate file type information
- Common groupings (images, videos, documents, etc.)
- Custom categories for specialized files

### 3. Frontend-Backend Communication

We'll use Tauri commands similar to the existing semantic search:
- Implement a `filename_search_command` with similar structure to `semantic_search_command`
- Allow for concurrent semantic and filename searches
- Use similar result structures for consistent UI handling

## Detailed Implementation Plan

Let's break down the most critical implementation details.
