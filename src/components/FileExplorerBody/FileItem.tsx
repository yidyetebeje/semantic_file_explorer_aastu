import FileIcon from './FileIcon';
import { convertFileSrc } from "@tauri-apps/api/core";
import { useState, useEffect } from 'react';

interface FileItemProps {
  name: string;
  type: string;
  size: number;
  thumbnail_path?: string | null;
  isSelected?: boolean;
}

const FileItem = ({ name, type, size, thumbnail_path, isSelected = false }: FileItemProps) => {
  const [assetUrl, setAssetUrl] = useState<string | null>(null);

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

  const iconSize = size;
  const fontSize = Math.max(10, size * 0.15);

  return (
    <div className={`relative cursor-pointer group flex flex-col items-center text-center ${isSelected ? 'scale-105' : ''}`}>
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
            type={type} 
            size={iconSize * 0.6}
            isDirectory={false} 
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
    </div>
  );
};

export default FileItem;