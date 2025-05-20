import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { useSetAtom } from 'jotai';
import { invoke } from '@tauri-apps/api/core';
import { SearchResultItem } from './SearchResultItem';
import { chatMessagesAtom, ChatMessage } from '@/store/chatAtoms';

// Mock Jotai
vi.mock('jotai');
const mockUseSetAtom = useSetAtom as jest.Mock;

// Mock Tauri's invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
const mockInvoke = invoke as jest.Mock;

describe('SearchResultItem', () => {
  let mockSetMessages = vi.fn();
  const mockResult = {
    file_path: '/path/to/test_file.txt',
    name: 'test_file.txt',
    score: 0.85,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockSetMessages = vi.fn();
    mockUseSetAtom.mockReturnValue(mockSetMessages);
  });

  test('renders search result information correctly', () => {
    render(<SearchResultItem result={mockResult} type="semantic" />);
    expect(screen.getByText(mockResult.name)).toBeInTheDocument();
    expect(screen.getByText(mockResult.file_path)).toBeInTheDocument();
    expect(screen.getByText(`Score: ${mockResult.score.toFixed(4)} (semantic)`)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'View Content' })).toBeInTheDocument();
  });

  test('calls get_document_content and updates messages on "View Content" click', async () => {
    mockInvoke.mockResolvedValueOnce('This is the content of the test file.'); // Mock file content response

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
       if (typeof fn === 'function') {
        currentMessages = fn(currentMessages);
      } else {
        currentMessages = fn;
      }
    });

    render(<SearchResultItem result={mockResult} type="filename" />);
    const viewContentButton = screen.getByRole('button', { name: 'View Content' });
    fireEvent.click(viewContentButton);

    // Check for "Fetching content..." message
    expect(mockSetMessages).toHaveBeenCalled(); 
    // Check if invoke was called for get_document_content
    expect(mockInvoke).toHaveBeenCalledWith('get_document_content', { filePath: mockResult.file_path });

    // Wait for content to be fetched and displayed
    await waitFor(() => {
      // Should be called for "Fetching" and then for the content itself
      expect(mockSetMessages).toHaveBeenCalledTimes(2); 
    });
  });

  test('handles error when fetching document content', async () => {
    mockInvoke.mockRejectedValueOnce('Failed to read file'); // Mock error response

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') {
        currentMessages = fn(currentMessages);
      } else {
        currentMessages = fn;
      }
    });
    
    render(<SearchResultItem result={mockResult} type="semantic" />);
    const viewContentButton = screen.getByRole('button', { name: 'View Content' });
    fireEvent.click(viewContentButton);

    expect(mockSetMessages).toHaveBeenCalled();
    expect(mockInvoke).toHaveBeenCalledWith('get_document_content', { filePath: mockResult.file_path });

    await waitFor(() => {
      // "Fetching..." message, then error message
      expect(mockSetMessages).toHaveBeenCalledTimes(2); 
    });
  });
});
