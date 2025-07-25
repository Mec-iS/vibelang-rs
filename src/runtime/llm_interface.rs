use anyhow::{Result, anyhow};
use reqwest;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;

pub struct LlmInterface {
    client: reqwest::blocking::Client,
    base_url: String,
}

#[derive(Debug, Clone)]
pub struct VibeValue {
    pub value_type: VibeValueType,
    pub data: VibeValueData,
}

#[derive(Debug, Clone)]
pub enum VibeValueType {
    Null,
    Boolean,
    Number,
    String,
}

#[derive(Debug, Clone)]
pub enum VibeValueData {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl LlmInterface {
    pub fn new() -> Result<Self> {
        let ollama_url = env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        // Try to detect if Ollama is running locally
        let use_ollama = Self::is_ollama_available(&ollama_url);

        let base_url = if use_ollama {
            // Use local Ollama - no API key needed
            ollama_url.clone()
        } else {
            panic!("Ollama not running locally")
        };

        println!("{:?}", ollama_url);

        Ok(Self {
            client: reqwest::blocking::Client::new(),
            base_url,
        })
    }

    fn is_ollama_available(url: &str) -> bool {
        let client = reqwest::blocking::Client::new();
        match client
            .get(&format!("{}/api/tags", url))
            .timeout(std::time::Duration::from_secs(2))
            .send()
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub fn execute_prompt(&self, prompt: &str, meaning: Option<&str>) -> Result<VibeValue> {
        let response = self.send_to_llm(prompt).unwrap();
        self.parse_response(&response, meaning)
    }

    fn send_to_llm(&self, prompt: &str) -> Result<String> {
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7
        });

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send();

        let payload = response.unwrap();

        if !payload.status().is_success() {
            let error_text = payload.text().unwrap();
            return Err(anyhow!("LLM API request failed: {}", error_text));
        }

        let response_json: Value = payload.json()?;

        let content = response_json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| anyhow!("Invalid response format from LLM API"))?;

        Ok(content.to_string())
    }

    fn parse_response(&self, response: &str, meaning: Option<&str>) -> Result<VibeValue> {
        match meaning {
            Some("temperature in Celsius") => {
                let temperature: f64 = response
                    .trim()
                    .parse()
                    .or_else(|_| self.extract_number_from_text(response))
                    .unwrap_or(0.0);

                Ok(VibeValue {
                    value_type: VibeValueType::Number,
                    data: VibeValueData::Number(temperature),
                })
            }
            Some("weather description") => Ok(VibeValue {
                value_type: VibeValueType::String,
                data: VibeValueData::String(response.to_string()),
            }),
            _ => Ok(VibeValue {
                value_type: VibeValueType::String,
                data: VibeValueData::String(response.to_string()),
            }),
        }
    }

    fn extract_number_from_text(&self, text: &str) -> Result<f64, std::num::ParseFloatError> {
        text.split_whitespace()
            .find_map(|word| {
                word.chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect::<String>()
                    .parse::<f64>()
                    .ok()
            })
            .ok_or_else(|| "0.0".parse::<f64>().unwrap_err())
    }
}

pub fn format_prompt(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (name, value) in variables {
        let placeholder = format!("{{{}}}", name);
        result = result.replace(&placeholder, value);
    }

    result
}

impl VibeValue {
    pub fn null() -> Self {
        Self {
            value_type: VibeValueType::Null,
            data: VibeValueData::Null,
        }
    }

    pub fn boolean(value: bool) -> Self {
        Self {
            value_type: VibeValueType::Boolean,
            data: VibeValueData::Boolean(value),
        }
    }

    pub fn number(value: f64) -> Self {
        Self {
            value_type: VibeValueType::Number,
            data: VibeValueData::Number(value),
        }
    }

    pub fn string(value: String) -> Self {
        Self {
            value_type: VibeValueType::String,
            data: VibeValueData::String(value),
        }
    }

    pub fn get_string(&self) -> Option<&String> {
        match &self.data {
            VibeValueData::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn get_number(&self) -> Option<f64> {
        match &self.data {
            VibeValueData::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match &self.data {
            VibeValueData::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn get_int(&self) -> Option<i32> {
        match &self.data {
            VibeValueData::Number(n) => Some(*n as i32),
            VibeValueData::String(s) => s.parse().ok(),
            VibeValueData::Boolean(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }
}
