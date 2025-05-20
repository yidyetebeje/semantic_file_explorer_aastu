import { Button } from "@/components/ui/button";
import {
  Globe,
  HardDrive,
  Laptop,
  Film,
  FileText,
  Download,
  Plus,
  Settings,
  Clock,
  Folder,
  Database,
} from "lucide-react";
import LibraryDropdown from "./LibraryDropdown";
import SidebarSection from "./SidebarSection";
import NavButton from "./NavButton";
import { useSetAtom, useAtom } from 'jotai';
import {
  navigateAtom,
  customLocationsAtom,
  addCustomLocationAtom,
  recentItemsAtom
} from '../../store/atoms';
import {
  getDocumentsDir,
  getDownloadsDir,
  getMoviesDir,
  getHostname,
  getHomeDir
} from "../../services/test";
import { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { basename } from '@tauri-apps/api/path';
import { CustomLocation } from "../../types/location";

const Sidebar = () => {
  const navigate = useSetAtom(navigateAtom);
  const addLocation = useSetAtom(addCustomLocationAtom);
  const [customLocations] = useAtom(customLocationsAtom);
  const [recentItems] = useAtom(recentItemsAtom);
  const [hostname, setHostname] = useState("Computer");
  const [homeDirPath, setHomeDirPath] = useState("/");


  useEffect(() => {
    getHostname()
      .then(name => setHostname(name))
      .catch(err => console.error("Failed to get hostname:", err));
    getHomeDir()
      .then(path => setHomeDirPath(path))
      .catch(err => console.error("Failed to get home directory:", err));
  }, []);

  const goToDocuments = async () => {
    try {
      const path = await getDocumentsDir();
      navigate(path);
    } catch (error) {
      console.error("Could not navigate to Documents:", error);
    }
  };

  const goToDownloads = async () => {
    try {
      const path = await getDownloadsDir();
      navigate(path);
    } catch (error) {
      console.error("Could not navigate to Downloads:", error);
    }
  };

  const goToMovies = async () => {
    try {
      const path = await getMoviesDir();
      navigate(path);
    } catch (error) {
      console.error("Could not navigate to Movies:", error);
    }
  };

  const handleMacintoshHDClick = () => navigate("/");
  const handleComputerClick = () => {
    if (homeDirPath && homeDirPath !== "/") {
      navigate(homeDirPath);
    } else {
      console.warn("Home directory path not available, navigating to root.");
      navigate("/");
    }
  };
  const handleAddLocationClick = async () => {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: "Select Folder to Add",
      });

      if (typeof selectedPath === 'string' && selectedPath) {
        const name = await basename(selectedPath);
        const newLocation: CustomLocation = { name, path: selectedPath };
        await addLocation(newLocation);
      } else {
        console.log("Folder selection cancelled or invalid.");
      }
    } catch (error) {
      console.error("Error opening folder dialog:", error);
    }
  };
  const handleSettingsClick = () => console.log("Settings clicked (not implemented)");
  const handleRecentClick = () => navigate('/recent-items');
  
  // Define a handler to show indexing status page
  const handleIndexingClick = () => {
    // Use the navigateAtom instead of window.location to stay within the SPA
    navigate('/indexing-status');
  };

  return (
    <div className="w-60 border-r border-gray-800 h-screen bg-gray-900 flex flex-col">
      {/* Library Selector */}
      <div className="p-4 border-b border-gray-800">
        <LibraryDropdown />
      </div>

      {/* Scrollable Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <SidebarSection title="Overview">
          <NavButton icon={HardDrive} label="Macintosh HD" onClick={handleMacintoshHDClick} />
          <NavButton icon={Laptop} label={hostname} onClick={handleComputerClick} />
        </SidebarSection>

        <SidebarSection title="Locations">
          <NavButton icon={Film} label="Movies" onClick={goToMovies} />
          <NavButton icon={FileText} label="Documents" onClick={goToDocuments} />
          <NavButton icon={Download} label="Downloads" onClick={goToDownloads} />
          {customLocations.map((location) => (
            <NavButton
              key={location.path}
              icon={Folder}
              label={location.name}
              onClick={() => navigate(location.path)}
            />
          ))}
        </SidebarSection>

        <Button 
          variant="ghost" 
          className="w-full justify-start text-gray-500 hover:bg-gray-800 cursor-pointer hover:text-gray-300"
          onClick={handleAddLocationClick}
        >
          <Plus className="mr-2 h-4 w-4" />
          Add Location
        </Button>
      </div>

      {/* Bottom Actions */}
      <div className="border-t border-gray-800 p-4 bg-gray-900">
        <NavButton icon={Database} label="Indexing Status" onClick={handleIndexingClick} />
        <NavButton icon={Settings} label="Settings" onClick={handleSettingsClick} />
        <NavButton 
          icon={Clock} 
          label={`Recent${recentItems.length > 0 ? ` (${recentItems.length})` : ''}`} 
          onClick={handleRecentClick}
        />
      </div>
    </div>
  );
};

export default Sidebar;
