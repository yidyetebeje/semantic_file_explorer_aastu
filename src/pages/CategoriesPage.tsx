import React, { useEffect } from 'react';
import { useAtom } from 'jotai';
import { invoke } from '@tauri-apps/api/core';
import {
  categoriesAtom,
  selectedCategoryAtom,
  categorizedFilesAtom,
  isLoadingCategoriesAtom,
  isLoadingFilesAtom,
  categoriesErrorAtom,
  filesErrorAtom,
  CategoryInfo,
} from '@/store/categoryAtoms';
import { FileInfo } from '@/types/file';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import FileGrid from '@/components/FileExplorerBody/FileGrid'; // Assuming this can be reused
import { toast } from 'react-hot-toast';

const CategoriesPage: React.FC = () => {
  const [categories, setCategories] = useAtom(categoriesAtom);
  const [selectedCategory, setSelectedCategory] = useAtom(selectedCategoryAtom);
  const [categorizedFiles, setCategorizedFiles] = useAtom(categorizedFilesAtom);
  const [isLoadingCategories, setIsLoadingCategories] = useAtom(isLoadingCategoriesAtom);
  const [isLoadingFiles, setIsLoadingFiles] = useAtom(isLoadingFilesAtom);
  const [categoriesError, setCategoriesError] = useAtom(categoriesErrorAtom);
  const [filesError, setFilesError] = useAtom(filesErrorAtom);

  useEffect(() => {
    setIsLoadingCategories(true);
    setCategoriesError(null);
    invoke<CategoryInfo[]>('get_all_categories')
      .then((fetchedCategories) => {
        setCategories(fetchedCategories);
      })
      .catch((err) => {
        console.error('Failed to fetch categories:', err);
        setCategoriesError(`Failed to load categories: ${err.toString()}`);
        toast.error(`Failed to load categories: ${err.toString()}`);
      })
      .finally(() => {
        setIsLoadingCategories(false);
      });
  }, [setCategories, setIsLoadingCategories, setCategoriesError]);

  const handleCategoryClick = (category: CategoryInfo) => {
    setSelectedCategory(category);
    setIsLoadingFiles(true);
    setFilesError(null);
    setCategorizedFiles([]); // Clear previous files

    // For now, pass null for base_path_str, backend should default to home dir
    invoke<FileInfo[]>('get_files_by_category', { categoryName: category.name, basePathStr: null })
      .then((files) => {
        setCategorizedFiles(files);
      })
      .catch((err) => {
        console.error(`Failed to fetch files for ${category.name}:`, err);
        setFilesError(`Failed to load files for ${category.name}: ${err.toString()}`);
        toast.error(`Failed to load files for ${category.name}: ${err.toString()}`);
      })
      .finally(() => {
        setIsLoadingFiles(false);
      });
  };

  return (
    <div className="flex flex-col h-full p-4 space-y-4">
      <h1 className="text-2xl font-semibold text-gray-100">File Categories</h1>

      {isLoadingCategories && <p className="text-gray-400">Loading categories...</p>}
      {categoriesError && <p className="text-red-500">{categoriesError}</p>}

      {!isLoadingCategories && !categoriesError && (
        <div className="flex flex-wrap gap-3">
          {categories.map((cat) => (
            <Button
              key={cat.name.toString()} // Assuming name is unique and stringifiable
              variant={selectedCategory?.name === cat.name ? 'default' : 'outline'}
              onClick={() => handleCategoryClick(cat)}
              className="shadow-md"
            >
              {cat.name.toString()}
            </Button>
          ))}
        </div>
      )}

      {selectedCategory && (
        <Card className="flex-grow flex flex-col bg-gray-800 border-gray-700">
          <CardHeader>
            <CardTitle className="text-gray-100">Files in "{selectedCategory.name.toString()}"</CardTitle>
          </CardHeader>
          <CardContent className="flex-grow flex flex-col">
            {isLoadingFiles && <p className="text-gray-400">Loading files...</p>}
            {filesError && <p className="text-red-500">{filesError}</p>}
            {!isLoadingFiles && !filesError && categorizedFiles.length === 0 && (
              <p className="text-gray-400">No files found in this category for the scanned location.</p>
            )}
            {!isLoadingFiles && !filesError && categorizedFiles.length > 0 && (
              <ScrollArea className="flex-grow">
                {/* Assuming FileGrid can take files directly. Adapt if needed. */}
                <FileGrid files={categorizedFiles} />
              </ScrollArea>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default CategoriesPage;
