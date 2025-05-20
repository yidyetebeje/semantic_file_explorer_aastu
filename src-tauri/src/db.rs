// src-tauri/src/db.rs

use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, FixedSizeListArray, TimestampSecondArray, Int32Array};
use arrow_array::builder::Float32Builder;
use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};
use lancedb::{connection::Connection, table::Table, Error as LanceError};
use lancedb::query::{QueryBase, ExecutableQuery, Select};
use futures::TryStreamExt; // For stream operations
use std::{path::{Path, PathBuf}, sync::Arc};
use std::fs;
use tempfile::TempDir; // Add this line for temporary directory support
use thiserror::Error;
use chrono::Utc;
use log::{info, warn, debug};

use lance_arrow::FixedSizeListArrayExt;
pub const TEXT_TABLE_NAME: &str = "documents";
pub const IMAGE_TABLE_NAME: &str = "images";
pub const TEXT_EMBEDDING_DIM: i32 = 384;  // BGESmallENV15 dimension
pub const IMAGE_EMBEDDING_DIM: i32 = 768; // NomicEmbedVisionV15 dimension
pub const AMHARIC_TEXT_TABLE_NAME: &str = "amharic_documents";
pub const AMHARIC_EMBEDDING_DIM: i32 = 384; // Dimension for multilingual-e5-small

pub const APP_DATA_DIR_NAME: &str = "semantic_file_explorer";

// For backward compatibility - use existing constant names internally
pub const TABLE_NAME: &str = TEXT_TABLE_NAME;
pub const EMBEDDING_DIM: i32 = TEXT_EMBEDDING_DIM;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("LanceDB connection failed: {0}")]
    ConnectionError(#[from] LanceError),
    #[error("Failed to create table schema: {0}")]
    SchemaError(#[from] arrow_schema::ArrowError),
    #[error("Table '{0}' already exists but with a different schema")]
    SchemaMismatch(String),
    #[error("Failed to create table '{0}': {1}")]
    TableCreationError(String, LanceError),
    #[error("Failed to open table '{0}': {1}")]
    TableOpenError(String, LanceError),
    #[error("Database path does not exist or is not a directory: {0}")]
    InvalidDbPath(String),
    #[error("IO error accessing database path '{0}': {1}")]
    IoError(String, #[source] std::io::Error),
    #[error("Record not found for path: {0}")]
    RecordNotFound(String),
    #[error("Embedding Error: {0}")]
    EmbeddingError(#[from] crate::embedder::EmbeddingError),
    #[error("Extractor Error: {0}")]
    ExtractorError(#[from] crate::extractor::ExtractorError),
    #[error("Failed to get application data directory: {0}")]
    AppDataDirError(String),
    #[error("Other DB Error: {0}")]
    Other(String),
    #[error("Image Embedding Error: {0}")]
    ImageEmbeddingError(#[from] crate::image_embedder::ImageEmbeddingError),
}

pub fn get_db_path() -> Result<PathBuf, DbError> {
    let app_data_dir = dirs::config_dir()
        .or_else(|| dirs::data_local_dir())
        .ok_or_else(|| DbError::AppDataDirError("Failed to locate application data directory".to_string()))?;
    let db_dir = app_data_dir.join(APP_DATA_DIR_NAME).join("lancedb");
    if !db_dir.exists() {
        fs::create_dir_all(&db_dir).map_err(|e| DbError::IoError(db_dir.display().to_string(), e))?;
    }
    Ok(db_dir)
}

fn create_amharic_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("file_path", DataType::Utf8, false),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("chunk_id", DataType::Int32, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                AMHARIC_EMBEDDING_DIM,
            ),
            true,
        ),
        Field::new("last_modified", DataType::Timestamp(TimeUnit::Second, None), false),
    ]))
}

fn create_text_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("file_path", DataType::Utf8, false),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("chunk_id", DataType::Int32, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                TEXT_EMBEDDING_DIM,
            ),
            true,
        ),
        Field::new("last_modified", DataType::Timestamp(TimeUnit::Second, None), false),
    ]))
}

/// Create the schema for image embeddings table
fn create_image_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("file_path", DataType::Utf8, false),
        Field::new("file_hash", DataType::Utf8, false),  // Hash of the image file
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                IMAGE_EMBEDDING_DIM,
            ),
            true,
        ),
        Field::new(
            "last_modified",
            DataType::Timestamp(TimeUnit::Second, None),
            false,
        ),
        // Additional fields specific to images
        Field::new("width", DataType::Int32, true),      // Image width in pixels
        Field::new("height", DataType::Int32, true),     // Image height in pixels
        Field::new("thumbnail_path", DataType::Utf8, true),  // Path to thumbnail if generated
    ]))
}

pub async fn connect_db() -> Result<Connection, DbError> {
    // Get the database path from application data directory
    let db_path = get_db_path()?;
    let db_path_str = db_path.to_string_lossy();
    
    // Ensure the path exists and is a directory
    if !db_path.exists() {
        std::fs::create_dir_all(&db_path).map_err(|e| DbError::IoError(db_path_str.to_string(), e))?;
        println!("Created database directory: {}", db_path_str);
    } else if !db_path.is_dir() {
        return Err(DbError::InvalidDbPath(format!(
            "DB path '{}' exists but is not a directory.", db_path_str
        )));
    }
    
    println!("Connecting to database: {}", db_path_str);
    lancedb::connect(db_path_str.as_ref()).execute().await.map_err(DbError::from)
}

// For backward compatibility with tests and other code that needs to specify a custom path
pub async fn connect_db_with_path(db_path: &str) -> Result<Connection, DbError> {
    let path = Path::new(db_path);
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| DbError::IoError(db_path.to_string(), e))?;
        println!("Created database directory: {}", db_path);
    } else if !path.is_dir() {
        return Err(DbError::InvalidDbPath(format!(
            "DB path '{}' exists but is not a directory.", db_path
        )));
    }
    println!("Connecting to database: {}", db_path);
    lancedb::connect(db_path).execute().await.map_err(DbError::from)
}

/// Open or create a text (document) table
pub async fn open_or_create_text_table(
    conn: &Connection,
) -> Result<Table, DbError> {
    open_or_create_table_with_schema(conn, TEXT_TABLE_NAME, create_text_schema()).await
}

/// Open or create an image table
pub async fn open_or_create_image_table(
    conn: &Connection,
) -> Result<Table, DbError> {
    open_or_create_table_with_schema(conn, IMAGE_TABLE_NAME, create_image_schema()).await
}

pub async fn open_or_create_amharic_text_table(
    conn: &Connection,
) -> Result<Table, DbError> {
    open_or_create_table_with_schema(conn, AMHARIC_TEXT_TABLE_NAME, create_amharic_schema()).await
}

/// Generic function to open or create a table with a specific schema
async fn open_or_create_table_with_schema(
    conn: &Connection,
    table_name: &str,
    expected_schema: SchemaRef,
) -> Result<Table, DbError> {
    let table_names = conn.table_names().execute().await?;

    if table_names.iter().any(|name| name == table_name) {
        println!("Opening existing table: {}", table_name);
        let table = conn
            .open_table(table_name)
            .execute()
            .await
            .map_err(|e| DbError::TableOpenError(table_name.to_string(), e))?;

        let existing_schema = table.schema().await?;
        if !schemas_compatible(&*existing_schema, &*expected_schema) {
             eprintln!(
                "Schema mismatch for table '{}':\nExpected: {:?}\nFound: {:?}",
                table_name, expected_schema, existing_schema
             );
            return Err(DbError::SchemaMismatch(table_name.to_string()));
        }
        Ok(table)
    } else {
        println!("Creating new table: {}", table_name);
        let batch = RecordBatch::new_empty(expected_schema.clone());
        let reader = RecordBatchIterator::new(vec![Ok(batch)], expected_schema);

        conn.create_table(table_name, Box::new(reader))
            .execute()
            .await
            .map_err(|e| DbError::TableCreationError(table_name.to_string(), e))
    }
}

fn schemas_compatible(schema1: &Schema, schema2: &Schema) -> bool {
    if schema1.fields.len() != schema2.fields.len() {
        return false;
    }
    for (f1, f2) in schema1.fields.iter().zip(schema2.fields.iter()) {
        if f1.name() != f2.name() || f1.data_type() != f2.data_type() {
            return false;
        }
    }
    true
}

/// Deletes a document from the table based on its file path.
pub async fn delete_document(table: &Table, file_path: &str) -> Result<(), DbError> {
    debug!("Deleting document: {}", file_path);
    // Use a SQL-like WHERE clause to specify the record to delete
    let predicate = format!("file_path = '{}'", file_path);
    table.delete(&predicate).await?; // Map LanceError to DbError via From
    Ok(())
}

/// Adds or updates a document record in the LanceDB table.
/// This performs a delete followed by an add, as LanceDB lacks native upsert.
///
/// Now supports multiple embeddings for a single document (chunking).
/// Each chunk gets a separate row with the same file_path and content_hash
/// but a different chunk_id.
pub async fn upsert_document(
    table: &Table,
    file_path: &str,
    content_hash: &str,
    embeddings: &[Vec<f32>],
) -> Result<(), DbError> {
    if embeddings.is_empty() {
        warn!("No embeddings provided for {}, skipping upsert", file_path);
        return Ok(());
    }

    debug!("Upserting document: {} with {} chunks", file_path, embeddings.len());
    
    // 1. Delete existing entries for this file path (ignore error if not found)
    let _ = delete_document(table, file_path).await; // Allow delete to fail if not present

    // 2. Prepare the new record batches
    let schema = create_text_schema(); // Get the schema
    let now_ts = Utc::now().timestamp();

    // Create batches for all embeddings/chunks
    let mut batches = Vec::with_capacity(embeddings.len());
    
    for (i, embedding) in embeddings.iter().enumerate() {
        // Create Arrow arrays for each record
        let file_path_array = StringArray::from(vec![file_path]);
        let content_hash_array = StringArray::from(vec![content_hash]);
        let chunk_id_array = Int32Array::from(vec![i as i32]);
        let last_modified_array = TimestampSecondArray::from(vec![now_ts]);

        // Create the FixedSizeList array for the embedding
        let mut embedding_builder = Float32Builder::new();
        embedding_builder.append_slice(embedding);
        let values_array = Arc::new(embedding_builder.finish()) as Arc<dyn arrow_array::Array>;
        let embedding_array = FixedSizeListArray::try_new_from_values(values_array, TEXT_EMBEDDING_DIM)
            .expect("Failed to create FixedSizeListArray");

        // Create the RecordBatch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(file_path_array),
                Arc::new(content_hash_array),
                Arc::new(chunk_id_array),
                Arc::new(embedding_array),
                Arc::new(last_modified_array),
            ],
        ).map_err(|e| DbError::SchemaError(e))?; // Convert ArrowError to DbError
        
        batches.push(Ok(batch));
    }

    // 3. Add all record batches to the table
    let reader = RecordBatchIterator::new(batches, schema);
    table.add(Box::new(reader)).execute().await?; // Map LanceError via From

    debug!("Successfully upserted document: {} with {} chunks", file_path, embeddings.len());
    Ok(())
}

pub async fn upsert_amharic_document(
    table: &Table,
    file_path: &str,
    content_hash: &str,
    embeddings: &[Vec<f32>],
) -> Result<(), DbError> {
    if embeddings.is_empty() {
        warn!("No embeddings provided for {}, skipping upsert", file_path);
        return Ok(());
    }

    debug!("Upserting Amharic document: {} with {} chunks", file_path, embeddings.len());
    
    // 1. Delete existing entries for this file path (ignore error if not found)
    let _ = delete_document(table, file_path).await; // Allow delete to fail if not present

    // 2. Prepare the new record batches
    let schema = create_amharic_schema(); // Get the schema
    let now_ts = Utc::now().timestamp();

    // Create batches for all embeddings/chunks
    let mut batches = Vec::with_capacity(embeddings.len());
    
    for (i, embedding) in embeddings.iter().enumerate() {
        // Create Arrow arrays for each record
        let file_path_array = StringArray::from(vec![file_path]);
        let content_hash_array = StringArray::from(vec![content_hash]);
        let chunk_id_array = Int32Array::from(vec![i as i32]);
        let last_modified_array = TimestampSecondArray::from(vec![now_ts]);

        // Create the FixedSizeList array for the embedding
        let mut embedding_builder = Float32Builder::new();
        embedding_builder.append_slice(embedding);
        let values_array = Arc::new(embedding_builder.finish()) as Arc<dyn arrow_array::Array>;
        let embedding_array = FixedSizeListArray::try_new_from_values(values_array, AMHARIC_EMBEDDING_DIM)
            .expect("Failed to create FixedSizeListArray");

        // Create the RecordBatch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(file_path_array),
                Arc::new(content_hash_array),
                Arc::new(chunk_id_array),
                Arc::new(embedding_array),
                Arc::new(last_modified_array),
            ],
        ).map_err(|e| DbError::SchemaError(e))?; // Convert ArrowError to DbError
        
        batches.push(Ok(batch));
    }

    // 3. Add all record batches to the table
    let reader = RecordBatchIterator::new(batches, schema);
    table.add(Box::new(reader)).execute().await?; // Map LanceError via From

    debug!("Successfully upserted Amharic document: {} with {} chunks", file_path, embeddings.len());
    Ok(())
}

/// Adds or updates an image record in the LanceDB image table.
pub async fn upsert_image(
    table: &Table,
    file_path: &str,
    file_hash: &str,
    embedding: &[f32],
    width: Option<i32>,
    height: Option<i32>,
    thumbnail_path: Option<&str>,
) -> Result<(), DbError> {
    debug!("Upserting image: {}", file_path);
    
    // 1. Delete existing entries for this file path (ignore error if not found)
    let _ = delete_document(table, file_path).await; // Allow delete to fail if not present

    // 2. Prepare the new record batch
    let schema = create_image_schema();
    let now_ts = Utc::now().timestamp();

    // Create Arrow arrays for the image record
    let file_path_array = StringArray::from(vec![file_path]);
    let file_hash_array = StringArray::from(vec![file_hash]);
    let last_modified_array = TimestampSecondArray::from(vec![now_ts]);
    let width_array = Int32Array::from(vec![width]);
    let height_array = Int32Array::from(vec![height]);
    let thumbnail_path_array = StringArray::from(vec![thumbnail_path]);

    // Create the FixedSizeList array for the embedding
    let mut embedding_builder = Float32Builder::new();
    embedding_builder.append_slice(embedding);
    let values_array = Arc::new(embedding_builder.finish()) as Arc<dyn arrow_array::Array>;
    let embedding_array = FixedSizeListArray::try_new_from_values(values_array, IMAGE_EMBEDDING_DIM)
        .expect("Failed to create FixedSizeListArray");

    // Create the RecordBatch
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(file_path_array),
            Arc::new(file_hash_array),
            Arc::new(embedding_array),
            Arc::new(last_modified_array),
            Arc::new(width_array),
            Arc::new(height_array),
            Arc::new(thumbnail_path_array),
        ],
    ).map_err(|e| DbError::SchemaError(e))?;

    // 3. Add the record batch to the table
    let reader = RecordBatchIterator::new(vec![Ok(batch)], schema);
    table.add(Box::new(reader)).execute().await?;

    debug!("Successfully upserted image: {}", file_path);
    Ok(())
}

/// Helper type for tests that creates a temporary directory for the DB
#[derive(Debug)]
pub struct TestDb {
    pub path: String,
    _dir: TempDir, // Keep TempDir to delete on drop
}

impl TestDb {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("Failed to create temp dir for test");
        let path = dir.path().to_str().unwrap().to_string();
        TestDb { path, _dir: dir }
    }
}
#[cfg(test)]
mod tests {
    use super::*; // Import necessary items from the parent module
    use lancedb::query::{ExecutableQuery, QueryBase}; // Restore query traits

    #[tokio::test]
    async fn test_db_connection_and_dir_creation() {
        let test_db = TestDb::new();
        let conn_result = connect_db_with_path(&test_db.path).await;
        assert!(conn_result.is_ok(), "Failed to connect to DB: {:?}", conn_result.err());
        assert!(Path::new(&test_db.path).exists(), "DB directory was not created");
        assert!(Path::new(&test_db.path).is_dir(), "DB path is not a directory");
    }

    #[tokio::test]
    async fn test_invalid_db_path_is_file() {
        let test_db = TestDb::new();
        let file_path = Path::new(&test_db.path).join("not_a_dir");
        std::fs::write(&file_path, "hello").expect("Failed to create dummy file");

        let conn_result = connect_db_with_path(file_path.to_str().unwrap()).await;
        assert!(conn_result.is_err());
        match conn_result.err().unwrap() {
            DbError::InvalidDbPath(_) => {}
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_create_and_open_table() {
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.expect("DB connection failed");

        let table_result1 = open_or_create_text_table(&conn).await;
        assert!(table_result1.is_ok(), "Create failed: {:?}", table_result1.err());
        let table1 = table_result1.unwrap();
        assert_eq!(table1.name(), TEXT_TABLE_NAME);

        let expected_schema = create_text_schema();
        let actual_schema = table1.schema().await.expect("Get schema failed");
        assert!(schemas_compatible(&*actual_schema, &*expected_schema), "Schema mismatch");

        let table_result2 = open_or_create_text_table(&conn).await;
        assert!(table_result2.is_ok(), "Open failed: {:?}", table_result2.err());
        let table2 = table_result2.unwrap();
        assert_eq!(table2.name(), TEXT_TABLE_NAME);

        let actual_schema2 = table2.schema().await.expect("Get schema 2 failed");
        assert!(schemas_compatible(&*actual_schema2, &*expected_schema), "Schema mismatch 2");

        let names = conn.table_names().execute().await.expect("Get names failed");
        assert!(names.contains(&TEXT_TABLE_NAME.to_string()), "Table name not found");
    }

    #[tokio::test]
    async fn test_schema_mismatch_error() {
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.expect("DB connection failed");

        let conflicting_schema = Arc::new(Schema::new(vec![
            Field::new("wrong_field", DataType::Int32, false),
        ]));
        let batch = RecordBatch::new_empty(conflicting_schema.clone());
        let reader = RecordBatchIterator::new(vec![Ok(batch)], conflicting_schema);
        conn.create_table(TEXT_TABLE_NAME, Box::new(reader))
            .execute()
            .await
            .expect("Manual create failed");

        let table_result = open_or_create_text_table(&conn).await;
        assert!(table_result.is_err(), "Expected schema mismatch error");

        match table_result.err().unwrap() {
            DbError::SchemaMismatch(name) => assert_eq!(name, TEXT_TABLE_NAME),
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    async fn setup_test_table() -> (TestDb, Connection, Table) {
        let test_db = TestDb::new();
        let conn = connect_db_with_path(&test_db.path).await.expect("DB connection failed");
        let table = open_or_create_text_table(&conn).await.expect("Creating table failed");
        (test_db, conn, table)
    }

    fn generate_dummy_embedding(seed: f32) -> Vec<f32> {
        (0..EMBEDDING_DIM).map(|i| seed + i as f32).collect()
    }

    #[tokio::test]
    async fn test_upsert_and_delete_document() {
        let (_test_db, _conn, table) = setup_test_table().await;

        let file_path1 = "/path/to/doc1.txt";
        let hash1 = "hash1";
        let embed1 = generate_dummy_embedding(1.0);

        // 1. Upsert initial document
        let upsert_result1 = upsert_document(&table, file_path1, hash1, &[embed1]).await;
        assert!(upsert_result1.is_ok(), "Upsert 1 failed: {:?}", upsert_result1.err());

        // Check if data exists (simple count)
        let count1 = table.count_rows(None).await.expect("Count failed");
        assert_eq!(count1, 1, "Expected 1 row after first upsert");

        // 2. Upsert the same document with a new hash (update)
        let hash2 = "hash2";
        let embed2 = generate_dummy_embedding(2.0);
        let upsert_result2 = upsert_document(&table, file_path1, hash2, &[embed2]).await;
        assert!(upsert_result2.is_ok(), "Upsert 2 failed: {:?}", upsert_result2.err());

        // Count should still be 1 after update
        let count2 = table.count_rows(None).await.expect("Count failed");
        assert_eq!(count2, 1, "Expected 1 row after update upsert");

        let predicate = format!("file_path = '{}'", file_path1);
        let query_result = table.query().only_if(predicate).execute().await;
        assert!(query_result.is_ok(), "Query failed: {:?}", query_result.err());

        // 3. Delete the document
        let delete_result = delete_document(&table, file_path1).await;
        assert!(delete_result.is_ok(), "Delete failed: {:?}", delete_result.err());

        // Count should be 0 after delete
        let count3 = table.count_rows(None).await.expect("Count failed");
        assert_eq!(count3, 0, "Expected 0 rows after delete");

        // 4. Delete non-existent document (should succeed without error)
        let delete_result_nonexistent = delete_document(&table, "/path/does/not/exist.txt").await;
        assert!(delete_result_nonexistent.is_ok(), "Delete non-existent failed: {:?}", delete_result_nonexistent.err());
    }
}

/// Force drops a table by removing it directly from the database
/// This is a more aggressive method than drop_table and should be used
/// only when regular drop_table fails
pub async fn force_drop_table(conn: &Connection, table_name: &str) -> Result<(), DbError> {
    info!("Force dropping table: {}", table_name);
    
    // Check if the table exists
    let table_names = conn.table_names().execute().await?;
    
    if !table_names.contains(&table_name.to_string()) {
        info!("Table '{}' does not exist, nothing to drop", table_name);
        return Ok(());
    }
    
    // Use the connection to execute a direct drop command
    // This bypasses any schema compatibility checks
    conn.drop_table(table_name).await?;
    
    info!("Successfully force-dropped table: {}", table_name);
    Ok(())
}

/// Clears all data from a LanceDB table without deleting the table itself
pub async fn clear_data(conn: &Connection, table_name: &str) -> Result<(), DbError> {
    info!("Clearing all data from table: {}", table_name);
    
    // Get the table
    let table = conn.open_table(table_name).execute().await
        .map_err(|e| DbError::TableOpenError(table_name.to_string(), e))?;
    
    // Delete all data from the table using a delete query with WHERE condition TRUE
    match table.delete("TRUE").await {
        Ok(_) => {
            info!("Successfully deleted all data from table: {}", table_name);
        },
        Err(e) => {
            warn!("Failed to delete data from table {}: {}", table_name, e);
            return Err(DbError::Other(format!("Failed to delete data: {}", e)));
        }
    }
    
    // Return success
    Ok(())
}

/// Gets statistics about the vector database, including document counts for each table
pub async fn get_vector_db_stats(conn: &Connection) -> Result<(usize, usize, usize), DbError> {
    info!("Getting vector database statistics");
    
    // Initialize document counts
    let mut text_docs_count = 0;
    let mut image_docs_count = 0;
    let mut amharic_docs_count = 0;
    
    // Try to get text documents count
    match conn.open_table(TEXT_TABLE_NAME).execute().await {
        Ok(table) => {
            // Execute a query to count all records
            // We'll use the count method directly on the stream
            let result = table.query().select(Select::All).execute().await;
            match result {
                Ok(data) => {
                    // Count the records by collecting and counting all batches
                    let count = match data.try_collect::<Vec<_>>().await {
                        Ok(batches) => {
                            let total = batches.iter().map(|batch| batch.num_rows()).sum();
                            debug!("Text documents count: {}", total);
                            total
                        },
                        Err(e) => {
                            warn!("Error counting text documents: {}", e);
                            0
                        }
                    };
                    text_docs_count = count;
                },
                Err(e) => {
                    warn!("Error counting text documents: {}", e);
                    // Continue with zero count
                }
            }
        },
        Err(e) => {
            // Table might not exist yet, which is fine
            debug!("Text table not found or cannot be opened: {}", e);
            // Continue with zero count
        }
    }
    
    // Try to get image documents count
    match conn.open_table(IMAGE_TABLE_NAME).execute().await {
        Ok(table) => {
            // Execute a query to count all records
            // We'll use the count method directly on the stream
            let result = table.query().select(Select::All).execute().await;
            match result {
                Ok(data) => {
                    // Count the records by collecting and counting all batches
                    let count = match data.try_collect::<Vec<_>>().await {
                        Ok(batches) => {
                            let total = batches.iter().map(|batch| batch.num_rows()).sum();
                            debug!("Image documents count: {}", total);
                            total
                        },
                        Err(e) => {
                            warn!("Error counting image documents: {}", e);
                            0
                        }
                    };
                    image_docs_count = count;
                },
                Err(e) => {
                    warn!("Error counting image documents: {}", e);
                    // Continue with zero count
                }
            }
        },
        Err(e) => {
            // Table might not exist yet, which is fine
            debug!("Image table not found or cannot be opened: {}", e);
            // Continue with zero count
        }
    }

    // Try to get Amharic text documents count
    match conn.open_table(AMHARIC_TEXT_TABLE_NAME).execute().await {
        Ok(table) => {
            let result = table.query().select(Select::All).execute().await;
            match result {
                Ok(data) => {
                    let count = match data.try_collect::<Vec<_>>().await {
                        Ok(batches) => {
                            let total = batches.iter().map(|batch| batch.num_rows()).sum();
                            debug!("Amharic text documents count: {}", total);
                            total
                        },
                        Err(e) => {
                            warn!("Error counting Amharic text documents: {}", e);
                            0
                        }
                    };
                    amharic_docs_count = count;
                },
                Err(e) => {
                    warn!("Error counting Amharic text documents: {}", e);
                }
            }
        },
        Err(e) => {
            debug!("Amharic text table not found or cannot be opened: {}", e);
        }
    }
    
    // Return the document counts
    Ok((text_docs_count, image_docs_count, amharic_docs_count))
}
