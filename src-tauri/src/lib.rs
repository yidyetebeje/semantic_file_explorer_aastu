// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use commands::benchmark_commands::run_benchmarks;
use commands::fs_commands::{
    get_documents_dir, get_downloads_dir, get_home_dir, get_hostname_command, get_movies_dir,
    list_directory_command, load_custom_locations, open_path_command, save_custom_locations,
};
use commands::file_operations::{
    copy_item, create_directory, delete_item, get_item_info, move_item, rename_item,
};
use commands::indexing_commands::{
    clear_index_command, get_indexing_stats_command, get_vector_db_stats_command,
    index_downloads_command, index_folder_command, run_startup_indexing,
};
use commands::search_commands::{
    add_file_to_index, clear_filename_index, filename_search_command, get_filename_index_stats,
    initialize_filename_index, remove_file_from_index, scan_directory_for_filename_index,
};
use commands::search_commands::{get_document_count, semantic_search_command};
use commands::env_commands::get_gemini_api_key;
use commands::chat_commands::{send_message_to_gemini, search_files, get_document_content};
use commands::category_commands::{get_all_categories, get_files_by_category};
pub mod benchmark;
pub mod chunker;
pub mod commands;
pub mod core;
pub mod db;
pub mod embedder;
pub mod embedding;
pub mod extractor;
pub mod image_embedder;
pub mod repair_db;
pub mod search;
pub mod watcher;
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

    builder
        .run(tauri::generate_context!())
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
        // File operations commands
        copy_item,
        move_item,
        delete_item,
        rename_item,
        create_directory,
        get_item_info,
        // Database repair command
        repair_database_command,
        // Env command
        get_gemini_api_key,
        // Chat commands
        send_message_to_gemini,
        search_files,
        get_document_content,
        // Category commands
        get_all_categories,
        get_files_by_category
    ])
}
