import { FC } from 'react';
// Import specific lucide icons needed
import {
  LucideIcon, // Base type
  Folder,
  FileText,
  Image as ImageIcon, // Rename Image to avoid conflict
  Video as VideoIcon,
  Music,
  FileArchive, 
  FileQuestion,
  File // Generic fallback file
} from 'lucide-react';
import { cn } from "@/lib/utils"; // Import cn utility if available, otherwise just concatenate strings

interface FileIconProps {
  type: string; // File type string from backend
  isDirectory: boolean;
  size?: number;
  className?: string;
}

// Maps file type string to a lucide icon component
const getLucideIconForFileType = (fileType: string): LucideIcon => {
  switch (fileType?.toLowerCase()) {
    case 'image':
    case 'png':
    case 'jpg':
    case 'jpeg':
    case 'gif':
    case 'svg':
    case 'webp':
    case 'bmp':
      return ImageIcon;
    case 'video':
    case 'mp4':
    case 'mov':
    case 'avi':
    case 'mkv':
    case 'webm':
      return VideoIcon;
    case 'audio':
    case 'mp3':
    case 'wav':
    case 'ogg':
    case 'flac':
    case 'aac':
      return Music;
    case 'pdf':
      return FileText; // Or a specific PDF icon if preferred
    case 'text':
    case 'txt':
    case 'md':
    case 'json':
    case 'yaml':
    case 'xml':
    case 'csv':
    case 'log':
    case 'html':
    case 'css':
    case 'js':
    case 'ts':
    case 'jsx':
    case 'tsx':
    case 'sh':
    case 'py':
    case 'rs':
      return FileText;
    case 'archive':
    case 'zip':
    case 'rar':
    case '7z':
    case 'tar':
    case 'gz':
    case 'bz2':
      return FileArchive;
    // Add more specific types as needed
    default:
      return File; // Generic file icon as fallback
      // return FileQuestion; // Alternative fallback
  }
};

const FileIcon: FC<FileIconProps> = ({ type, isDirectory, size = 24, className = "" }) => { 
  // Determine the appropriate Lucide icon component
  const IconComponent: LucideIcon = isDirectory 
    ? Folder 
    : getLucideIconForFileType(type);

  // Determine the color class based on whether it's a directory
  const colorClass = isDirectory ? "text-blue-500" : "text-white";

  // Render the chosen Lucide icon component
  return (
    <IconComponent 
      size={size} 
      // Combine conditional color class with incoming className
      className={cn(colorClass, className)} 
      aria-label={isDirectory ? 'Folder' : type} // Accessibility
      strokeWidth={1.5} // Optional: adjust stroke width 
    />
  );
};

export default FileIcon;