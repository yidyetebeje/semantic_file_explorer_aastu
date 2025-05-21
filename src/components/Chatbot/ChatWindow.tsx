import React from 'react';
import { useAtom } from 'jotai';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
// Use a more common icon if PaperPlaneIcon is specific to Radix and not available project-wide
// For example, using Send from lucide-react
import { Send as PaperPlaneIcon } from 'lucide-react'; 
import { chatMessagesAtom, chatInputAtom, isChatOpenAtom, ChatMessage } from '@/store/chatAtoms';
import { invoke } from '@tauri-apps/api/core';
import { SearchResultItem } from './SearchResultItem'; // Import the new component

export const ChatWindow: React.FC = () => {
  const [messages, setMessages] = useAtom(chatMessagesAtom);
  const [input, setInput] = useAtom(chatInputAtom);
  const [isOpen, setIsOpen] = useAtom(isChatOpenAtom);

  const handleSend = () => {
    if (input.trim() === '') return;
    // Add user message
    const userMessage = { id: Date.now().toString(), text: input, sender: 'user' as const, timestamp: new Date() };
    setMessages((prev) => [...prev, userMessage]);
    setInput('');

    // Call backend
    if (input.startsWith('/search ')) {
      const query = input.substring('/search '.length);
      setMessages(prev => [...prev, { id: (Date.now() + 1).toString(), text: `Searching for: "${query}"`, sender: 'system', timestamp: new Date() }]);
      invoke<any>('search_files', { query }) // Expecting JSON response
        .then(results => {
          setMessages(prev => [
            ...prev,
            { 
              id: (Date.now() + 2).toString(), 
              text: "Search results:", 
              sender: 'system', 
              timestamp: new Date(),
              metadata: { searchResults: results }
            }
          ]);
        })
        .catch(error => {
          console.error("Error searching files:", error);
          setMessages(prev => [
            ...prev,
            { id: (Date.now() + 2).toString(), text: `Error searching files: ${error}`, sender: 'system', timestamp: new Date() }
          ]);
        });
    } else if (input.startsWith('/summarize ')) {
      const filePath = input.substring('/summarize '.length).trim();
      if (filePath === '') {
        setMessages(prev => [...prev, { id: (Date.now() + 1).toString(), text: "Usage: /summarize <file_path>", sender: 'system', timestamp: new Date() }]);
        return;
      }
      setMessages(prev => [...prev, { id: (Date.now() + 1).toString(), text: `Summarizing file: "${filePath}"...`, sender: 'system', timestamp: new Date() }]);
      
      invoke<string>('summarize_file', { filePath })
        .then(summary => {
          setMessages(prev => [
            ...prev,
            { id: (Date.now() + 2).toString(), text: `Summary for ${filePath}:\n${summary}`, sender: 'bot' as const, timestamp: new Date(), metadata: {isSummary: true} }
          ]);
        })
        .catch(error => {
          console.error(`Error summarizing file ${filePath}:`, error);
          setMessages(prev => [
            ...prev,
            { id: (Date.now() + 2).toString(), text: `Error summarizing file "${filePath}": ${error}`, sender: 'system', timestamp: new Date() }
          ]);
        });
    } else {
      // Regular message to Gemini
      invoke<string>('send_message_to_gemini', { message: input })
        .then((response) => {
          setMessages((prev) => [
            ...prev,
            { id: (Date.now() + 1).toString(), text: response, sender: 'bot' as const, timestamp: new Date() },
          ]);
        })
        .catch((error) => {
          console.error("Error sending message to Gemini:", error);
          setMessages((prev) => [
            ...prev,
            { id: (Date.now() + 1).toString(), text: `Error: ${error}`, sender: 'system' as const, timestamp: new Date() },
          ]);
        });
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="fixed bottom-20 right-5 w-96 h-[500px] bg-white dark:bg-gray-800 shadow-xl rounded-lg flex flex-col border border-gray-300 dark:border-gray-700">
      <div className="p-4 border-b dark:border-gray-700">
        <h2 className="text-lg font-semibold">Chatbot</h2>
      </div>
      <ScrollArea className="flex-grow p-4">
        {messages.map((msg) => {
          if (msg.metadata?.searchResults) {
            const { semantic_search, filename_search } = msg.metadata.searchResults;
            return (
              <div key={msg.id} className="mb-3 p-3 rounded-lg bg-gray-100 dark:bg-gray-700 w-full text-gray-900 dark:text-white">
                <p className="text-sm font-semibold mb-2">{msg.text}</p>
                {semantic_search?.results && semantic_search.results.length > 0 && (
                  <>
                    <h4 className="text-xs font-medium mt-2 mb-1">Semantic Results:</h4>
                    {semantic_search.results.map((item: any, index: number) => ( // Consider defining a type for search results
                      <SearchResultItem key={`semantic-${index}`} result={item} type="semantic" />
                    ))}
                  </>
                )}
                {filename_search?.results && filename_search.results.length > 0 && (
                  <>
                    <h4 className="text-xs font-medium mt-2 mb-1">Filename Results:</h4>
                    {filename_search.results.map((item: any, index: number) => ( // Consider defining a type for search results
                      <SearchResultItem key={`filename-${index}`} result={item} type="filename" />
                    ))}
                  </>
                )}
                {(semantic_search?.results?.length === 0 || !semantic_search?.results) && 
                 (filename_search?.results?.length === 0 || !filename_search?.results) && (
                  <p className="text-xs text-gray-500 dark:text-gray-400">No results found.</p>
                )}
                 <p className="text-xs opacity-70 mt-1 text-right">
                  {new Date(msg.timestamp).toLocaleTimeString()}
                </p>
              </div>
            );
          }
          return (
            <div
              key={msg.id}
              className={`mb-3 p-3 rounded-lg max-w-[80%] ${
                msg.sender === 'user'
                  ? 'bg-blue-500 text-white self-end ml-auto'
                : msg.sender === 'bot'
                  ? msg.metadata?.isSummary // Check for summary metadata
                    ? 'bg-purple-600 text-white self-start mr-auto' // Dedicated summary style
                    : 'bg-green-500 text-white self-start mr-auto' // Regular bot message
                  : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white self-start mr-auto' // System messages
              }`}
            >
              <p className="text-sm whitespace-pre-wrap">{msg.text}</p>
              <p className="text-xs opacity-70 mt-1">
                {new Date(msg.timestamp).toLocaleTimeString()} by {msg.sender}
              </p>
            </div>
          );
        })}
      </ScrollArea>
      <div className="p-4 border-t dark:border-gray-700 flex items-center">
        <Input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && handleSend()}
          placeholder="Type a message..."
          className="flex-grow"
        />
        <Button onClick={handleSend} className="ml-2">
          <PaperPlaneIcon className="w-5 h-5" />
        </Button>
      </div>
    </div>
  );
};
