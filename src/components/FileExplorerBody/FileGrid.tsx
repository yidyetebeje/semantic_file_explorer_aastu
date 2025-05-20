import { FileInfo, ViewMode } from '../../types/file';
import FileItem from './FileItem';
import FileList from './FileList';
import { Toaster } from 'react-hot-toast';

interface FileGridProps {
  files: FileInfo[];
  viewMode: ViewMode;
  fileSize: number;  
  gapSize: number;   
  selectedFile: FileInfo | null;
  onFileSelect?: (file: FileInfo) => void;
  onFileDoubleClick?: (file: FileInfo) => void;
  onFileOperation?: (type: string, file: FileInfo) => void;
}

const FileGrid = ({ 
  files, 
  viewMode, 
  fileSize, 
  gapSize, 
  selectedFile,
  onFileSelect, 
  onFileDoubleClick,
  onFileOperation 
}: FileGridProps) => {
  if (viewMode === 'list') {
    return <FileList 
      files={files} 
      onFileSelect={onFileSelect} 
      onFileDoubleClick={onFileDoubleClick} 
      fileSize={fileSize} 
      gapSize={gapSize}
      selectedFile={selectedFile}
    />;
  }

  const gapClass = `gap-${gapSize}`;

  return (
    <div className="w-full">
      <Toaster position="top-right" />
      <div className={`grid grid-cols-3 xs:grid-cols-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 2xl:grid-cols-12 ${gapClass} p-4 rounded-lg`}>
        {files.map((file) => {
          const isSelected = Boolean(selectedFile && selectedFile.path === file.path);
          
          return file.is_directory ? (
            <div key={file.path}>
              <FileItem
                file={file}
                size={fileSize}
                isSelected={isSelected}
                onSelect={onFileSelect}
                onDoubleClick={onFileDoubleClick}
                onFileOperation={onFileOperation}
              />
            </div>
          ) : (
            <div key={file.path}>
              <FileItem
                file={file}
                size={fileSize}
                isSelected={isSelected}
                onSelect={onFileSelect}
                onDoubleClick={onFileDoubleClick}
                onFileOperation={onFileOperation}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default FileGrid;