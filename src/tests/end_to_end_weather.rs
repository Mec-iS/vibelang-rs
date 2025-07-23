use crate::compiler::codegen::*;
use crate::compiler::parser::*;
use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::Result;
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn test_end_to_end_vibelang_workflow() -> Result<()> {
    // Step 1: Generate Rust code from VibeLang definition
    let vibelang_source = r#"
        type Weather = Meaning<String>("weather description");
        
        fn get_weather(city: String) -> Weather {
            prompt "What is the weather like in {city}? Provide a brief description.";
        }
        
        fn get_temperature(city: String) -> Meaning<Int>("temperature in Celsius") {
            prompt "What is the current temperature in {city}? Reply with just the number.";
        }
    "#;

    let ast = parse_string(vibelang_source)?;
    assert_eq!(ast.node_type, AstNodeType::Program);

    // Step 2: Use local project directory instead of tempdir
    let project_dir = std::env::current_dir()?;
    let generated_dir = project_dir.join("generated");
    std::fs::create_dir_all(&generated_dir)?;

    let rust_file_path = generated_dir.join("vibelang_workflow.rs");
    let cargo_toml_path = generated_dir.join("Cargo.toml");

    // Generate Rust code
    let mut codegen = CodeGenerator::new();
    codegen.generate(&ast, rust_file_path.to_str().unwrap())?;

    // Verify the generated file exists
    assert!(rust_file_path.exists());
    println!("Generated Rust file at: {:?}", rust_file_path);

    // Step 3: Create Cargo.toml for the generated project
    let cargo_toml_content = r#"
[package]
name = "vibelang-generated"
version = "0.1.0"
edition = "2024"

[dependencies]
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json", "blocking"] }

[[bin]]
name = "vibelang_workflow"
path = "vibelang_workflow.rs"
"#;

    std::fs::write(&cargo_toml_path, cargo_toml_content)?;
    println!("Created Cargo.toml at: {:?}", cargo_toml_path);

    // Step 4: Add main function to the generated code
    let generated_content = std::fs::read_to_string(&rust_file_path)?;
    let enhanced_content = format!(
        "{}\n\nfn main() {{\n    println!(\"VibeLang Workflow Started\");\n    let city = \"Paris\".to_string();\n    let weather = get_weather(city.clone());\n    println!(\"Weather: {{}}\", weather);\n    let temp = get_temperature(city);\n    println!(\"Temperature: {{}}\", temp);\n}}",
        generated_content
    );

    std::fs::write(&rust_file_path, enhanced_content)?;

    // Step 5: Compile the generated Rust code
    let compile_output = std::process::Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&generated_dir)
        .output()?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        eprintln!("Compilation failed: {}", stderr);
        panic!("Failed to compile generated Rust code");
    }

    println!("✅ Generated Rust code compiled successfully");
    println!("You can inspect the files at: {:?}", generated_dir);

    Ok(())
}
