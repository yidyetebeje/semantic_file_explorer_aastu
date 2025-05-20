import { useAtom } from 'jotai';
import { format } from 'date-fns';
import { Clock, Search, File, Folder, X, ArrowLeft } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { 
  recentItemsAtom, 
  recentSearchesAtom, 
  navigateAtom, 
  searchQueryAtom, 
  triggerSearchAtom, 
  searchModeAtom,
  type RecentItem,
  type RecentSearchItem 
} from '../../store/atoms';
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
    <div className="px-5 py-4 hover:bg-gray-800/30 rounded-xl border border-gray-800 hover:border-gray-700 flex items-center group transition-all shadow-sm hover:shadow-md">
      <div className="mr-5 p-3 rounded-full bg-purple-900/20 text-purple-400 flex-shrink-0">
        <Icon className="h-6 w-6" />
      </div>
      <div 
        className="flex-grow cursor-pointer" 
        onClick={() => onNavigate(item.path, item.type)}
      >
        <div className="font-medium text-base text-gray-200">{item.name}</div>
        <div className="text-sm text-gray-400 flex items-center flex-wrap gap-2">
          <span className="bg-gray-800 px-2 py-0.5 rounded-full text-xs">{item.type === 'directory' ? 'Folder' : item.fileType || 'File'}</span>
          <span className="text-gray-500">•</span>
          <span>{formattedDate}</span>
        </div>
        <div className="text-xs text-gray-500 truncate mt-2">{item.path}</div>
      </div>
      <button 
        className="opacity-0 group-hover:opacity-100 text-gray-500 hover:text-white hover:bg-gray-700 transition-all p-2 rounded-full ml-2 flex-shrink-0"
        onClick={(e) => {
          e.stopPropagation();
          onRemove(item.path);
        }}
        aria-label="Remove from recents"
      >
        <X size={16} />
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
  const modeLabel = search.mode === 'semantic' ? 'Semantic Search' : 'Filename Search';
  const bgColorClass = search.mode === 'semantic' ? 'bg-blue-900/20 text-blue-400' : 'bg-green-900/20 text-green-400';
  const searchTypeClass = search.mode === 'semantic' ? 'bg-blue-900/30 text-blue-300' : 'bg-green-900/30 text-green-300';

  return (
    <div className="px-5 py-4 hover:bg-gray-800/30 rounded-xl border border-gray-800 hover:border-gray-700 flex items-center group transition-all shadow-sm hover:shadow-md">
      <div className={`mr-5 p-3 rounded-full ${bgColorClass} flex-shrink-0`}>
        <Search className="h-6 w-6" />
      </div>
      <div 
        className="flex-grow cursor-pointer" 
        onClick={() => onSelect(search.query, search.mode)}
      >
        <div className="font-medium text-base text-gray-200">"{search.query}"</div>
        <div className="text-sm text-gray-400 flex items-center flex-wrap gap-2">
          <span className={`px-2 py-0.5 rounded-full text-xs ${searchTypeClass}`}>{modeLabel}</span>
          <span className="text-gray-500">•</span>
          <span>{formattedDate}</span>
        </div>
      </div>
      <button 
        className="opacity-0 group-hover:opacity-100 text-gray-500 hover:text-white hover:bg-gray-700 transition-all p-2 rounded-full ml-2 flex-shrink-0"
        onClick={(e) => {
          e.stopPropagation();
          onRemove(search.query, search.mode);
        }}
        aria-label="Remove from recent searches"
      >
        <X size={16} />
      </button>
    </div>
  );
};

const RecentItemsPage = () => {
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

  const handleBackToFiles = () => {
    navigate('');
  };

  return (
    <div className="flex flex-col h-full max-w-5xl mx-auto py-6">
      <div className="flex flex-col mb-8">
        <div className="flex items-center mb-2">
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={handleBackToFiles} 
            className="text-gray-400 hover:text-white"
          >
            <ArrowLeft className="h-4 w-4 mr-1" />
            Back to Files
          </Button>
        </div>
        <h1 className="text-3xl font-bold text-white flex items-center">
          <Clock className="mr-3 h-7 w-7" />
          Recent Activity
        </h1>
      </div>

      <Tabs defaultValue="items" className="flex-1 w-full">
        <div className="mb-8">
          <TabsList className="bg-gray-800 border border-gray-800 w-full max-w-md flex mb-0 p-1 rounded-xl">
            <TabsTrigger 
              value="items" 
              className="flex-1 text-base py-3 px-6 text-white data-[state=active]:bg-gray-900 data-[state=active]:text-white rounded-lg transition-all"
            >
              Files & Folders
            </TabsTrigger>
            <TabsTrigger 
              value="searches" 
              className="flex-1 text-base py-3 px-6 text-white data-[state=active]:bg-gray-900 data-[state=active]:text-white rounded-lg transition-all"
            >
              Searches
            </TabsTrigger>
          </TabsList>
        </div>
        
        <TabsContent value="items" className="flex-1">
          {recentItems.length > 0 ? (
            <>
              <div className="flex justify-between items-center mb-6">
                <span className="text-sm text-gray-400 font-medium">
                  {recentItems.length} {recentItems.length === 1 ? 'item' : 'items'}
                </span>
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={clearAllRecentItems}
                  className="text-gray-400 hover:text-white border-gray-700 hover:border-gray-600"
                >
                  Clear all
                </Button>
              </div>
              <div className="space-y-3">
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
            <div className="text-center py-24 bg-gray-800/10 rounded-xl border border-gray-800/40">
              <Folder className="h-20 w-20 mx-auto text-gray-600 mb-6" />
              <p className="text-xl text-gray-300 mb-3">No recent files or folders</p>
              <p className="text-gray-500 max-w-md mx-auto">Your recently opened files and folders will appear here</p>
            </div>
          )}
        </TabsContent>
        
        <TabsContent value="searches" className="flex-1">
          {recentSearches.length > 0 ? (
            <>
              <div className="flex justify-between items-center mb-6">
                <span className="text-sm text-gray-400 font-medium">
                  {recentSearches.length} {recentSearches.length === 1 ? 'search' : 'searches'}
                </span>
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={clearAllRecentSearches}
                  className="text-gray-400 hover:text-white border-gray-700 hover:border-gray-600"
                >
                  Clear all
                </Button>
              </div>
              <div className="space-y-3">
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
            <div className="text-center py-24 bg-gray-800/10 rounded-xl border border-gray-800/40">
              <Search className="h-20 w-20 mx-auto text-gray-600 mb-6" />
              <p className="text-xl text-gray-300 mb-3">No recent searches</p>
              <p className="text-gray-500 max-w-md mx-auto">Your search history will appear here</p>
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default RecentItemsPage;
