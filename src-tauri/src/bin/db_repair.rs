use log::{info, warn, error};
use std::path::PathBuf;
use std::fs;
use semantic_file_explorer::db::{connect_db, get_db_path, TEXT_TABLE_NAME, IMAGE_TABLE_NAME};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("Starting database repair tool...");
    println!("WARNING: This will delete and recreate your database tables!");
    
    // First try the normal way through LanceDB API
    match repair_using_api().await {
        Ok(_) => {
            println!("Database successfully repaired using LanceDB API.");
            return Ok(());
        },
        Err(e) => {
            println!("API-based repair failed: {}", e);
            println!("Trying more aggressive repair method...");
        }
    }
    
    // If that fails, try the manual file removal approach
    match repair_using_filesystem().await {
        Ok(_) => {
            println!("Database successfully repaired by filesystem manipulation.");
            Ok(())
        },
        Err(e) => {
            error!("All repair methods failed!");
            Err(e.into())
        }
    }
}

async fn repair_using_api() -> Result<(), String> {
    info!("Attempting database repair using LanceDB API");
    
    // Get a connection to the database
    let conn = connect_db().await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        format!("Database connection failed: {}", e)
    })?;
    
    // Get table names
    let table_names = conn.table_names().execute().await.map_err(|e| {
        format!("Failed to list tables: {}", e)
    })?;
    
    // Check for and drop tables with potential schema issues
    let tables_to_check = [TEXT_TABLE_NAME, IMAGE_TABLE_NAME];
    let mut success = true;
    
    for table_name in tables_to_check.iter() {
        // Check if the table exists
        if table_names.contains(&table_name.to_string()) {
            info!("Found table '{}', attempting to drop it", table_name);
            
            // Try to drop the table
            match conn.drop_table(*table_name).await {
                Ok(_) => {
                    info!("Successfully dropped table '{}'", table_name);
                },
                Err(e) => {
                    error!("Failed to drop table '{}': {}", table_name, e);
                    success = false;
                }
            }
        } else {
            info!("Table '{}' not found, nothing to repair", table_name);
        }
    }
    
    if success {
        Ok(())
    } else {
        Err("Failed to drop one or more tables".to_string())
    }
}

async fn repair_using_filesystem() -> Result<(), Box<dyn std::error::Error>> {
    info!("Attempting database repair by filesystem manipulation");
    
    // Get database directory
    let db_path = get_db_path()?;
    
    // Check and remove both text and image tables
    let tables_to_check = [TEXT_TABLE_NAME, IMAGE_TABLE_NAME];
    
    for table_name in tables_to_check.iter() {
        let table_dir = db_path.join(table_name);
        
        // Remove table directory if it exists
        if table_dir.exists() && table_dir.is_dir() {
            info!("Found table directory: {}", table_dir.display());
            fs::remove_dir_all(&table_dir)?;
            info!("Successfully removed table directory for '{}'", table_name);
        } else {
            info!("Table directory for '{}' not found at {}, nothing to repair", 
                 table_name, table_dir.display());
        }
        
        // Also try to remove any LanceDB metadata for this table
        let metadata_file = db_path.join(format!("{}.lance", table_name));
        if metadata_file.exists() {
            fs::remove_file(&metadata_file)?;
            info!("Removed metadata file: {}", metadata_file.display());
        }
    }
    
    // Additionally, clean up any Tantivy locks that might be causing issues
    let tantivy_lock_path = PathBuf::from(format!("{}/home/{}/.tantivy-lock", 
                                         std::env::var("HOME").unwrap_or_default(), 
                                         std::env::var("USER").unwrap_or_default()));
    if tantivy_lock_path.exists() {
        fs::remove_file(&tantivy_lock_path).ok(); // Ignore errors here
        info!("Removed Tantivy lock file if it existed");
    }
    
    Ok(())
} 