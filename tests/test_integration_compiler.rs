// tests/compiler_tests.rs
use anyhow::Result;
use tempfile::tempdir;
use vibelang::compiler::{
    codegen::CodeGenerator, parser::parse_source, project_builder::ProjectBuilder,
};
use vibelang::config::Config;
use vibelang::runtime::client::LlmClient;

#[test]
fn test_end_to_end_compilation() -> Result<()> {
    // 1. Setup: Define source code and create a temporary output directory.
    let vibe_source = r#"
        type Capital = Meaning<String>("the capital city of a country");
        fn get_capital(country: String) -> Capital {
            prompt "What is the capital of {country}?";
        }
    "#;
    let temp_dir = tempdir()?;
    let output_path = temp_dir.path();

    // 2. Execution: Run the core compiler logic.
    let config = Config::from_env();
    let llm_client = LlmClient::new(config)?;
    let ast = parse_source(vibe_source)?;
    let generated_code = CodeGenerator::new().generate(&ast)?;

    let builder = ProjectBuilder::new(&llm_client);
    builder.build(output_path, vibe_source, &generated_code)?;

    // 3. Verification: Check that the expected files were created.
    let cargo_toml_path = output_path.join("Cargo.toml");
    let main_rs_path = output_path.join("src/main.rs");

    assert!(cargo_toml_path.exists(), "Cargo.toml was not created");
    assert!(main_rs_path.exists(), "src/main.rs was not created");

    let main_content = std::fs::read_to_string(main_rs_path)?;
    assert!(
        main_content.contains("pub fn get_capital"),
        "get_capital function not found in generated code"
    );

    // You could even try to compile and run the generated project here for a full-cycle test.

    Ok(())
}
