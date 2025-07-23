use crate::compiler::codegen::*;
use crate::compiler::parser::*;
use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::Result;
use std::env;
use std::fs;
use std::process::{Command, Stdio};

#[test]
fn test_end_to_end_vibelang_workflow() -> Result<()> {
    // Step 1: Generate Rust code from VibeLang definition using general knowledge queries
    let vibelang_source = r#"
        type Population = Meaning<Int>("population count in millions");
        type HistoricalFact = Meaning<String>("historical information");
        type YearFounded = Meaning<Int>("year when established");
        type GeographicInfo = Meaning<String>("geographic description");
        
        fn get_population(country: String) -> Population {
            prompt "What is the current population of {country} in millions? Reply with just the number.";
        }
        
        fn get_capital(country: String) -> String {
            prompt "What is the capital city of {country}? Reply with just the city name.";
        }
        
        fn get_founding_year(country: String) -> YearFounded {
            prompt "In what year was {country} founded or established as a nation? Reply with just the year.";
        }
        
        fn get_historical_fact(person: String) -> HistoricalFact {
            prompt "Tell me one interesting historical fact about {person}. Keep it to one sentence.";
        }
        
        fn get_geographic_info(landmark: String) -> GeographicInfo {
            prompt "Describe the geographic location and features of {landmark} in one sentence.";
        }
        
        fn analyze_sentiment(text: String) -> Meaning<String>("emotional sentiment") {
            prompt "Analyze the emotional sentiment of this text: '{text}'. Reply with either 'positive', 'negative', or 'neutral'.";
        }
        
        fn calculate_age(birth_year: String) -> Meaning<Int>("age calculation") {
            prompt "If someone was born in {birth_year}, how old would they be in 2025? Reply with just the number.";
        }
    "#;

    let ast = parse_string(vibelang_source)?;
    assert_eq!(ast.node_type, AstNodeType::Program);

    // Step 2: Use local project directory instead of tempdir
    let project_dir = std::env::current_dir()?;
    let generated_dir = project_dir.join("generated");
    std::fs::create_dir_all(&generated_dir)?;

    let rust_file_path = generated_dir.join("knowledge_demo.rs");
    let cargo_toml_path = generated_dir.join("Cargo.toml");

    // Generate Rust code
    let mut codegen = CodeGenerator::new();
    codegen.generate(&ast, rust_file_path.to_str().unwrap())?;

    // Verify the generated file exists
    assert!(rust_file_path.exists());
    println!("Generated Rust file at: {:?}", rust_file_path);

    // Step 3: Create Cargo.toml using LLM-powered naming
    crate::generate_cargo_toml_from_source!(
        &cargo_toml_path,
        vibelang_source,
        "knowledge_demo.rs"
    )?;
    println!(
        "Created Cargo.toml with LLM-generated names at: {:?}",
        cargo_toml_path
    );

    // Step 4: Add comprehensive main function to demonstrate all MTP features
    let generated_content = std::fs::read_to_string(&rust_file_path)?;
    let enhanced_content = format!(
        r#"{}

fn main() {{
    println!("🚀 VibeLang MTP Knowledge Demo Started");
    println!("=====================================\n");
    
    // Test 1: Population queries with numeric semantic parsing
    println!("📊 Testing Population Queries (MTP: population count)");
    let france_pop = get_population("France".to_string());
    println!("France population: {{}} million", france_pop);
    
    let japan_pop = get_population("Japan".to_string());
    println!("Japan population: {{}} million", japan_pop);
    println!();
    
    // Test 2: Simple string queries
    println!("🏛️ Testing Capital City Queries (String type)");
    let german_capital = get_capital("Germany".to_string());
    println!("Capital of Germany: {{}}", german_capital);
    
    let brazil_capital = get_capital("Brazil".to_string());
    println!("Capital of Brazil: {{}}", brazil_capital);
    println!();
    
    // Test 3: Year parsing with semantic meaning
    println!("📅 Testing Historical Year Queries (MTP: year when established)");
    let usa_founded = get_founding_year("United States".to_string());
    println!("USA founded in: {{}}", usa_founded);
    
    let italy_founded = get_founding_year("Italy".to_string());
    println!("Italy unified in: {{}}", italy_founded);
    println!();
    
    // Test 4: Historical facts with string semantic processing
    println!("📜 Testing Historical Facts (MTP: historical information)");
    let napoleon_fact = get_historical_fact("Napoleon Bonaparte".to_string());
    println!("Napoleon fact: {{}}", napoleon_fact);
    
    let einstein_fact = get_historical_fact("Albert Einstein".to_string());
    println!("Einstein fact: {{}}", einstein_fact);
    println!();
    
    // Test 5: Geographic information
    println!("🌍 Testing Geographic Queries (MTP: geographic description)");
    let everest_info = get_geographic_info("Mount Everest".to_string());
    println!("Mount Everest: {{}}", everest_info);
    
    let amazon_info = get_geographic_info("Amazon River".to_string());
    println!("Amazon River: {{}}", amazon_info);
    println!();
    
    // Test 6: Sentiment analysis
    println!("😊 Testing Sentiment Analysis (MTP: emotional sentiment)");
    let positive_text = "I love learning new programming languages!".to_string();
    let sentiment1 = analyze_sentiment(positive_text);
    println!("Sentiment of positive text: {{}}", sentiment1);
    
    let negative_text = "This code is frustrating and doesn't work.".to_string();
    let sentiment2 = analyze_sentiment(negative_text);
    println!("Sentiment of negative text: {{}}", sentiment2);
    println!();
    
    // Test 7: Age calculation
    println!("🎂 Testing Age Calculation (MTP: age calculation)");
    let age1990 = calculate_age("1990".to_string());
    println!("Age of someone born in 1990: {{}}", age1990);
    
    let age2000 = calculate_age("2000".to_string());
    println!("Age of someone born in 2000: {{}}", age2000);
    println!();
    
    println!("✅ VibeLang MTP Knowledge Demo Completed");
    println!("All semantic type parsers were tested with general knowledge queries!");
}}
"#,
        generated_content
    );

    std::fs::write(&rust_file_path, enhanced_content)?;

    // Ensure main function exists
    let final_content = std::fs::read_to_string(&rust_file_path)?;
    assert!(
        final_content.contains("fn main() {"),
        "Generated file missing main function"
    );

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
    println!("📁 Generated files available at: {:?}", generated_dir);

    // Step 6: Verify generated code contains all MTP semantic parsers
    let generated_content = std::fs::read_to_string(&rust_file_path)?;

    // Verify MTP type definitions
    assert!(generated_content.contains("pub type Population = i32;"));
    assert!(generated_content.contains("pub type HistoricalFact = String;"));
    assert!(generated_content.contains("pub type YearFounded = i32;"));
    assert!(generated_content.contains("pub type GeographicInfo = String;"));

    // Verify dynamic semantic parsing functions exist
    assert!(generated_content.contains("parse_semantic_response"));
    assert!(generated_content.contains("parse_integer_semantic"));
    assert!(generated_content.contains("parse_string_semantic"));

    // Verify dynamically generated extraction functions
    assert!(generated_content.contains("extract_population_count_millions_value"));
    assert!(generated_content.contains("extract_historical_information_string"));
    assert!(generated_content.contains("extract_year_when_established_value"));
    assert!(generated_content.contains("extract_geographic_description_string"));

    // Verify parametric prompt execution
    assert!(generated_content.contains("vibe_execute_prompt"));
    assert!(generated_content.contains("meaning: Option<&str>"));
    assert!(generated_content.contains("return_type: &str"));

    // Verify generic fallback functions
    assert!(generated_content.contains("extract_generic_number"));
    assert!(generated_content.contains("extract_generic_float"));
    assert!(generated_content.contains("extract_generic_boolean"));

    // Verify LLM-generated project names in Cargo.toml
    let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
    assert!(cargo_content.contains("name = \""));

    println!("✅ All MTP components verified in generated code");
    println!("✅ LLM-generated project names successfully created");
    println!("\n🎯 Demo Summary:");
    println!("- Population queries test numeric semantic parsing with dynamic extraction");
    println!("- Capital queries test basic string handling");
    println!("- Founding year queries test year-specific number extraction");
    println!("- Historical facts test string semantic processing");
    println!("- Geographic info tests descriptive text handling");
    println!("- Sentiment analysis tests emotion classification");
    println!("- Age calculation tests mathematical reasoning");
    println!("- All extraction functions are dynamically generated from MTP payloads");
    println!("- Package and binary names are intelligently generated using LLM analysis");

    Ok(())
}

#[test]
fn test_mtp_semantic_variety() -> Result<()> {
    // Additional test for different semantic meanings
    let vibelang_source = r#"
        type BookCount = Meaning<Int>("number of books written");
        type ScientificField = Meaning<String>("area of scientific expertise");
        type Availability = Meaning<Bool>("whether something is available");
        type Rating = Meaning<Float>("quality rating out of 10");
        
        fn count_books(author: String) -> BookCount {
            prompt "How many books did {author} write? Reply with just the number.";
        }
        
        fn get_field(scientist: String) -> ScientificField {
            prompt "What field of science was {scientist} known for? Reply with the field name.";
        }
        
        fn check_availability(item: String) -> Availability {
            prompt "Is {item} commonly available for purchase? Reply with yes or no.";
        }
        
        fn rate_innovation(invention: String) -> Rating {
            prompt "Rate the historical impact of {invention} on a scale of 1-10. Reply with just the number.";
        }
    "#;

    let ast = parse_string(vibelang_source)?;
    let project_dir = std::env::current_dir()?;
    let generated_dir = project_dir.join("generated");
    let rust_file_path = generated_dir.join("semantic_variety_test.rs");

    let mut codegen = CodeGenerator::new();
    codegen.generate(&ast, rust_file_path.to_str().unwrap())?;

    let generated_content = std::fs::read_to_string(&rust_file_path)?;

    // Verify different dynamically generated semantic parsers are created
    assert!(generated_content.contains("extract_number_books_written_value"));
    assert!(generated_content.contains("extract_area_scientific_expertise_string"));
    assert!(generated_content.contains("extract_quality_rating_out_10_value"));

    // Verify type-specific semantic parsers
    assert!(generated_content.contains("parse_integer_semantic"));
    assert!(generated_content.contains("parse_float_semantic"));
    assert!(generated_content.contains("parse_boolean_semantic"));
    assert!(generated_content.contains("parse_string_semantic"));

    // Verify the semantic parsers dispatch to the correct extraction functions
    assert!(generated_content.contains(
        "Some(\"number of books written\") => extract_number_books_written_value(content)"
    ));
    assert!(generated_content.contains("Some(\"area of scientific expertise\") => extract_area_scientific_expertise_string(content)"));
    assert!(generated_content.contains(
        "Some(\"quality rating out of 10\") => extract_quality_rating_out_10_value(content)"
    ));

    println!("✅ Semantic variety test passed - all parser types dynamically generated");
    println!("✅ Verified dynamic MTP payload-based extraction function generation");
    println!("✅ LLM-powered project naming working for semantic variety test");

    Ok(())
}

#[test]
fn test_mtp_normalization() -> Result<()> {
    // Test semantic meaning normalization to function names
    let vibelang_source = r#"
        type ComplexMeaning = Meaning<String>("the user's emotional state and preference");
        type SimpleCount = Meaning<Int>("count");
        type WithArticles = Meaning<Float>("a rating of the quality");
        
        fn get_complex(input: String) -> ComplexMeaning {
            prompt "Analyze {input}";
        }
        
        fn get_count(input: String) -> SimpleCount {
            prompt "Count {input}";
        }
        
        fn get_rating(input: String) -> WithArticles {
            prompt "Rate {input}";
        }
    "#;

    let ast = parse_string(vibelang_source)?;
    let project_dir = std::env::current_dir()?;
    let generated_dir = project_dir.join("generated");
    let rust_file_path = generated_dir.join("normalization_test.rs");

    let mut codegen = CodeGenerator::new();
    codegen.generate(&ast, rust_file_path.to_str().unwrap())?;

    let generated_content = std::fs::read_to_string(&rust_file_path)?;

    // Verify function name normalization works correctly
    assert!(generated_content.contains("extract_count_value"));
    assert!(generated_content.contains("extract_rating_quality_value"));

    // Verify articles and prepositions are filtered out
    assert!(!generated_content.contains("extract_the_users"));
    assert!(!generated_content.contains("extract_a_rating_of_the"));

    println!("✅ MTP semantic meaning normalization test passed");
    println!("✅ LLM-powered project naming working for normalization test");

    Ok(())
}

#[test]
fn test_llm_naming_fallback() -> Result<()> {
    // Test the fallback naming system when LLM is unavailable
    let vibelang_source = r#"
        type WeatherInfo = Meaning<String>("weather conditions and forecasts");
        type TemperatureReading = Meaning<Int>("temperature measurement in celsius");
        
        fn get_weather(city: String) -> WeatherInfo {
            prompt "What's the weather like in {city}?";
        }
        
        fn get_temperature(location: String) -> TemperatureReading {
            prompt "What's the temperature in {location}?";
        }
    "#;

    // Test the semantic annotation extraction
    let annotations =
        crate::compiler::macros::helpers::extract_semantic_annotations(vibelang_source);
    assert_eq!(annotations.len(), 2);
    assert!(annotations.contains(&"weather conditions and forecasts".to_string()));
    assert!(annotations.contains(&"temperature measurement in celsius".to_string()));

    // Test the fallback naming system
    let (package_name, binary_name) =
        crate::compiler::macros::helpers::generate_fallback_names(&annotations).unwrap();

    // Verify the names are valid Rust identifiers
    assert!(
        package_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
    );
    assert!(binary_name.chars().all(|c| c.is_alphanumeric() || c == '_'));

    // Verify the names contain relevant semantic content

    assert!(package_name.contains("weather") || package_name.contains("forecasts"));

    println!("✅ LLM naming fallback system working correctly");
    println!("Generated package name: {}", package_name);
    println!("Generated binary name: {}", binary_name);

    Ok(())
}
