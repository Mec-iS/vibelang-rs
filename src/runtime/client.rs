use crate::config::Config;
use crate::runtime::llm_provider::LlmProvider; // Import the new trait
use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde_json::json;

pub struct LlmClient {
    client: Client,
    config: Config,
}

impl LlmClient {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            config,
        })
    }
}

// Implement the LlmProvider trait for the real LlmClient
impl LlmProvider for LlmClient {
    fn generate(&self, prompt: &str) -> Result<String> {
        let request_body = json!({
            "model": &self.config.ollama_model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.5
            }
        });

        let response = self
            .client
            .post(format!("{}/api/generate", &self.config.ollama_base_url))
            .json(&request_body)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "LLM API request failed with status {}: {}",
                response.status(),
                response.text()?
            ));
        }

        let response_json: serde_json::Value = response.json()?;
        let content = response_json["response"].as_str().ok_or_else(|| {
            anyhow!(
                "Invalid response format from LLM API: `response` field missing or not a string"
            )
        })?;

        Ok(content.to_string())
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            config: Config::default(),
        }
    }
}
