/**
 * Represents a user-defined custom location for the sidebar.
 */
export interface CustomLocation {
  /** The display name for the location (e.g., "My Project Folder"). */
  name: string;
  /** The absolute file system path to the location. */
  path: string;
} 