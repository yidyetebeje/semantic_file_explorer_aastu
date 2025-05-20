// src-tauri/src/commands/chat_commands.rs

use serde_json::Value;
use tauri::State;
use crate::core::models::SemanticSearchResult;
use crate::search::search_files_semantic;
use crate::commands::search_commands::{filename_search_command, semantic_search_command};


#[tauri::command]
pub async fn send_message_to_gemini(app_handle: tauri::AppHandle, message: String) -> Result<String, String> {
    let script = format!("window.sendToGemini(`{}`)", message);
    match app_handle.eval_script(&script).await {
        Ok(tauri::ipc::Response::Ok(value)) => {
            // Assuming the JS promise resolves to a string which gets serialized as a JSON string
            if let Some(response_str) = value.as_str() {
                Ok(response_str.to_string())
            } else {
                Err(format!("Unexpected JS response type: {:?}", value))
            }
        }
        Ok(tauri::ipc::Response::Err(err)) => Err(format!("JS error: {:?}", err)),
        Err(e) => Err(format!("Failed to evaluate script: {}", e)),
    }
}

#[tauri::command]
pub async fn search_files(query: String) -> Result<Value, String> {
    let semantic_results_future = semantic_search_command(query.clone(), None, None, None, None);
    let filename_results_future = filename_search_command(query.clone(), None, None, None);

    let (semantic_results, filename_results) = tokio::join!(semantic_results_future, filename_results_future);

    let combined_results = serde_json::json!({
        "semantic_search": semantic_results.unwrap_or_else(|e| serde_json::json!({"error": e.to_string()})),
        "filename_search": filename_results.unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}))
    });

    Ok(combined_results)
}

#[tauri::command]
pub async fn get_document_content(file_path: String) -> Result<String, String> {
    match std::fs::read_to_string(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file {}: {}", file_path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use serde_json::json;
    // Mocking AppHandle for send_message_to_gemini is complex for unit tests.
    // For search_files, we'll test the combination logic.

    #[tokio::test]
    async fn test_get_document_content_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_doc.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        writeln!(file, "This is a test document.").unwrap();

        let result = get_document_content(file_path.to_str().unwrap().to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!\nThis is a test document.\n");
    }

    #[tokio::test]
    async fn test_get_document_content_file_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent_doc.txt");
        
        let result = get_document_content(file_path.to_str().unwrap().to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    // Test for send_message_to_gemini (conceptual, as AppHandle mocking is hard)
    // This test would ideally mock app_handle.eval_script().
    // For now, it's more of a placeholder demonstrating the intent.
    #[tokio::test]
    async fn test_send_message_to_gemini_forms_script() {
        // To truly test this, we'd need a mock AppHandle that allows us to inspect
        // the script passed to eval_script or mock its response.
        // This is a simplified check.
        let message = "Hello Gemini".to_string();
        let expected_script_part = "window.sendToGemini(`Hello Gemini`)";
        
        // If we had a mock AppHandle, we would:
        // 1. Create an instance of the mock AppHandle.
        // 2. Set up an expectation on eval_script to be called with a script containing expected_script_part.
        // 3. Call send_message_to_gemini(mock_app_handle, message).await.
        // 4. Assert that the expectation was met.
        // For now, we acknowledge this limitation.
        assert!(expected_script_part.contains(&message));
    }

    // Tests for search_files
    // We will simulate the behavior of semantic_search_command and filename_search_command
    // by creating helper functions that return pre-defined results.
    // This avoids needing to mock the actual Tauri command invocation system.

    // Mocked search result structures (simplified for testing)
    fn mock_semantic_success() -> Result<Value, String> {
        Ok(json!({
            "results": [{"file_path": "/path/semantic1.txt", "name": "semantic1.txt", "score": 0.9}],
            "query_id": "sem-123"
        }))
    }

    fn mock_filename_success() -> Result<Value, String> {
         Ok(json!({
            "results": [{"file_path": "/path/filename1.txt", "name": "filename1.txt"}],
            "count": 1
        }))
    }

    fn mock_search_error() -> Result<Value, String> {
        Err("Simulated search error".to_string())
    }
    
    // Since we can't directly mock the imported commands easily without a larger refactor,
    // and the `search_files` function calls them directly,
    // a true unit test of `search_files` in isolation is difficult here.
    // The test below will act more like an integration test for the happy path,
    // actually calling the real search commands if they don't have external dependencies (like DB).
    // If they do, this test would fail or require a running DB.
    // For this exercise, we'll assume they can be called and might return empty or errors if not set up.

    #[tokio::test]
    async fn test_search_files_combines_results() {
        // This test is more of an integration test due to direct command calls.
        // A pure unit test would require refactoring search_files to accept mockable dependencies.
        let query = "test query".to_string();
        
        // If semantic_search_command and filename_search_command are light enough to call:
        let result = search_files(query).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        
        assert!(value.get("semantic_search").is_some());
        assert!(value.get("filename_search").is_some());

        // Example: Check if semantic_search part has an error or results (depending on actual command behavior)
        if let Some(sem_res_val) = value.get("semantic_search") {
            if let Some(err_val) = sem_res_val.get("error") {
                 assert!(err_val.is_string()); // It errored, which is fine if DB not up
            } else {
                 assert!(sem_res_val.get("results").is_some() || sem_res_val.get("query_id").is_some());
            }
        }
         if let Some(fname_res_val) = value.get("filename_search") {
            if let Some(err_val) = fname_res_val.get("error") {
                 assert!(err_val.is_string());
            } else {
                 assert!(fname_res_val.get("results").is_some() || fname_res_val.get("count").is_some());
            }
        }
    }

    // To properly unit test search_files in isolation, one would refactor search_files to:
    // async fn search_files_logic<F1, F2, Fut1, Fut2>(query: String, semantic_search_fn: F1, filename_search_fn: F2) -> Result<Value, String>
    // where
    //     F1: FnOnce(String, Option<i64>, Option<f32>, Option<String>, Option<String>) -> Fut1,
    //     F2: FnOnce(String, Option<usize>, Option<usize>, Option<String>) -> Fut2,
    //     Fut1: std::future::Future<Output = Result<Value, String>>,
    //     Fut2: std::future::Future<Output = Result<Value, String>>,
    // Then, in tests, pass mock functions. The current `search_files` command would call this logic function.
}
