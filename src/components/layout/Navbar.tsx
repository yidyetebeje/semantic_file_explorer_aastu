import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { 
  ArrowLeft, 
  ArrowRight, 
  Search, 
  XCircle,
  LayoutGrid, 
  List, 
  Maximize,
  Filter
} from "lucide-react";
import { useAtomValue, useSetAtom, useAtom } from 'jotai';
import {
  canGoBackAtom,
  canGoForwardAtom,
  goBackAtom,
  goForwardAtom,
  searchQueryAtom,
  triggerSearchAtom,
  isSearchingAtom,
  hasSearchedAtom,
  currentPathAtom,
  viewModeAtom,
  searchModeAtom,
  availableFileCategoriesAtom,
  selectedFileCategoriesAtom,
  maxDistanceAtom
} from '../../store/atoms'; // Adjust path if needed
import { Window } from "@tauri-apps/api/window"; // Import Window

const Navbar = () => {
  // Get navigation state and setters
  const canGoBack = useAtomValue(canGoBackAtom);
  const canGoForward = useAtomValue(canGoForwardAtom);
  const goBack = useSetAtom(goBackAtom);
  const goForward = useSetAtom(goForwardAtom);
  
  // Get search state and setters
  const [searchQuery, setSearchQuery] = useAtom(searchQueryAtom);
  const triggerSearch = useSetAtom(triggerSearchAtom);
  const isSearching = useAtomValue(isSearchingAtom);
  const setHasSearched = useSetAtom(hasSearchedAtom);
  
  // Search mode state
  const [searchMode, setSearchMode] = useAtom(searchModeAtom);
  
  // File type filter state
  const availableFileCategories = useAtomValue(availableFileCategoriesAtom);
  const [selectedFileCategories, setSelectedFileCategories] = useAtom(selectedFileCategoriesAtom);
  const [maxDistance, setMaxDistance] = useAtom(maxDistanceAtom);

  // Get current path
  const currentPath = useAtomValue(currentPathAtom);

  // Extract directory name from path
  const currentDirectoryName = currentPath.split(/[\/]/).pop() || "Home"; // Handle potential empty path or root

  // Get view mode setter
  const setViewMode = useSetAtom(viewModeAtom);

  // Handler for Enter key press in search input
  const handleSearchKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      triggerSearch();
    }
  };
  
  // Clear search query
  const handleClearSearch = () => {
    setSearchQuery('');
    setHasSearched(false);
  };
  
  // Handle search input change
  const handleSearchInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setSearchQuery(newValue);
    // Clear the hasSearched flag when the user types
    setHasSearched(false);
  };
  
  // Trigger search when search button is clicked
  const handleSearchClick = () => {
    if (searchQuery.trim()) {
      triggerSearch();
    }
  };

  // Handler for maximize button
  const handleMaximizeClick = async () => {
    try {
      const appWindow = Window.getCurrent(); // Get the current window instance using Window.getCurrent()
      await appWindow.toggleMaximize();
    } catch (error) {
      console.error("Failed to toggle maximize:", error);
    }
  };

  return (
    <div className="h-14 border-b border-gray-800 flex items-center justify-between px-4 bg-gray-900/95 backdrop-blur supports-[backdrop-filter]:bg-gray-900/60">
      {/* Left Section */}
      <div className="flex items-center gap-2">
        {/* Back Button */}
        <Button 
          variant="ghost" 
          size="icon" 
          className={`hover:bg-gray-800 ${!canGoBack ? 'opacity-50 cursor-not-allowed' : ''}`}
          onClick={goBack}
          disabled={!canGoBack}
          title="Go Back"
        >
          <ArrowLeft className={`h-4 w-4 ${!canGoBack ? 'text-gray-600' : 'text-gray-400'}`} />
        </Button>
        {/* Forward Button */}
        <Button 
          variant="ghost" 
          size="icon" 
          className={`hover:bg-gray-800 ${!canGoForward ? 'opacity-50 cursor-not-allowed' : ''}`}
          onClick={goForward}
          disabled={!canGoForward}
          title="Go Forward"
        >
          <ArrowRight className={`h-4 w-4 ${!canGoForward ? 'text-gray-600' : 'text-gray-400'}`} />
        </Button>
        <div className="flex items-center gap-2 ml-4">
          <span className="text-sm font-medium text-gray-300">{currentDirectoryName}</span>
        </div>
      </div>

      {/* Search Bar */}
      <div className="flex-1 max-w-xl mx-4">
        <div className="flex items-center gap-2">
          {/* Search Mode Toggle - Now beside the search box */}
          <div className="flex bg-gray-800/70 p-1 rounded-md border border-gray-700">
            <Button
              variant={searchMode === 'semantic' ? "default" : "ghost"}
              size="sm"
              className={`h-7 px-2 text-xs ${searchMode === 'semantic' ? 'bg-purple-700 text-purple-50' : 'text-gray-400 hover:text-gray-300'}`}
              onClick={() => setSearchMode('semantic')}
            >
              Semantic
            </Button>
            <Button
              variant={searchMode === 'filename' ? "default" : "ghost"}
              size="sm"
              className={`h-7 px-2 text-xs ${searchMode === 'filename' ? 'bg-purple-700 text-purple-50' : 'text-gray-400 hover:text-gray-300'}`}
              onClick={() => setSearchMode('filename')}
            >
              Filename
            </Button>
          </div>

          {/* Search Input with Button */}
          <div className="relative flex-1 flex items-center">
            <Search className="absolute left-2 top-2.5 h-4 w-4 text-gray-500" />
            <Input
              placeholder={searchMode === 'semantic' ? "Search Semantically..." : "Search by Filename..."}
              className="w-full pl-8 pr-16 bg-gray-800/50 border-gray-700 text-gray-300 focus:border-purple-600 focus:ring-purple-600"
              value={searchQuery}
              onChange={handleSearchInputChange}
              onKeyDown={handleSearchKeyDown}
            />
            
            {/* Clear button - only show when there's a query */}
            {searchQuery && (
              <button 
                className="absolute right-16 top-2.5 text-gray-400 hover:text-gray-300 focus:outline-none"
                onClick={handleClearSearch}
                title="Clear search"
              >
                <XCircle className="h-4 w-4" />
              </button>
            )}
            
            {/* Search button */}
            <Button 
              className="absolute right-1 top-1 h-8 px-2 text-xs bg-purple-800 hover:bg-purple-700 text-purple-100"
              onClick={handleSearchClick}
              disabled={isSearching || !searchQuery.trim()}
            >
              {isSearching ? 'Searching...' : 'Search'}
            </Button>
          </div>

          {/* File Type Filter - Only show for filename search */}
          {searchMode === 'filename' && (
            <div className="flex items-center">
              <Popover>
                <PopoverTrigger asChild>
                  <Filter className="h-4 w-4 text-gray-400 hover:text-gray-300" />
                </PopoverTrigger>
                <PopoverContent 
                  align="start" 
                  className="w-64 bg-gray-900 border-gray-700 text-gray-300 p-2 shadow-xl animate-in fade-in-0 zoom-in-95"
                  sideOffset={5}
                  onClick={(e: React.MouseEvent) => e.stopPropagation()} /* Prevent closing when clicking inside */
                >
                  <div>
                    <h3 className="text-sm font-medium text-gray-300 px-1 mb-2">File Types</h3>
                    <div className="p-1 grid grid-cols-2 gap-1 mt-1">
                      {availableFileCategories.map((category) => (
                        <div key={category} className="flex items-center p-1 rounded hover:bg-gray-800">
                          <Checkbox
                            id={`filter-${category}`}
                            checked={selectedFileCategories.includes(category)}
                            onCheckedChange={(checked: boolean | 'indeterminate') => {
                              if (checked === true) {
                                setSelectedFileCategories([...selectedFileCategories, category]);
                              } else {
                                setSelectedFileCategories(
                                  selectedFileCategories.filter((c) => c !== category)
                                );
                              }
                            }}
                            className="h-3.5 w-3.5 border-gray-600 rounded-sm"
                          />
                          <label 
                            htmlFor={`filter-${category}`}
                            className="ml-2 text-xs cursor-pointer w-full"
                          >
                            {category}
                          </label>
                        </div>
                      ))}
                    </div>
                    
                    <div className="h-px bg-gray-700 my-2" />
                    <h3 className="text-sm font-medium text-gray-300 px-1 mb-2">Fuzzy Matching</h3>
                    <div className="p-2">
                      <div className="w-full">
                        <div className="flex justify-between items-center">
                          <label className="text-xs">Max Edit Distance:</label>
                          <span className="text-xs font-medium bg-purple-800/70 text-purple-100 px-2 py-0.5 rounded-full">{maxDistance}</span>
                        </div>
                        <input
                          type="range"
                          min="0"
                          max="5"
                          value={maxDistance}
                          onChange={(e) => setMaxDistance(parseInt(e.target.value))}
                          className="w-full h-1.5 bg-gray-700 rounded-lg appearance-none cursor-pointer mt-2 accent-purple-600"
                        />
                        <div className="flex justify-between text-xs text-gray-400 mt-1 px-0.5">
                          <span>Exact Match</span>
                          <span>Fuzzy Match</span>
                        </div>
                      </div>
                    </div>
                    
                    <div className="h-px bg-gray-700 my-2" />
                    <div className="flex justify-end">
                      <Button 
                        variant="outline" 
                        size="sm"
                        className="h-7 text-xs border-gray-700 hover:bg-gray-800 hover:text-white"
                        onClick={() => setSelectedFileCategories([])}
                      >
                        Clear All Filters
                      </Button>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
            </div>
          )}
        </div>
      </div>

      {/* Right Section */}
      <div className="flex items-center gap-2">
        <Button 
          variant="ghost" 
          size="icon" 
          className="hover:bg-gray-800" 
          onClick={() => setViewMode('grid')} 
          title="Grid View"
        >
          <LayoutGrid className="h-4 w-4 text-gray-400" />
        </Button>
        <Button 
          variant="ghost" 
          size="icon" 
          className="hover:bg-gray-800" 
          onClick={() => setViewMode('list')} 
          title="List View"
        >
          <List className="h-4 w-4 text-gray-400" />
        </Button>
        <Button 
          variant="ghost" 
          size="icon" 
          className="hover:bg-gray-800" 
          onClick={handleMaximizeClick} 
          title="Maximize"
        >
          <Maximize className="h-4 w-4 text-gray-400" />
        </Button>
      </div>
    </div>
  );
};

export default Navbar;