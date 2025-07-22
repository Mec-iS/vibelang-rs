use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstNodeType {
    // Core program structure
    Program,
    FunctionDecl,
    FunctionBody,
    TypeDecl,
    VarDecl,
    ClassDecl,
    ClassBody,
    MemberVar,
    Import,
    
    // Type system
    BasicType,
    MeaningType,
    
    // Parameters and arguments
    ParamList,
    Parameter,
    
    // Statements
    Block,
    ExprStmt,
    ReturnStmt,
    PromptBlock,
    
    // Expressions
    CallExpr,
    Identifier,
    
    // Literals
    StringLiteral,
    IntLiteral,
    FloatLiteral,
    BoolLiteral,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub node_type: AstNodeType,
    pub children: Vec<Box<AstNode>>,
    pub properties: HashMap<String, PropertyValue>,
    pub line: usize,
    pub column: usize,
    pub parent: Option<*mut AstNode>,
}

impl AstNode {
    pub fn new(node_type: AstNodeType) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            properties: HashMap::new(),
            line: 0,
            column: 0,
            parent: None,
        }
    }

    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(Box::new(child));
    }

    // Property setters matching C API
    pub fn set_string(&mut self, name: &str, value: &str) {
        self.properties.insert(name.to_string(), PropertyValue::String(value.to_string()));
    }

    pub fn set_int(&mut self, name: &str, value: i64) {
        self.properties.insert(name.to_string(), PropertyValue::Int(value));
    }

    pub fn set_float(&mut self, name: &str, value: f64) {
        self.properties.insert(name.to_string(), PropertyValue::Float(value));
    }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.properties.insert(name.to_string(), PropertyValue::Bool(value));
    }

    // Property getters matching C API
    pub fn get_string(&self, name: &str) -> Option<&String> {
        match self.properties.get(name) {
            Some(PropertyValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.properties.get(name) {
            Some(PropertyValue::Int(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.properties.get(name) {
            Some(PropertyValue::Float(f)) => Some(*f),
            _ => None,
        }
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.properties.get(name) {
            Some(PropertyValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }
}

pub fn extract_string_value(node: &AstNode) -> Option<&String> {
    match node.node_type {
        AstNodeType::StringLiteral => node.get_string("value"),
        AstNodeType::Identifier => node.get_string("name"),
        _ => None,
    }
}
