// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use commands::fs_commands::{
    get_documents_dir, get_downloads_dir, get_home_dir, get_movies_dir, list_directory_command,
    open_path_command, save_custom_locations, load_custom_locations,
    get_hostname_command
};
use commands::benchmark_commands::run_benchmarks;
use commands::search_commands::{semantic_search_command, get_document_count};
use commands::indexing_commands::{
    index_downloads_command, run_startup_indexing, get_indexing_stats_command,
    clear_index_command, index_folder_command, get_vector_db_stats_command
};
use commands::search_commands::{filename_search_command, add_file_to_index, remove_file_from_index, get_filename_index_stats, clear_filename_index, scan_directory_for_filename_index, initialize_filename_index};
pub mod commands;
pub mod core;
pub mod db;
pub mod embedding;
pub mod watcher;
pub mod extractor;
pub mod embedder;
pub mod benchmark;
pub mod search;
pub mod chunker;
pub mod repair_db;
pub mod image_embedder;
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn repair_database_command() -> Result<String, String> {
    repair_db::repair_database().await?;
    Ok("Database successfully repaired".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();
    tracing::info!("Application starting up...");

    // Start indexing the Downloads folder and initializing filename index in the background
    // Use spawn_blocking to run the async code without requiring an existing runtime
    std::thread::spawn(|| {
        // Create a new runtime for this thread
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            tracing::info!("Starting background indexing processes");
            
            // Initialize the semantic search index
            tracing::info!("Starting Downloads folder indexing for semantic search");
            run_startup_indexing().await;
            
            // Initialize the filename index with common directories
            tracing::info!("Starting filename index initialization");
            // match initialize_filename_index().await {
            //     Ok(stats) => tracing::info!("Filename index initialized: {:?}", stats),
            //     Err(e) => tracing::error!("Failed to initialize filename index: {}", e),
            // }
        });
    });

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    let builder = register_commands(builder);
    
    builder.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        // Filesystem commands
        list_directory_command,
        get_home_dir,
        open_path_command,
        get_downloads_dir,
        get_movies_dir,
        get_documents_dir,
        load_custom_locations,
        save_custom_locations,
        get_hostname_command,
        
        // Semantic search commands
        semantic_search_command,
        get_document_count,
        
        // Filename search commands
        filename_search_command,
        add_file_to_index,
        remove_file_from_index,
        get_filename_index_stats,
        clear_filename_index,
        scan_directory_for_filename_index,
        initialize_filename_index,
        
        // Indexing commands
        index_downloads_command,
        index_folder_command,
        get_indexing_stats_command,
        clear_index_command,
        get_vector_db_stats_command,
        
        // Benchmark commands
        run_benchmarks,
        
        // Database repair command
        repair_database_command
    ])
}
