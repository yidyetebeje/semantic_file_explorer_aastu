import { useState, FC, useRef, useEffect } from 'react';

import { ViewMode } from '../../types/file';
import NavigationIcons from './NavigationIcons';
import ViewModeButtons from './ViewModeButtons';
import RightSectionIcons from './RightSectionIcons';
import DropdownMenu from './DropdownMenu';

interface TopBarProps {
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  onSizeChange: (size: number) => void;
  onGapChange: (gap: number) => void;
  onInspectorToggle: () => void;
  isInspectorActive: boolean;
  currentPath: string;
}

const TopBar: FC<TopBarProps> = ({
  viewMode,
  onViewModeChange,
  onSizeChange,
  onGapChange,
  onInspectorToggle,
  isInspectorActive,
  currentPath,
}) => {
  const [showDropdown, setShowDropdown] = useState<boolean>(false);

  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false);
      }
    };

    if (showDropdown) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showDropdown]);

  const handleSizeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newSize = parseInt(e.target.value);
    onSizeChange(newSize);
  };

  const handleGapChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newGap = parseInt(e.target.value);
    onGapChange(newGap);
  };

  return (
    <div className="bg-gray-800/50 p-1 flex items-center rounded-b-lg shadow-lg fixed top-0 left-0 right-0 z-10">
      <NavigationIcons />
      <div className="flex items-center flex-grow mx-2">
        <input
          type="text"
          value={currentPath}
          readOnly
          className="bg-gray-700 text-gray-300 rounded-md p-1 focus:outline-none focus:ring-2 focus:ring-blue-500 text-xs flex-grow min-w-0"
        />
      </div>
      <ViewModeButtons viewMode={viewMode} onViewModeChange={onViewModeChange} />
      <div className="h-4 border-l border-gray-600 mx-2" />
      <RightSectionIcons
        showDropdown={showDropdown}
        setShowDropdown={setShowDropdown}
        isInspectorActive={isInspectorActive}
        onInspectorToggle={onInspectorToggle}
      />
      <DropdownMenu
        showDropdown={showDropdown}
        dropdownRef={dropdownRef}
        handleSizeChange={handleSizeChange}
        handleGapChange={handleGapChange}
      />
    </div>
  );
};

export default TopBar;