use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ollama_base_url: String,
    pub ollama_model: String,
}

impl Config {
    pub fn from_env() -> Self {
        let ollama_base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let ollama_model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3.1".to_string());
        
        Self { ollama_base_url, ollama_model }
    }
}
