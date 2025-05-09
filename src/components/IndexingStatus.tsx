import { useEffect, useState } from 'react';
import { useAtom, useSetAtom } from 'jotai';
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { 
    indexingStatsAtom,
    isIndexingAtom,
    triggerIndexingAtom,
    fetchIndexingStatsAtom,
    selectedFolderPathAtom,
    triggerFolderIndexingAtom,
    clearIndexDataAtom,
    isClearingIndexAtom,
    // Vector DB stats atom
    vectorDbStatsAtom,
    fetchVectorDbStatsAtom,
    // Filename indexing atoms
    filenameIndexStatsAtom,
    isFilenameIndexingAtom,
    selectedFolderForFilenameIndexAtom,
    fetchFilenameIndexStatsAtom,
    clearFilenameIndexDataAtom,
    scanDirectoryForFilenameIndexAtom,
    initializeFilenameIndexAtom,
    isClearingFilenameIndexAtom,
    filenameIndexingResultAtom
} from '../store/atoms';
import { 
    Database, 
    FileCheck, 
    FileX, 
    FileClock, 
    RefreshCw, 
    ChevronDown, 
    ChevronRight, 
    Trash, 
    FolderOpen,
    FileText,
    Image
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DialogClose } from "@radix-ui/react-dialog";
import { open } from '@tauri-apps/plugin-dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AlertCircle } from "lucide-react";

const IndexingStatus = () => {
    // Semantic indexing state
    const [indexingStats] = useAtom(indexingStatsAtom);
    const [isIndexing, setIsIndexing] = useAtom(isIndexingAtom);
    const [isClearing] = useAtom(isClearingIndexAtom);
    const triggerIndexing = useSetAtom(triggerIndexingAtom);
    const fetchStats = useSetAtom(fetchIndexingStatsAtom);
    const [folderPath, setFolderPath] = useAtom(selectedFolderPathAtom);
    const triggerFolderIndexing = useSetAtom(triggerFolderIndexingAtom);
    const clearIndex = useSetAtom(clearIndexDataAtom);
    
    // Vector DB stats
    const [vectorDbStats] = useAtom(vectorDbStatsAtom);
    const fetchVectorDbStats = useSetAtom(fetchVectorDbStatsAtom);
    
    // Filename indexing state
    const [filenameIndexStats] = useAtom(filenameIndexStatsAtom);
    const [isFilenameIndexing] = useAtom(isFilenameIndexingAtom);
    const [isClearingFilenameIndex] = useAtom(isClearingFilenameIndexAtom);
    const [filenameIndexingResult] = useAtom(filenameIndexingResultAtom);
    const [folderPathForFilename, setFolderPathForFilename] = useAtom(selectedFolderForFilenameIndexAtom);
    const fetchFilenameStats = useSetAtom(fetchFilenameIndexStatsAtom);
    const scanDirectoryForFilenameIndex = useSetAtom(scanDirectoryForFilenameIndexAtom);
    const initializeFilenameIndex = useSetAtom(initializeFilenameIndexAtom);
    const clearFilenameIndex = useSetAtom(clearFilenameIndexDataAtom);
    
    // UI state
    const [showIndexedFiles, setShowIndexedFiles] = useState(false);
    const [showFailedFiles, setShowFailedFiles] = useState(false);
    const [showConfirmClear, setShowConfirmClear] = useState(false);
    const [showConfirmClearFilename, setShowConfirmClearFilename] = useState(false);
    const [selectedTab, setSelectedTab] = useState("downloads");
    const [selectedFilenameTab, setSelectedFilenameTab] = useState("initialize");
    const [clearSuccess, setClearSuccess] = useState<string | null>(null);
    const [clearFilenameSuccess, setClearFilenameSuccess] = useState<string | null>(null);

    // Fetch semantic indexing stats on mount and every 2 seconds while indexing
    useEffect(() => {
        fetchStats();
        fetchVectorDbStats(); // Fetch vector database stats on mount
        
        // Set up interval to fetch stats if indexing is in progress
        let interval: NodeJS.Timeout | null = null;
        if (isIndexing) {
            interval = setInterval(() => {
                fetchStats();
                fetchVectorDbStats(); // Also update vector DB stats during indexing
            }, 2000);
        }
        
        // Clean up interval on unmount
        return () => {
            if (interval) clearInterval(interval);
        };
    }, [fetchStats, fetchVectorDbStats, isIndexing]);
    
    // Fetch filename indexing stats on mount and every 2 seconds while indexing
    useEffect(() => {
        fetchFilenameStats();
        
        // Set up interval to fetch stats if indexing is in progress
        let interval: NodeJS.Timeout | null = null;
        if (isFilenameIndexing) {
            interval = setInterval(() => {
                fetchFilenameStats();
            }, 2000);
        }
        
        // Clean up interval on unmount
        return () => {
            if (interval) clearInterval(interval);
        };
    }, [fetchFilenameStats, isFilenameIndexing]);

    // Add an effect to update isIndexing based on stat changes
    useEffect(() => {
        if (indexingStats && !isIndexing) {
            // If we have stats but aren't indexing, make sure we're in the correct state
            fetchStats();
        }
    }, [indexingStats, isIndexing, fetchStats]);

    // Another effect to reset isIndexing if stuck
    useEffect(() => {
        // If indexing flag is stuck, reset it after 10 seconds of no changes to stats
        let timeout: NodeJS.Timeout | null = null;
        
        if (isIndexing) {
            timeout = setTimeout(() => {
                // Check if stats have been updated recently
                const lastUpdate = indexingStats?.time_taken_ms || 0;
                if (lastUpdate > 0 && Date.now() - lastUpdate > 10000) {
                    console.log("Indexing appears to be stuck, resetting state");
                    setIsIndexing(false);
                }
            }, 10000);
        }
        
        return () => {
            if (timeout) clearTimeout(timeout);
        };
    }, [isIndexing, indexingStats, setIsIndexing]);

    const handleStartDownloadsIndexing = () => {
        triggerIndexing();
    };

    const handleStartFolderIndexing = async () => {
        if (!folderPath) {
            const selectedPath = await open({
                directory: true,
                multiple: false,
                title: "Select Folder to Index",
            });

            if (typeof selectedPath === 'string' && selectedPath) {
                setFolderPath(selectedPath);
                // We can trigger indexing immediately after selecting the folder
                triggerFolderIndexing();
            }
        } else {
            triggerFolderIndexing();
        }
    };

    const handleSelectFolder = async () => {
        const selectedPath = await open({
            directory: true,
            multiple: false,
            title: "Select Folder to Index",
        });

        if (typeof selectedPath === 'string' && selectedPath) {
            setFolderPath(selectedPath);
        }
    };

    const handleClearIndex = async () => {
        setShowConfirmClear(false);
        await clearIndex();
        setClearSuccess("Successfully cleared all indexed data.");
        setTimeout(() => {
            setClearSuccess(null);
        }, 5000);
    };
    
    const handleClearFilenameIndex = async () => {
        setShowConfirmClearFilename(false);
        await clearFilenameIndex();
        setClearFilenameSuccess("Successfully cleared filename index data.");
        setTimeout(() => {
            setClearFilenameSuccess(null);
        }, 5000);
    };
    
    const handleInitializeFilenameIndex = () => {
        initializeFilenameIndex();
    };
    
    const handleScanDirectoryForFilename = async () => {
        if (!folderPathForFilename) {
            const selectedPath = await open({
                directory: true,
                multiple: false,
                title: "Select Folder to Index Filenames",
            });

            if (typeof selectedPath === 'string' && selectedPath) {
                setFolderPathForFilename(selectedPath);
                // We can trigger indexing immediately after selecting the folder
                scanDirectoryForFilenameIndex();
            }
        } else {
            scanDirectoryForFilenameIndex();
        }
    };
    
    const handleSelectFolderForFilename = async () => {
        const selectedPath = await open({
            directory: true,
            multiple: false,
            title: "Select Folder to Index Filenames",
        });

        if (typeof selectedPath === 'string' && selectedPath) {
            setFolderPathForFilename(selectedPath);
        }
    };

    // Helper function to format time
    const formatTime = (ms: number) => {
        if (ms < 1000) return `${ms}ms`;
        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);
        if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        }
        return `${seconds}s`;
    };

    // Helper function to extract filename from a path
    const getFileName = (path: string) => {
        return path.split('/').pop() || path;
    };

    return (
        <div className="container mx-auto mt-8 px-4">
            <div className="flex items-center justify-between mb-8">
                <div>
                    <h1 className="text-3xl font-bold text-white">Indexing Status</h1>
                    <p className="text-gray-400 mt-2">
                        View the status of document indexing and manage the indexing process
                    </p>
                </div>
                <div className="flex space-x-2">
                    <Button
                        onClick={() => setShowConfirmClear(true)}
                        disabled={isIndexing || isClearing}
                        variant="destructive"
                    >
                        <Trash className="mr-2 h-4 w-4" />
                        Clear Semantic Index
                    </Button>
                    <Button
                        onClick={() => setShowConfirmClearFilename(true)}
                        disabled={isFilenameIndexing || isClearingFilenameIndex}
                        variant="destructive"
                    >
                        <Trash className="mr-2 h-4 w-4" />
                        Clear Filename Index
                    </Button>
                </div>
            </div>

            {clearSuccess && (
                <Alert className="mb-4 bg-green-800 border-green-700">
                    <AlertCircle className="h-4 w-4 text-green-400" />
                    <AlertTitle>Success</AlertTitle>
                    <AlertDescription>{clearSuccess}</AlertDescription>
                </Alert>
            )}
            
            {clearFilenameSuccess && (
                <Alert className="mb-4 bg-green-800 border-green-700">
                    <AlertCircle className="h-4 w-4 text-green-400" />
                    <AlertTitle>Success</AlertTitle>
                    <AlertDescription>{clearFilenameSuccess}</AlertDescription>
                </Alert>
            )}

            {isIndexing && (
                <div className="mb-8">
                    <h2 className="text-white text-lg font-medium mb-2">Semantic Indexing in Progress</h2>
                    <Progress value={indexingStats ? 
                        (indexingStats.files_indexed / (indexingStats.files_processed || 1)) * 100 : 0
                    } className="h-2 mb-2" />
                    <p className="text-gray-400 text-sm">
                        Indexing documents, please wait...
                    </p>
                </div>
            )}
            
            {isFilenameIndexing && (
                <div className="mb-8">
                    <h2 className="text-white text-lg font-medium mb-2">Filename Indexing in Progress</h2>
                    <Progress value={75} className="h-2 mb-2" />
                    <p className="text-gray-400 text-sm">
                        Indexing filenames, please wait...
                    </p>
                </div>
            )}

            {/* Semantic Indexing Options */}
            <h2 className="text-xl font-bold text-white mb-3">Semantic Content Indexing</h2>
            <Tabs defaultValue="downloads" value={selectedTab} onValueChange={setSelectedTab} className="mb-8">
                <TabsList className="grid grid-cols-2 bg-gray-800">
                    <TabsTrigger value="downloads">Index Downloads Folder</TabsTrigger>
                    <TabsTrigger value="custom">Index Custom Folder</TabsTrigger>
                </TabsList>
                <TabsContent value="downloads">
                    <Card className="bg-gray-800 border-gray-700 text-white">
                        <CardHeader>
                            <CardTitle>Index Downloads Folder</CardTitle>
                            <CardDescription className="text-gray-400">
                                Index all PDF files in your Downloads folder.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <p className="text-gray-300 mb-4">
                                This will search your Downloads folder for PDF files and index them for semantic search.
                                The process may take several minutes depending on the number of files.
                            </p>
                        </CardContent>
                        <CardFooter className="flex justify-end">
                            <Button
                                onClick={handleStartDownloadsIndexing}
                                disabled={isIndexing}
                                className="bg-purple-600 hover:bg-purple-700"
                            >
                                <RefreshCw className={`mr-2 h-4 w-4 ${isIndexing ? 'animate-spin' : ''}`} />
                                {isIndexing ? 'Indexing...' : 'Start Indexing'}
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>
                <TabsContent value="custom">
                    <Card className="bg-gray-800 border-gray-700 text-white">
                        <CardHeader>
                            <CardTitle>Index Custom Folder</CardTitle>
                            <CardDescription className="text-gray-400">
                                Select and index a specific folder on your computer.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="flex items-center mb-4">
                                <div className="flex-1 bg-gray-700 p-2 rounded truncate mr-2">
                                    {folderPath || "No folder selected"}
                                </div>
                                <Button 
                                    onClick={handleSelectFolder} 
                                    variant="outline"
                                    className="ml-2"
                                    disabled={isIndexing}
                                >
                                    <FolderOpen className="h-4 w-4 mr-2" />
                                    Browse
                                </Button>
                            </div>
                        </CardContent>
                        <CardFooter className="flex justify-end">
                            <Button
                                onClick={handleStartFolderIndexing}
                                disabled={isIndexing}
                                className="bg-purple-600 hover:bg-purple-700"
                            >
                                <RefreshCw className={`mr-2 h-4 w-4 ${isIndexing ? 'animate-spin' : ''}`} />
                                {isIndexing ? 'Indexing...' : 'Start Indexing'}
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>
            </Tabs>
            
            {/* Filename Indexing Options */}
            <h2 className="text-xl font-bold text-white mb-3 mt-8">Filename Indexing</h2>
            <Tabs defaultValue="initialize" value={selectedFilenameTab} onValueChange={setSelectedFilenameTab} className="mb-8">
                <TabsList className="grid grid-cols-2 bg-gray-800">
                    <TabsTrigger value="initialize">Initialize Common Directories</TabsTrigger>
                    <TabsTrigger value="custom">Index Custom Folder</TabsTrigger>
                </TabsList>
                
                <TabsContent value="initialize" className="mt-4">
                    <Card className="bg-gray-900 border-gray-800">
                        <CardHeader>
                            <CardTitle className="text-lg font-medium text-gray-100">Initialize Filename Index</CardTitle>
                            <CardDescription className="text-gray-400">
                                This will index filenames in common directories like Downloads, Documents, and Desktop.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <span className="font-medium text-gray-300">Total Files Indexed:</span>
                                        <span className="ml-2 text-gray-400">{filenameIndexStats?.file_count || 0}</span>
                                    </div>
                                </div>
                                
                                {filenameIndexingResult && (
                                    <div className="mt-3 p-3 bg-gray-800 rounded-md">
                                        <p className="text-gray-300 text-sm">Last Operation Results:</p>
                                        <p className="text-gray-400 text-sm mt-1">Files Added: {filenameIndexingResult.files_added}</p>
                                    </div>
                                )}
                            </div>
                        </CardContent>
                        <CardFooter className="flex justify-end">
                            <Button
                                onClick={handleInitializeFilenameIndex}
                                disabled={isFilenameIndexing}
                                className="bg-purple-600 hover:bg-purple-700"
                            >
                                <RefreshCw className={`mr-2 h-4 w-4 ${isFilenameIndexing ? 'animate-spin' : ''}`} />
                                {isFilenameIndexing ? 'Indexing...' : 'Initialize Index'}
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>
                
                <TabsContent value="custom" className="mt-4">
                    <Card className="bg-gray-900 border-gray-800">
                        <CardHeader>
                            <CardTitle className="text-lg font-medium text-gray-100">Index Custom Folder</CardTitle>
                            <CardDescription className="text-gray-400">
                                Select a specific folder to index all filenames within it.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-4">
                                <div className="flex items-center">
                                    <span className="text-gray-300 w-24">Folder Path:</span>
                                    <input 
                                        type="text" 
                                        value={folderPathForFilename || ''} 
                                        readOnly 
                                        className="flex-1 bg-gray-800 border-gray-700 rounded text-gray-300 text-sm px-2 py-1"
                                        placeholder="No folder selected"
                                    />
                                    <Button 
                                        onClick={handleSelectFolderForFilename} 
                                        variant="outline"
                                        className="ml-2"
                                        disabled={isFilenameIndexing}
                                    >
                                        <FolderOpen className="h-4 w-4 mr-2" />
                                        Browse
                                    </Button>
                                </div>
                                
                                {filenameIndexingResult && filenameIndexingResult.directory && (
                                    <div className="mt-3 p-3 bg-gray-800 rounded-md">
                                        <p className="text-gray-300 text-sm">Last Operation Results:</p>
                                        <p className="text-gray-400 text-sm mt-1">Directory: {filenameIndexingResult.directory}</p>
                                        <p className="text-gray-400 text-sm">Files Added: {filenameIndexingResult.files_added}</p>
                                    </div>
                                )}
                            </div>
                        </CardContent>
                        <CardFooter className="flex justify-end">
                            <Button
                                onClick={handleScanDirectoryForFilename}
                                disabled={isFilenameIndexing}
                                className="bg-purple-600 hover:bg-purple-700"
                            >
                                <RefreshCw className={`mr-2 h-4 w-4 ${isFilenameIndexing ? 'animate-spin' : ''}`} />
                                {isFilenameIndexing ? 'Indexing...' : 'Start Indexing'}
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>
            </Tabs>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader className="pb-2">
                        <CardTitle className="text-lg flex items-center">
                            <Database className="mr-2 h-5 w-5 text-purple-400" />
                            Total Files
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-3xl font-bold">{indexingStats?.files_processed || 0}</p>
                    </CardContent>
                </Card>

                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader className="pb-2">
                        <CardTitle className="text-lg flex items-center">
                            <FileCheck className="mr-2 h-5 w-5 text-green-400" />
                            Indexed Files
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-3xl font-bold">{indexingStats?.db_inserts || 0}</p>
                    </CardContent>
                </Card>

                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader className="pb-2">
                        <CardTitle className="text-lg flex items-center">
                            <FileClock className="mr-2 h-5 w-5 text-yellow-400" />
                            Skipped Files
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-3xl font-bold">{indexingStats?.files_skipped || 0}</p>
                    </CardContent>
                </Card>

                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader className="pb-2">
                        <CardTitle className="text-lg flex items-center">
                            <FileX className="mr-2 h-5 w-5 text-red-400" />
                            Failed Files
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-3xl font-bold">{indexingStats?.files_failed || 0}</p>
                    </CardContent>
                </Card>
            </div>

            <h2 className="text-xl font-bold text-white mb-4">Content Type Breakdown</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader>
                        <CardTitle className="text-lg flex items-center">
                            <FileText className="mr-2 h-5 w-5 text-blue-400" />
                            Text Files
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="pt-0">
                        <div className="grid grid-cols-3 gap-2 mb-4">
                            <div className="bg-gray-700 p-4 rounded-lg text-center">
                                <p className="text-gray-400 text-sm">Processed</p>
                                <p className="text-2xl font-bold">{indexingStats?.text_files_processed || 0}</p>
                            </div>
                            <div className="bg-gray-700 p-4 rounded-lg text-center">
                                <p className="text-gray-400 text-sm">Indexed</p>
                                <p className="text-2xl font-bold text-green-400">{indexingStats?.text_files_indexed || 0}</p>
                            </div>
                            <div className="bg-gray-700 p-4 rounded-lg text-center">
                                <p className="text-gray-400 text-sm">Failed</p>
                                <p className="text-2xl font-bold text-red-400">{indexingStats?.text_files_failed || 0}</p>
                            </div>
                        </div>
                        
                        {indexingStats && indexingStats?.text_files_processed && indexingStats.text_files_processed > 0 && (
                            <div className="mt-4">
                                <p className="text-sm text-gray-400 mb-2">Indexing Progress</p>
                                <Progress 
                                    value={(indexingStats.text_files_indexed || 0) / (indexingStats.text_files_processed || 1) * 100} 
                                    className="h-2"
                                />
                            </div>
                        )}
                    </CardContent>
                </Card>
                
                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader>
                        <CardTitle className="text-lg flex items-center">
                            <Image className="mr-2 h-5 w-5 text-purple-400" />
                            Image Files
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="pt-0">
                        <div className="grid grid-cols-3 gap-2 mb-4">
                            <div className="bg-gray-700 p-4 rounded-lg text-center">
                                <p className="text-gray-400 text-sm">Processed</p>
                                <p className="text-2xl font-bold">{indexingStats?.image_files_processed || 0}</p>
                            </div>
                            <div className="bg-gray-700 p-4 rounded-lg text-center">
                                <p className="text-gray-400 text-sm">Indexed</p>
                                <p className="text-2xl font-bold text-green-400">{indexingStats?.image_files_indexed || 0}</p>
                            </div>
                            <div className="bg-gray-700 p-4 rounded-lg text-center">
                                <p className="text-gray-400 text-sm">Failed</p>
                                <p className="text-2xl font-bold text-red-400">{indexingStats?.image_files_failed || 0}</p>
                            </div>
                        </div>
                        
                        {indexingStats && indexingStats?.image_files_processed && indexingStats.image_files_processed > 0 && (
                            <div className="mt-4">
                                <p className="text-sm text-gray-400 mb-2">Indexing Progress</p>
                                <Progress 
                                    value={(indexingStats.image_files_indexed || 0) / (indexingStats.image_files_processed || 1) * 100} 
                                    className="h-2"
                                />
                            </div>
                        )}
                    </CardContent>
                </Card>
            </div>

            {/* Vector Database Statistics Card */}
            <Card className="bg-gray-800 border-gray-700 text-white mb-8">
                <CardHeader>
                    <CardTitle>Vector Database Statistics</CardTitle>
                    <CardDescription className="text-gray-400">
                        Current information about indexed documents in the database
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="space-y-2">
                        <div className="flex justify-between">
                            <span className="text-gray-400">Total Documents</span>
                            <span className="font-medium text-blue-400">
                                {vectorDbStats ? vectorDbStats.total_documents_count : '0'}
                            </span>
                        </div>
                        <div className="flex justify-between">
                            <div className="flex items-center">
                                <FileText className="h-4 w-4 mr-2 text-purple-400" />
                                <span className="text-gray-400">Text Documents</span>
                            </div>
                            <span className="font-medium text-purple-400">
                                {vectorDbStats ? vectorDbStats.text_documents_count : '0'}
                            </span>
                        </div>
                        <div className="flex justify-between">
                            <div className="flex items-center">
                                <Image className="h-4 w-4 mr-2 text-green-400" />
                                <span className="text-gray-400">Image Documents</span>
                            </div>
                            <span className="font-medium text-green-400">
                                {vectorDbStats ? vectorDbStats.image_documents_count : '0'}
                            </span>
                        </div>
                    </div>
                </CardContent>
                <CardFooter className="text-xs text-gray-500">
                    Real-time data from your vector database
                </CardFooter>
            </Card>

            {/* Last Indexing Process Card */}
            <Card className="bg-gray-800 border-gray-700 text-white mb-8">
                <CardHeader>
                    <CardTitle>Last Indexing Process</CardTitle>
                    <CardDescription className="text-gray-400">
                        Details about the most recent indexing operation
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="space-y-2">
                        <div className="flex justify-between">
                            <span className="text-gray-400">Status</span>
                            <span className={`font-medium ${indexingStats?.success ? 'text-green-400' : 'text-red-400'}`}>
                                {indexingStats?.success ? 'Completed Successfully' : 'Failed'}
                            </span>
                        </div>
                        <div className="flex justify-between">
                            <span className="text-gray-400">Time Taken</span>
                            <span className="font-medium">{indexingStats ? formatTime(indexingStats.time_taken_ms) : 'N/A'}</span>
                        </div>
                        <div className="flex justify-between">
                            <span className="text-gray-400">Success Rate</span>
                            <span className="font-medium">
                                {indexingStats && indexingStats.files_processed > 0
                                    ? `${Math.round((indexingStats.files_indexed / indexingStats.files_processed) * 100)}%`
                                    : 'N/A'
                                }
                            </span>
                        </div>
                    </div>
                    {indexingStats?.message && (
                        <div className="pt-4 border-t border-gray-700">
                            <h4 className="text-sm font-medium text-gray-300 mb-2">Message:</h4>
                            <p className="text-gray-400 text-sm">{indexingStats.message}</p>
                        </div>
                    )}
                </CardContent>
                <CardFooter className="text-xs text-gray-500">
                    Last updated: {new Date().toLocaleString()}
                </CardFooter>
            </Card>

            {/* File Lists Section */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                {/* Indexed Files */}
                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader className="pb-3 cursor-pointer" onClick={() => setShowIndexedFiles(!showIndexedFiles)}>
                        <CardTitle className="text-lg flex items-center justify-between">
                            <div className="flex items-center">
                                <FileCheck className="mr-2 h-5 w-5 text-green-400" />
                                Indexed Files
                            </div>
                            {showIndexedFiles ? 
                                <ChevronDown className="h-5 w-5 text-gray-400" /> : 
                                <ChevronRight className="h-5 w-5 text-gray-400" />
                            }
                        </CardTitle>
                        <CardDescription className="text-gray-400">
                            {indexingStats?.indexed_files?.length || 0} files successfully indexed
                        </CardDescription>
                    </CardHeader>
                    {showIndexedFiles && (
                        <CardContent className="max-h-60 overflow-y-auto">
                            {indexingStats?.indexed_files?.length ? (
                                <ul className="space-y-1">
                                    {indexingStats.indexed_files.map((file, index) => (
                                        <li key={index} className="truncate text-sm text-gray-300">
                                            <span className="text-green-400 mr-2">•</span> {getFileName(file)}
                                            <span className="block ml-5 text-xs text-gray-500 truncate">{file}</span>
                                        </li>
                                    ))}
                                </ul>
                            ) : (
                                <p className="text-gray-500 text-sm">No indexed files to show</p>
                            )}
                        </CardContent>
                    )}
                </Card>

                {/* Failed Files */}
                <Card className="bg-gray-800 border-gray-700 text-white">
                    <CardHeader className="pb-3 cursor-pointer" onClick={() => setShowFailedFiles(!showFailedFiles)}>
                        <CardTitle className="text-lg flex items-center justify-between">
                            <div className="flex items-center">
                                <FileX className="mr-2 h-5 w-5 text-red-400" />
                                Failed Files
                            </div>
                            {showFailedFiles ? 
                                <ChevronDown className="h-5 w-5 text-gray-400" /> : 
                                <ChevronRight className="h-5 w-5 text-gray-400" />
                            }
                        </CardTitle>
                        <CardDescription className="text-gray-400">
                            {indexingStats?.failed_files?.length || 0} files failed to index
                        </CardDescription>
                    </CardHeader>
                    {showFailedFiles && (
                        <CardContent className="max-h-60 overflow-y-auto">
                            {indexingStats?.failed_files?.length ? (
                                <ul className="space-y-1">
                                    {indexingStats.failed_files.map((file, index) => (
                                        <li key={index} className="truncate text-sm text-gray-300">
                                            <span className="text-red-400 mr-2">•</span> {getFileName(file)}
                                            <span className="block ml-5 text-xs text-gray-500 truncate">{file}</span>
                                        </li>
                                    ))}
                                </ul>
                            ) : (
                                <p className="text-gray-500 text-sm">No failed files to show</p>
                            )}
                        </CardContent>
                    )}
                </Card>
            </div>

            {/* Confirm Clear Semantic Index Dialog */}
            <Dialog open={showConfirmClear} onOpenChange={setShowConfirmClear}>
                <DialogContent className="bg-gray-900 border-gray-800 text-gray-100">
                    <DialogHeader>
                        <DialogTitle>Clear Semantic Index Data</DialogTitle>
                        <DialogDescription className="text-gray-400">
                            Are you sure you want to clear all semantically indexed content data? This action cannot be undone.
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter className="justify-between">
                        <DialogClose asChild>
                            <Button variant="outline" className="border-gray-700 hover:bg-gray-800">
                                Cancel
                            </Button>
                        </DialogClose>
                        <Button onClick={handleClearIndex} variant="destructive" className="hover:bg-red-700">
                            Clear Semantic Index
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
            
            {/* Confirm Clear Filename Index Dialog */}
            <Dialog open={showConfirmClearFilename} onOpenChange={setShowConfirmClearFilename}>
                <DialogContent className="bg-gray-900 border-gray-800 text-gray-100">
                    <DialogHeader>
                        <DialogTitle>Clear Filename Index Data</DialogTitle>
                        <DialogDescription className="text-gray-400">
                            Are you sure you want to clear all indexed filename data? This action cannot be undone.
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter className="justify-between">
                        <DialogClose asChild>
                            <Button variant="outline" className="border-gray-700 hover:bg-gray-800">
                                Cancel
                            </Button>
                        </DialogClose>
                        <Button onClick={handleClearFilenameIndex} variant="destructive" className="hover:bg-red-700">
                            Clear Filename Index
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
};

export default IndexingStatus; 