import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const formatDate = (dateString: string): string => {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
};

/**
 * Formats a file size in bytes into a human-readable string (KB, MB, GB).
 * Returns '--' if the size is null or undefined.
 */
export const formatSize = (bytes: number | null | undefined): string => {
  if (bytes === null || bytes === undefined || isNaN(bytes)) {
    return '--';
  }
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  // Handle potential edge case where i might be out of bounds (very large numbers)
  const index = Math.min(i, sizes.length - 1);

  // Format the number with 1 decimal place for KB and above
  const formattedSize = parseFloat((bytes / Math.pow(k, index)).toFixed(index === 0 ? 0 : 1));

  return `${formattedSize} ${sizes[index]}`;
};

