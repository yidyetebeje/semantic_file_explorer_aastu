// RecentPanel component to show recently accessed files and searches
import { useAtom } from 'jotai';
import { recentItemsAtom, recentSearchesAtom, navigateAtom, searchQueryAtom, triggerSearchAtom, searchModeAtom } from '../../store/atoms';
import { type RecentItem, type RecentSearchItem } from '../../store/atoms';
import { format } from 'date-fns';
import { Clock, Search, File, Folder, X } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { openPath } from '../../services/test';

interface RecentItemProps {
  item: RecentItem;
  onNavigate: (path: string, type: 'file' | 'directory') => void;
  onRemove: (path: string) => void;
}

const RecentItemComponent = ({ item, onNavigate, onRemove }: RecentItemProps) => {
  const Icon = item.type === 'directory' ? Folder : File;
  const formattedDate = format(new Date(item.accessedAt), 'MMM d, yyyy h:mm a');

  return (
    <div className="py-2 px-3 hover:bg-gray-800 rounded-md flex items-center group">
      <Icon className="mr-2 h-4 w-4 text-gray-400 flex-shrink-0" />
      <div 
        className="flex-grow truncate cursor-pointer" 
        onClick={() => onNavigate(item.path, item.type)}
      >
        <div className="font-medium text-sm text-gray-300 truncate">{item.name}</div>
        <div className="text-xs text-gray-500 truncate">{formattedDate}</div>
      </div>
      <button 
        className="opacity-0 group-hover:opacity-100 text-gray-500 hover:text-gray-300 transition-opacity"
        onClick={(e) => {
          e.stopPropagation();
          onRemove(item.path);
        }}
        aria-label="Remove from recents"
      >
        <X size={14} />
      </button>
    </div>
  );
};

interface RecentSearchProps {
  search: RecentSearchItem;
  onSelect: (query: string, mode: 'semantic' | 'filename') => void;
  onRemove: (query: string, mode: 'semantic' | 'filename') => void;
}

const RecentSearchComponent = ({ search, onSelect, onRemove }: RecentSearchProps) => {
  const formattedDate = format(new Date(search.timestamp), 'MMM d, yyyy h:mm a');
  const modeLabel = search.mode === 'semantic' ? 'Semantic' : 'Filename';

  return (
    <div className="py-2 px-3 hover:bg-gray-800 rounded-md flex items-center group">
      <Search className="mr-2 h-4 w-4 text-gray-400 flex-shrink-0" />
      <div 
        className="flex-grow truncate cursor-pointer" 
        onClick={() => onSelect(search.query, search.mode)}
      >
        <div className="font-medium text-sm text-gray-300 truncate">{search.query}</div>
        <div className="text-xs text-gray-500 flex items-center">
          <span className="mr-2">{modeLabel}</span>
          <span>{formattedDate}</span>
        </div>
      </div>
      <button 
        className="opacity-0 group-hover:opacity-100 text-gray-500 hover:text-gray-300 transition-opacity"
        onClick={(e) => {
          e.stopPropagation();
          onRemove(search.query, search.mode);
        }}
        aria-label="Remove from recent searches"
      >
        <X size={14} />
      </button>
    </div>
  );
};

const RecentPanel = () => {
  const [recentItems, setRecentItems] = useAtom(recentItemsAtom);
  const [recentSearches, setRecentSearches] = useAtom(recentSearchesAtom);
  const [, setSearchQuery] = useAtom(searchQueryAtom);
  const [, setSearchMode] = useAtom(searchModeAtom);
  const navigate = useAtom(navigateAtom)[1];
  const triggerSearch = useAtom(triggerSearchAtom)[1];

  const handleNavigate = async (path: string, type: 'file' | 'directory') => {
    if (type === 'directory') {
      navigate(path);
    } else {
      try {
        await openPath(path);
      } catch (err) {
        console.error("Failed to open file:", err);
      }
    }
  };

  const handleRemoveRecentItem = (path: string) => {
    setRecentItems(items => items.filter(item => item.path !== path));
  };

  const handleSearchSelect = (query: string, mode: 'semantic' | 'filename') => {
    setSearchQuery(query);
    setSearchMode(mode);
    triggerSearch();
  };

  const handleRemoveRecentSearch = (query: string, mode: 'semantic' | 'filename') => {
    setRecentSearches(searches => 
      searches.filter(s => !(s.query === query && s.mode === mode))
    );
  };

  const clearAllRecentItems = () => {
    setRecentItems([]);
  };

  const clearAllRecentSearches = () => {
    setRecentSearches([]);
  };

  return (
    <div className="w-full h-full flex flex-col">
      <div className="border-b border-gray-800 pb-3 pt-2 px-4">
        <h2 className="text-lg font-semibold text-gray-200 flex items-center">
          <Clock className="mr-2 h-5 w-5" />
          Recent Activity
        </h2>
      </div>

      <Tabs defaultValue="items" className="flex-1 overflow-hidden flex flex-col">
        <TabsList className="p-2 bg-transparent w-full">
          <TabsTrigger value="items" className="flex-1">Files & Folders</TabsTrigger>
          <TabsTrigger value="searches" className="flex-1">Searches</TabsTrigger>
        </TabsList>
        
        <TabsContent value="items" className="flex-1 overflow-y-auto p-2 data-[state=inactive]:hidden">
          {recentItems.length > 0 ? (
            <>
              <div className="flex justify-between items-center px-2 mb-2">
                <span className="text-xs text-gray-500">Recent Files & Folders</span>
                <button 
                  className="text-xs text-gray-500 hover:text-gray-300"
                  onClick={clearAllRecentItems}
                >
                  Clear all
                </button>
              </div>
              <div className="space-y-1">
                {recentItems.map(item => (
                  <RecentItemComponent 
                    key={item.path} 
                    item={item} 
                    onNavigate={handleNavigate}
                    onRemove={handleRemoveRecentItem}
                  />
                ))}
              </div>
            </>
          ) : (
            <div className="text-center text-gray-500 py-8">
              <p>No recent items</p>
              <p className="text-xs mt-1">Your recently opened files and folders will appear here</p>
            </div>
          )}
        </TabsContent>
        
        <TabsContent value="searches" className="flex-1 overflow-y-auto p-2 data-[state=inactive]:hidden">
          {recentSearches.length > 0 ? (
            <>
              <div className="flex justify-between items-center px-2 mb-2">
                <span className="text-xs text-gray-500">Recent Searches</span>
                <button 
                  className="text-xs text-gray-500 hover:text-gray-300"
                  onClick={clearAllRecentSearches}
                >
                  Clear all
                </button>
              </div>
              <div className="space-y-1">
                {recentSearches.map((search, index) => (
                  <RecentSearchComponent 
                    key={`${search.query}-${search.mode}-${index}`} 
                    search={search} 
                    onSelect={handleSearchSelect}
                    onRemove={handleRemoveRecentSearch}
                  />
                ))}
              </div>
            </>
          ) : (
            <div className="text-center text-gray-500 py-8">
              <p>No recent searches</p>
              <p className="text-xs mt-1">Your recent searches will appear here</p>
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default RecentPanel;
