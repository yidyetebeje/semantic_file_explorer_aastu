import { atom } from 'jotai';

export interface ChatMessage {
  id: string;
  text: string;
  sender: 'user' | 'bot' | 'system';
  timestamp: Date;
  metadata?: Record<string, any>; // For search results, etc.
}

export const chatMessagesAtom = atom<ChatMessage[]>([]);
export const isChatOpenAtom = atom<boolean>(false);
export const chatInputAtom = atom<string>('');
