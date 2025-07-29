use anyhow::Result;
use vibelang::compiler::{codegen::CodeGenerator, parser::parse_source};

#[test]
fn test_joke_generation_payload() -> Result<()> {
    // --- Arrange ---
    let vibe_source = r#"
        // Define a type for the generated joke text
        type Joke = Meaning<String>("a short humorous line");
        type Topic = Meaning<String>("topic for the joke");

        fn tellJoke(topic: Topic) -> Joke {
            prompt "Tell me a short joke about {topic}.";
        }
    "#;

    // --- Act ---
    // Run the source through the parser and code generator.
    let ast = parse_source(vibe_source)?;
    let generated_code = CodeGenerator::new().generate(&ast, false)?;

    // --- Assert ---
    // Verify that the type aliases were created correctly.
    assert!(
        generated_code.contains("pub type Joke = String;"),
        "Joke type alias is incorrect."
    );
    assert!(
        generated_code.contains("pub type Topic = String;"),
        "Topic type alias is incorrect."
    );

    // Verify the function signature is correct.
    assert!(
        generated_code.contains("pub fn tellJoke(llm: &LlmClient, topic: Topic) -> Joke"),
        "tellJoke function signature is incorrect."
    );

    // Verify the prompt template is correctly embedded.
    assert!(
        generated_code
            .contains(r#"let mut template = "Tell me a short joke about {topic}.".to_string();"#),
        "Prompt template was not correctly generated."
    );

    // Verify that the semantic meaning was correctly inherited from the `Joke` return type.
    assert!(
        generated_code.contains(r#"let meaning = Some("a short humorous line");"#),
        "Semantic meaning was not inherited from the return type."
    );

    // Verify the correct conversion method is called.
    assert!(
        generated_code.contains("result.into_string()"),
        "Incorrect return type conversion method was used."
    );

    Ok(())
}

#[test]
fn test_greeting_generation_payload_with_inline_meaning() -> Result<()> {
    // --- Arrange ---
    let vibe_source = r#"
        type Greeting = Meaning<String>("a friendly greeting");
        type Language = Meaning<String>("language name");

        fn generateGreeting(name: Meaning<String>("person's name"), 
                           language: Language) -> Greeting {
            prompt "Generate a friendly greeting for a person named {name} in {language}.";
        }
    "#;

    // --- Act ---
    let ast = parse_source(vibe_source)?;
    let generated_code = CodeGenerator::new().generate(&ast, false)?;

    // --- Assert ---
    // This test is particularly important as it verifies how the compiler handles
    // a `Meaning` type defined inline as a function parameter.

    // Verify type aliases.
    assert!(generated_code.contains("pub type Greeting = String;"));
    assert!(generated_code.contains("pub type Language = String;"));

    // Verify the function signature. The inline `Meaning` for `name`
    // should be compiled down to its base Rust type (`String`).
    assert!(
        generated_code.contains(
            "pub fn generateGreeting(llm: &LlmClient, name: String, language: Language) -> Greeting"
        ),
        "generateGreeting function signature is incorrect. Check handling of inline Meaning parameter."
    );

    // Verify that both placeholders are correctly handled.
    assert!(
        generated_code.contains(r#"template = template.replace("{name}", &name.to_string());"#)
    );
    assert!(
        generated_code
            .contains(r#"template = template.replace("{language}", &language.to_string());"#)
    );

    // Verify the semantic meaning is inherited from the `Greeting` return type.
    assert!(
        generated_code.contains(r#"let meaning = Some("a friendly greeting");"#),
        "Semantic meaning was not inherited correctly."
    );

    Ok(())
}
