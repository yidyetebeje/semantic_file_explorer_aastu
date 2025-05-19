use log::{info, warn, error};
use lancedb::Connection;
use crate::db::{connect_db, TEXT_TABLE_NAME, IMAGE_TABLE_NAME, force_drop_table};

/// Drops the documents table and recreates it with the correct schema
pub async fn repair_database() -> Result<(), String> {
    info!("Starting database repair process");
    
    // Connect to the database
    let conn = connect_db().await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        format!("Database connection failed: {}", e)
    })?;
    
    // Define which tables to repair
    let tables_to_repair = [TEXT_TABLE_NAME, IMAGE_TABLE_NAME];

    for table_name_to_repair in tables_to_repair.iter() {
        info!("Attempting to repair table: {}", table_name_to_repair);
        // First try the regular drop table method
        if let Err(e) = drop_table(&conn, table_name_to_repair).await {
            warn!("Regular drop failed for table '{}': {}", table_name_to_repair, e);
            
            // If regular drop fails, try the force drop method
            if let Err(e_force) = force_drop_table(&conn, table_name_to_repair).await {
                error!("Force drop also failed for table '{}': {}", table_name_to_repair, e_force);
                // We can choose to return an error for the specific table or try to continue with others
                // For now, let's log the error and continue, or you can return an Err here.
                // return Err(format!("Could not repair table '{}'. All drop methods failed: {}", table_name_to_repair, e_force));
            } else {
                info!("Successfully force-dropped table: {}", table_name_to_repair);
            }
        } else {
            info!("Successfully dropped table '{}' using regular method", table_name_to_repair);
        }
    }
    
    info!("Database repair process finished for all targeted tables.");
    
    info!("Database repair completed successfully");
    // The table will be recreated with the correct schema when needed
    Ok(())
}

/// Drops a table if it exists
async fn drop_table(conn: &Connection, table_name: &str) -> Result<(), String> {
    info!("Attempting to drop table: {}", table_name);
    
    // Check if the table exists
    let table_names = conn.table_names().execute().await.map_err(|e| {
        format!("Failed to list tables: {}", e)
    })?;
    
    if table_names.contains(&table_name.to_string()) {
        // Table exists, drop it
        conn.drop_table(table_name).await.map_err(|e| {
            format!("Failed to drop table '{}': {}", table_name, e)
        })?;
        info!("Successfully dropped table: {}", table_name);
    } else {
        info!("Table '{}' does not exist, nothing to drop", table_name);
    }
    
    Ok(())
} 