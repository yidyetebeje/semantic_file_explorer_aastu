use log::{info, warn, error};
use lancedb::Connection;
use crate::db::{connect_db, TABLE_NAME, force_drop_table};

/// Drops the documents table and recreates it with the correct schema
pub async fn repair_database() -> Result<(), String> {
    info!("Starting database repair process");
    
    // Connect to the database
    let conn = connect_db().await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        format!("Database connection failed: {}", e)
    })?;
    
    // First try the regular drop table method
    if let Err(e) = drop_table(&conn, TABLE_NAME).await {
        warn!("Regular table drop failed: {}", e);
        
        // If regular drop fails, try the force drop method
        if let Err(e) = force_drop_table(&conn, TABLE_NAME).await.map_err(|e| {
            format!("Force drop table error: {}", e)
        }) {
            error!("Force drop table also failed: {}", e);
            return Err(format!("Could not repair database. All drop methods failed: {}", e));
        } else {
            info!("Successfully force-dropped the table");
        }
    } else {
        info!("Successfully dropped the table using regular method");
    }
    
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