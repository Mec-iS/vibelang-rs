use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)] // Added Clone for convenience
pub struct Config {
    pub ollama_base_url: String,
    pub ollama_model: String,
}

impl Config {
    /// Creates a configuration by reading from environment variables,
    /// falling back to standard defaults.
    pub fn from_env() -> Self {
        Self {
            ollama_base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.1".to_string()),
        }
    }
}

// NEW: Implement the Default trait for Config.
impl Default for Config {
    /// Provides a default configuration for testing or basic usage.
    fn default() -> Self {
        Self {
            ollama_base_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.1".to_string(),
        }
    }
}
