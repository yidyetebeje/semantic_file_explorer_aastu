import { atom } from "jotai";
import { FileInfo, ViewMode } from "../types/file";
import { CustomLocation } from "../types/location";
import { SearchResponse, SearchResult, FilenameSearchResult, FileCategory } from "../types/search"; 
import { atomWithStorage } from 'jotai/utils';
import {
  fetchDirectoryContents,
  getHomeDir,
  loadCustomLocations,
  saveCustomLocations,
} from "../services/test";
import { 
  semanticSearch, 
  filenameSearch,
  indexDownloads, 
  getIndexingStats,
  indexFolder,
  clearIndexData,
  getFilenameIndexStats,
  clearFilenameIndex,
  scanDirectoryForFilenameIndex,
  initializeFilenameIndex,
  getVectorDatabaseStats
} from "../services/commands";

// --- Indexing State Atoms ---
export interface IndexingStats {
  files_processed: number;
  files_indexed: number;
  files_skipped: number;
  files_failed: number;
  time_taken_ms: number;
  success: boolean;
  message: string;
  indexed_files: string[];
  failed_files: string[];
  
  // New fields for multimodal indexing
  db_inserts?: number;
  text_files_processed?: number;
  text_files_indexed?: number;
  text_files_failed?: number;
  image_files_processed?: number;
  image_files_indexed?: number;
  image_files_failed?: number;
}

// --- Vector Database Stats ---
export interface VectorDatabaseStats {
  text_documents_count: number;
  image_documents_count: number;
  total_documents_count: number;
}

export const indexingStatsAtom = atom<IndexingStats | null>(null);
export const isIndexingAtom = atom<boolean>(false);
export const indexedFilesAtom = atom<string[]>([]);
export const vectorDbStatsAtom = atom<VectorDatabaseStats | null>(null);

// Atom to trigger indexing and update the stats
export const triggerIndexingAtom = atom(
  null,
  async (_get, set) => {
    try {
      set(isIndexingAtom, true);
      const stats = await indexDownloads();
      set(indexingStatsAtom, stats);
    } catch (error) {
      console.error('Error during indexing:', error);
    } finally {
      set(isIndexingAtom, false);
    }
  }
);

// Atom to fetch the current indexing status
export const fetchIndexingStatsAtom = atom(
  null,
  async (_get, set) => {
    try {
      const stats = await getIndexingStats();
      set(indexingStatsAtom, stats);
    } catch (error) {
      console.error('Error fetching indexing stats:', error);
    }
  }
);

// Atom to fetch the current vector database statistics
export const fetchVectorDbStatsAtom = atom(
  null,
  async (_get, set) => {
    try {
      const stats = await getVectorDatabaseStats();
      set(vectorDbStatsAtom, stats);
    } catch (error) {
      console.error('Error fetching vector database stats:', error);
    }
  }
);

// --- Basic UI State Atoms ---
// ... (keep existing atoms like viewModeAtom, etc.)
export const viewModeAtom = atom<ViewMode>("grid");
export const fileSizeAtom = atom<number>(80);
export const gapSizeAtom = atom<number>(4);
export const selectedFileAtom = atom<FileInfo | null>(null);
export const showInspectorAtom = atom<boolean>(false);

// --- Core File Explorer State Atoms ---

// Atom to store the current directory path
export const currentPathAtom = atom<string>(""); // Current path remains central

// Atom to store the files/folders in the current directory
export const directoryFilesAtom = atom<FileInfo[]>([]);

// --- Search Mode ---
export type SearchMode = 'semantic' | 'filename';
export const searchModeAtom = atom<SearchMode>('semantic');

// --- File Categories for Filtering ---
export const availableFileCategoriesAtom = atom<FileCategory[]>([
  'Document',
  'Image',
  'Video',
  'Audio',
  'Archive',
  'Code',
  'Other'
]);

export const selectedFileCategoriesAtom = atom<FileCategory[]>([]);

// --- Filename Search Max Distance ---
export const maxDistanceAtom = atom<number>(2); // Default Levenshtein distance

// --- Shared Search State Atoms ---
export const searchQueryAtom = atom<string>("");
export const isSearchingAtom = atom<boolean>(false);
export const searchErrorAtom = atom<string | null>(null);
export const hasSearchedAtom = atom<boolean>(false); // Track when a search has been explicitly performed

// --- Semantic Search Results ---
export const semanticSearchResultsAtom = atom<SearchResult[]>([]);

// --- Filename Search Results ---
export const filenameSearchResultsAtom = atom<FilenameSearchResult[]>([]);

// --- Combined Results Atom (derived) ---
export const searchResultsAtom = atom<(SearchResult | FilenameSearchResult)[]>((get) => {
  const searchMode = get(searchModeAtom);
  return searchMode === 'semantic'
    ? get(semanticSearchResultsAtom)
    : get(filenameSearchResultsAtom);
});

// Write-only atom to trigger a search based on the current mode
export const triggerSearchAtom = atom(
  null,
  async (_get, set) => {
    const query = _get(searchQueryAtom);
    const searchMode = _get(searchModeAtom);
    
    if (!query.trim()) {
      // Don't search if query is empty or whitespace
      set(searchErrorAtom, null);
      if (searchMode === 'semantic') {
        set(semanticSearchResultsAtom, []);
      } else {
        set(filenameSearchResultsAtom, []);
      }
      return;
    }

    set(isSearchingAtom, true);
    set(searchErrorAtom, null);
    // Clear previous results for the current search mode
    if (searchMode === 'semantic') {
      set(semanticSearchResultsAtom, []);
    } else {
      set(filenameSearchResultsAtom, []);
    }
    set(hasSearchedAtom, true); // Set the flag to indicate a search has been performed

    // Add search query to recent searches
    set(addToRecentSearchesAtom, query);

    try {
      if (searchMode === 'semantic') {
        // Semantic search
        console.log(`Triggering semantic search for: "${query}"`);
        const response: SearchResponse = await semanticSearch({ query }); 
        console.log("Semantic search response:", response);
        set(semanticSearchResultsAtom, response.results);
      } else {
        // Filename search
        console.log(`Triggering filename search for: "${query}"`);
        const maxDistance = _get(maxDistanceAtom);
        const categories = _get(selectedFileCategoriesAtom);
        const response = await filenameSearch({ 
          query, 
          max_distance: maxDistance, 
          categories: categories.length > 0 ? categories : undefined
        });
        console.log("Filename search response:", response);
        set(filenameSearchResultsAtom, response.results);
      }
    } catch (error) {
      console.error(`${searchMode} search failed:`, error);
      const errorMessage = error instanceof Error ? error.message : String(error);
      set(searchErrorAtom, `Search failed: ${errorMessage}`);
      if (searchMode === 'semantic') {
        set(semanticSearchResultsAtom, []);
      } else {
        set(filenameSearchResultsAtom, []);
      }
    } finally {
      set(isSearchingAtom, false);
    }
  }
);

// Atom to store loading state
export const isLoadingAtom = atom<boolean>(false);

// Atom to store potential errors
export const errorAtom = atom<string | null>(null);

// Atom to store active folder filters
export const folderFilterCategoriesAtom = atom<FileCategory[]>([]);

// Boolean atom to track if folder filtering is active
export const isFolderFilterActiveAtom = atom<boolean>(false);

// Derived atom to filter visible files based on active filters and hidden files
export const visibleFilesAtom = atom((get) => {
  const allFiles = get(directoryFilesAtom);
  const activeFilters = get(folderFilterCategoriesAtom);
  const isFilterActive = get(isFolderFilterActiveAtom);
  
  // First filter out hidden files (always active)
  const nonHiddenFiles = allFiles.filter(file => !file.name.startsWith('.'));
  
  // If folder filters are not active, return all non-hidden files
  if (!isFilterActive || activeFilters.length === 0) {
    return nonHiddenFiles;
  }
  
  // Apply category filters to non-hidden files
  return nonHiddenFiles.filter(file => {
    // Directories are always shown
    if (file.is_directory) return true;
    
    // For files, check if their type matches any of the active filters
    const fileType = file.file_type.toLowerCase();
    
    // Match file type to categories
    const matchesCategory = activeFilters.some(category => {
      switch(category) {
        case 'Document':
          return fileType.includes('document') || 
                 fileType.includes('pdf') || 
                 fileType.includes('txt') || 
                 fileType.includes('doc') || 
                 fileType.includes('text');
        case 'Image':
          return fileType.includes('image') || 
                 fileType.includes('photo') ||
                 fileType.includes('png') ||
                 fileType.includes('jpg') ||
                 fileType.includes('jpeg');
        case 'Video':
          return fileType.includes('video') || 
                 fileType.includes('movie');
        case 'Audio':
          return fileType.includes('audio') || 
                 fileType.includes('sound') || 
                 fileType.includes('music');
        case 'Archive':
          return fileType.includes('archive') || 
                 fileType.includes('zip') || 
                 fileType.includes('compressed');
        case 'Code':
          return fileType.includes('code') || 
                 fileType.includes('source') || 
                 fileType.includes('text');
        case 'Other':
          // Files that don't match any other category
          return !fileType.includes('document') && 
                 !fileType.includes('image') && 
                 !fileType.includes('video') && 
                 !fileType.includes('audio') && 
                 !fileType.includes('archive') && 
                 !fileType.includes('code');
        default:
          return false;
      }
    });
    
    return matchesCategory;
  });
});

// Atom to apply folder filters
export const applyFolderFiltersAtom = atom(
  null,
  (_get, set, categories: FileCategory[]) => {
    if (categories.length === 0) {
      set(isFolderFilterActiveAtom, false);
    } else {
      set(folderFilterCategoriesAtom, categories);
      set(isFolderFilterActiveAtom, true);
    }
  }
);

// Atom to clear folder filters
export const clearFolderFiltersAtom = atom(
  null,
  (_get, set) => {
    set(folderFilterCategoriesAtom, []);
    set(isFolderFilterActiveAtom, false);
  }
);

// --- Navigation History State ---
export const pathHistoryAtom = atom<string[]>([]);
export const historyIndexAtom = atom<number>(-1);

// --- Custom Locations State ---
export const customLocationsAtom = atom<CustomLocation[]>([]);

// Atom to trigger loading custom locations on startup
export const loadLocationsOnInitAtom = atom(
  null,
  async (_get, set) => {
    try {
      const loadedLocations = await loadCustomLocations();
      set(customLocationsAtom, loadedLocations);
    } catch (error) {
      console.error("Failed to load custom locations on init:", error);
      set(customLocationsAtom, []);
    }
  }
);

// Atom to add a new custom location and save the updated list
export const addCustomLocationAtom = atom(
  null,
  async (get, set, newLocation: CustomLocation) => {
    const currentLocations = get(customLocationsAtom);
    if (currentLocations.some(loc => loc.path === newLocation.path)) {
        console.warn("Location already exists:", newLocation.path);
        return; 
    }
    const updatedLocations = [...currentLocations, newLocation];
    set(customLocationsAtom, updatedLocations);
    try {
      await saveCustomLocations(updatedLocations);
    } catch (error) {
      console.error("Failed to save custom locations after adding:", error);
      set(customLocationsAtom, currentLocations);
    }
  }
);

// --- Derived Atoms / Atoms with Logic ---

// Derived atoms for navigation button state
export const canGoBackAtom = atom((get) => get(historyIndexAtom) > 0);
export const canGoForwardAtom = atom((get) => get(historyIndexAtom) < get(pathHistoryAtom).length - 1);

// Atom to handle navigating to a new path (updates history)
export const navigateAtom = atom(
  null,
  (get, set, newPath: string) => {
    const history = get(pathHistoryAtom);
    const currentIndex = get(historyIndexAtom);

    if (history[currentIndex] === newPath) {
      return;
    }

    const newHistory = history.slice(0, currentIndex + 1);
    newHistory.push(newPath);

    set(pathHistoryAtom, newHistory);
    set(historyIndexAtom, newHistory.length - 1);
    set(currentPathAtom, newPath);
  }
);

// Atom to navigate back in history
export const goBackAtom = atom(
  null,
  (get, set) => {
    if (get(canGoBackAtom)) {
      const newIndex = get(historyIndexAtom) - 1;
      set(historyIndexAtom, newIndex);
      const history = get(pathHistoryAtom);
      set(currentPathAtom, history[newIndex]);
    }
  }
);

// Atom to navigate forward in history
export const goForwardAtom = atom(
  null,
  (get, set) => {
    if (get(canGoForwardAtom)) {
      const newIndex = get(historyIndexAtom) + 1;
      set(historyIndexAtom, newIndex);
      const history = get(pathHistoryAtom);
      set(currentPathAtom, history[newIndex]);
    }
  }
);

// Atom to fetch and set the initial home directory (modified)
export const loadHomeDirAtom = atom(
  null,
  async (_get, set) => {
    try {
      const homeDir = await getHomeDir();
      set(navigateAtom, homeDir);
      set(errorAtom, null);
    } catch (err) {
      console.error("Failed to get home directory:", err);
      set(errorAtom, "Failed to load home directory.");
    }
  }
);

// Asynchronous atom to load directory contents (no change needed here)
export const loadDirectoryAtom = atom(
  null,
  async (_get, set) => {
    const path = _get(currentPathAtom);
    if (!path) return;

    set(isLoadingAtom, true);
    set(errorAtom, null);
    set(directoryFilesAtom, []);

    try {
      const files = await fetchDirectoryContents(path);
      set(directoryFilesAtom, files);
    } catch (error) {
      console.error(`Error fetching directory "${path}":`, error);
      set(errorAtom, `Failed to load directory: ${path}`);
      set(directoryFilesAtom, []);
    } finally {
      set(isLoadingAtom, false);
    }
  }
);

// Add these new atoms
export const selectedFolderPathAtom = atom<string | null>(null);
export const isClearingIndexAtom = atom<boolean>(false);

// --- Filename Indexing State Atoms ---
export interface FilenameIndexStats {
  file_count: number;
}

export const filenameIndexStatsAtom = atom<FilenameIndexStats | null>(null);
export const isFilenameIndexingAtom = atom<boolean>(false);
export const selectedFolderForFilenameIndexAtom = atom<string | null>(null);
export const isClearingFilenameIndexAtom = atom<boolean>(false);

// --- Recent Files and Searches Atoms ---
export interface RecentItem {
  path: string;
  name: string;
  accessedAt: number; // timestamp
  type: 'file' | 'directory';
  fileType?: string;
}

export interface RecentSearchItem {
  query: string;
  mode: SearchMode;
  timestamp: number;
}

// Store recent items with persistence (using localStorage)
export const recentItemsAtom = atomWithStorage<RecentItem[]>('recentItems', []);
export const recentSearchesAtom = atomWithStorage<RecentSearchItem[]>('recentSearches', []);

// Maximum number of recent items to keep
export const MAX_RECENT_ITEMS = 10;
export const MAX_RECENT_SEARCHES = 10;

// Action atom to add a file or directory to recent items
export const addToRecentItemsAtom = atom(
  null,
  (get, set, item: Omit<RecentItem, 'accessedAt'>) => {
    const recentItems = get(recentItemsAtom);
    
    // Don't add if the item is already at the top of the list
    if (recentItems.length > 0 && recentItems[0].path === item.path) {
      return;
    }
    
    // Create new item with current timestamp
    const newItem: RecentItem = {
      ...item,
      accessedAt: Date.now()
    };
    
    // Filter out this item if it already exists (to avoid duplicates)
    const filteredItems = recentItems.filter(existing => existing.path !== item.path);
    
    // Add new item to the beginning of the array and limit the size
    set(recentItemsAtom, [newItem, ...filteredItems].slice(0, MAX_RECENT_ITEMS));
  }
);

// Action atom to add a search query to recent searches
export const addToRecentSearchesAtom = atom(
  null,
  (get, set, query: string) => {
    if (!query.trim()) return;
    
    const recentSearches = get(recentSearchesAtom);
    const searchMode = get(searchModeAtom);
    
    // Create new search item
    const newSearchItem: RecentSearchItem = {
      query,
      mode: searchMode,
      timestamp: Date.now()
    };
    
    // Filter out this query if it already exists
    const filteredItems = recentSearches.filter(
      existing => !(existing.query === query && existing.mode === searchMode)
    );
    
    // Add new item to the beginning and limit the size
    set(recentSearchesAtom, [newSearchItem, ...filteredItems].slice(0, MAX_RECENT_SEARCHES));
  }
);
export const filenameIndexingResultAtom = atom<{ directory?: string, files_added: number, errors?: string[] } | null>(null);

// Atom to trigger indexing for a specific folder
export const triggerFolderIndexingAtom = atom(
  null,
  async (_get, set) => {
    const folderPath = _get(selectedFolderPathAtom);
    if (!folderPath) {
      console.error('No folder path selected for indexing');
      return;
    }

    try {
      set(isIndexingAtom, true);
      const stats = await indexFolder(folderPath);
      set(indexingStatsAtom, stats);
    } catch (error) {
      console.error(`Error during indexing folder ${folderPath}:`, error);
    } finally {
      set(isIndexingAtom, false);
    }
  }
);

// Atom to clear all indexed data
export const clearIndexDataAtom = atom(
  null,
  async (_get, set) => {
    try {
      set(isClearingIndexAtom, true);
      const result = await clearIndexData();
      
      if (result.success) {
        // Reset the stats
        set(indexingStatsAtom, {
          files_processed: 0,
          files_indexed: 0,
          files_skipped: 0,
          files_failed: 0,
          time_taken_ms: 0,
          success: true,
          message: result.message,
          indexed_files: [],
          failed_files: []
        });
      } else {
        console.error('Failed to clear index data:', result.message);
      }
    } catch (error) {
      console.error('Error clearing index data:', error);
    } finally {
      set(isClearingIndexAtom, false);
    }
  }
);

// Atom to fetch filename index stats
export const fetchFilenameIndexStatsAtom = atom(
  null,
  async (_get, set) => {
    try {
      const stats = await getFilenameIndexStats();
      set(filenameIndexStatsAtom, stats);
    } catch (error) {
      console.error('Error fetching filename index stats:', error);
    }
  }
);

// Atom to clear filename index
export const clearFilenameIndexDataAtom = atom(
  null,
  async (_get, set) => {
    try {
      set(isClearingFilenameIndexAtom, true);
      await clearFilenameIndex();
      
      // Reset the stats
      set(filenameIndexStatsAtom, {
        file_count: 0
      });
      set(filenameIndexingResultAtom, null);
    } catch (error) {
      console.error('Error clearing filename index data:', error);
    } finally {
      set(isClearingFilenameIndexAtom, false);
    }
  }
);

// Atom to scan a directory for filename indexing
export const scanDirectoryForFilenameIndexAtom = atom(
  null,
  async (_get, set) => {
    const folderPath = _get(selectedFolderForFilenameIndexAtom);
    if (!folderPath) {
      console.error('No folder path selected for filename indexing');
      return;
    }

    try {
      set(isFilenameIndexingAtom, true);
      const result = await scanDirectoryForFilenameIndex(folderPath);
      set(filenameIndexingResultAtom, result);
      
      // Update the stats
      const stats = await getFilenameIndexStats();
      set(filenameIndexStatsAtom, stats);
    } catch (error) {
      console.error(`Error during filename indexing of folder ${folderPath}:`, error);
    } finally {
      set(isFilenameIndexingAtom, false);
    }
  }
);

// Atom to initialize filename index
export const initializeFilenameIndexAtom = atom(
  null,
  async (_get, set) => {
    try {
      set(isFilenameIndexingAtom, true);
      const result = await initializeFilenameIndex();
      set(filenameIndexingResultAtom, { files_added: result.total_files_added });
      
      // Update the stats
      const stats = await getFilenameIndexStats();
      set(filenameIndexStatsAtom, stats);
    } catch (error) {
      console.error('Error initializing filename index:', error);
    } finally {
      set(isFilenameIndexingAtom, false);
    }
  }
);
