// src/runtime/types.rs

#[derive(Debug, Clone)]
pub enum VibeValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl VibeValue {
    /// Converts VibeValue to an i32.
    /// Panics if the conversion is not possible or logical.
    pub fn into_i32(self) -> i32 {
        match self {
            VibeValue::Number(n) => n as i32,
            VibeValue::String(s) => s.parse::<i32>().unwrap_or_else(|e| {
                panic!("Failed to convert LLM response '{}' to i32: {}", s, e)
            }),
            _ => panic!("Cannot convert {:?} to i32", self),
        }
    }

    /// Converts VibeValue to an f64.
    /// Panics if the conversion is not possible or logical.
    pub fn into_f64(self) -> f64 {
        match self {
            VibeValue::Number(n) => n,
            VibeValue::String(s) => s.parse::<f64>().unwrap_or_else(|e| {
                panic!("Failed to convert LLM response '{}' to f64: {}", s, e)
            }),
            _ => panic!("Cannot convert {:?} to f64", self),
        }
    }

    /// Converts VibeValue to a bool.
    /// Panics if the conversion is not possible or logical.
    pub fn into_bool(self) -> bool {
        match self {
            VibeValue::Boolean(b) => b,
            VibeValue::String(s) => {
                let val = s.to_lowercase();
                if val == "true" { true }
                else if val == "false" { false }
                else { panic!("Cannot convert LLM response '{}' to bool", s) }
            },
            _ => panic!("Cannot convert {:?} to bool", self),
        }
    }

    /// Converts VibeValue to a String.
    /// This conversion is always possible.
    pub fn into_string(self) -> String {
        match self {
            VibeValue::String(s) => s,
            VibeValue::Number(n) => n.to_string(),
            VibeValue::Boolean(b) => b.to_string(),
            VibeValue::Null => String::new(),
        }
    }
}
