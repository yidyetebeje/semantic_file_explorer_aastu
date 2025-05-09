import React from 'react';
import { useAtomValue, useSetAtom } from 'jotai';
import { FileIcon, FileTextIcon, FileImageIcon, FileArchiveIcon, FileCodeIcon, ExternalLink } from 'lucide-react';
import { searchResultsAtom, searchQueryAtom, isSearchingAtom, searchErrorAtom, hasSearchedAtom, navigateAtom, selectedFileAtom } from '../../store/atoms';
import { SearchResult, FilenameSearchResult } from '@/types/search';
import { Button } from '../ui/button';
import { Skeleton } from '../ui/skeleton';
import { formatDistance } from 'date-fns';
import { openPath } from '../../services/test';

// Helper function to determine the icon based on file path
const getFileIcon = (filePath: string) => {
  const extension = filePath.split('.').pop()?.toLowerCase();
  
  if (!extension) return <FileIcon className="h-5 w-5 text-gray-400" />;
  
  switch (extension) {
    case 'pdf':
      return <FileTextIcon className="h-5 w-5 text-red-500" />;
    case 'txt':
    case 'md':
    case 'rtf':
    case 'doc':
    case 'docx':
      return <FileTextIcon className="h-5 w-5 text-blue-500" />;
    case 'jpg':
    case 'jpeg':
    case 'png':
    case 'gif':
    case 'webp':
      return <FileImageIcon className="h-5 w-5 text-green-500" />;
    case 'zip':
    case 'rar':
    case 'tar':
    case 'gz':
      return <FileArchiveIcon className="h-5 w-5 text-yellow-500" />;
    case 'js':
    case 'ts':
    case 'jsx':
    case 'tsx':
    case 'py':
    case 'rs':
    case 'go':
    case 'java':
    case 'html':
    case 'css':
      return <FileCodeIcon className="h-5 w-5 text-purple-500" />;
    default:
      return <FileIcon className="h-5 w-5 text-gray-400" />;
  }
};

// Helper function to format file path for display
const formatFilePath = (filePath: string) => {
  // Extract file name and directory
  const parts = filePath.split('/');
  const fileName = parts.pop() || '';
  const directory = parts.join('/');
  
  return { fileName, directory };
};

// Helper function to truncate hash
const truncateHash = (hash: string, length: number) => {
  return hash.substring(0, length) + '...';
};

// Component for a single search result
const SearchResultItem = ({ result }: { result: SearchResult | FilenameSearchResult }) => {
  const { fileName, directory } = formatFilePath(result.file_path);
  const lastModified = new Date(result.last_modified * 1000); // Convert Unix timestamp to JS Date
  
  // Determine if this is a semantic or filename search result
  const isSemanticResult = 'content_hash' in result;
  
  const navigate = useSetAtom(navigateAtom);
  const setSelectedFile = useSetAtom(selectedFileAtom);
  const setHasSearched = useSetAtom(hasSearchedAtom);
  
  // Navigate to the file's directory and highlight the file
  const handleNavigateToFile = () => {
    // Navigate to the file's directory
    navigate(directory);
    
    // Create a simulated file info object to highlight the selected file
    setSelectedFile({
      name: fileName,
      path: result.file_path,
      is_directory: false,
      file_type: fileName.split('.').pop() || 'Unknown',
      size: null, // We don't have this information from the search result
      modified: result.last_modified,
    });
    
    // Hide the search results after navigation
    setHasSearched(false);
  };
  
  // Open the file directly
  const handleOpenFile = async (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent the parent click handler from firing
    
    try {
      await openPath(result.file_path);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };
  
  return (
    <div 
      className="p-3 hover:bg-gray-800/50 rounded-md transition-colors cursor-pointer" 
      onClick={handleNavigateToFile}
    >
      <div className="flex items-center gap-3">
        {getFileIcon(result.file_path)}
        <div className="flex-1 min-w-0">
          <div className="flex justify-between items-start">
            <p className="font-medium text-gray-200 truncate">{fileName}</p>
            <div className="text-right text-gray-500 text-xs mt-2">
              {/* Different format based on type of result */}
              {isSemanticResult && (
                <code className="text-xs">{truncateHash((result as SearchResult).content_hash, 6)}</code>
              )}
              <div>Score: {result.score.toFixed(2)}</div>
              <div>{formatDistance(lastModified, new Date(), { addSuffix: true })}</div>
              {!isSemanticResult && (
                <div>Distance: {(result as FilenameSearchResult).distance}</div>
              )}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <div className="inline-flex items-center px-2 py-1 rounded-full bg-purple-900/30 text-purple-400 text-xs">
            {Math.round(result.score * 100)}% match
          </div>
          <Button 
            variant="ghost" 
            size="icon" 
            onClick={handleOpenFile}
            title="Open file"
            className="h-8 w-8 text-gray-400 hover:text-gray-200 hover:bg-gray-700"
          >
            <ExternalLink className="h-4 w-4" />
          </Button>
        </div>
      </div>
      {isSemanticResult && (
        <p className="text-gray-400 text-sm line-clamp-2 mt-1">
          {/* Use appropriate property from result */}
          {"No excerpt available"}
        </p>
      )}
    </div>
  );
};

// Loading skeleton for search results
const SearchResultSkeleton = () => (
  <div className="p-3 rounded-md">
    <div className="flex items-center gap-3">
      <Skeleton className="h-5 w-5 rounded-md" />
      <div className="flex-1">
        <Skeleton className="h-5 w-3/4 mb-1" />
        <Skeleton className="h-4 w-1/2" />
      </div>
      <Skeleton className="h-6 w-16 rounded-full" />
    </div>
  </div>
);

const EnhancedSearchResults: React.FC = () => {
  const searchResults = useAtomValue(searchResultsAtom);
  const searchQuery = useAtomValue(searchQueryAtom);
  const isSearching = useAtomValue(isSearchingAtom);
  const searchError = useAtomValue(searchErrorAtom);
  const hasSearched = useAtomValue(hasSearchedAtom);
  
  // Only show results if a search has been explicitly performed via Enter or Search button
  if (!hasSearched) {
    return null;
  }
  
  return (
    <div className="fixed top-16 right-4 z-50 w-full max-w-lg shadow-xl">
      <div className="bg-gray-900 border border-gray-800 rounded-lg overflow-hidden">
        <div className="p-3 bg-gray-800/50 border-b border-gray-800">
          <h3 className="font-medium text-gray-200">
            {isSearching ? (
              'Searching...'
            ) : searchResults.length > 0 ? (
              `Found ${searchResults.length} result${searchResults.length === 1 ? '' : 's'} for "${searchQuery}"`
            ) : searchQuery ? (
              `No results found for "${searchQuery}"`
            ) : (
              'Search Results'
            )}
          </h3>
        </div>
        
        <div className="max-h-96 overflow-y-auto">
          {isSearching ? (
            // Show loading skeletons while searching
            <div className="divide-y divide-gray-800">
              {[...Array(3)].map((_, i) => (
                <SearchResultSkeleton key={i} />
              ))}
            </div>
          ) : searchError ? (
            // Show error message
            <div className="p-4 text-center">
              <p className="text-red-400 mb-2">{searchError}</p>
              <Button 
                variant="outline" 
                size="sm" 
                className="text-gray-400 border-gray-700 hover:text-gray-200 hover:border-gray-600"
              >
                Try Again
              </Button>
            </div>
          ) : searchResults.length > 0 ? (
            // Show search results
            <div className="divide-y divide-gray-800">
              {searchResults.map((result) => (
                <SearchResultItem key={result.file_path} result={result} />
              ))}
            </div>
          ) : searchQuery ? (
            // No results found for query
            <div className="p-4 text-center text-gray-400">
              <p>No matching files found.</p>
              <p className="text-sm mt-1">Try using different keywords or check your indexing status.</p>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
};

export default EnhancedSearchResults; 