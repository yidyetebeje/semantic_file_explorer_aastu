import { useEffect, useState } from "react";
import { useSetAtom, useAtom } from 'jotai';
import { useAtomValue } from 'jotai'; 
import "./App.css";
import FileExplorer from "./components/FileExplorerBody/FileExplorer";
import MainLayout from "./components/layout/MainLayout";
import EnhancedSearchResults from "./components/Search/EnhancedSearchResults"; 
import IndexingStatus from "./components/IndexingStatus";
import { 
  loadHomeDirAtom, 
  loadLocationsOnInitAtom, 
  searchQueryAtom, 
  hasSearchedAtom,
  currentPathAtom
} from "./store/atoms"; 

function App() {
  const loadHomeDir = useSetAtom(loadHomeDirAtom);
  const loadLocations = useSetAtom(loadLocationsOnInitAtom);
  const [currentPath] = useAtom(currentPathAtom);
  const [currentView, setCurrentView] = useState<string>('');

  useEffect(() => {
    loadHomeDir();
    loadLocations();
    console.log("App mounted, initiating home directory and location load.");
  }, [loadHomeDir, loadLocations]);

  // Listen for changes to the currentPath atom
  useEffect(() => {
    // Check if the path is a special route
    if (currentPath === '/indexing-status') {
      setCurrentView('indexing-status');
    } else {
      setCurrentView('file-explorer');
    }
  }, [currentPath]);

  const renderContent = () => {
    switch (currentView) {
      case 'indexing-status':
        return <IndexingStatus />;
      case 'file-explorer':
      default:
        return (
          <>
            {/* Always render FileExplorer on the main route */}
            <FileExplorer />
            
            {/* Show search results only when a search has been performed */}
            <EnhancedSearchResults />
          </>
        );
    }
  };

  return (
    <main className="container mx-auto min-h-screen min-w-full bg-gray-900 bg-gradient-to-br from-gray-900 via-purple-900/30 to-gray-900">
      <MainLayout>
        {renderContent()}
      </MainLayout>
    </main>
  );
}

export default App;
