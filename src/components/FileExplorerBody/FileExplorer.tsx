import { useEffect } from "react";
import { useAtom, useAtomValue, useSetAtom } from 'jotai';
import { FileInfo } from "../../types/file";
import FileGrid from "./FileGrid";
import FilterToggle from "./FilterToggle";
import { Grid, List, X, Filter } from "lucide-react";
import { openPath } from "../../services/test";
import {
  viewModeAtom,
  fileSizeAtom,
  gapSizeAtom,
  selectedFileAtom,
  showInspectorAtom,
  currentPathAtom,
  isLoadingAtom,
  errorAtom,
  loadDirectoryAtom,
  visibleFilesAtom,
  navigateAtom,
  addToRecentItemsAtom,
  isFolderFilterActiveAtom,
  folderFilterCategoriesAtom
} from '../../store/atoms';

const FileExplorer = () => {
  const [viewMode, setViewMode] = useAtom(viewModeAtom);
  const [fileSize] = useAtom(fileSizeAtom);
  const [gapSize] = useAtom(gapSizeAtom);
  const [selectedFile, setSelectedFile] = useAtom(selectedFileAtom);
  const [showInspector, setShowInspector] = useAtom(showInspectorAtom);
  const [isFilterActive] = useAtom(isFolderFilterActiveAtom);
  const [activeFilters] = useAtom(folderFilterCategoriesAtom);

  const currentPath = useAtomValue(currentPathAtom);
  const files = useAtomValue(visibleFilesAtom);
  const isLoading = useAtomValue(isLoadingAtom);
  const error = useAtomValue(errorAtom);

  const loadDirectory = useSetAtom(loadDirectoryAtom);
  const navigate = useSetAtom(navigateAtom);
  const addToRecentItems = useSetAtom(addToRecentItemsAtom);

  useEffect(() => {
    console.log(`Path changed to: ${currentPath}, triggering directory load.`);
    loadDirectory();
  }, [currentPath, loadDirectory]);

  useEffect(() => {
    if (selectedFile && files.length > 0) {
      const matchingFile = files.find(file => file.path === selectedFile.path);
      
      if (matchingFile) {
        setSelectedFile(matchingFile);
        setShowInspector(true);
      }
    }
  }, [files, selectedFile, setSelectedFile, setShowInspector]);

  const handleFileSelect = (file: FileInfo) => {
    setSelectedFile(file);
    setShowInspector(true);
    console.log("Selected:", file);
  };

  const handleFileDoubleClick = async (file: FileInfo) => {
    console.log("Double-clicked:", file);
    if (file.is_directory) {
      navigate(file.path);
      // Add directory to recent items
      addToRecentItems({
        path: file.path,
        name: file.name,
        type: 'directory',
        fileType: 'directory'
      });
    } else {
      try {
        await openPath(file.path);
        // Add file to recent items
        addToRecentItems({
          path: file.path,
          name: file.name,
          type: 'file',
          fileType: file.file_type || ''
        });
      } catch (err) {
        console.error("Failed to open file:", err);
      }
    }
  };

  // Toggle inspector visibility
  const handleCloseInspector = () => {
    setShowInspector(false);
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-purple-950/30 to-gray-900 w-full">
      <div className="flex justify-between items-center mb-4 gap-2 pt-2 px-2">
        {/* Left side: Filter toggle */}
        {!isLoading && !error && (
          <FilterToggle />
        )}
        
        {/* Right side: View mode toggles */}
        <div className="flex gap-2">
          <button
            onClick={() => setViewMode("grid")}
            className={`p-2 rounded ${viewMode === "grid" ? "bg-blue-500 text-white" : "bg-gray-700 text-gray-300"}`}
          >
            <Grid size={20} />
          </button>
          <button
            onClick={() => setViewMode("list")}
            className={`p-2 rounded ${viewMode === "list" ? "bg-blue-500 text-white" : "bg-gray-700 text-gray-300"}`}
          >
            <List size={20} />
          </button>
        </div>
      </div>

      <div className="fixed top-0 left-0 right-0 z-10 ">
     

        <div className="w-full h-px bg-gray-500/20 backdrop-blur-sm"></div>
      </div>

      {showInspector && selectedFile && (
        <div className="fixed top-16 right-4 z-20 pt-4">
          <div className="bg-gray-800/80 backdrop-blur-sm rounded-lg p-4 shadow-lg w-64 border border-gray-700/50 relative">
            <button 
              onClick={handleCloseInspector}
              className="absolute top-2 right-2 text-gray-400 hover:text-gray-200 transition-colors"
              aria-label="Close inspector"
            >
              <X size={16} />
            </button>
            
            <p className="text-gray-200 font-bold text-center break-words pr-4">
              {selectedFile.name}
            </p>

            <div className="w-full h-px bg-gray-600 my-2"></div>

            <div className="space-y-1">
              <p className="text-gray-400 text-sm">Type: {selectedFile.file_type}</p>
              {selectedFile.size !== null && (
                 <p className="text-gray-400 text-sm">Size: {selectedFile.size.toLocaleString()} bytes</p>
              )}
              <p className="text-gray-400 text-sm">
                Modified:{" "}
                 {selectedFile.modified ? new Date(selectedFile.modified * 1000).toLocaleDateString() : 'N/A'}
               </p>
            </div>
            
            <button
              onClick={async () => {
                try {
                  await openPath(selectedFile.path);
                  // Add file to recent items when opened from inspector
                  addToRecentItems({
                    path: selectedFile.path,
                    name: selectedFile.name,
                    type: 'file',
                    fileType: selectedFile.file_type || ''
                  });
                } catch (err) {
                  console.error("Failed to open file:", err);
                }
              }}
              className="mt-4 w-full py-1.5 px-3 bg-purple-700 hover:bg-purple-600 text-white rounded-md text-sm transition-colors"
            >
              Open File
            </button>
          </div>
        </div>
      )}

      <div className="pt-16 px-4 pb-4">
        {isLoading && <p className="text-center text-gray-400">Loading...</p>}
        {error && <p className="text-center text-red-500">Error: {error}</p>}
        
        {!isLoading && !error && (
          <>
            {/* File Results with Filter Indicators */}
            <div className="mb-4">
              {isFilterActive && (
                <div className="flex items-center mb-4 px-1 py-2 bg-gray-800/30 rounded-lg border border-gray-700/30 text-sm text-gray-400">
                  <Filter className="h-4 w-4 mr-2 text-purple-400 flex-shrink-0" />
                  <span>Showing {files.length} {files.length === 1 ? 'item' : 'items'} matching </span>
                  <span className="font-medium text-purple-400 mx-1">{activeFilters.join(', ')}</span>
                  <span>filter{activeFilters.length !== 1 ? 's' : ''}</span>
                </div>
              )}
              
              <FileGrid
                files={files}
                viewMode={viewMode}
                fileSize={fileSize}
                gapSize={gapSize}
                onFileSelect={handleFileSelect}
                onFileDoubleClick={handleFileDoubleClick}
                selectedFile={selectedFile}
              />
            </div>
            
            {files.length === 0 && (
              <div className="text-center py-12 bg-gray-800/10 rounded-lg border border-gray-800/30">
                {isFilterActive ? (
                  <>
                    <p className="text-gray-300 text-lg mb-2">No matching files found</p>
                    <p className="text-gray-500">Try adjusting your filters or changing directories</p>
                  </>
                ) : (
                  <p className="text-gray-500">Directory is empty or not accessible.</p>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default FileExplorer;
