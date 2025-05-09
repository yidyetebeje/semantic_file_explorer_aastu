import { Grid, List } from 'lucide-react';

interface ViewModeButtonsProps {
  viewMode: 'grid' | 'list';
  onViewModeChange: (mode: 'grid' | 'list') => void;
}

const ViewModeButtons = ({ viewMode, onViewModeChange }: ViewModeButtonsProps) => {
  return (
    <div className="flex items-center gap-1">
      <button
        onClick={() => onViewModeChange('grid')}
        className={`p-2 rounded ${viewMode === 'grid' ? 'bg-blue-500' : 'bg-gray-700'}`}
      >
        <Grid size={15} className="text-gray-300" />
      </button>
      <button
        onClick={() => onViewModeChange('list')}
        className={`p-2 rounded ${viewMode === 'list' ? 'bg-blue-500' : 'bg-gray-700'}`}
      >
        <List size={15} className="text-gray-300" />
      </button>
    </div>
  );
};

export default ViewModeButtons;