use crate::runtime::llm_provider::LlmProvider;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Handles the scaffolding of the generated Rust project.
/// It is generic over any type T that implements the LlmProvider trait.
pub struct ProjectBuilder<'a, T: LlmProvider> {
    llm_client: &'a T,
}

impl<'a, T: LlmProvider> ProjectBuilder<'a, T> {
    /// Creates a new ProjectBuilder with a reference to an LLM provider.
    pub fn new(llm_client: &'a T) -> Self {
        Self { llm_client }
    }

    /// Builds the project structure in the output directory.
    ///
    /// # Arguments
    /// * `output_dir` - Path where the generated project directory will be created.
    /// * `vibelang_source` - The original VibeLang source code.
    /// * `generated_rust_code` - The generated Rust code.
    /// * `as_lib` - If true, generates a library crate; if false, generates a binary crate.
    pub fn build(
        &self,
        output_dir: &Path,
        vibelang_source: &str,
        generated_rust_code: &str,
        as_lib: bool,
    ) -> Result<()> {
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let (package_name, bin_name) = self.generate_project_names(vibelang_source)?;
        let vibelang_version = self.get_vibelang_version()?;
        let cargo_content = self.create_cargo_toml_content(&package_name, &bin_name, as_lib, &vibelang_version)?;
        
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;

        // Generate either lib.rs or main.rs based on as_lib parameter
        if as_lib {
            fs::write(src_dir.join("lib.rs"), generated_rust_code)?;
        } else {
            fs::write(src_dir.join("main.rs"), generated_rust_code)?;
        }

        Ok(())
    }

    /// Reads the current vibelang version from the library's Cargo.toml file.
    fn get_vibelang_version(&self) -> Result<String> {
        // Try to find the Cargo.toml file in the current workspace
        let possible_paths = [
            Path::new("Cargo.toml"),
            Path::new("../Cargo.toml"),
            Path::new("../../Cargo.toml"),
        ];

        for path in &possible_paths {
            if path.exists() {
                let cargo_content = fs::read_to_string(path)?;
                if let Some(version) = self.extract_version_from_cargo_toml(&cargo_content) {
                    return Ok(version);
                }
            }
        }

        // Fallback to default version if Cargo.toml not found
        Ok("0.1.0".to_string())
    }

    /// Extracts the version from Cargo.toml content.
    fn extract_version_from_cargo_toml(&self, content: &str) -> Option<String> {
        let version_regex = Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).ok()?;
        version_regex.captures(content)?.get(1).map(|m| m.as_str().to_string())
    }

    /// Creates the Cargo.toml content for the project.
    ///
    /// # Arguments
    /// * `package_name` - The name of the package.
    /// * `bin_name` - The name of the binary (only used for binary crates).
    /// * `as_lib` - If true, generates library configuration; if false, generates binary configuration.
    /// * `vibelang_version` - The version of vibelang to use as dependency.
    fn create_cargo_toml_content(
        &self,
        package_name: &str,
        bin_name: &str,
        as_lib: bool,
        vibelang_version: &str,
    ) -> Result<String> {
        if as_lib {
            Ok(format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
vibelang = "{}"
anyhow = "1.0"
reqwest = {{ version = "0.12", features = ["json", "blocking"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}

[lib]
name = "{}"
crate-type = ["rlib"]
"#,
                package_name,
                vibelang_version,
                package_name.replace("-", "_")
            ))
        } else {
            Ok(format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
vibelang = "{}"
anyhow = "1.0"
reqwest = {{ version = "0.12", features = ["json", "blocking"] }}
serde_json = "1.0"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
                package_name, vibelang_version, bin_name
            ))
        }
    }

    fn extract_semantic_annotations(&self, source: &str) -> Vec<String> {
        let re = Regex::new(r#"Meaning<.+?>"(.+?)""#).unwrap();
        re.captures_iter(source)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn generate_project_names(&self, source: &str) -> Result<(String, String)> {
        let annotations = self.extract_semantic_annotations(source);
        if annotations.is_empty() {
            return Ok(("vibe-project".to_string(), "vibeapp".to_string()));
        }

        // This can generate context-aware project
        // let context = annotations.join(", ");
        // let prompt = format!(
        //     "Based on these concepts: {}, suggest a snake_case Rust package name and a binary name. Respond ONLY with JSON in the format {{\"packagename\": \"name\", \"binname\": \"name\"}}",
        //     context
        // );

        // if let Ok(response) = self.llm_client.generate(&prompt) {
        //     if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
        //         let package = json["packagename"].as_str().unwrap_or("vibe-project").replace("_", "-");
        //         let binary = json["binname"].as_str().unwrap_or("vibeapp").to_string();
        //         return Ok((package, binary));
        //     }
        // }

        Ok(("vibe-project".to_string(), "vibeapp".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::llm_provider::MockLlmProvider;

    #[test]
    fn test_version_extraction() {
        let mock_client = MockLlmProvider::new();
        let builder = ProjectBuilder::new(&mock_client);
        
        let cargo_content = r#"[package]
name = "vibelang"
version = "0.2.5"
edition = "2024"

[dependencies]
serde = "1.0"
"#;
        
        let version = builder.extract_version_from_cargo_toml(cargo_content);
        assert_eq!(version, Some("0.2.5".to_string()));
    }

    #[test]
    fn test_cargo_toml_generation_binary_with_version() {
        let mock_client = MockLlmProvider::new();
        let builder = ProjectBuilder::new(&mock_client);
        let content = builder.create_cargo_toml_content("mycoolpackage", "myapp", false, "0.2.5").unwrap();
        assert!(content.contains(r#"name = "mycoolpackage""#));
        assert!(content.contains(r#"name = "myapp""#));
        assert!(content.contains("[[bin]]"));
        assert!(content.contains(r#"vibelang = "0.2.5""#));
        assert!(!content.contains("[lib]"));
    }

    #[test]
    fn test_cargo_toml_generation_library_with_version() {
        let mock_client = MockLlmProvider::new();
        let builder = ProjectBuilder::new(&mock_client);
        let content = builder.create_cargo_toml_content("mycoolpackage", "myapp", true, "0.2.5").unwrap();
        assert!(content.contains(r#"name = "mycoolpackage""#));
        assert!(content.contains("[lib]"));
        assert!(content.contains("tokio"));
        assert!(content.contains(r#"vibelang = "0.2.5""#));
        assert!(!content.contains("[[bin]]"));
    }

    #[test]
    fn test_name_generation_with_valid_llm_json() {
        let mut mock_client = MockLlmProvider::new();
        mock_client
            .expect_generate()
            .times(1)
            .returning(|_| Ok(r#"{"packagename": "geography_tools", "binname": "capitalfinder"}"#.to_string()));

        let builder = ProjectBuilder::new(&mock_client);
        let source = r#"type Capital = Meaning<String>("the capital city of a country")"#;
        let (package_name, bin_name) = builder.generate_project_names(source).unwrap();

        assert_eq!(package_name, "geography-tools");
        assert_eq!(bin_name, "capitalfinder");
    }

    #[test]
    fn test_name_generation_falls_back_on_invalid_json() {
        let mut mock_client = MockLlmProvider::new();
        mock_client
            .expect_generate()
            .times(1)
            .returning(|_| Ok("I am not valid JSON.".to_string()));

        let builder = ProjectBuilder::new(&mock_client);
        let source = r#"type Capital = Meaning<String>("a capital city")"#;
        let (package_name, bin_name) = builder.generate_project_names(source).unwrap();

        assert_eq!(package_name, "vibe-project");
        assert_eq!(bin_name, "vibeapp");
    }

    #[test]
    fn test_name_generation_falls_back_with_no_annotations() {
        let mock_client = MockLlmProvider::new();
        
        let builder = ProjectBuilder::new(&mock_client);
        let source = r#"fn get_year() -> Int { prompt "What year is it?" }"#;
        let (package_name, bin_name) = builder.generate_project_names(source).unwrap();

        assert_eq!(package_name, "vibe-project");
        assert_eq!(bin_name, "vibeapp");
    }
}
