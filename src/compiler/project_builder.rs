// src/compiler/project_builder.rs
use crate::runtime::client::LlmClient;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Handles the scaffolding of the generated Rust project.
pub struct ProjectBuilder<'a> {
    llm_client: &'a LlmClient,
}

impl<'a> ProjectBuilder<'a> {
    /// Creates a new ProjectBuilder with a reference to the LLM client.
    pub fn new(llm_client: &'a LlmClient) -> Self {
        Self { llm_client }
    }

    /// Builds the entire project structure.
    pub fn build(
        &self,
        output_dir: &Path,
        vibelang_source: &str,
        generated_rust_code: &str,
    ) -> Result<()> {
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // 1. Determine project names, using the LLM for suggestions.
        let (package_name, bin_name) = self.generate_project_names(vibelang_source)?;

        // 2. Generate and write the Cargo.toml file.
        let cargo_content = self.create_cargo_toml_content(&package_name, &bin_name)?;
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;

        // 3. Write the generated Rust code to src/main.rs.
        fs::write(src_dir.join("main.rs"), generated_rust_code)?;

        Ok(())
    }

    /// Generates the content for the Cargo.toml file.
    fn create_cargo_toml_content(&self, package_name: &str, bin_name: &str) -> Result<String> {
        Ok(format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
# Make sure the path to the vibelang library is correct for your setup.
# Using a git dependency or publishing to crates.io is recommended for real-world usage.
vibelang = {{ path = "../" }}
anyhow = "1.0"
reqwest = {{ version = "0.12", features = ["json", "blocking"] }}
serde_json = "1.0"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
            package_name, bin_name
        ))
    }

    /// Extracts semantic "Meaning" annotations from the source to guide LLM naming.
    fn extract_semantic_annotations(&self, source: &str) -> Vec<String> {
        let re = Regex::new(r#"Meaning<.+?>\("([^"]+)"\)"#).unwrap();
        re.captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Uses the LLM to suggest a project name based on semantic annotations.
    /// Falls back to a default name if the LLM fails or no annotations are found.
    fn generate_project_names(&self, source: &str) -> Result<(String, String)> {
        let annotations = self.extract_semantic_annotations(source);
        if annotations.is_empty() {
            return Ok(("vibe-project".to_string(), "vibe_app".to_string()));
        }

        let context = annotations.join(", ");
        let prompt = format!(
            "Based on these concepts: [{}], suggest a snake_case Rust package name and a binary name. Respond ONLY with JSON in the format: {{\"package_name\": \"name\", \"bin_name\": \"name\"}}",
            context
        );

        if let Ok(response) = self.llm_client.generate(&prompt) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                let package = json["package_name"].as_str().unwrap_or("vibe-project").replace('-', "_");
                let binary = json["bin_name"].as_str().unwrap_or("vibe_app").to_string();
                return Ok((package, binary));
            }
        }
        
        // Fallback
        Ok(("vibe-project".to_string(), "vibe_app".to_string()))
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_toml_generation() {
        let content = ProjectBuilder::new(&LlmClient::new(Default::default()).unwrap())
            .create_cargo_toml_content("my_cool_package", "my_app")
            .unwrap();

        assert!(content.contains(r#"name = "my_cool_package""#));
        assert!(content.contains(r#"name = "my_app""#));
        assert!(content.contains(r#"path = "src/main.rs""#));
    }
}
