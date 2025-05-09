import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem
} from "@/components/ui/dropdown-menu";
import { FolderOpen } from "lucide-react";
import { useState, useEffect } from 'react';
import { getHostname } from '../../services/test'; // Assuming path is correct

const LibraryDropdown = () => {
  const [hostname, setHostname] = useState("Library"); // Default name

  useEffect(() => {
    getHostname()
      .then(name => setHostname(name ? `${name}'s Library` : "My Library"))
      .catch(err => {
        console.error("Failed to get hostname for Library:", err);
        setHostname("My Library"); // Fallback name
      });
  }, []);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="w-full justify-start text-gray-300 hover:bg-gray-800 hover:text-gray-300 cursor-pointer">
          <FolderOpen className="mr-2 h-4 w-4" />
          {hostname} {/* Use dynamic hostname */}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-48 bg-gray-900 text-gray-300">
        <DropdownMenuItem>Library 1</DropdownMenuItem>
        <DropdownMenuItem>Library 2</DropdownMenuItem>
        <DropdownMenuItem>Library 3</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export default LibraryDropdown;  