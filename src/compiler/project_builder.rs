use crate::runtime::llm_provider::LlmProvider; // Import the trait
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Handles the scaffolding of the generated Rust project.
/// It is generic over any type `T` that implements the LlmProvider trait.
pub struct ProjectBuilder<'a, T: LlmProvider> {
    llm_client: &'a T,
}

impl<'a, T: LlmProvider> ProjectBuilder<'a, T> {
    /// Creates a new ProjectBuilder with a reference to an LLM provider.
    pub fn new(llm_client: &'a T) -> Self {
        Self { llm_client }
    }

    // -- The rest of the implementation is unchanged --

    pub fn build(
        &self,
        output_dir: &Path,
        vibelang_source: &str,
        generated_rust_code: &str,
    ) -> Result<()> {
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let (package_name, bin_name) = self.generate_project_names(vibelang_source)?;
        let cargo_content = self.create_cargo_toml_content(&package_name, &bin_name)?;
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;
        fs::write(src_dir.join("main.rs"), generated_rust_code)?;

        Ok(())
    }

    fn create_cargo_toml_content(&self, package_name: &str, bin_name: &str) -> Result<String> {
        Ok(format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
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

    fn extract_semantic_annotations(&self, source: &str) -> Vec<String> {
        let re = Regex::new(r#"Meaning<.+?>\("([^"]+)"\)"#).unwrap();
        re.captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }
    
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
        
        Ok(("vibe-project".to_string(), "vibe_app".to_string()))
    }
}


// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    // Import the automatically generated mock struct from the LlmProvider module.
    use crate::runtime::llm_provider::MockLlmProvider;

    #[test]
    fn test_cargo_toml_generation() {
        // This test doesn't involve the LLM, so we create a mock but don't
        // set any expectations on its `generate` method.
        let mock_client = MockLlmProvider::new();
        let builder = ProjectBuilder::new(&mock_client);
        let content = builder.create_cargo_toml_content("my_cool_package", "my_app").unwrap();

        assert!(content.contains(r#"name = "my_cool_package""#));
        assert!(content.contains(r#"name = "my_app""#));
    }

    #[test]
    fn test_name_generation_with_valid_llm_json() {
        // 1. Create an instance of the automock.
        let mut mock_client = MockLlmProvider::new();

        // 2. Set expectations: The `generate` method should be called once
        //    and return a valid JSON string.
        mock_client.expect_generate()
            .times(1)
            .returning(|_| Ok(r#"{"package_name": "geography_tools", "bin_name": "capital_finder"}"#.to_string()));

        // 3. Pass the mock to the builder and execute the logic.
        let builder = ProjectBuilder::new(&mock_client);
        let source = r#"type Capital = Meaning<String>("the capital city of a country");"#;
        let (package_name, bin_name) = builder.generate_project_names(source).unwrap();

        // 4. Assert the result.
        assert_eq!(package_name, "geography_tools");
        assert_eq!(bin_name, "capital_finder");
    }

    #[test]
    fn test_name_generation_falls_back_on_invalid_json() {
        let mut mock_client = MockLlmProvider::new();

        // Set expectation: `generate` is called once but returns a non-JSON string.
        mock_client.expect_generate()
            .times(1)
            .returning(|_| Ok("I am not valid JSON.".to_string()));

        let builder = ProjectBuilder::new(&mock_client);
        let source = r#"type Capital = Meaning<String>("a capital city");"#;
        let (package_name, bin_name) = builder.generate_project_names(source).unwrap();

        // Assert that the builder logic gracefully falls back to default names.
        assert_eq!(package_name, "vibe-project");
        assert_eq!(bin_name, "vibe_app");
    }

    #[test]
    fn test_name_generation_falls_back_with_no_annotations() {
        let mut mock_client = MockLlmProvider::new();

        // Set expectation: The `generate` method should NEVER be called.
        // This is a powerful feature of mockall that verifies the logic short-circuits correctly.
        mock_client.expect_generate()
            .times(0);

        let builder = ProjectBuilder::new(&mock_client);
        let source = "fn get_year() -> Int { prompt \"What year is it?\"; }"; // No 'Meaning' annotations
        let (package_name, bin_name) = builder.generate_project_names(source).unwrap();

        // Assert the fallback behavior.
        assert_eq!(package_name, "vibe-project");
        assert_eq!(bin_name, "vibe_app");
    }
}
