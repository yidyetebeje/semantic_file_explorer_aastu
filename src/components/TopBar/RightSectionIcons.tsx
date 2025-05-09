import { Key, Tag, Eye } from 'lucide-react';

interface RightSectionIconsProps {
  showDropdown: boolean;
  setShowDropdown: (show: boolean) => void;
  isInspectorActive: boolean;
  onInspectorToggle: () => void;
}

const RightSectionIcons = ({
  showDropdown,
  setShowDropdown,
  isInspectorActive,
  onInspectorToggle,
}: RightSectionIconsProps) => {
  return (
    <div className="flex items-center gap-1 ml-2">
      <button className="flex items-center p-1 hover:bg-gray-700 rounded-md transition">
        <Key size={15} className="text-gray-300" />
      </button>
      <button className="flex items-center p-1 hover:bg-gray-700 rounded-md transition">
        <Tag size={15} className="text-gray-300" />
      </button>
      <button
        className="flex items-center p-1 hover:bg-gray-700 rounded-md transition"
        onClick={() => setShowDropdown(!showDropdown)}
      >
        <img
          src="/images/4737438_equalizer_filter_filtering_mixer_sorting_icon.png"
          alt="Sorting Icon"
          className="w-4 h-4 filter invert"
        />
      </button>
      <div className="h-4 border-l border-gray-600 mx-2" />
      <button
        className={`flex items-center p-1 rounded-md transition ${
          isInspectorActive ? 'bg-blue-500' : 'hover:bg-gray-700'
        }`}
        onClick={onInspectorToggle}
      >
        <Eye size={15} className={`${isInspectorActive ? 'text-white' : 'text-gray-300'}`} />
      </button>
    </div>
  );
};

export default RightSectionIcons;