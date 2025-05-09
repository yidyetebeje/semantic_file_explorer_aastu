import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useAtomValue, useSetAtom } from 'jotai';
import {
  canGoBackAtom,
  canGoForwardAtom,
  goBackAtom,
  goForwardAtom
} from '../../store/atoms'; // Adjust path if needed

const NavigationIcons = () => {
  // Get navigation state and setters
  const canGoBack = useAtomValue(canGoBackAtom);
  const canGoForward = useAtomValue(canGoForwardAtom);
  const goBack = useSetAtom(goBackAtom);
  const goForward = useSetAtom(goForwardAtom);

  return (
    <div className="flex items-center">
      <button
        onClick={goBack}
        disabled={!canGoBack}
        // Add conditional styling for disabled state
        className={`p-1 hover:bg-gray-700 rounded-md transition ${!canGoBack ? 'opacity-50 cursor-not-allowed text-gray-500' : 'text-gray-300'}`}
        title="Go Back"
      >
        {/* Ensure icon color also reflects disabled state if needed */}
        <ChevronLeft size={16} />
      </button>
      <button
        onClick={goForward}
        disabled={!canGoForward}
        // Add conditional styling for disabled state
        className={`p-1 hover:bg-gray-700 rounded-md transition ml-1 ${!canGoForward ? 'opacity-50 cursor-not-allowed text-gray-500' : 'text-gray-300'}`}
        title="Go Forward"
      >
        {/* Ensure icon color also reflects disabled state if needed */}
        <ChevronRight size={16} />
      </button>
    </div>
  );
};

export default NavigationIcons;