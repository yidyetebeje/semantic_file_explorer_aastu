import { useAtomValue } from 'jotai';
import { Loader2, AlertTriangle, FileText, Image, FileVideo, FileAudio, Archive, Code, FileQuestion } from 'lucide-react';
import {
  searchResultsAtom,
  isSearchingAtom,
  searchErrorAtom,
  searchModeAtom
} from '@/store/atoms';
import { SearchResult, FilenameSearchResult, FileCategory } from '@/types/search';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
// Helper function to get the appropriate icon for file categories
const getCategoryIcon = (category: FileCategory) => {
  switch (category) {
    case 'Document':
      return <FileText className="h-4 w-4 mr-2 flex-shrink-0" />;
    case 'Image':
      return <Image className="h-4 w-4 mr-2 flex-shrink-0" />;
    case 'Video':
      return <FileVideo className="h-4 w-4 mr-2 flex-shrink-0" />;
    case 'Audio':
      return <FileAudio className="h-4 w-4 mr-2 flex-shrink-0" />;
    case 'Archive':
      return <Archive className="h-4 w-4 mr-2 flex-shrink-0" />;
    case 'Code':
      return <Code className="h-4 w-4 mr-2 flex-shrink-0" />;
    default:
      return <FileQuestion className="h-4 w-4 mr-2 flex-shrink-0" />;
  }
};

// Helper function to format file size
const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
};

// Helper function to format date from unix timestamp
const formatDate = (timestamp: number): string => {
  return new Date(timestamp * 1000).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
};

const SearchResultsDisplay = () => {
  const results = useAtomValue(searchResultsAtom);
  const isLoading = useAtomValue(isSearchingAtom);
  const error = useAtomValue(searchErrorAtom);
  const searchMode = useAtomValue(searchModeAtom);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        <Loader2 className="mr-2 h-6 w-6 animate-spin" />
        <span>Searching...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-red-500 px-4">
        <AlertTriangle className="h-8 w-8 mb-2" />
        <span className="font-semibold mb-1">Search Error</span>
        <p className="text-center text-sm text-red-400">{error}</p>
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        <span>No results found. Try a different search query.</span>
      </div>
    );
  }

  return (
    <ScrollArea className="h-full p-4">
      <h2 className="text-lg font-semibold text-gray-200 mb-3">
        {searchMode === 'semantic' ? 'Semantic' : 'Filename'} Search Results ({results.length})
      </h2>
      <div className="space-y-3">
        {searchMode === 'semantic' 
          ? (results as SearchResult[]).map((result, index) => (
              <Card key={`${result.content_hash}-${index}`} className="bg-gray-800/60 border-gray-700 hover:bg-gray-800 transition-colors duration-150">
                <CardHeader className="p-3">
                  <CardTitle className="text-sm font-medium text-purple-300 flex items-center">
                    <FileText className="h-4 w-4 mr-2 flex-shrink-0" />
                    <span className="truncate" title={result.file_path}>{result.file_path}</span>
                  </CardTitle>
                </CardHeader>
                <CardContent className="p-3 pt-0 text-xs text-gray-400">
                  <p>Relevance: {(result.score * 100).toFixed(1)}%</p>
                  <p>Modified: {formatDate(result.last_modified)}</p>
                </CardContent>
              </Card>
            ))
          : (results as FilenameSearchResult[]).map((result, index) => (
              <Card key={`${result.file_path}-${index}`} className="bg-gray-800/60 border-gray-700 hover:bg-gray-800 transition-colors duration-150">
                <CardHeader className="p-3">
                  <CardTitle className="text-sm font-medium text-purple-300 flex items-center">
                    {getCategoryIcon(result.category)}
                    <span className="truncate flex-grow" title={result.file_path}>{result.file_path}</span>
                    {result.distance > 0 && (
                      <span className="text-xs px-1.5 py-0.5 rounded-full bg-purple-900/50 text-purple-200 ml-2">
                        Distance: {result.distance}
                      </span>
                    )}
                  </CardTitle>
                </CardHeader>
                <CardContent className="p-3 pt-0 text-xs grid grid-cols-2 gap-x-4 text-gray-400">
                  <p>Match: {(result.score * 100).toFixed(1)}%</p>
                  <p>Size: {formatFileSize(result.size)}</p>
                  <p>Modified: {formatDate(result.last_modified)}</p>
                  <p>Type: {result.category}</p>
                </CardContent>
              </Card>
            ))
        }
      </div>
    </ScrollArea>
  );
};

export default SearchResultsDisplay;
