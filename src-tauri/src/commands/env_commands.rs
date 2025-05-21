// WARNING: This function returns a hardcoded API key.
// This is a major security risk and should NOT be used in production.
// Hardcoding API keys makes them vulnerable to being compromised if the
// application's source code or compiled binaries are accessed by unauthorized parties.
//
// For production environments, it is strongly recommended to use a more secure method
// for managing API keys, such as:
// 1. Environment Variables: Load the API key from an environment variable at runtime.
//    This keeps the key out of the source code.
// 2. Secure Configuration Files: Store the API key in a configuration file that is
//    not checked into version control and has restricted access permissions.
// 3. Secrets Management Services: Utilize services like HashiCorp Vault, AWS Secrets Manager,
//    or Google Cloud Secret Manager for storing and accessing API keys securely.
//
// This hardcoded key is for development and testing purposes ONLY.
#[tauri::command]
pub fn get_gemini_api_key() -> String {
    "AIzaSyCtOY0CKOUrbGCqSkgMH70m2a0BgkigBDg".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gemini_api_key_returns_expected_key() {
        // This test confirms that the function returns the hardcoded API key.
        // It's a simple test but ensures the function behaves as expected and
        // that the key hasn't been accidentally changed during refactoring.
        let expected_key = "AIzaSyCtOY0CKOUrbGCqSkgMH70m2a0BgkigBDg".to_string();
        assert_eq!(get_gemini_api_key(), expected_key);
    }
}
