import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { useAtom } from 'jotai';
import { invoke } from '@tauri-apps/api/core';
import { ChatWindow } from './ChatWindow';
import { chatMessagesAtom, chatInputAtom, isChatOpenAtom, ChatMessage } from '@/store/chatAtoms';

// Mock Jotai
vi.mock('jotai');
const mockUseAtom = useAtom as jest.Mock;

// Mock Tauri's invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
const mockInvoke = invoke as jest.Mock;

// Mock SearchResultItem
vi.mock('./SearchResultItem', () => ({
  SearchResultItem: vi.fn(({ result, type }) => (
    <div data-testid={`search-result-${type}-${result.name}`}>Mocked SearchResultItem: {result.name}</div>
  )),
}));


describe('ChatWindow', () => {
  let mockSetMessages = vi.fn();
  let mockSetInput = vi.fn();
  let mockSetIsOpen = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockSetMessages = vi.fn();
    mockSetInput = vi.fn();
    mockSetIsOpen = vi.fn();

    // Default mock implementation for useAtom
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [[], mockSetMessages];
      if (atom === chatInputAtom) return ['', mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen]; // Assume chat is open for most tests
      return [undefined, vi.fn()];
    });
  });

  test('renders chat window when open', () => {
    render(<ChatWindow />);
    expect(screen.getByText('Chatbot')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument();
  });

  test('does not render when closed', () => {
    mockUseAtom.mockImplementation((atom) => {
      if (atom === isChatOpenAtom) return [false, mockSetIsOpen];
      return [undefined, vi.fn()];
    });
    const { container } = render(<ChatWindow />);
    expect(container.firstChild).toBeNull();
  });

  test('displays messages from chatMessagesAtom', () => {
    const messages: ChatMessage[] = [
      { id: '1', text: 'Hello User', sender: 'bot', timestamp: new Date() },
      { id: '2', text: 'Hello Bot', sender: 'user', timestamp: new Date() },
    ];
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [messages, mockSetMessages];
      if (atom === chatInputAtom) return ['', mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });
    render(<ChatWindow />);
    expect(screen.getByText('Hello User')).toBeInTheDocument();
    expect(screen.getByText('Hello Bot')).toBeInTheDocument();
  });

  test('updates input field on change', () => {
    render(<ChatWindow />);
    const inputField = screen.getByPlaceholderText('Type a message...');
    fireEvent.change(inputField, { target: { value: 'Test message' } });
    expect(mockSetInput).toHaveBeenCalledWith('Test message');
  });

  test('sends a regular message and displays bot response', async () => {
    mockInvoke.mockResolvedValueOnce('Gemini says hi'); // Mock Gemini response
    
    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') {
        currentMessages = fn(currentMessages);
      } else {
        currentMessages = fn;
      }
    });
     mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return ['User message', mockSetInput]; // User has typed a message
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);
    const sendButton = screen.getByRole('button'); // Assuming send button is the only one or identifiable
    fireEvent.click(sendButton);

    // Check if user message is added
    expect(mockSetMessages).toHaveBeenCalled();
    // Check if input is cleared
    expect(mockSetInput).toHaveBeenCalledWith('');
    // Check if invoke was called for Gemini
    expect(mockInvoke).toHaveBeenCalledWith('send_message_to_gemini', { message: 'User message' });

    // Wait for bot response to be processed
    await waitFor(() => {
      expect(mockSetMessages).toHaveBeenCalledTimes(3); // Initial, user message, bot message
    });
  });

  test('sends a /search command and displays system message', async () => {
    mockInvoke.mockResolvedValueOnce({ semantic_search: {results: []}, filename_search: {results: []} }); // Mock search response
    
    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
       if (typeof fn === 'function') {
        currentMessages = fn(currentMessages);
      } else {
        currentMessages = fn;
      }
    });
     mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return ['/search test query', mockSetInput]; // User has typed a search command
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);
    const sendButton = screen.getByRole('button');
    fireEvent.click(sendButton);
    
    expect(mockSetMessages).toHaveBeenCalled();
    expect(mockSetInput).toHaveBeenCalledWith('');
    expect(mockInvoke).toHaveBeenCalledWith('search_files', { query: 'test query' });

    await waitFor(() => {
      // Initial, user message, "Searching for..." system message, "Search results:" system message
      expect(mockSetMessages).toHaveBeenCalledTimes(4); 
    });
  });
  
  
  test('renders search results when message metadata contains them', async () => {
    const searchData = {
      semantic_search: { results: [{ name: 'semantic_doc.txt', file_path: '/sem/doc.txt', score: 0.9 }] },
      filename_search: { results: [{ name: 'filename_doc.txt', file_path: '/fname/doc.txt' }] }
    };
    const messagesWithSearchResults: ChatMessage[] = [
      { 
        id: 'search-res-1', 
        text: 'Search results:', 
        sender: 'system', 
        timestamp: new Date(),
        metadata: { searchResults: searchData }
      }
    ];

    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [messagesWithSearchResults, mockSetMessages];
      if (atom === chatInputAtom) return ['', mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);

    expect(screen.getByText('Search results:')).toBeInTheDocument();
    // Check if our mocked SearchResultItem is rendered for semantic results
    expect(screen.getByTestId('search-result-semantic-semantic_doc.txt')).toBeInTheDocument();
    expect(screen.getByText('Mocked SearchResultItem: semantic_doc.txt')).toBeInTheDocument();
    // Check for filename results
    expect(screen.getByTestId('search-result-filename-filename_doc.txt')).toBeInTheDocument();
    expect(screen.getByText('Mocked SearchResultItem: filename_doc.txt')).toBeInTheDocument();
  });

  test('handles errors from send_message_to_gemini invoke', async () => {
    mockInvoke.mockRejectedValueOnce('Network Error');
    
    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') currentMessages = fn(currentMessages); else currentMessages = fn;
    });
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return ['Hello', mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);
    fireEvent.click(screen.getByRole('button')); // Send button

    await waitFor(() => {
      // User message, then system error message
      expect(mockSetMessages).toHaveBeenCalledTimes(2); 
    });
     // Assuming currentMessages is updated by the mockSetMessages
    const lastMessage = currentMessages[currentMessages.length - 1];
    expect(lastMessage.sender).toBe('system');
    expect(lastMessage.text).toContain('Error: Network Error');
  });

  test('handles errors from search_files invoke', async () => {
    mockInvoke.mockRejectedValueOnce('Search Failed');
    
    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') currentMessages = fn(currentMessages); else currentMessages = fn;
    });
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return ['/search error test', mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });
    
    render(<ChatWindow />);
    fireEvent.click(screen.getByRole('button'));

    await waitFor(() => {
      // User message, "Searching for..." message, then system error message
      expect(mockSetMessages).toHaveBeenCalledTimes(3);
    });
    const lastMessage = currentMessages[currentMessages.length - 1];
    expect(lastMessage.sender).toBe('system');
    expect(lastMessage.text).toContain('Error searching files: Search Failed');
  });

  // Tests for /summarize command
  test('sends /summarize command and displays success message', async () => {
    const filePath = '/path/to/document.txt';
    const summaryText = 'This is a summary of the document.';
    mockInvoke.mockResolvedValueOnce(summaryText); // Mock for summarize_file

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') currentMessages = fn(currentMessages); else currentMessages = fn;
    });
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return [`/summarize ${filePath}`, mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);
    fireEvent.click(screen.getByRole('button')); // Send button

    // 1. User message
    // 2. System message "Summarizing file..."
    // 3. Bot message with summary
    
    expect(mockInvoke).toHaveBeenCalledWith('summarize_file', { filePath });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(3));

    const userMessage = currentMessages.find(msg => msg.sender === 'user' && msg.text === `/summarize ${filePath}`);
    const systemMessage = currentMessages.find(msg => msg.sender === 'system' && msg.text.includes(`Summarizing file: "${filePath}"`));
    const botMessage = currentMessages.find(msg => msg.sender === 'bot' && msg.text.includes(summaryText));

    expect(userMessage).toBeDefined();
    expect(systemMessage).toBeDefined();
    expect(botMessage).toBeDefined();
    expect(botMessage?.metadata?.isSummary).toBe(true);
  });

  test('sends /summarize command and displays error message on failure', async () => {
    const filePath = '/path/to/error_document.txt';
    const errorMessage = 'Could not summarize the document.';
    mockInvoke.mockRejectedValueOnce(errorMessage); // Mock for summarize_file failure

    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') currentMessages = fn(currentMessages); else currentMessages = fn;
    });
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return [`/summarize ${filePath}`, mockSetInput];
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);
    fireEvent.click(screen.getByRole('button')); // Send button

    // 1. User message
    // 2. System message "Summarizing file..."
    // 3. System message with error
    expect(mockInvoke).toHaveBeenCalledWith('summarize_file', { filePath });
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(3));
    
    const userMessage = currentMessages.find(msg => msg.sender === 'user' && msg.text === `/summarize ${filePath}`);
    const systemSummarizingMessage = currentMessages.find(msg => msg.sender === 'system' && msg.text.includes(`Summarizing file: "${filePath}"`));
    const systemErrorMessage = currentMessages.find(msg => msg.sender === 'system' && msg.text.includes(`Error summarizing file "${filePath}": ${errorMessage}`));

    expect(userMessage).toBeDefined();
    expect(systemSummarizingMessage).toBeDefined();
    expect(systemErrorMessage).toBeDefined();
  });

  test('/summarize command with no file path shows usage message', async () => {
    let currentMessages: ChatMessage[] = [];
    mockSetMessages.mockImplementation((fn) => {
      if (typeof fn === 'function') currentMessages = fn(currentMessages); else currentMessages = fn;
    });
    mockUseAtom.mockImplementation((atom) => {
      if (atom === chatMessagesAtom) return [currentMessages, mockSetMessages];
      if (atom === chatInputAtom) return ['/summarize ', mockSetInput]; // Empty file path
      if (atom === isChatOpenAtom) return [true, mockSetIsOpen];
      return [undefined, vi.fn()];
    });

    render(<ChatWindow />);
    fireEvent.click(screen.getByRole('button'));

    // 1. User message
    // 2. System message with usage information
    await waitFor(() => expect(mockSetMessages).toHaveBeenCalledTimes(2));
    expect(mockInvoke).not.toHaveBeenCalledWith('summarize_file', expect.anything());
    
    const userMessage = currentMessages.find(msg => msg.sender === 'user' && msg.text === '/summarize ');
    const systemMessage = currentMessages.find(msg => msg.sender === 'system' && msg.text === 'Usage: /summarize <file_path>');
    
    expect(userMessage).toBeDefined();
    expect(systemMessage).toBeDefined();
  });
});
