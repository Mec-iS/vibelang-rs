use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::compiler;
use crate::compiler::project_builder::ProjectBuilder;
use crate::config::Config;
use crate::runtime::client::LlmClient;

/// Compiles a VibeLang source file, scaffolds a project, and runs it.
///
/// This function handles the end-to-end process:
/// 1. Reads and compiles the VibeLang source into Rust code.
/// 2. Uses the `ProjectBuilder` to create a new Cargo project in the specified output directory.
/// 3. Executes `cargo run` within the new project's directory to compile and run the binary.
///
/// # Arguments
/// * `source_path` - Path to the input `.vibe` file.
/// * `output_dir` - Path where the "generated" project directory will be created.
pub fn run_file<P: AsRef<Path>>(source_path: P, output_dir: P, as_lib: bool) -> Result<()> {
    let source_path = source_path.as_ref();
    let output_dir = output_dir.as_ref();

    // Step 1: Generate the Rust code from the source file.
    println!("⚙️  [1/3] Compiling VibeLang source from: {:?}", source_path);
    let source_code = fs::read_to_string(source_path)?;
    let generated_code = compiler::compile(&source_code, as_lib)?;

    // Step 2: Build the project structure in the 'generated' directory.
    println!("⚙️  [2/3] Generating project structure at: {:?}", output_dir);
    let config = Config::from_env();
    let llm_client = LlmClient::new(config)?;
    let project_builder = ProjectBuilder::new(&llm_client);
    project_builder.build(output_dir, &source_code, &generated_code, as_lib)?;

    if as_lib == true {
        println!("\n✅ Library file has been created at {:?}", output_dir);
        return Ok(());
    }

    // Step 3: Compile and run the generated project's binary.
    println!("⚙️  [3/3] Compiling and running the generated project...");
    let status = Command::new("cargo")
        .arg("run")
        .current_dir(output_dir)
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to compile or run the generated project. Review the output above for errors.");
    }

    Ok(())
}
