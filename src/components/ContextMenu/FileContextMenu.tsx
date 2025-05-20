import React, { useState } from 'react';
import { 
  Copy, 
  Scissors, 
  Trash2, 
  Edit2, 
  Info, 
  ExternalLink
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { save as saveDialog } from '@tauri-apps/plugin-dialog';
import { FileInfo } from '../../types/file';

interface FileContextMenuProps {
  file: FileInfo;
  x: number;
  y: number;
  onClose: () => void;
  onDelete: () => void;
  onRename: (newName: string) => void;
  onCopy: (destination: string) => void;
  onMove: (destination: string) => void;
  onShowProperties: () => void; // Added for Properties modal
  isOpen: boolean;
}

const FileContextMenu: React.FC<FileContextMenuProps> = ({
  file,
  x,
  y,
  onClose,
  onDelete,
  onRename,
  onCopy,
  onMove,
  onShowProperties, // Added
  isOpen
}) => {
  const [isRenaming, setIsRenaming] = useState(false);
  const [newName, setNewName] = useState(file.name);
  if (!isOpen) return null;

  const handleCopy = async () => {
    try {
      const selectedPath = await saveDialog({
        title: 'Select Destination',
        defaultPath: file.is_directory ? file.name : '',
      });
      
      if (selectedPath) {
        onCopy(selectedPath);
      }
    } catch (error) {
      console.error('Error selecting destination for copy:', error);
    }
    onClose();
  };

  const handleMove = async () => {
    try {
      const selectedPath = await saveDialog({
        title: 'Select Destination',
        defaultPath: file.is_directory ? file.name : '',
      });
      
      if (selectedPath) {
        onMove(selectedPath);
      }
    } catch (error) {
      console.error('Error selecting destination for move:', error);
    }
    onClose();
  };

  const handleDelete = () => {
    if (confirm(`Are you sure you want to delete "${file.name}"?`)) {
      onDelete();
    }
    onClose();
  };

  const handleRenameClick = () => {
    setIsRenaming(true);
  };

  const handleRenameSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newName.trim() && newName !== file.name) {
      onRename(newName);
    }
    setIsRenaming(false);
    onClose();
  };

  const handleOpen = () => {
    invoke('open_path_command', { path: file.path })
      .catch(error => console.error('Failed to open file:', error));
    onClose();
  };

  // Calculate position to ensure menu stays within viewport
  // Determine dimensions based on current view (Properties modal is no longer handled here)
  const baseWidth = isRenaming ? 224 : 224; // Rename width or ContextMenu width
  const baseHeightEstimate = isRenaming ? 100 : 300; // Rename height or ContextMenu height
  const buffer = 10; // Buffer from window edges

  let finalLeft = x;
  let finalTop = y;

  // Adjust if it goes off the right edge
  if (x + baseWidth + buffer > window.innerWidth) {
    finalLeft = x - baseWidth;
  }
  // Adjust if it goes off the left edge (or was pushed off by right edge adjustment)
  if (finalLeft < buffer) {
    finalLeft = buffer;
  }
  // Final check to ensure it's not wider than window if pushed from both sides (rare)
  if (finalLeft + baseWidth + buffer > window.innerWidth) {
    finalLeft = window.innerWidth - baseWidth - buffer;
  }

  // Adjust if it goes off the bottom edge
  if (y + baseHeightEstimate + buffer > window.innerHeight) {
    finalTop = y - baseHeightEstimate;
  }
  // Adjust if it goes off the top edge (or was pushed off by bottom edge adjustment)
  if (finalTop < buffer) {
    finalTop = buffer;
  }
  // Final check for height
  if (finalTop + baseHeightEstimate + buffer > window.innerHeight) {
    finalTop = window.innerHeight - baseHeightEstimate - buffer;
  }

  // Diagnostic logs for positioning
  console.log('[FileContextMenu] Input Coords (cursor):', { x, y });
  console.log('[FileContextMenu] Calculated finalLeft, finalTop:', { finalLeft, finalTop });
  console.log('[FileContextMenu] Window Inner W/H:', { innerWidth: window.innerWidth, innerHeight: window.innerHeight });
  console.log('[FileContextMenu] Menu W/H:', { baseWidth, baseHeightEstimate });

  const menuStyle: React.CSSProperties = {
    position: 'fixed',
    left: finalLeft,
    top: finalTop,
    // zIndex is handled by Tailwind classes on the divs
  };

  if (isRenaming) {
    return (
      <div 
        className="absolute bg-gray-800 border border-gray-700 rounded-md shadow-lg p-3 w-full z-[9997]"
        style={menuStyle}
      >
        <form onSubmit={handleRenameSubmit}>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            className="w-full bg-gray-700 text-white p-2 rounded-md outline-none focus:ring-2 focus:ring-purple-500"
            autoFocus
          />
          <div className="flex justify-end space-x-2 mt-3">
            <button
              type="button"
              onClick={() => {
                setIsRenaming(false);
                onClose();
              }}
              className="px-3 py-1 bg-gray-700 hover:bg-gray-600 text-white rounded-md text-sm"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white rounded-md text-sm"
            >
              Rename
            </button>
          </div>
        </form>
      </div>
    );
  }

  return (
    <div 
      className="absolute bg-gray-800 border border-gray-700 rounded-md shadow-lg py-1 w-56 z-[9998]"
      style={menuStyle}
    >
      <button 
        onClick={handleOpen}
        className="w-full text-left px-3 py-2 text-sm flex items-center text-gray-200 hover:bg-gray-600 hover:text-white"
      >
        <ExternalLink size={16} className="mr-2 text-purple-400" /> 
        Open
      </button>
      
      <div className="border-t border-gray-700 my-1"></div>
      
      <button 
        onClick={handleCopy}
        className="w-full text-left px-3 py-2 text-sm flex items-center text-gray-200 hover:bg-gray-600 hover:text-white"
      >
        <Copy size={16} className="mr-2 text-blue-400" /> 
        Copy to...
      </button>
      
      <button 
        onClick={handleMove}
        className="w-full text-left px-3 py-2 text-sm flex items-center text-gray-200 hover:bg-gray-600 hover:text-white"
      >
        <Scissors size={16} className="mr-2 text-yellow-400" /> 
        Move to...
      </button>
      
      <button 
        onClick={handleRenameClick}
        className="w-full text-left px-3 py-2 text-sm flex items-center text-gray-200 hover:bg-gray-600 hover:text-white"
      >
        <Edit2 size={16} className="mr-2 text-green-400" /> 
        Rename
      </button>
      
      <button 
        onClick={handleDelete}
        className="w-full text-left px-3 py-2 text-sm flex items-center text-gray-200 hover:bg-gray-600 hover:text-white"
      >
        <Trash2 size={16} className="mr-2 text-red-400" /> 
        Delete
      </button>
      
      <div className="border-t border-gray-700 my-1"></div>
      
      <button 
        onClick={() => { onShowProperties(); onClose(); }}
        className="w-full text-left px-3 py-2 text-sm flex items-center text-gray-200 hover:bg-gray-600 hover:text-white"
      >
        <Info size={16} className="mr-2 text-purple-400" /> 
        Properties
      </button>
    </div>
  );
};

export default FileContextMenu;
