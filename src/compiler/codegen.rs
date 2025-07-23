use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub struct CodeGenerator {
    indent_level: usize,
    current_return_type: Option<String>,
    current_semantic_meaning: Option<String>,
    type_aliases: HashMap<String, String>,
    semantic_meanings: HashMap<String, (String, String)>, // meaning -> (rust_type, normalized_name)
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            current_return_type: None,
            current_semantic_meaning: None,
            type_aliases: HashMap::new(),
            semantic_meanings: HashMap::new(),
        }
    }

    /// Collect all semantic meanings from MTP type declarations
    fn collect_semantic_meanings(&mut self, ast: &AstNode) {
        for child in &ast.children {
            if child.node_type == AstNodeType::TypeDecl {
                if let Some(type_name) = child.get_string("name") {
                    if !child.children.is_empty() {
                        let type_child = &child.children[0];
                        if type_child.node_type == AstNodeType::MeaningType {
                            if let Some(meaning) = type_child.get_string("meaning") {
                                let base_type = self.determine_base_type(type_child);
                                let normalized_name =
                                    self.normalize_meaning_to_function_name(meaning);
                                self.semantic_meanings
                                    .insert(meaning.clone(), (base_type, normalized_name));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Convert semantic meaning text to a valid function name
    fn normalize_meaning_to_function_name(&self, meaning: &str) -> String {
        meaning
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .split('_')
            .filter(|s| {
                !s.is_empty() && s != &"a" && s != &"an" && s != &"the" && s != &"of" && s != &"in"
            })
            .collect::<Vec<_>>()
            .join("_")
    }

    pub fn generate(&mut self, ast: &AstNode, output_path: &str) -> Result<()> {
        let mut file = File::create(output_path)?;

        // First collect all semantic meanings from MTP declarations
        self.collect_semantic_meanings(ast);

        self.generate_headers(&mut file)?;

        // First pass: Generate type declarations to build alias map
        for child in &ast.children {
            if child.node_type == AstNodeType::TypeDecl {
                self.generate_type_declaration(&mut file, child)?;
            }
        }

        // Second pass: Generate other declarations
        for child in &ast.children {
            match child.node_type {
                AstNodeType::FunctionDecl => self.generate_function(&mut file, child)?,
                AstNodeType::ClassDecl => self.generate_struct_declaration(&mut file, child)?,
                AstNodeType::TypeDecl => {} // Already processed
                _ => {}
            }
        }

        Ok(())
    }

    fn generate_headers(&self, file: &mut File) -> Result<()> {
        // Generate basic headers and imports
        crate::generate_file_header!(file);
        crate::generate_imports!(file);

        // Generate parametric vibe_execute_prompt
        crate::generate_parametric_vibe_execute_prompt!(file);

        // Generate dynamic semantic parser based on collected meanings
        self.generate_dynamic_semantic_parser(file)?;

        // Generate dynamic extraction utilities based on MTP payloads
        self.generate_dynamic_extraction_utilities(file)?;

        // Generate other utilities
        crate::generate_format_prompt_function!(file);
        crate::generate_vibe_value_enum!(file);

        Ok(())
    }

    /// Generate semantic parser that dispatches to dynamically created extraction functions
    fn generate_dynamic_semantic_parser(&self, file: &mut File) -> Result<()> {
        writeln!(
            file,
            "fn parse_semantic_response(content: &str, meaning: Option<&str>, return_type: &str) -> VibeValue {{"
        )?;
        writeln!(
            file,
            "    // Parse based on resolved primitive type, not type alias"
        )?;
        writeln!(file, "    match return_type {{")?;
        writeln!(
            file,
            "        \"i32\" => parse_integer_semantic(content, meaning),"
        )?;
        writeln!(
            file,
            "        \"f64\" => parse_float_semantic(content, meaning),"
        )?;
        writeln!(
            file,
            "        \"bool\" => parse_boolean_semantic(content, meaning),"
        )?;
        writeln!(
            file,
            "        _ => parse_string_semantic(content, meaning),"
        )?;
        writeln!(file, "    }}")?;
        writeln!(file, "}}")?;
        writeln!(file)?;

        // Generate type-specific parsers with dynamic dispatch
        self.generate_integer_semantic_parser(file)?;
        self.generate_float_semantic_parser(file)?;
        self.generate_boolean_semantic_parser(file)?;
        self.generate_string_semantic_parser(file)?;

        Ok(())
    }

    fn generate_integer_semantic_parser(&self, file: &mut File) -> Result<()> {
        writeln!(
            file,
            "fn parse_integer_semantic(content: &str, meaning: Option<&str>) -> VibeValue {{"
        )?;
        writeln!(file, "    // Try direct parsing first")?;
        writeln!(
            file,
            "    if let Ok(value) = content.trim().parse::<i32>() {{"
        )?;
        writeln!(file, "        return VibeValue::Number(value as f64);")?;
        writeln!(file, "    }}")?;
        writeln!(file)?;
        writeln!(
            file,
            "    // Extract number based on semantic meaning from MTP payload"
        )?;
        writeln!(file, "    let extracted = match meaning {{")?;

        // Generate match arms for each integer semantic meaning
        for (meaning, (rust_type, normalized_name)) in &self.semantic_meanings {
            if rust_type == "i32" {
                writeln!(
                    file,
                    "        Some(\"{}\") => extract_{}_value(content),",
                    meaning, normalized_name
                )?;
            }
        }

        writeln!(file, "        _ => extract_generic_number(content),")?;
        writeln!(file, "    }};")?;
        writeln!(file)?;
        writeln!(file, "    VibeValue::Number(extracted as f64)")?;
        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_float_semantic_parser(&self, file: &mut File) -> Result<()> {
        writeln!(
            file,
            "fn parse_float_semantic(content: &str, meaning: Option<&str>) -> VibeValue {{"
        )?;
        writeln!(
            file,
            "    if let Ok(value) = content.trim().parse::<f64>() {{"
        )?;
        writeln!(file, "        return VibeValue::Number(value);")?;
        writeln!(file, "    }}")?;
        writeln!(file)?;
        writeln!(file, "    let extracted = match meaning {{")?;

        // Generate match arms for each float semantic meaning
        for (meaning, (rust_type, normalized_name)) in &self.semantic_meanings {
            if rust_type == "f64" {
                writeln!(
                    file,
                    "        Some(\"{}\") => extract_{}_value(content),",
                    meaning, normalized_name
                )?;
            }
        }

        writeln!(file, "        _ => extract_generic_float(content),")?;
        writeln!(file, "    }};")?;
        writeln!(file)?;
        writeln!(file, "    VibeValue::Number(extracted)")?;
        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_boolean_semantic_parser(&self, file: &mut File) -> Result<()> {
        writeln!(
            file,
            "fn parse_boolean_semantic(content: &str, meaning: Option<&str>) -> VibeValue {{"
        )?;
        writeln!(file, "    let content_lower = content.to_lowercase();")?;
        writeln!(file)?;
        writeln!(file, "    // Direct boolean parsing")?;
        writeln!(
            file,
            "    if content_lower.contains(\"true\") || content_lower.contains(\"yes\") {{"
        )?;
        writeln!(file, "        return VibeValue::Boolean(true);")?;
        writeln!(file, "    }}")?;
        writeln!(
            file,
            "    if content_lower.contains(\"false\") || content_lower.contains(\"no\") {{"
        )?;
        writeln!(file, "        return VibeValue::Boolean(false);")?;
        writeln!(file, "    }}")?;
        writeln!(file)?;
        writeln!(
            file,
            "    // Semantic-based boolean extraction from MTP payload"
        )?;
        writeln!(file, "    let result = match meaning {{")?;

        // Generate match arms for each boolean semantic meaning
        for (meaning, (rust_type, normalized_name)) in &self.semantic_meanings {
            if rust_type == "bool" {
                writeln!(
                    file,
                    "        Some(\"{}\") => extract_{}_boolean(content),",
                    meaning, normalized_name
                )?;
            }
        }

        writeln!(file, "        _ => extract_generic_boolean(content),")?;
        writeln!(file, "    }};")?;
        writeln!(file)?;
        writeln!(file, "    VibeValue::Boolean(result)")?;
        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_string_semantic_parser(&self, file: &mut File) -> Result<()> {
        writeln!(
            file,
            "fn parse_string_semantic(content: &str, meaning: Option<&str>) -> VibeValue {{"
        )?;
        writeln!(file, "    let processed = match meaning {{")?;

        // Generate match arms for each string semantic meaning
        for (meaning, (rust_type, normalized_name)) in &self.semantic_meanings {
            if rust_type == "String" {
                writeln!(
                    file,
                    "        Some(\"{}\") => extract_{}_string(content),",
                    meaning, normalized_name
                )?;
            }
        }

        writeln!(file, "        _ => content.trim().to_string(),")?;
        writeln!(file, "    }};")?;
        writeln!(file)?;
        writeln!(file, "    VibeValue::String(processed)")?;
        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    /// Generate extraction functions dynamically based on MTP semantic meanings
    fn generate_dynamic_extraction_utilities(&self, file: &mut File) -> Result<()> {
        writeln!(
            file,
            "// Dynamically generated extraction functions based on MTP payloads"
        )?;
        writeln!(file)?;

        // Generate extraction functions for each semantic meaning
        for (meaning, (rust_type, normalized_name)) in &self.semantic_meanings {
            match rust_type.as_str() {
                "i32" => {
                    self.generate_integer_extraction_function(file, meaning, normalized_name)?
                }
                "f64" => self.generate_float_extraction_function(file, meaning, normalized_name)?,
                "bool" => {
                    self.generate_boolean_extraction_function(file, meaning, normalized_name)?
                }
                _ => self.generate_string_extraction_function(file, meaning, normalized_name)?,
            }
        }

        // Generate generic fallback functions
        self.generate_generic_extraction_functions(file)?;

        Ok(())
    }

    fn generate_integer_extraction_function(
        &self,
        file: &mut File,
        meaning: &str,
        normalized_name: &str,
    ) -> Result<()> {
        writeln!(
            file,
            "fn extract_{}_value(text: &str) -> i32 {{",
            normalized_name
        )?;
        writeln!(
            file,
            "    // Extract integer for semantic meaning: \"{}\"",
            meaning
        )?;

        // Generate specific extraction logic based on meaning content
        if meaning.contains("books") || meaning.contains("count") || meaning.contains("number") {
            writeln!(
                file,
                "    // Look for book count or general number patterns"
            )?;
            writeln!(file, "    text.split_whitespace()")?;
            writeln!(file, "        .find_map(|word| word.parse::<i32>().ok())")?;
            writeln!(file, "        .unwrap_or(0)")?;
        } else if meaning.contains("year")
            || meaning.contains("age")
            || meaning.contains("founded")
            || meaning.contains("established")
        {
            writeln!(
                file,
                "    // Look for year/age patterns in reasonable ranges"
            )?;
            writeln!(file, "    for word in text.split_whitespace() {{")?;
            writeln!(
                file,
                "        if let Ok(value) = word.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<i32>() {{"
            )?;
            if meaning.contains("age") {
                writeln!(
                    file,
                    "            if value > 0 && value < 150 {{ return value; }}"
                )?;
            } else {
                writeln!(
                    file,
                    "            if value >= 1000 && value <= 2100 {{ return value; }}"
                )?;
            }
            writeln!(file, "        }}")?;
            writeln!(file, "    }}")?;
            writeln!(file, "    0")?;
        } else if meaning.contains("population") {
            writeln!(
                file,
                "    // Look for population numbers, possibly in millions"
            )?;
            writeln!(file, "    text.split_whitespace()")?;
            writeln!(file, "        .find_map(|word| {{")?;
            writeln!(file, "            word.chars()")?;
            writeln!(
                file,
                "                .filter(|c| c.is_ascii_digit() || *c == '.')"
            )?;
            writeln!(file, "                .collect::<String>()")?;
            writeln!(file, "                .parse::<f64>().ok()")?;
            writeln!(file, "                .map(|f| f as i32)")?;
            writeln!(file, "        }})")?;
            writeln!(file, "        .unwrap_or(0)")?;
        } else {
            writeln!(file, "    // Generic integer extraction")?;
            writeln!(file, "    extract_generic_number(text)")?;
        }

        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_float_extraction_function(
        &self,
        file: &mut File,
        meaning: &str,
        normalized_name: &str,
    ) -> Result<()> {
        writeln!(
            file,
            "fn extract_{}_value(text: &str) -> f64 {{",
            normalized_name
        )?;
        writeln!(
            file,
            "    // Extract float for semantic meaning: \"{}\"",
            meaning
        )?;

        if meaning.contains("rating") || meaning.contains("scale") || meaning.contains("quality") {
            writeln!(
                file,
                "    // Look for rating patterns: \"4.5/5\", \"8 out of 10\", \"7.2\""
            )?;
            writeln!(file, "    text.split_whitespace()")?;
            writeln!(file, "        .find_map(|word| {{")?;
            writeln!(file, "            word.chars()")?;
            writeln!(
                file,
                "                .filter(|c| c.is_ascii_digit() || *c == '.')"
            )?;
            writeln!(file, "                .collect::<String>()")?;
            writeln!(file, "                .parse::<f64>().ok()")?;
            writeln!(file, "        }})")?;
            writeln!(file, "        .unwrap_or(0.0)")?;
        } else if meaning.contains("price") || meaning.contains("cost") {
            writeln!(
                file,
                "    // Look for price patterns: \"$25.99\", \"25.99 USD\""
            )?;
            writeln!(file, "    text.split_whitespace()")?;
            writeln!(file, "        .find_map(|word| {{")?;
            writeln!(file, "            word.chars()")?;
            writeln!(
                file,
                "                .filter(|c| c.is_ascii_digit() || *c == '.')"
            )?;
            writeln!(file, "                .collect::<String>()")?;
            writeln!(file, "                .parse::<f64>().ok()")?;
            writeln!(file, "        }})")?;
            writeln!(file, "        .unwrap_or(0.0)")?;
        } else if meaning.contains("percentage") || meaning.contains("percent") {
            writeln!(
                file,
                "    // Look for percentage patterns: \"75%\", \"75 percent\""
            )?;
            writeln!(file, "    text.split_whitespace()")?;
            writeln!(file, "        .find_map(|word| {{")?;
            writeln!(
                file,
                "            word.trim_end_matches('%').parse::<f64>().ok()"
            )?;
            writeln!(file, "        }})")?;
            writeln!(file, "        .unwrap_or(0.0)")?;
        } else {
            writeln!(file, "    // Generic float extraction")?;
            writeln!(file, "    extract_generic_float(text)")?;
        }

        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_boolean_extraction_function(
        &self,
        file: &mut File,
        meaning: &str,
        normalized_name: &str,
    ) -> Result<()> {
        writeln!(
            file,
            "fn extract_{}_boolean(text: &str) -> bool {{",
            normalized_name
        )?;
        writeln!(
            file,
            "    // Extract boolean for semantic meaning: \"{}\"",
            meaning
        )?;
        writeln!(file, "    let text_lower = text.to_lowercase();")?;

        if meaning.contains("available") || meaning.contains("availability") {
            writeln!(
                file,
                "    let available_words = [\"available\", \"in stock\", \"yes\", \"true\", \"ready\", \"open\"];"
            )?;
            writeln!(
                file,
                "    available_words.iter().any(|&word| text_lower.contains(word))"
            )?;
        } else if meaning.contains("recommendation") || meaning.contains("recommend") {
            writeln!(
                file,
                "    let recommend_words = [\"recommend\", \"suggest\", \"yes\", \"should\", \"good\", \"advised\"];"
            )?;
            writeln!(
                file,
                "    recommend_words.iter().any(|&word| text_lower.contains(word))"
            )?;
        } else if meaning.contains("sentiment") {
            writeln!(
                file,
                "    let positive_words = [\"positive\", \"good\", \"happy\", \"excellent\", \"great\"];"
            )?;
            writeln!(
                file,
                "    positive_words.iter().any(|&word| text_lower.contains(word))"
            )?;
        } else {
            writeln!(file, "    // Generic boolean extraction")?;
            writeln!(
                file,
                "    text_lower.contains(\"yes\") || text_lower.contains(\"true\") || text_lower.contains(\"positive\")"
            )?;
        }

        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_string_extraction_function(
        &self,
        file: &mut File,
        meaning: &str,
        normalized_name: &str,
    ) -> Result<()> {
        writeln!(
            file,
            "fn extract_{}_string(text: &str) -> String {{",
            normalized_name
        )?;
        writeln!(
            file,
            "    // Extract string for semantic meaning: \"{}\"",
            meaning
        )?;

        if meaning.contains("field") || meaning.contains("area") || meaning.contains("expertise") {
            writeln!(
                file,
                "    // Extract field/area of expertise - usually a single specialized term"
            )?;
            writeln!(file, "    text.split_whitespace()")?;
            writeln!(
                file,
                "        .find(|word| word.len() > 3 && word.chars().all(|c| c.is_alphabetic()))"
            )?;
            writeln!(file, "        .unwrap_or(\"unknown\")")?;
            writeln!(file, "        .to_string()")?;
        } else if meaning.contains("description")
            || meaning.contains("information")
            || meaning.contains("geographic")
        {
            writeln!(
                file,
                "    // Extract descriptive information - keep full text but trim"
            )?;
            writeln!(file, "    text.trim().to_string()")?;
        } else if meaning.contains("summary") {
            writeln!(
                file,
                "    // Extract summary - first sentence or limited length"
            )?;
            writeln!(
                file,
                "    if let Some(first_sentence) = text.split('.').next() {{"
            )?;
            writeln!(file, "        first_sentence.trim().to_string()")?;
            writeln!(file, "    }} else if text.len() > 100 {{")?;
            writeln!(file, "        format!(\"{{}}...\", &text[..97])")?;
            writeln!(file, "    }} else {{")?;
            writeln!(file, "        text.trim().to_string()")?;
            writeln!(file, "    }}")?;
        } else if meaning.contains("historical") {
            writeln!(
                file,
                "    // Extract historical facts, keep first sentence or up to 200 chars"
            )?;
            writeln!(
                file,
                "    if let Some(first_sentence) = text.split('.').next() {{"
            )?;
            writeln!(file, "        first_sentence.trim().to_string()")?;
            writeln!(file, "    }} else if text.len() > 200 {{")?;
            writeln!(file, "        format!(\"{{}}...\", &text[..197])")?;
            writeln!(file, "    }} else {{")?;
            writeln!(file, "        text.trim().to_string()")?;
            writeln!(file, "    }}")?;
        } else {
            writeln!(file, "    // Generic string extraction")?;
            writeln!(file, "    text.trim().to_string()")?;
        }

        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_generic_extraction_functions(&self, file: &mut File) -> Result<()> {
        writeln!(file, "// Generic fallback extraction functions")?;
        writeln!(file)?;

        writeln!(file, "fn extract_generic_number(text: &str) -> i32 {{")?;
        writeln!(file, "    text.split_whitespace()")?;
        writeln!(file, "        .find_map(|word| word.parse::<i32>().ok())")?;
        writeln!(file, "        .unwrap_or(0)")?;
        writeln!(file, "}}")?;
        writeln!(file)?;

        writeln!(file, "fn extract_generic_float(text: &str) -> f64 {{")?;
        writeln!(file, "    text.split_whitespace()")?;
        writeln!(file, "        .find_map(|word| word.parse::<f64>().ok())")?;
        writeln!(file, "        .unwrap_or(0.0)")?;
        writeln!(file, "}}")?;
        writeln!(file)?;

        writeln!(file, "fn extract_generic_boolean(text: &str) -> bool {{")?;
        writeln!(file, "    let text_lower = text.to_lowercase();")?;
        writeln!(
            file,
            "    text_lower.contains(\"true\") || text_lower.contains(\"yes\") || text_lower.contains(\"positive\")"
        )?;
        writeln!(file, "}}")?;
        writeln!(file)?;

        Ok(())
    }

    // ... (rest of the existing methods remain the same)

    fn collect_parameters(&self, func: &AstNode) -> Vec<String> {
        let mut params = Vec::new();

        for child in &func.children {
            if child.node_type == AstNodeType::ParamList {
                for param in &child.children {
                    if param.node_type == AstNodeType::Parameter {
                        if let Some(param_name) = param.get_string("name") {
                            let param_type = self.determine_parameter_type(param);
                            params.push(format!("{}: {}", param_name, param_type));
                        }
                    }
                }
                break;
            }
        }

        params
    }

    fn generate_function_body(&mut self, file: &mut File, func: &AstNode) -> Result<()> {
        for child in &func.children {
            if child.node_type == AstNodeType::FunctionBody {
                for stmt in &child.children {
                    self.generate_statement(file, stmt)?;
                }
                break;
            }
        }
        Ok(())
    }

    fn generate_statement(&mut self, file: &mut File, stmt: &AstNode) -> Result<()> {
        match stmt.node_type {
            AstNodeType::VarDecl => self.generate_variable_declaration(file, stmt),
            AstNodeType::ReturnStmt => self.generate_return_statement(file, stmt),
            AstNodeType::PromptBlock => self.generate_prompt_block(file, stmt),
            AstNodeType::ExprStmt => self.generate_expression_statement(file, stmt),
            AstNodeType::Block => self.generate_block(file, stmt),
            _ => {
                self.write_indent(file)?;
                writeln!(file, "// Unsupported statement type")?;
                Ok(())
            }
        }
    }

    fn generate_prompt_block(&mut self, file: &mut File, prompt: &AstNode) -> Result<()> {
        let template_str = prompt
            .get_string("template")
            .ok_or_else(|| anyhow!("Prompt template not found"))?;

        let variables = self.extract_template_variables(template_str);

        self.write_indent(file)?;
        writeln!(file, "{{")?;
        self.indent_level += 1;

        self.write_indent(file)?;
        writeln!(file, "let prompt_template = \"{}\";", template_str)?;

        if !variables.is_empty() {
            self.write_indent(file)?;
            writeln!(file, "let mut variables = HashMap::new();")?;

            for var in &variables {
                self.write_indent(file)?;
                writeln!(
                    file,
                    "variables.insert(\"{}\".to_string(), {}.to_string());",
                    var, var
                )?;
            }

            self.write_indent(file)?;
            writeln!(
                file,
                "let formatted_prompt = format_prompt(prompt_template, &variables);"
            )?;
        } else {
            self.write_indent(file)?;
            writeln!(file, "let formatted_prompt = prompt_template.to_string();")?;
        }

        // Use parametric prompt execution with semantic meaning and return type
        let return_type = self.determine_return_type_from_context(prompt);
        let meaning_context = self.current_semantic_meaning.as_deref();

        self.write_indent(file)?;
        match meaning_context {
            Some(meaning) => {
                writeln!(
                    file,
                    "let prompt_result = vibe_execute_prompt(&formatted_prompt, Some(\"{}\"), \"{}\");",
                    meaning, return_type
                )?;
            }
            None => {
                writeln!(
                    file,
                    "let prompt_result = vibe_execute_prompt(&formatted_prompt, None, \"{}\");",
                    return_type
                )?;
            }
        }

        // Generate type-aware return conversion
        self.write_indent(file)?;
        self.generate_type_conversion(file, &return_type)?;

        self.indent_level -= 1;
        self.write_indent(file)?;
        writeln!(file, "}}")?;

        Ok(())
    }

    fn resolve_type_alias(&self, type_name: &str) -> String {
        // First check if it's a known type alias
        if let Some(resolved_type) = self.type_aliases.get(type_name) {
            return resolved_type.clone();
        }

        // If not found in aliases, return the original type name
        // (could be a primitive type already)
        type_name.to_string()
    }

    fn generate_type_conversion(&mut self, file: &mut File, return_type: &str) -> Result<()> {
        // Resolve type alias to underlying type
        let resolved_type = self.resolve_type_alias(return_type);

        match resolved_type.as_str() {
            "i32" => {
                writeln!(file, "return match prompt_result {{")?;
                self.indent_level += 1;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Number(n) => n as i32,")?;
                self.write_indent(file)?;
                writeln!(
                    file,
                    "VibeValue::String(s) => s.parse::<i32>().unwrap_or(0),"
                )?;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Boolean(b) => if b {{ 1 }} else {{ 0 }},")?;
                self.write_indent(file)?;
                writeln!(file, "_ => 0,")?;
                self.indent_level -= 1;
                self.write_indent(file)?;
                writeln!(file, "}};")?;
            }
            "f64" => {
                writeln!(file, "return match prompt_result {{")?;
                self.indent_level += 1;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Number(n) => n,")?;
                self.write_indent(file)?;
                writeln!(
                    file,
                    "VibeValue::String(s) => s.parse::<f64>().unwrap_or(0.0),"
                )?;
                self.write_indent(file)?;
                writeln!(
                    file,
                    "VibeValue::Boolean(b) => if b {{ 1.0 }} else {{ 0.0 }},"
                )?;
                self.write_indent(file)?;
                writeln!(file, "_ => 0.0,")?;
                self.indent_level -= 1;
                self.write_indent(file)?;
                writeln!(file, "}};")?;
            }
            "bool" => {
                writeln!(file, "return match prompt_result {{")?;
                self.indent_level += 1;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Boolean(b) => b,")?;
                self.write_indent(file)?;
                writeln!(
                    file,
                    "VibeValue::String(s) => s.to_lowercase().contains(\"true\") || s.to_lowercase().contains(\"yes\"),"
                )?;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Number(n) => n != 0.0,")?;
                self.write_indent(file)?;
                writeln!(file, "_ => false,")?;
                self.indent_level -= 1;
                self.write_indent(file)?;
                writeln!(file, "}};")?;
            }
            _ => {
                // Default to String return - this is the only case that should return strings
                writeln!(file, "return match prompt_result {{")?;
                self.indent_level += 1;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::String(s) => s,")?;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Number(n) => n.to_string(),")?;
                self.write_indent(file)?;
                writeln!(file, "VibeValue::Boolean(b) => b.to_string(),")?;
                self.write_indent(file)?;
                writeln!(file, "_ => String::new(),")?;
                self.indent_level -= 1;
                self.write_indent(file)?;
                writeln!(file, "}};")?;
            }
        }
        Ok(())
    }

    fn determine_return_type_from_context(&self, _prompt: &AstNode) -> String {
        if let Some(return_type) = &self.current_return_type {
            return_type.clone()
        } else {
            "String".to_string()
        }
    }

    fn generate_function(&mut self, file: &mut File, func: &AstNode) -> Result<()> {
        let func_name = func
            .get_string("name")
            .ok_or_else(|| anyhow!("Function missing name"))?;

        let return_type = self.determine_return_type(func);
        let semantic_meaning = self.extract_semantic_meaning_from_function(func);

        // Store context for prompt block generation
        self.current_return_type = Some(return_type.clone());
        self.current_semantic_meaning = semantic_meaning;

        write!(file, "pub fn {}(", func_name)?;

        let params = self.collect_parameters(func);
        write!(file, "{}) -> {} {{", params.join(", "), return_type)?;
        writeln!(file)?;

        self.indent_level += 1;
        self.generate_function_body(file, func)?;
        self.indent_level -= 1;

        writeln!(file, "}}")?;
        writeln!(file)?;

        // Clear context
        self.current_return_type = None;
        self.current_semantic_meaning = None;

        Ok(())
    }

    fn generate_variable_declaration(&mut self, file: &mut File, var: &AstNode) -> Result<()> {
        let var_name = var
            .get_string("name")
            .ok_or_else(|| anyhow!("Variable missing name"))?;

        let var_type = self.infer_variable_type(var);

        self.write_indent(file)?;
        write!(file, "let {}: {} = ", var_name, var_type)?;

        // Find initialization expression
        for child in &var.children {
            if !matches!(
                child.node_type,
                AstNodeType::BasicType | AstNodeType::MeaningType
            ) {
                self.generate_expression(file, child)?;
                break;
            }
        }

        writeln!(file, ";")?;
        Ok(())
    }

    fn generate_expression(&mut self, file: &mut File, expr: &AstNode) -> Result<()> {
        match expr.node_type {
            AstNodeType::StringLiteral => {
                let empty = &format!("");
                let value = expr.get_string("value").unwrap_or(&empty);
                write!(file, "\"{}\".to_string()", value)?;
            }
            AstNodeType::IntLiteral => {
                let value = expr.get_int("value").unwrap_or(0);
                write!(file, "{}", value)?;
            }
            AstNodeType::FloatLiteral => {
                let value = expr.get_float("value").unwrap_or(0.0);
                write!(file, "{}", value)?;
            }
            AstNodeType::BoolLiteral => {
                let value = expr.get_bool("value").unwrap_or(false);
                write!(file, "{}", value)?;
            }
            AstNodeType::Identifier => {
                let empty = format!("{}", "unknown_identifier");
                let name = expr.get_string("name").unwrap_or(&empty);
                write!(file, "{}", name)?;
            }
            AstNodeType::CallExpr => {
                let empty = format!("{}", "unknown_function");
                let func_name = expr.get_string("function").unwrap_or(&empty);
                write!(file, "{}(", func_name)?;

                for (i, arg) in expr.children.iter().enumerate() {
                    if i > 0 {
                        write!(file, ", ")?;
                    }
                    self.generate_expression(file, arg)?;
                }

                write!(file, ")")?;
            }
            _ => {
                write!(file, "/* Unsupported expression */")?;
            }
        }
        Ok(())
    }

    fn generate_struct_declaration(&mut self, file: &mut File, class: &AstNode) -> Result<()> {
        let struct_name = class
            .get_string("name")
            .ok_or_else(|| anyhow!("Struct missing name"))?;

        writeln!(file, "#[derive(Debug, Clone)]")?;
        writeln!(file, "pub struct {} {{", struct_name)?;

        self.indent_level += 1;
        self.generate_struct_fields(file, class)?;
        self.indent_level -= 1;

        writeln!(file, "}}")?;
        writeln!(file)?;

        self.generate_impl_block(file, class, struct_name)?;

        Ok(())
    }

    fn generate_struct_fields(&mut self, file: &mut File, class: &AstNode) -> Result<()> {
        for child in &class.children {
            if child.node_type == AstNodeType::MemberVar {
                if let Some(member_name) = child.get_string("name") {
                    if !child.children.is_empty() {
                        let member_type = self.determine_base_type(&child.children[0]);
                        self.write_indent(file)?;
                        writeln!(file, "pub {}: {},", member_name, member_type)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn generate_impl_block(
        &mut self,
        file: &mut File,
        class: &AstNode,
        struct_name: &str,
    ) -> Result<()> {
        writeln!(file, "impl {} {{", struct_name)?;
        self.indent_level += 1;

        for child in &class.children {
            if child.node_type == AstNodeType::FunctionDecl {
                self.generate_method(file, child)?;
            }
        }

        self.indent_level -= 1;
        writeln!(file, "}}")?;
        writeln!(file)?;
        Ok(())
    }

    fn generate_method(&mut self, file: &mut File, func: &AstNode) -> Result<()> {
        let func_name = func
            .get_string("name")
            .ok_or_else(|| anyhow!("Function missing name"))?;

        let return_type = self.determine_return_type(func);
        let params = self.collect_parameters(func);

        self.write_indent(file)?;
        write!(file, "pub fn {}(&self", func_name)?;

        for param in params {
            write!(file, ", {}", param)?;
        }

        writeln!(file, ") -> {} {{", return_type)?;

        self.indent_level += 1;
        self.generate_function_body(file, func)?;
        self.indent_level -= 1;

        self.write_indent(file)?;
        writeln!(file, "}}")?;
        writeln!(file)?;

        Ok(())
    }

    fn determine_return_type(&self, func: &AstNode) -> String {
        for child in &func.children {
            match child.node_type {
                AstNodeType::BasicType => {
                    if let Some(type_name) = child.get_string("type") {
                        return self.map_to_rust_type(type_name);
                    }
                }
                AstNodeType::MeaningType => {
                    if !child.children.is_empty() {
                        let base_child = &child.children[0];
                        if let Some(base_type_name) = base_child.get_string("type") {
                            return self.map_to_rust_type(base_type_name);
                        }
                    }
                }
                _ => {}
            }
        }
        "String".to_string()
    }

    fn extract_semantic_meaning_from_function(&self, func: &AstNode) -> Option<String> {
        for child in &func.children {
            if child.node_type == AstNodeType::MeaningType {
                return child.get_string("meaning").map(|s| s.clone());
            }
        }
        None
    }

    fn determine_parameter_type(&self, param: &AstNode) -> String {
        for child in &param.children {
            if let Some(type_name) = self.extract_type_name(child) {
                return self.map_to_rust_type(&type_name);
            }
        }
        "()".to_string()
    }

    fn infer_variable_type(&self, var: &AstNode) -> String {
        for child in &var.children {
            if let Some(type_name) = self.extract_type_name(child) {
                return self.map_to_rust_type(&type_name);
            }
        }

        for child in &var.children {
            match child.node_type {
                AstNodeType::StringLiteral => return "String".to_string(),
                AstNodeType::IntLiteral => return "i32".to_string(),
                AstNodeType::FloatLiteral => return "f64".to_string(),
                AstNodeType::BoolLiteral => return "bool".to_string(),
                _ => {}
            }
        }

        "()".to_string()
    }

    fn determine_base_type(&self, type_node: &AstNode) -> String {
        if let Some(type_name) = self.extract_type_name(type_node) {
            self.map_to_rust_type(&type_name)
        } else {
            "()".to_string()
        }
    }

    fn extract_type_name(&self, node: &AstNode) -> Option<String> {
        match node.node_type {
            AstNodeType::BasicType => node.get_string("type").map(|s| s.clone()),
            AstNodeType::MeaningType => {
                if !node.children.is_empty() {
                    self.extract_type_name(&node.children[0])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn map_to_rust_type(&self, vibe_type: &str) -> String {
        match vibe_type {
            "Int" => "i32".to_string(),
            "Float" => "f64".to_string(),
            "String" => "String".to_string(),
            "Bool" => "bool".to_string(),
            "void" => "()".to_string(),
            _ => vibe_type.to_string(),
        }
    }

    fn extract_template_variables(&self, template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        if !var_name.is_empty() {
                            variables.push(var_name);
                        }
                        break;
                    }
                    var_name.push(ch);
                }
            }
        }

        variables
    }

    fn generate_return_statement(&mut self, file: &mut File, ret: &AstNode) -> Result<()> {
        self.write_indent(file)?;
        write!(file, "return")?;

        if !ret.children.is_empty() {
            write!(file, " ")?;
            self.generate_expression(file, &ret.children[0])?;
        }

        writeln!(file, ";")?;
        Ok(())
    }

    fn generate_expression_statement(
        &mut self,
        file: &mut File,
        expr_stmt: &AstNode,
    ) -> Result<()> {
        self.write_indent(file)?;
        if !expr_stmt.children.is_empty() {
            self.generate_expression(file, &expr_stmt.children[0])?;
        }
        writeln!(file, ";")?;
        Ok(())
    }

    fn generate_block(&mut self, file: &mut File, block: &AstNode) -> Result<()> {
        self.write_indent(file)?;
        writeln!(file, "{{")?;

        self.indent_level += 1;
        for child in &block.children {
            self.generate_statement(file, child)?;
        }
        self.indent_level -= 1;

        self.write_indent(file)?;
        writeln!(file, "}}")?;
        Ok(())
    }

    fn generate_type_declaration(&mut self, file: &mut File, type_decl: &AstNode) -> Result<()> {
        let type_name = type_decl
            .get_string("name")
            .ok_or_else(|| anyhow!("Type declaration missing name"))?;

        writeln!(file, "// MTP Type {}", type_name)?;

        if !type_decl.children.is_empty() {
            let child = &type_decl.children[0];

            if child.node_type == AstNodeType::MeaningType {
                if let Some(meaning) = child.get_string("meaning") {
                    writeln!(file, "// Semantic meaning: \"{}\"", meaning)?;
                }
            }

            let base_type = self.determine_base_type(child);
            writeln!(file, "pub type {} = {};", type_name, base_type)?;

            // Store the alias mapping for later resolution
            self.type_aliases.insert(type_name.clone(), base_type);
        }

        writeln!(file)?;
        Ok(())
    }

    fn write_indent(&self, file: &mut File) -> Result<()> {
        for _ in 0..self.indent_level {
            write!(file, "    ")?;
        }
        Ok(())
    }
}
