use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use tera::{Context, Tera};

pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template("main.rs.tera", include_str!("../../templates/main.rs.tera"))
        .expect("Failed to parse template");
    tera
});

#[derive(Serialize)]
struct TypeAlias {
    name: String,
    base_type: String,
    meaning: Option<String>,
}

#[derive(Serialize)]
struct SemanticHandler {
    meaning: String,
    normalized_name: String,
}

#[derive(Serialize)]
struct SemanticTypeGroup {
    rust_type: String,
    handlers: Vec<SemanticHandler>,
}

#[derive(Serialize)]
struct FunctionParam {
    name: String,
    rust_type: String,
    test_value: String,
}

#[derive(Serialize)]
struct Function {
    name: String,
    params: Vec<FunctionParam>,
    return_type: String,
    return_base_type: String,
    semantic_meaning: Option<String>,
    prompt_template: String,
}

pub struct CodeGenerator {}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(&self, ast: &AstNode) -> Result<String> {
        // ... (context setup and type processing is unchanged) ...
        let mut context = Context::new();

        let mut type_aliases = Vec::new();
        let mut semantic_meanings: HashMap<String, (String, String)> = HashMap::new();
        let mut type_alias_map: HashMap<String, String> = HashMap::new();
        let mut type_meaning_map: HashMap<String, String> = HashMap::new();

        for node in &ast.children {
            if let AstNodeType::TypeDecl = node.node_type {
                self.process_type_decl_node(
                    node,
                    &mut type_aliases,
                    &mut semantic_meanings,
                    &mut type_alias_map,
                    &mut type_meaning_map,
                );
            }
        }
        
        let mut functions = Vec::new();
        for node in &ast.children {
            if let AstNodeType::FunctionDecl = node.node_type {
                functions.push(self.process_function_node(
                    node,
                    &type_alias_map,
                    &type_meaning_map,
                )?);
            }
        }

        // ... (semantic group processing and rendering is unchanged) ...
        let mut grouped_semantics: HashMap<String, Vec<SemanticHandler>> = HashMap::new();
        for (meaning, (rust_type, normalized_name)) in semantic_meanings {
            let group = grouped_semantics.entry(rust_type).or_default();
            group.push(SemanticHandler {
                meaning,
                normalized_name,
            });
        }
        let semantic_type_groups: Vec<SemanticTypeGroup> = grouped_semantics
            .into_iter()
            .map(|(rust_type, handlers)| SemanticTypeGroup {
                rust_type,
                handlers,
            })
            .collect();

        context.insert("type_aliases", &type_aliases);
        context.insert("functions", &functions);
        context.insert("semantic_type_groups", &semantic_type_groups);

        let rendered = TEMPLATES.render("main.rs.tera", &context)?;
        Ok(rendered)
    }

    fn generate_test_value(&self, base_rust_type: &str) -> String {
        match base_rust_type {
            "i32"    => "123".to_string(),
            "f64"    => "45.6".to_string(),
            "bool"   => "true".to_string(),
            "String" => "\"Test Topic\".to_string()".to_string(),
            // Fallback for unknown types.
            _        => panic!("Cannot generate test value for this type"),
        }
    }

    // ... (other helper functions like normalize_meaning_to_function_name, map_to_rust_type, get_type_info_from_node, and process_type_decl_node are unchanged) ...
    fn normalize_meaning_to_function_name(&self, meaning: &str) -> String {
        meaning
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .split('_')
            .filter(|s| !s.is_empty() && !["a", "an", "the", "of", "in"].contains(s))
            .collect::<Vec<_>>()
            .join("_")
    }

    fn map_to_rust_type(&self, vibe_type: &str) -> String {
        match vibe_type {
            "Int" => "i32".to_string(),
            "Float" => "f64".to_string(),
            "String" => "String".to_string(),
            "Bool" => "bool".to_string(),
            _ => vibe_type.to_string(),
        }
    }

    fn get_type_info_from_node(&self, type_node: &AstNode) -> (String, String, Option<String>) {
        match type_node.node_type {
            AstNodeType::BasicType => {
                let alias = type_node.get_string("type").unwrap().to_string();
                let base_type = self.map_to_rust_type(&alias);
                (alias, base_type, None)
            }
            AstNodeType::MeaningType => {
                let meaning = type_node.get_string("meaning").cloned();
                let (base_alias, base_type, _) =
                    self.get_type_info_from_node(&type_node.children[0]);
                (base_alias, base_type, meaning)
            }
            _ => ("()".to_string(), "()".to_string(), None),
        }
    }
    
    fn process_type_decl_node(
        &self,
        node: &AstNode,
        type_aliases: &mut Vec<TypeAlias>,
        semantic_meanings: &mut HashMap<String, (String, String)>,
        type_alias_map: &mut HashMap<String, String>,
        type_meaning_map: &mut HashMap<String, String>,
    ) {
        let name = node.get_string("name").unwrap().clone();
        let type_def_node = &node.children[0];
        let (_, base_type, meaning) = self.get_type_info_from_node(type_def_node);

        if let Some(m) = &meaning {
            let normalized = self.normalize_meaning_to_function_name(m);
            semantic_meanings.insert(m.clone(), (base_type.clone(), normalized));
            type_meaning_map.insert(name.clone(), m.clone());
        }

        type_alias_map.insert(name.clone(), base_type.clone());
        type_aliases.push(TypeAlias {
            name,
            base_type,
            meaning,
        });
    }


    fn process_function_node(
        &self,
        node: &AstNode,
        type_alias_map: &HashMap<String, String>,
        type_meaning_map: &HashMap<String, String>,
    ) -> Result<Function> {
        let name = node.get_string("name").unwrap().clone();
        let mut params = Vec::new();
        let mut return_type = "()".to_string();
        let mut return_base_type = "()".to_string();
        let mut semantic_meaning = None;
        let mut prompt_template = String::new();

        for child in &node.children {
            match child.node_type {
                AstNodeType::ParamList => {
                    for param_node in &child.children {
                        let param_name = param_node.get_string("name").unwrap().clone();
                        let (param_alias, param_base, _) =
                            self.get_type_info_from_node(&param_node.children[0]);
                        
                        let param_rust_type = if type_alias_map.contains_key(&param_alias) {
                            param_alias
                        } else {
                            param_base.clone()
                        };
                        
                        // UPDATED: Generate a test value for the parameter.
                        let test_value = self.generate_test_value(&param_base);

                        params.push(FunctionParam {
                            name: param_name,
                            rust_type: param_rust_type,
                            test_value, // Add the generated value here.
                        });
                    }
                }
                // ... (rest of the function processing is unchanged) ...
                AstNodeType::BasicType | AstNodeType::MeaningType => {
                    let (vibe_type_name, initial_base_type, mut direct_meaning) =
                        self.get_type_info_from_node(child);

                    let final_base_type = type_alias_map
                        .get(&vibe_type_name)
                        .cloned()
                        .unwrap_or(initial_base_type);

                    let signature_type = if type_alias_map.contains_key(&vibe_type_name) {
                        vibe_type_name.clone()
                    } else {
                        final_base_type.clone()
                    };

                    return_type = signature_type;
                    return_base_type = final_base_type;

                    if direct_meaning.is_none() {
                        if let Some(inherited_meaning) = type_meaning_map.get(&return_type) {
                            direct_meaning = Some(inherited_meaning.clone());
                        }
                    }
                    semantic_meaning = direct_meaning;
                }
                AstNodeType::Block => {
                    for stmt in &child.children {
                        if stmt.node_type == AstNodeType::PromptBlock {
                            prompt_template = stmt.get_string("template").unwrap().clone();
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Function {
            name,
            params,
            return_type,
            return_base_type,
            semantic_meaning,
            prompt_template,
        })
    }
}
