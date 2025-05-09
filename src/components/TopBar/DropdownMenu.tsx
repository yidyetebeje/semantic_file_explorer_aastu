import { useAtom } from 'jotai';
import { fileSizeAtom, gapSizeAtom } from '../../store/atoms';

interface DropdownMenuProps {
  showDropdown: boolean;
  dropdownRef: React.RefObject<HTMLDivElement>;
  handleSizeChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handleGapChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

const DropdownMenu = ({
  showDropdown,
  dropdownRef,
  handleSizeChange,
  handleGapChange,
}: DropdownMenuProps) => {
  const [fileSize] = useAtom(fileSizeAtom);
  const [gapSize] = useAtom(gapSizeAtom);

  return (
    <>
      {showDropdown && (
        <div
          ref={dropdownRef}
          className="absolute bg-gray-800 rounded-md p-2 shadow-lg right-0 top-12 z-20"
        >
          <div className="mb-2">
            <label className="block text-gray-300 text-xs">File Size (px):</label>
            <input
              type="range"
              min="50"
              max="200"
              value={fileSize}
              onChange={handleSizeChange}
              className="w-full"
            />
          </div>
          <div>
            <label className="block text-gray-300 text-xs">Gap Size (px):</label>
            <input
              type="range"
              min="0"
              max="20"
              value={gapSize}
              onChange={handleGapChange}
              className="w-full"
            />
          </div>
        </div>
      )}
    </>
  );
};

export default DropdownMenu;