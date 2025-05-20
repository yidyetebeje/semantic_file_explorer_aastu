import { atom } from 'jotai';
import { FileInfo } from '@/types/file'; // Assuming FileInfo type is available

// From backend: src-tauri/src/commands/category_commands.rs
export interface CategoryInfo {
  name: String;
  extensions: string[];
  keywords: string[];
  // file_count: Option<usize>; // If you add this later
}

export const categoriesAtom = atom<CategoryInfo[]>([]);
export const selectedCategoryAtom = atom<CategoryInfo | null>(null);
export const categorizedFilesAtom = atom<FileInfo[]>([]);

export const isLoadingCategoriesAtom = atom<boolean>(true);
export const isLoadingFilesAtom = atom<boolean>(false);
export const categoriesErrorAtom = atom<string | null>(null);
export const filesErrorAtom = atom<string | null>(null);
