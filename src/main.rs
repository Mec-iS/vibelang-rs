// src/main.rs
use anyhow::Result;
use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;
use vibelang::compiler::codegen::CodeGenerator;
use vibelang::compiler::parser::parse_source;
use vibelang::compiler::project_builder::ProjectBuilder;
use vibelang::config::Config;
use vibelang::runtime::client::LlmClient;

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the VibeLang (.vibe) input file
    #[arg(required = true)]
    input_file: PathBuf,

    /// Directory to place the generated Rust project
    #[arg(short, long, default_value = "./generated")]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_env();

    println!("Reading source file: {:?}", &args.input_file);
    let source = fs::read_to_string(&args.input_file)?;

    // 1. Parse the source code into an AST
    println!("Parsing VibeLang source...");
    let ast = parse_source(&source)?;

    // 2. Generate the Rust code from the AST
    println!("Generating Rust code...");
    let codegen = CodeGenerator::new();
    let generated_code = codegen.generate(&ast)?;

    // 3. Set up the output project using the ProjectBuilder
    println!("Creating project structure at: {:?}", &args.output_dir);
    let llm_client = LlmClient::new(config)?;
    let project_builder = ProjectBuilder::new(&llm_client);

    // Delegate all project creation logic to the builder.
    project_builder.build(&args.output_dir, &source, &generated_code)?;

    println!("\n✅ Compilation successful!");
    println!("Generated project is located in: {:?}", &args.output_dir);

    Ok(())
}
