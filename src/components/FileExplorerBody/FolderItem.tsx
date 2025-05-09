import FileIcon from './FileIcon';

interface FolderItemProps {
  name: string;
  size: number;
  isSelected?: boolean;
}

const FolderItem = ({ name, size, isSelected = false }: FolderItemProps) => {
  return (
    <div className={`relative cursor-pointer group ${isSelected ? 'scale-105' : ''}`}>
      <div className={`flex flex-col items-center justify-center p-1 rounded-md ${isSelected ? 'bg-purple-600/30 ring-2 ring-purple-500' : 'group-hover:bg-white/10'} transition-all`}>
        <div className="relative">
          <FileIcon type="folder" size={size} isDirectory={true} />
          <div className={`absolute inset-0 ${isSelected ? 'bg-black/20' : 'bg-black/0 group-hover:bg-black/10'} rounded-lg transition-colors`} />
        </div>
        {/* Folder Name */}
        <p
          className={`text-sm ${isSelected ? 'text-white font-bold' : 'text-gray-300 group-hover:text-white group-hover:font-bold'} text-center truncate w-full mt-1`}
          style={{ fontSize: `${Math.max(10, size * 0.15)}px` }}
          title={name}
        >
          {name}
        </p>
      </div>
    </div>
  );
};

export default FolderItem;