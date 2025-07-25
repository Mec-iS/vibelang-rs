pub mod codegen;
pub mod parser;
pub mod project_builder;

use anyhow::Result;
use codegen::CodeGenerator;
use parser::parse_source;

/// A convenience function to compile VibeLang source code directly into Rust code.
///
/// This function encapsulates the parsing (AST creation) and code generation steps.
///
/// # Arguments
/// * `source` - A string slice containing the VibeLang source code.
///
/// # Returns
/// A `Result` containing the generated Rust code as a `String`, or an error if
/// parsing or code generation fails.
pub fn compile(source: &str) -> Result<String> {
    // Step 1: Parse the source code into an Abstract Syntax Tree (AST).
    let ast = parse_source(source)?;

    // Step 2: Generate the Rust code from the AST.
    let codegen = CodeGenerator::new();
    let generated_code = codegen.generate(&ast)?;

    Ok(generated_code)
}
