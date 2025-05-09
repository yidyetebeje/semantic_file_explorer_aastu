import { Eye } from 'lucide-react';

interface InspectorButtonProps {
  isInspectorActive: boolean;
  onInspectorToggle: () => void;
}

const InspectorButton = ({ isInspectorActive, onInspectorToggle }: InspectorButtonProps) => {
  return (
    <button
      className={`flex items-center p-1 rounded-md transition ${
        isInspectorActive ? 'bg-blue-500' : 'hover:bg-gray-700'
      }`}
      onClick={onInspectorToggle}
    >
      <Eye size={15} className={`${isInspectorActive ? 'text-white' : 'text-gray-300'}`} />
    </button>
  );
};

export default InspectorButton;