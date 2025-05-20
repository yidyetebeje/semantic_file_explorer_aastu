import FileIcon from './FileIcon';
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { basename, dirname, join } from "@tauri-apps/api/path";
import { useState, useEffect, useRef } from 'react';
import FileContextMenu from '../ContextMenu/FileContextMenu';
import { FileInfo } from '../../types/file';
import { toast } from 'react-hot-toast';

// Helper function to format file size
const formatSize = (sizeInBytes: number): string => {
  if (sizeInBytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(sizeInBytes) / Math.log(k));
  if (i < 0 || i >= sizes.length) return parseFloat(sizeInBytes.toFixed(2)) + ' Bytes'; // Fallback for very small/large
  return parseFloat((sizeInBytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

interface FileItemProps {
  file: FileInfo;
  size: number;
  isSelected?: boolean;
  onSelect?: (file: FileInfo) => void;
  onDoubleClick?: (file: FileInfo) => void;
  onFileOperation?: (type: string, file: FileInfo) => void;
}

const FileItem = ({ 
  file, 
  size, 
  isSelected = false, 
  onSelect, 
  onDoubleClick,
  onFileOperation
}: FileItemProps) => {
  const [assetUrl, setAssetUrl] = useState<string | null>(null);
  const [menuPosition, setMenuPosition] = useState<{ x: number, y: number } | null>(null);
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const itemRef = useRef<HTMLDivElement>(null);
  const [isPropertiesModalOpen, setIsPropertiesModalOpen] = useState(false);
  const [detailedFileInfo, setDetailedFileInfo] = useState<any | null>(null); // Consider a more specific type
  const propertiesModalRef = useRef<HTMLDivElement>(null);
  const { name, file_type, thumbnail_path, path } = file;

  useEffect(() => {
    if (thumbnail_path) {
      try {
        const url = convertFileSrc(thumbnail_path);
        setAssetUrl(url);
      } catch (error) {
        console.error("Error converting thumbnail path:", thumbnail_path, error);
        setAssetUrl(null);
      }
    } else {
      setAssetUrl(null);
    }
  }, [thumbnail_path]);

  useEffect(() => {
    // Close context menu when clicking outside
    const handleClickOutside = (event: MouseEvent) => {
      if (itemRef.current && !itemRef.current.contains(event.target as Node)) {
        setIsMenuOpen(false);
      }
    };

    // Add event listener when menu is open
    if (isMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isMenuOpen]);

  // Effect to close Properties modal on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (propertiesModalRef.current && !propertiesModalRef.current.contains(event.target as Node)) {
        setIsPropertiesModalOpen(false);
      }
    };
    if (isPropertiesModalOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isPropertiesModalOpen]);

  // Effect to fetch detailed file info when Properties modal opens
  useEffect(() => {
    if (isPropertiesModalOpen && file.path) {
      invoke<any>('get_item_info', { path: file.path }) // Specify return type for invoke if known
        .then(info => {
          setDetailedFileInfo(info);
        })
        .catch(error => {
          console.error('Failed to get file info for modal:', error);
          toast.error('Could not load file properties.');
          setIsPropertiesModalOpen(false); // Close modal on error
        });
    } else if (!isPropertiesModalOpen) {
      setDetailedFileInfo(null); // Clear data when modal closes or is not open
    }
  }, [isPropertiesModalOpen, file.path]);

  const iconSize = size;
  const fontSize = Math.max(10, size * 0.15);

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setMenuPosition({ x: e.clientX, y: e.clientY });
    console.log(e.clientX, e.clientY);
    setIsMenuOpen(true);
  };

  const handleClick = () => {
    if (onSelect) {
      onSelect(file);
    }
  };

  const handleDoubleClick = () => {
    if (onDoubleClick) {
      onDoubleClick(file);
    }
  };

  // File operations
  const handleDelete = async () => {
    try {
      await invoke('delete_item', { path: file.path });
      toast.success(`Deleted ${file.name}`);
      if (onFileOperation) {
        onFileOperation('delete', file);
      }
    } catch (error) {
      console.error('Failed to delete file:', error);
      toast.error(`Failed to delete ${file.name}`);
    }
  };

  const handleRename = async (newName: string) => {
    try {
      // Get the parent directory and construct the new path
      const parentDir = await dirname(file.path);
      const newPath = await join(parentDir, newName);
      
      await invoke('rename_item', { path: file.path, newName });
      toast.success(`Renamed ${file.name} to ${newName}`);
      if (onFileOperation) {
        onFileOperation('rename', {
          ...file,
          name: newName,
          path: newPath
        });
      }
    } catch (error) {
      console.error('Failed to rename file:', error);
      toast.error(`Failed to rename ${file.name}`);
    }
  };

  const handleCopy = async (destination: string) => {
    try {
      // Get the base filename
      const fileName = await basename(file.path);
      
      // Construct the full destination path
      const destPath = await join(destination, fileName);
      
      await invoke('copy_item', { source: file.path, destination: destPath });
      toast.success(`Copied ${file.name}`);
      if (onFileOperation) {
        onFileOperation('copy', file);
      }
    } catch (error) {
      console.error('Failed to copy file:', error);
      toast.error(`Failed to copy ${file.name}`);
    }
  };

  const handleShowPropertiesModal = () => {
    setIsMenuOpen(false); // Close context menu first
    setIsPropertiesModalOpen(true);
  };

  const handleMove = async (destination: string) => {
    try {
      // Get the base filename
      const fileName = await basename(file.path);
      
      // Construct the full destination path
      const destPath = await join(destination, fileName);
      
      await invoke('move_item', { source: file.path, destination: destPath });
      toast.success(`Moved ${file.name}`);
      if (onFileOperation) {
        onFileOperation('move', file);
      }
    } catch (error) {
      console.error('Failed to move file:', error);
      toast.error(`Failed to move ${file.name}`);
    }
  };

  return (
    <div 
      ref={itemRef}
      className={`relative cursor-pointer group flex flex-col items-center text-center ${isSelected ? 'scale-105' : ''}`}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onContextMenu={handleContextMenu}
    >
      <div 
        className={`relative flex items-center justify-center rounded-md ${isSelected ? 'bg-purple-600/30 ring-2 ring-purple-500' : 'group-hover:bg-white/10'} transition-all mb-1 overflow-hidden`}
        style={{ width: `${iconSize}px`, height: `${iconSize}px` }}
      >
        {assetUrl ? (
          <img 
            src={assetUrl} 
            alt={name} 
            className="object-cover w-full h-full"
            loading="lazy"
          />
        ) : (
          <FileIcon 
            type={file_type} 
            size={iconSize * 0.6}
            isDirectory={file.is_directory} 
          />
        )}
      </div>
      
      <p
        className={`text-sm ${isSelected ? 'text-white font-bold' : 'text-gray-300 group-hover:text-white group-hover:font-bold'} text-center truncate w-full`}
        style={{ fontSize: `${fontSize}px` }}
        title={name}
      >
        {name}
      </p>

      {isMenuOpen && menuPosition && (
        <FileContextMenu
          file={file}
          x={menuPosition.x}
          y={menuPosition.y}
          onClose={() => setIsMenuOpen(false)}
          onDelete={handleDelete}
          onRename={handleRename}
          onCopy={handleCopy}
          onMove={handleMove}
          onShowProperties={handleShowPropertiesModal} // Added prop
          isOpen={isMenuOpen}
        />
      )}

      {/* Properties Modal */}
      {isPropertiesModalOpen && detailedFileInfo && (
        <div
          ref={propertiesModalRef}
          className="absolute left-full ml-2 top-0 w-80 bg-gray-800 border border-gray-700 rounded-md shadow-lg p-4 z-30"
          onClick={(e) => e.stopPropagation()} // Prevent click from bubbling to itemRef's handler
        >
          <h3 className="text-white font-medium border-b border-gray-700 pb-2 mb-3">File Information</h3>
          <div className="text-sm text-gray-300 space-y-2">
            <p><span className="text-purple-400">Name:</span> {detailedFileInfo.name || file.name}</p>
            <p><span className="text-purple-400">Type:</span> {detailedFileInfo.is_dir ? 'Directory' : (detailedFileInfo.file_type || 'File')}</p>
            {!detailedFileInfo.is_dir && typeof detailedFileInfo.size === 'number' && (
              <p><span className="text-purple-400">Size:</span> {formatSize(detailedFileInfo.size)}</p>
            )}
            {detailedFileInfo.modified && (
              <p><span className="text-purple-400">Modified:</span> {new Date(detailedFileInfo.modified * 1000).toLocaleString()}</p>
            )}
            {detailedFileInfo.created && (
              <p><span className="text-purple-400">Created:</span> {new Date(detailedFileInfo.created * 1000).toLocaleString()}</p>
            )}
            {detailedFileInfo.path && <p><span className="text-purple-400">Path:</span> <span className="text-xs break-all">{detailedFileInfo.path}</span></p>}
            {typeof detailedFileInfo.readonly === 'boolean' && (
              <p><span className="text-purple-400">Read-only:</span> {detailedFileInfo.readonly ? 'Yes' : 'No'}</p>
            )}
          </div>
          <div className="mt-4 flex justify-end">
            <button 
              onClick={() => setIsPropertiesModalOpen(false)}
              className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white rounded-md text-sm"
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default FileItem;