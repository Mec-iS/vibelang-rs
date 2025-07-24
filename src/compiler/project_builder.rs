// src/compiler/project_builder.rs
use crate::runtime::client::LlmClient;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

pub struct ProjectBuilder {
    llm_client: LlmClient,
}

impl ProjectBuilder {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }

    pub fn create_project(
        &self,
        output_dir: &Path,
        vibelang_source: &str,
        rust_file_name: &str,
        rust_code: &str,
    ) -> Result<()> {
        fs::create_dir_all(output_dir)?;

        // Write the generated Rust code
        fs::write(output_dir.join(rust_file_name), rust_code)?;

        // Generate and write Cargo.toml
        let annotations = self.extract_semantic_annotations(vibelang_source);
        let (package_name, bin_name) = self.generate_names(&annotations)?;

        let cargo_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
vibelang = {{ path = "../" }} # Assuming vibelang is in a parent dir for dev
anyhow = "1.0"
reqwest = {{ version = "0.12", features = ["json", "blocking"] }}
serde_json = "1.0"

[[bin]]
name = "{}"
path = "{}"
"#,
            package_name, bin_name, rust_file_name
        );
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;

        Ok(())
    }

    fn extract_semantic_annotations(&self, source: &str) -> Vec<String> {
        let re = Regex::new(r#"Meaning<.+?>\("([^"]+)"\)"#).unwrap();
        re.captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn generate_names(&self, annotations: &[String]) -> Result<(String, String)> {
        if annotations.is_empty() {
            return Ok(("vibe-project".to_string(), "main".to_string()));
        }

        let context = annotations.join(", ");
        let prompt = format!(
            "Based on these concepts: [{}], suggest a snake_case Rust package name and a binary name. Respond in JSON format: {{\"package\": \"name\", \"binary\": \"name\"}}",
            context
        );

        if let Ok(response) = self.llm_client.generate(&prompt) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                 let package = json["package"].as_str().unwrap_or("vibe-project").to_string();
                 let binary = json["binary"].as_str().unwrap_or("main").to_string();
                 return Ok((package, binary));
            }
        }
        
        // Fallback
        Ok(("vibe-project".to_string(), "main".to_string()))
    }
}
