// src-tauri/src/commands/env_commands.rs
#[tauri::command]
pub fn get_gemini_api_key() -> Result<String, String> {
    match std::env::var("GEMINI_API_KEY") {
        Ok(key) => Ok(key),
        Err(_) => Err("GEMINI_API_KEY not found in environment".to_string()),
    }
}
