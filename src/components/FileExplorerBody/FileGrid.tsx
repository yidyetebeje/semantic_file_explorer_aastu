import { FileInfo, ViewMode } from '../../types/file';
import FileItem from './FileItem';
import FolderItem from './FolderItem';
import FileList from './FileList';

interface FileGridProps {
  files: FileInfo[];
  viewMode: ViewMode;
  fileSize: number;  
  gapSize: number;   
  selectedFile: FileInfo | null;
  onFileSelect?: (file: FileInfo) => void;
  onFileDoubleClick?: (file: FileInfo) => void;
}

const FileGrid = ({ 
  files, 
  viewMode, 
  fileSize, 
  gapSize, 
  selectedFile,
  onFileSelect, 
  onFileDoubleClick 
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
      <div className={`grid grid-cols-3 xs:grid-cols-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 2xl:grid-cols-12 ${gapClass} p-4 rounded-lg`}>
        {files.map((file) => {
          const isSelected = Boolean(selectedFile && selectedFile.path === file.path);
          
          return file.is_directory ? (
            <div
              key={file.path}
              className={`cursor-pointer transition-all ${isSelected ? 'scale-105' : ''}`}
              onClick={() => onFileSelect?.(file)}
              onDoubleClick={() => onFileDoubleClick?.(file)}
            >
              <FolderItem
                name={file.name}
                size={fileSize}
                isSelected={isSelected}
              />
            </div>
          ) : (
            <div
              key={file.path}
              className={`cursor-pointer transition-all ${isSelected ? 'scale-105' : ''}`}
              onClick={() => onFileSelect?.(file)}
              onDoubleClick={() => onFileDoubleClick?.(file)}
            >
              <FileItem
                name={file.name}
                type={file.file_type}
                size={fileSize}
                thumbnail_path={file.thumbnail_path}
                isSelected={isSelected}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default FileGrid;