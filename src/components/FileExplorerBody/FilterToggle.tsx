import { useState, useRef, useEffect } from 'react';
import { useAtom, useSetAtom } from 'jotai';
import { 
  availableFileCategoriesAtom, 
  folderFilterCategoriesAtom, 
  isFolderFilterActiveAtom,
  applyFolderFiltersAtom,
  clearFolderFiltersAtom
} from '../../store/atoms';
import { FileCategory } from '../../types/search';
import { Filter, Check, X } from 'lucide-react';
import { Badge } from '../../components/ui/badge';
import { Button } from '../../components/ui/button';

const FilterToggle = () => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  
  const [availableCategories] = useAtom(availableFileCategoriesAtom);
  const [activeFilters] = useAtom(folderFilterCategoriesAtom);
  const [isFilterActive] = useAtom(isFolderFilterActiveAtom);
  const applyFilters = useSetAtom(applyFolderFiltersAtom);
  const clearFilters = useSetAtom(clearFolderFiltersAtom);
  
  // Local state for selected categories before applying
  const [selectedCategories, setSelectedCategories] = useState<FileCategory[]>(activeFilters);
  
  // Update local selection when active filters change
  useEffect(() => {
    setSelectedCategories(activeFilters);
  }, [activeFilters]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const handleCategoryToggle = (category: FileCategory) => {
    setSelectedCategories(prev => {
      if (prev.includes(category)) {
        return prev.filter(c => c !== category);
      } else {
        return [...prev, category];
      }
    });
  };

  const handleApplyFilters = () => {
    applyFilters(selectedCategories);
    setIsOpen(false);
  };

  const handleClearFilters = () => {
    clearFilters();
    setSelectedCategories([]);
    setIsOpen(false);
  };

  const getCategoryIcon = (category: FileCategory) => {
    switch (category) {
      case 'Document': return '📄';
      case 'Image': return '🖼️';
      case 'Video': return '🎬';
      case 'Audio': return '🎵';
      case 'Archive': return '🗄️';
      case 'Code': return '💻';
      case 'Other': return '❓';
      default: return '📂';
    }
  };

  const getCategoryColor = (category: FileCategory) => {
    switch (category) {
      case 'Document': return 'bg-blue-900/20 text-blue-400 border-blue-500/30';
      case 'Image': return 'bg-purple-900/20 text-purple-400 border-purple-500/30';
      case 'Video': return 'bg-red-900/20 text-red-400 border-red-500/30';
      case 'Audio': return 'bg-green-900/20 text-green-400 border-green-500/30';
      case 'Archive': return 'bg-yellow-900/20 text-yellow-400 border-yellow-500/30';
      case 'Code': return 'bg-cyan-900/20 text-cyan-400 border-cyan-500/30';
      case 'Other': return 'bg-gray-900/20 text-gray-400 border-gray-500/30';
      default: return 'bg-gray-900/20 text-gray-400 border-gray-500/30';
    }
  };

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={`p-2 rounded flex items-center ${isFilterActive 
          ? "bg-purple-600 text-white" 
          : "bg-gray-700 text-gray-300 hover:bg-gray-600"}`}
        title="Filter files"
      >
        <Filter size={20} />
        {isFilterActive && activeFilters.length > 0 && (
          <span className="ml-1 text-xs font-medium">{activeFilters.length}</span>
        )}
      </button>
      
      {isOpen && (
        <div className="absolute left-0 top-full mt-2 z-50 w-72 bg-gray-800 rounded-lg border border-gray-700 shadow-xl overflow-hidden">
          <div className="flex items-center justify-between p-3 border-b border-gray-700">
            <div className="flex items-center">
              <Filter className="w-4 h-4 mr-2 text-gray-400" />
              <h3 className="text-sm font-medium text-gray-200">Filter Files</h3>
            </div>
            
            <div className="flex items-center space-x-2">
              {isFilterActive && (
                <Badge variant="outline" className="bg-purple-900/30 text-purple-300 border-purple-500/30 px-2 py-0.5 text-xs">
                  {activeFilters.length} active
                </Badge>
              )}
              <button 
                onClick={() => setIsOpen(false)}
                className="text-gray-400 hover:text-white p-1 rounded-full hover:bg-gray-700/50"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
          
          <div className="p-3">
            <div className="flex flex-wrap gap-1.5 mb-3">
              {availableCategories.map(category => (
                <button
                  key={category}
                  className={`
                    py-1 px-2 rounded-full border text-xs font-medium transition-all flex items-center
                    ${
                      selectedCategories.includes(category) 
                      ? getCategoryColor(category) + ' shadow-sm' 
                      : 'bg-gray-800 text-gray-400 border-gray-700/50 hover:bg-gray-700/50'
                    }
                  `}
                  onClick={() => handleCategoryToggle(category)}
                >
                  <span className="mr-1">{getCategoryIcon(category)}</span>
                  {category}
                  {selectedCategories.includes(category) && (
                    <Check className="ml-1 h-2.5 w-2.5" />
                  )}
                </button>
              ))}
            </div>
            
            <div className="flex justify-between gap-2 mt-4">
              <Button
                variant="outline"
                size="sm"
                onClick={handleClearFilters}
                className="text-gray-400 hover:text-white border-gray-700 hover:border-gray-600 text-xs"
                disabled={!isFilterActive && selectedCategories.length === 0}
              >
                Clear
              </Button>
              
              <Button
                variant="default"
                size="sm"
                onClick={handleApplyFilters}
                className="bg-purple-600 hover:bg-purple-700 text-white text-xs"
                disabled={selectedCategories.length === 0}
              >
                Apply Filters
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default FilterToggle;
