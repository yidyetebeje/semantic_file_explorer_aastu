import { useState, useEffect } from 'react';
import { useAtom, useSetAtom } from 'jotai';
import { 
  availableFileCategoriesAtom, 
  folderFilterCategoriesAtom, 
  isFolderFilterActiveAtom,
  applyFolderFiltersAtom,
  clearFolderFiltersAtom,
  currentPathAtom
} from '../../store/atoms';
import { FileCategory } from '../../types/search';
import { Check, Filter, X } from 'lucide-react';
import { Button } from '../../components/ui/button';
import { Badge } from '../../components/ui/badge';

const FilterPanel = () => {
  const [availableCategories] = useAtom(availableFileCategoriesAtom);
  const [activeFilters] = useAtom(folderFilterCategoriesAtom);
  const [isFilterActive] = useAtom(isFolderFilterActiveAtom);
  const [currentPath] = useAtom(currentPathAtom);
  const applyFilters = useSetAtom(applyFolderFiltersAtom);
  const clearFilters = useSetAtom(clearFolderFiltersAtom);
  
  // Local state for selected categories before applying
  const [selectedCategories, setSelectedCategories] = useState<FileCategory[]>([]);
  
  // Reset selections when directory changes or when filters are cleared
  useEffect(() => {
    setSelectedCategories(activeFilters);
  }, [currentPath, activeFilters]);

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
  };

  const handleClearFilters = () => {
    clearFilters();
    setSelectedCategories([]);
  };

  const getCategoryIcon = (category: FileCategory) => {
    // You can use different icons for each category if needed
    switch (category) {
      case 'Document':
        return '📄';
      case 'Image':
        return '🖼️';
      case 'Video':
        return '🎬';
      case 'Audio':
        return '🎵';
      case 'Archive':
        return '🗄️';
      case 'Code':
        return '💻';
      case 'Other':
        return '❓';
      default:
        return '📂';
    }
  };

  const getCategoryColor = (category: FileCategory) => {
    switch (category) {
      case 'Document':
        return 'bg-blue-900/20 text-blue-400 border-blue-500/30';
      case 'Image':
        return 'bg-purple-900/20 text-purple-400 border-purple-500/30';
      case 'Video':
        return 'bg-red-900/20 text-red-400 border-red-500/30';
      case 'Audio':
        return 'bg-green-900/20 text-green-400 border-green-500/30';
      case 'Archive':
        return 'bg-yellow-900/20 text-yellow-400 border-yellow-500/30';
      case 'Code':
        return 'bg-cyan-900/20 text-cyan-400 border-cyan-500/30';
      case 'Other':
        return 'bg-gray-900/20 text-gray-400 border-gray-500/30';
      default:
        return 'bg-gray-900/20 text-gray-400 border-gray-500/30';
    }
  };

  return (
    <div className="mb-6 bg-gray-800/60 border border-gray-700/50 rounded-lg p-4 transition-all">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center">
          <Filter className="w-5 h-5 mr-2 text-gray-400" />
          <h3 className="text-base font-medium text-gray-200">Filter Files</h3>
        </div>
        
        <div className="flex items-center space-x-2">
          {isFilterActive && (
            <Badge variant="outline" className="bg-purple-900/30 text-purple-300 border-purple-500/30 px-2 py-1">
              {activeFilters.length} {activeFilters.length === 1 ? 'filter' : 'filters'} active
            </Badge>
          )}
          {isFilterActive && (
            <Button 
              variant="ghost" 
              size="icon" 
              className="h-7 w-7 text-gray-400 hover:text-white"
              onClick={handleClearFilters}
              title="Clear filters"
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
      </div>
      
      <div className="flex flex-wrap gap-2 mb-4">
        {availableCategories.map(category => (
          <button
            key={category}
            className={`
              py-1 px-3 rounded-full border text-sm font-medium transition-all flex items-center
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
              <Check className="ml-1 h-3 w-3" />
            )}
          </button>
        ))}
      </div>
      
      <div className="flex justify-end">
        <Button
          variant="default"
          size="sm"
          onClick={handleApplyFilters}
          className="bg-purple-600 hover:bg-purple-700 text-white"
          disabled={selectedCategories.length === 0 && !isFilterActive}
        >
          Apply Filters
        </Button>
      </div>
    </div>
  );
};

export default FilterPanel;
