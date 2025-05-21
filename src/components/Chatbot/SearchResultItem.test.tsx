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
    // Verify new buttons are rendered
    expect(screen.getByRole('button', { name: 'Summarize' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Open File' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Copy Path' })).toBeInTheDocument();
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

  // --- Tests for Summarize button ---
  test('"Summarize" button click invokes summarize_file and updates messages on success', async () => {
    const summaryText = "This is a summary.";
    mockInvoke.mockResolvedValueOnce(summaryText); // For summarize_file

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      currentMessages = typeof fn === 'function' ? fn(currentMessages) : fn;
    });

    render(<SearchResultItem result={mockResult} type="semantic" />);
    fireEvent.click(screen.getByRole('button', { name: 'Summarize' }));

    expect(mockInvoke).toHaveBeenCalledWith('summarize_file', { filePath: mockResult.file_path });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(2)); // "Summarizing..." and actual summary

    expect(currentMessages.find(msg => msg.text.includes(`Summarizing file: "${mockResult.name}"`))).toBeDefined();
    const summaryMessage = currentMessages.find(msg => msg.sender === 'bot' && msg.text.includes(summaryText));
    expect(summaryMessage).toBeDefined();
    expect(summaryMessage?.metadata?.isSummary).toBe(true);
  });

  test('"Summarize" button click handles error from summarize_file', async () => {
    const errorMsg = "Failed to summarize";
    mockInvoke.mockRejectedValueOnce(errorMsg); // For summarize_file

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      currentMessages = typeof fn === 'function' ? fn(currentMessages) : fn;
    });

    render(<SearchResultItem result={mockResult} type="semantic" />);
    fireEvent.click(screen.getByRole('button', { name: 'Summarize' }));

    expect(mockInvoke).toHaveBeenCalledWith('summarize_file', { filePath: mockResult.file_path });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(2)); // "Summarizing..." and error

    expect(currentMessages.find(msg => msg.text.includes(`Summarizing file: "${mockResult.name}"`))).toBeDefined();
    expect(currentMessages.find(msg => msg.text.includes(`Error summarizing file "${mockResult.name}": ${errorMsg}`))).toBeDefined();
  });

  // --- Tests for Open File button ---
  test('"Open File" button click invokes open_file_external and updates messages on success', async () => {
    mockInvoke.mockResolvedValueOnce(undefined); // For open_file_external

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      currentMessages = typeof fn === 'function' ? fn(currentMessages) : fn;
    });

    render(<SearchResultItem result={mockResult} type="semantic" />);
    fireEvent.click(screen.getByRole('button', { name: 'Open File' }));

    expect(mockInvoke).toHaveBeenCalledWith('open_file_external', { path: mockResult.file_path });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(2)); // "Attempting to open..." and success/error

    expect(currentMessages.find(msg => msg.text.includes(`Attempting to open: "${mockResult.name}"`))).toBeDefined();
    expect(currentMessages.find(msg => msg.text.includes(`Successfully requested to open "${mockResult.name}"`))).toBeDefined();
  });

  test('"Open File" button click handles error from open_file_external', async () => {
    const errorMsg = "Cannot open";
    mockInvoke.mockRejectedValueOnce(errorMsg); // For open_file_external

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      currentMessages = typeof fn === 'function' ? fn(currentMessages) : fn;
    });
    
    render(<SearchResultItem result={mockResult} type="semantic" />);
    fireEvent.click(screen.getByRole('button', { name: 'Open File' }));

    expect(mockInvoke).toHaveBeenCalledWith('open_file_external', { path: mockResult.file_path });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(2));

    expect(currentMessages.find(msg => msg.text.includes(`Attempting to open: "${mockResult.name}"`))).toBeDefined();
    expect(currentMessages.find(msg => msg.text.includes(`Error opening file "${mockResult.name}": ${errorMsg}`))).toBeDefined();
  });

  // --- Tests for Copy Path button ---
  test('"Copy Path" button click invokes copy_to_clipboard and updates messages on success', async () => {
    mockInvoke.mockResolvedValueOnce(undefined); // For copy_to_clipboard

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      currentMessages = typeof fn === 'function' ? fn(currentMessages) : fn;
    });

    render(<SearchResultItem result={mockResult} type="semantic" />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy Path' }));

    expect(mockInvoke).toHaveBeenCalledWith('copy_to_clipboard', { text: mockResult.file_path });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(1)); // Only one message for copy path

    expect(currentMessages.find(msg => msg.text.includes(`Path for "${mockResult.name}" copied to clipboard.`))).toBeDefined();
  });

  test('"Copy Path" button click handles error from copy_to_clipboard', async () => {
    const errorMsg = "Clipboard fail";
    mockInvoke.mockRejectedValueOnce(errorMsg); // For copy_to_clipboard

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      currentMessages = typeof fn === 'function' ? fn(currentMessages) : fn;
    });

    render(<SearchResultItem result={mockResult} type="semantic" />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy Path' }));

    expect(mockInvoke).toHaveBeenCalledWith('copy_to_clipboard', { text: mockResult.file_path });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(1));

    expect(currentMessages.find(msg => msg.text.includes(`Error copying path for "${mockResult.name}": ${errorMsg}`))).toBeDefined();
  });
});
