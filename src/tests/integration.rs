use crate::compiler::parser::*;
use crate::compiler::codegen::*;
use crate::utils::ast::{AstNode, AstNodeType};
use crate::runtime::llm_interface::{LlmInterface, VibeValueType, format_prompt};
use std::env;

#[tokio::test]
async fn test_complete_mtp_workflow() {   
    let source = r#"
        type Weather = Meaning<String>("weather description");
        
        fn get_weather(city: String) -> Weather {
            prompt "What is the weather in {city}?";
        }
        
        fn get_temperature(city: String) -> Meaning<Int>("temperature in Celsius") {
            prompt "What is the temperature in {city}?";
        }
    "#;
    
    let ast = parse_string(source).unwrap();
    assert_eq!(ast.node_type, AstNodeType::Program);
    assert_eq!(ast.children.len(), 3); // type decl + 2 functions
    
    // Test code generation
    let mut codegen = CodeGenerator::new();
    let result = codegen.generate(&ast, "/tmp/test_mtp.rs");
    assert!(result.is_ok());
    
    use std::path::Path;
    assert!(Path::new(&"/tmp/test_mtp.rs").exists());

    // Verify generated C code contains MTP elements
    let generated = std::fs::read_to_string("/tmp/test_mtp.rs").unwrap();
    println!("{:?}", generated);

    assert!(generated.contains("get_temperature(city: String) -> i32"));
    assert!(generated.contains("get_weather(city: String) -> Weather"));
    assert!(generated.contains("pub type Weather = String;"));
    assert!(generated.contains("result = result.replace(&format!(\"{}\", name), value);"));
}

#[test]
fn test_parser_meaning_types() {
    let source = r#"
        type Temperature = Meaning<Int>("temperature in Celsius");
        type Sentiment = Meaning<String>("emotional sentiment analysis");
    "#;
    
    let ast = parse_string(source).unwrap();
    assert_eq!(ast.children.len(), 2);
    
    let temp_type = &ast.children[0];
    assert_eq!(temp_type.node_type, AstNodeType::TypeDecl);
    assert_eq!(temp_type.get_string("name").unwrap(), "Temperature");
    
    let meaning_node = &temp_type.children[0];
    assert_eq!(meaning_node.node_type, AstNodeType::MeaningType);
    assert_eq!(meaning_node.get_string("meaning").unwrap(), "temperature in Celsius");
    
    let base_type = &meaning_node.children[0];
    assert_eq!(base_type.node_type, AstNodeType::BasicType);
    assert_eq!(base_type.get_string("type").unwrap(), "Int");
}

#[test]
fn test_format_prompt() {
    let template = "Weather in {city} on {day}";
    let mut variables = std::collections::HashMap::new();
    variables.insert("city".to_string(), "Tokyo".to_string());
    variables.insert("day".to_string(), "Monday".to_string());
    
    let result = format_prompt(template, &variables);
    assert_eq!(result, "Weather in Tokyo on Monday");
}

#[test] 
fn test_advanced_prompt_parsing() {
    let source = r#"
        fn analyze_sentiment(text: String) -> Meaning<String>("emotional sentiment") {
            let processed_text: String = preprocess(text);
            prompt "Analyze the sentiment of: {processed_text}";
        }
    "#;
    
    let ast = parse_string(source).unwrap();
    let function = &ast.children[0];
    
    assert_eq!(function.node_type, AstNodeType::FunctionDecl);
    assert_eq!(function.get_string("name").unwrap(), "analyze_sentiment");
    
    // Check return type is MeaningType
    let return_type = &function.children[1]; // params, return_type, body
    assert_eq!(return_type.node_type, AstNodeType::MeaningType);
    assert_eq!(return_type.get_string("meaning").unwrap(), "emotional sentiment");
}
