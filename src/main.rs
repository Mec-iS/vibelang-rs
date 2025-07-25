use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use vibelang::runnable;

/// A command-line tool to compile and execute a VibeLang (.vibe) file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the VibeLang source file to execute.
    #[arg(required = true)]
    input_file: PathBuf,

    /// The directory where the generated Rust project will be placed.
    #[arg(short, long, default_value = "./generated")]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("--- VibeLang Project Runner ---");
    
    // Run parser and code generation.
    runnable::run_file(&cli.input_file, &cli.output_dir)?;

    println!("\n✅ Process finished successfully.");
    Ok(())
}
