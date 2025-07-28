use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use vibelang::runnable;

/// A command-line tool to compile and execute a VibeLang .vibe file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the VibeLang source file to execute.
    #[arg(required = true)]
    input_file: PathBuf,

    /// The directory where the generated Rust project will be placed.
    #[arg(short, long, default_value = ".generated")]
    output_dir: PathBuf,

    /// Generate as a library crate instead of a binary crate.
    #[arg(long, default_value_t = false)]
    as_lib: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("--- VibeLang Project Runner ---");
    
    // Run parser and code generation.
    runnable::run_file(&cli.input_file, &cli.output_dir, cli.as_lib)?;

    println!("\n✅ Process finished successfully.");
    Ok(())
}
