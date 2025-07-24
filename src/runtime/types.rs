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
            VibeValue::String(s) => s
                .parse::<i32>()
                .unwrap_or_else(|e| panic!("Failed to convert LLM response '{}' to i32: {}", s, e)),
            _ => panic!("Cannot convert {:?} to i32", self),
        }
    }

    /// Converts VibeValue to an f64.
    /// Panics if the conversion is not possible or logical.
    pub fn into_f64(self) -> f64 {
        match self {
            VibeValue::Number(n) => n,
            VibeValue::String(s) => s
                .parse::<f64>()
                .unwrap_or_else(|e| panic!("Failed to convert LLM response '{}' to f64: {}", s, e)),
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
                if val == "true" {
                    true
                } else if val == "false" {
                    false
                } else {
                    panic!("Cannot convert LLM response '{}' to bool", s)
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Tests for into_i32 ---
    #[test]
    fn test_vibe_value_into_i32_from_number() {
        assert_eq!(VibeValue::Number(42.9).into_i32(), 42);
        assert_eq!(VibeValue::Number(-10.0).into_i32(), -10);
    }

    #[test]
    fn test_vibe_value_into_i32_from_string() {
        assert_eq!(VibeValue::String("123".to_string()).into_i32(), 123);
    }

    #[test]
    #[should_panic(expected = "Failed to convert LLM response 'abc' to i32")]
    fn test_vibe_value_into_i32_panics_on_invalid_string() {
        VibeValue::String("abc".to_string()).into_i32();
    }

    #[test]
    #[should_panic(expected = "Cannot convert Boolean(true) to i32")]
    fn test_vibe_value_into_i32_panics_on_invalid_type() {
        VibeValue::Boolean(true).into_i32();
    }

    // --- Tests for into_f64 ---
    #[test]
    fn test_vibe_value_into_f64() {
        assert_eq!(VibeValue::Number(3.14).into_f64(), 3.14);
        assert_eq!(VibeValue::String("99.9".to_string()).into_f64(), 99.9);
    }

    // --- Tests for into_bool ---
    #[test]
    fn test_vibe_value_into_bool() {
        assert_eq!(VibeValue::Boolean(true).into_bool(), true);
        assert_eq!(VibeValue::String("true".to_string()).into_bool(), true);
        assert_eq!(VibeValue::String("FALSE".to_string()).into_bool(), false);
    }

    #[test]
    #[should_panic(expected = "Cannot convert LLM response 'maybe' to bool")]
    fn test_vibe_value_into_bool_panics_on_invalid_string() {
        VibeValue::String("maybe".to_string()).into_bool();
    }

    // --- Tests for into_string ---
    #[test]
    fn test_vibe_value_into_string_conversion() {
        assert_eq!(
            VibeValue::String("hello".to_string()).into_string(),
            "hello"
        );
        assert_eq!(VibeValue::Number(-12.5).into_string(), "-12.5");
        assert_eq!(VibeValue::Boolean(true).into_string(), "true");
        assert_eq!(VibeValue::Null.into_string(), "");
    }
}
