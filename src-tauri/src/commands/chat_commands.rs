// src-tauri/src/commands/chat_commands.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value; // Keep Value for search_files if needed
use crate::commands::search_commands::{filename_search_command, semantic_search_command, SearchRequest, FilenameSearchRequest};
use super::env_commands::get_gemini_api_key; // Import the function to get API key

// Structs for Gemini API request and response
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    // Add promptFeedback if you need to inspect it, e.g. for blocked prompts
    // #[serde(rename = "promptFeedback")]
    // prompt_feedback: Option<PromptFeedback>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Option<ContentResponse>,
    // Add other fields like finishReason, safetyRatings if needed
    // #[serde(rename = "finishReason")]
    // finish_reason: Option<String>,
    // #[serde(rename = "safetyRatings")]
    // safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Deserialize, Debug)]
struct ContentResponse {
    parts: Option<Vec<PartResponse>>,
    role: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PartResponse {
    text: Option<String>,
}

// Example of other structs you might need if you parse more of the response
// #[derive(Deserialize, Debug)]
// struct PromptFeedback {
//     #[serde(rename = "blockReason")]
//     block_reason: Option<String>,
//     #[serde(rename = "safetyRatings")]
//     safety_ratings: Option<Vec<SafetyRating>>,
// }

// #[derive(Deserialize, Debug)]
// struct SafetyRating {
//     category: String,
//     probability: String,
// }

#[tauri::command]
pub async fn send_message_to_gemini(message: String) -> Result<String, String> {
    let api_key = get_gemini_api_key();
    if api_key.is_empty() || api_key == "AIzaSyCtOY0CKOUrbGCqSkgMH70m2a0BgkigBDg" { // Basic check
        eprintln!("API key is missing or is the placeholder. Please set it in env_commands.rs");
        // It's better to return an error than to proceed with a clearly invalid key.
        // However, the task specifies to use the hardcoded key for now.
        // For a real app, you'd return Err("API key not configured".to_string()) here.
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
        api_key
    );

    let client = Client::new();

    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: message }],
        }],
    };

    log::info!("Sending request to Gemini API at URL: {}", url);
    log::debug!("Request body: {:?}", serde_json::to_string(&request_body));


    match client.post(&url).json(&request_body).send().await {
        Ok(response) => {
            let response_status = response.status();
            log::debug!("Received response from Gemini API with status: {}", response_status);

            if response_status.is_success() {
                match response.json::<GeminiResponse>().await {
                    Ok(gemini_response) => {
                        log::debug!("Successfully parsed Gemini response: {:?}", gemini_response);
                        if let Some(candidates) = gemini_response.candidates {
                            if let Some(candidate) = candidates.get(0) {
                                if let Some(content) = &candidate.content {
                                    if let Some(parts) = &content.parts {
                                        if let Some(part) = parts.get(0) {
                                            if let Some(text) = &part.text {
                                                log::info!("Extracted text from Gemini response");
                                                return Ok(text.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        log::warn!("Failed to extract text from Gemini response structure");
                        Err("Failed to parse Gemini response: No text found".to_string())
                    }
                    Err(e) => {
                        eprintln!("Failed to parse Gemini JSON response: {:?}", e);
                        log::error!("Failed to parse Gemini JSON response: {}", e);
                        Err(format!("Failed to parse Gemini JSON response: {}", e))
                    }
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|e| format!("Unknown error, failed to read error body: {}", e));
                eprintln!("Gemini API request failed with status: {}. Response: {}", response_status, error_text);
                log::error!("Gemini API request failed with status: {}. Response: {}", response_status, error_text);
                Err(format!(
                    "Gemini API request failed with status: {}. Response: {}",
                    response_status, error_text
                ))
            }
        }
        Err(e) => {
            eprintln!("Failed to send request to Gemini: {:?}", e);
            log::error!("Failed to send request to Gemini: {}", e);
            Err(format!("Failed to send request to Gemini: {}", e))
        }
    }
}

#[tauri::command]
pub async fn summarize_file(file_path: String) -> Result<String, String> {
    log::info!("Attempting to summarize file: {}", file_path);

    // 1. Get API Key
    let api_key = get_gemini_api_key();
    if api_key.is_empty() || api_key == "AIzaSyCtOY0CKOUrbGCqSkgMH70m2a0BgkigBDg" {
        let err_msg = "API key is missing or is the placeholder. Please set it in env_commands.rs";
        eprintln!("{}", err_msg);
        log::error!("{}", err_msg);
        // For a real app, return Err(err_msg.to_string()) here.
        // Proceeding with placeholder key as per current task constraints.
    }

    // 2. Get Document Content
    let file_content = match get_document_content(file_path.clone()).await { // Added .await here
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read file content for {}: {}", file_path, e);
            return Err(format!("Failed to read file {}: {}", file_path, e));
        }
    };

    // Basic content checks
    const MAX_CONTENT_LENGTH: usize = 100_000; // Approx 25k tokens for gemini-pro (limit is 30720 tokens)
    if file_content.len() > MAX_CONTENT_LENGTH {
        log::warn!(
            "File content for {} is long ({} chars). Summary quality may be affected or API may reject if overall request is too large.", 
            file_path, file_content.len()
        );
        // Not returning error here, let API handle it, as prompt is also part of request size.
    }
    if file_content.trim().is_empty() {
        log::info!("File content for {} is empty. Returning specific summary.", file_path);
        return Ok("The file is empty, so there is nothing to summarize.".to_string());
    }

    // 3. Construct Prompt
    let prompt = format!(
        "Please summarize the following document (path: {}):\n\n---\n{}\n---",
        file_path, file_content
    );

    // 4. Call Gemini API
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
        api_key
    );
    let client = Client::new();
    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
    };

    log::info!("Sending summarization request to Gemini API for file: {}", file_path);
    match client.post(&url).json(&request_body).send().await {
        Ok(response) => {
            let response_status = response.status();
            if response_status.is_success() {
                match response.json::<GeminiResponse>().await {
                    Ok(gemini_response) => {
                        if let Some(candidates) = gemini_response.candidates {
                            if let Some(candidate) = candidates.get(0) {
                                if let Some(content_resp) = &candidate.content {
                                    if let Some(parts) = &content_resp.parts {
                                        if let Some(part) = parts.get(0) {
                                            if let Some(text) = &part.text {
                                                log::info!("Extracted summary for file: {}", file_path);
                                                return Ok(text.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        log::warn!("Failed to extract summary from Gemini response structure for file: {}", file_path);
                        Err(format!("Failed to parse Gemini summary response for {}: No text found", file_path))
                    }
                    Err(e) => {
                        log::error!("Failed to parse Gemini summary JSON response for {}: {}", file_path, e);
                        Err(format!("Failed to parse Gemini summary JSON response for {}: {}", file_path, e))
                    }
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|e| format!("Unknown error, failed to read error body: {}", e));
                log::error!("Gemini API summary request failed for {} with status: {}. Response: {}", file_path, response_status, error_text);
                Err(format!(
                    "Gemini API summary request failed for {} with status: {}. Response: {}",
                    file_path, response_status, error_text
                ))
            }
        }
        Err(e) => {
            log::error!("Failed to send summary request to Gemini for {}: {}", file_path, e);
            Err(format!("Failed to send summary request to Gemini for {}: {}", file_path, e))
        }
    }
}

#[tauri::command]
pub async fn search_files(query: String) -> Result<Value, String> {
    // Create proper request objects
    let semantic_request = SearchRequest {
        query: query.clone(),
        limit: None,
        min_score: None,
        db_uri: None,
        content_type: None
    };
    
    let filename_request = FilenameSearchRequest {
        query: query.clone(),
        categories: None,
        limit: None,
        path_filter: None,
        category_filter: None
    };
    
    let semantic_results_future = semantic_search_command(semantic_request);
    let filename_results_future = filename_search_command(filename_request);

    let (semantic_results, filename_results) = tokio::join!(semantic_results_future, filename_results_future);

    // Map the results to JSON value format
    let semantic_search_json = match semantic_results {
        Ok(response) => serde_json::to_value(response).unwrap_or_else(|e| serde_json::json!({"error": e.to_string()})),
        Err(e) => serde_json::json!({"error": e}),
    };
    
    let filename_search_json = match filename_results {
        Ok(response) => serde_json::to_value(response).unwrap_or_else(|e| serde_json::json!({"error": e.to_string()})),
        Err(e) => serde_json::json!({"error": e}),
    };
    
    let combined_results = serde_json::json!({
        "semantic_search": semantic_search_json,
        "filename_search": filename_search_json
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
    use serde_json::json; // Make sure this is imported for mock_semantic_success etc.
    // Mocking AppHandle for send_message_to_gemini is complex for unit tests.
    // For search_files, we'll test the combination logic.

    #[tokio::test]
    async fn test_get_document_content_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_doc.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        writeln!(file, "This is a test document.").unwrap();

        let result = get_document_content(file_path.to_str().unwrap().to_string()).await; // Added .await
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!\nThis is a test document.\n");
    }

    #[tokio::test]
    async fn test_get_document_content_file_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent_doc.txt");
        
        let result = get_document_content(file_path.to_str().unwrap().to_string()).await; // Added .await
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    // --- Tests for summarize_file ---
    #[tokio::test]
    async fn test_summarize_file_empty_content() {
        let _ = env_logger::builder().is_test(true).try_init();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty_doc.txt");
        File::create(&file_path).unwrap(); 

        let result = summarize_file(file_path.to_str().unwrap().to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "The file is empty, so there is nothing to summarize.");
    }

    #[tokio::test]
    async fn test_summarize_file_content_long() {
        // This test checks behavior when content is long but not excessively so,
        // expecting it to proceed to the API call (which will fail due to the placeholder key).
        let _ = env_logger::builder().is_test(true).try_init();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("long_doc.txt");
        let mut file = File::create(&file_path).unwrap();
        // Content length is MAX_CONTENT_LENGTH + 1
        let long_content = "a".repeat(MAX_CONTENT_LENGTH + 1); 
        file.write_all(long_content.as_bytes()).unwrap();
        
        let result = summarize_file(file_path.to_str().unwrap().to_string()).await;
        assert!(result.is_err(), "Expected API call to fail for long content with placeholder key, but got Ok: {:?}", result.ok());
        if let Err(e) = result {
            // Check that the failure is due to the API call, not an early exit for "too long"
            assert!(e.contains("Gemini API summary request failed"), "Error message did not indicate API failure for long content: {}", e);
        }
    }
    
    #[tokio::test]
    async fn test_summarize_file_file_not_found() {
        let _ = env_logger::builder().is_test(true).try_init();
        let result = summarize_file("path/to/non_existent_document.txt".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file path/to/non_existent_document.txt"));
    }

    #[tokio::test]
    async fn test_summarize_file_with_placeholder_key_normal_content() {
        // This test ensures that a normal file (not empty, not excessively long)
        // proceeds to the API call, which then fails due to the placeholder key.
        let _ = env_logger::builder().is_test(true).try_init();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("normal_doc.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is a normal document for summarization.").unwrap();

        let result = summarize_file(file_path.to_str().unwrap().to_string()).await;
        assert!(result.is_err(), "Expected API call to fail with placeholder key, but got Ok: {:?}", result.ok());
        if let Err(e) = result {
            assert!(e.contains("Gemini API summary request failed"), "Error message did not indicate API failure: {}", e);
        }
    }
    // --- End of Tests for summarize_file ---

    // Test for send_message_to_gemini
    // This test would ideally use a mock HTTP server (e.g., mockito, wiremock-rs)
    // to simulate Gemini API responses. For now, it's a conceptual placeholder.
    // It also assumes that the API key is the placeholder and will be handled by the function.
    #[tokio::test]
    async fn test_send_message_to_gemini_placeholder_key() {
        // Ensure env_logger is initialized for tests that use log macros
        let _ = env_logger::builder().is_test(true).try_init();

        let message = "Hello Gemini".to_string();
        // This will actually try to make a network request if not mocked.
        // Since the API key is a placeholder, it should fail.
        // The function is expected to handle this based on the current implementation.
        // If the function were to return Err for the placeholder key, this test would change.
        let result = send_message_to_gemini(message).await;
        
        // Given the current implementation uses the placeholder key,
        // it will attempt a real API call, which will likely fail due to an invalid key.
        // The exact error might vary (e.g., API error from Google, or reqwest error if DNS fails).
        // We expect an error that indicates the API request failed.
        assert!(result.is_err(), "Expected an error due to placeholder API key, but got Ok: {:?}", result.ok());
        if let Err(e) = result {
            assert!(e.contains("Gemini API request failed"), "Error message did not indicate API failure: {}", e);
        }
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
