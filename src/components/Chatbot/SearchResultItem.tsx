import React from 'react';
import { Button } from '@/components/ui/button';
import { ChatMessage, chatMessagesAtom } from '@/store/chatAtoms';
import { useSetAtom } from 'jotai';
import { invoke } from '@tauri-apps/api/core';

interface SearchResult {
  file_path: string;
  name: string;
  score?: number; // For semantic search
  // Add other relevant fields from your backend search result structure
}

interface SearchResultItemProps {
  result: SearchResult;
  type: 'semantic' | 'filename';
}

export const SearchResultItem: React.FC<SearchResultItemProps> = ({ result, type }) => {
  const setMessages = useSetAtom(chatMessagesAtom);

  // Helper function to add messages, can be outside if preferred and setMessages is passed
  const addChatMessageHelper = (
    text: string,
    sender: 'user' | 'bot' | 'system',
    metadata?: any 
  ) => {
    setMessages((prev) => [
      ...prev,
      {
        id: Date.now().toString() + Math.random().toString(36).substring(2,7), // More unique ID
        text,
        sender,
        timestamp: new Date(),
        metadata,
      },
    ]);
  };

  const handleSummarizeClickInternal = () => {
    addChatMessageHelper(
      `Summarizing file: "${result.name}"...`,
      'system'
    );

    invoke<string>('summarize_file', { filePath: result.file_path })
      .then(summary => {
        addChatMessageHelper(
          `Summary for ${result.name}:\n${summary}`,
          'bot',
          { isSummary: true, filePath: result.file_path } 
        );
      })
      .catch(error => {
        console.error(`Error summarizing file ${result.name}:`, error);
        addChatMessageHelper(
          `Error summarizing file "${result.name}": ${error}`,
          'system'
        );
      });
  };

  const handleResultClick = async () => {
    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        text: `Fetching content for: ${result.name}...`,
        sender: 'system',
        timestamp: new Date(),
      },
    ]);

    try {
      const content = await invoke<string>('get_document_content', { filePath: result.file_path });
      // Display a snippet or summary if content is too long
      const snippet = content.length > 500 ? content.substring(0, 497) + '...' : content;
      setMessages(prev => [
        ...prev,
        {
          id: (Date.now() + 1).toString(),
          text: `Content for ${result.name}:\n\n${snippet}`,
          sender: 'system',
          timestamp: new Date(),
          metadata: { filePath: result.file_path, fullContent: content }
        },
      ]);
    } catch (error) {
      console.error('Error fetching document content:', error);
      setMessages(prev => [
        ...prev,
        {
          id: (Date.now() + 1).toString(),
          text: `Error fetching content for ${result.name}: ${error}`,
          sender: 'system',
          timestamp: new Date(),
        },
      ]);
    }
  };

  return (
    <div className="mb-2 p-2 border rounded-md dark:border-gray-600">
      <p className="text-sm font-medium">{result.name}</p>
      <p className="text-xs text-gray-500 dark:text-gray-400">{result.file_path}</p>
      {result.score && <p className="text-xs">Score: {result.score.toFixed(4)} ({type})</p>}
      <Button variant="link" size="sm" onClick={handleResultClick} className="p-0 h-auto text-xs mr-2">
        View Content
      </Button>
      <Button variant="link" size="sm" onClick={handleSummarizeClickInternal} className="p-0 h-auto text-xs mr-2">
        Summarize
      </Button>
      <Button variant="link" size="sm" onClick={handleOpenFileClick} className="p-0 h-auto text-xs mr-2">
        Open File
      </Button>
      <Button variant="link" size="sm" onClick={handleCopyPathClick} className="p-0 h-auto text-xs">
        Copy Path
      </Button>
    </div>
  );

  function handleOpenFileClick() {
    addChatMessageHelper(
      `Attempting to open: "${result.name}"...`,
      'system'
    );
    invoke<void>('open_file_external', { path: result.file_path })
      .then(() => {
        addChatMessageHelper(
          `Successfully requested to open "${result.name}". Check your system.`,
          'system'
        );
      })
      .catch(error => {
        console.error(`Error opening file ${result.name}:`, error);
        addChatMessageHelper(
          `Error opening file "${result.name}": ${error}`,
          'system'
        );
      });
  }

  function handleCopyPathClick() {
    invoke<void>('copy_to_clipboard', { text: result.file_path })
      .then(() => {
        addChatMessageHelper(
          `Path for "${result.name}" copied to clipboard.`,
          'system'
        );
      })
      .catch(error => {
        console.error(`Error copying path for ${result.name}:`, error);
        addChatMessageHelper(
          `Error copying path for "${result.name}": ${error}`,
          'system'
        );
      });
  }
};
