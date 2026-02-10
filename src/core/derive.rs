use crate::core::types::Value;

pub fn derive(value: &Value) -> Value {
    match value {
        Value::Str(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Value::Int(n)
            } else if let Ok(f) = s.parse::<f64>() {
                Value::Float(f)
            } else if s.to_lowercase() == "true" {
                Value::Bool(true)
            } else if s.to_lowercase() == "false" {
                Value::Bool(false)
            } else {
                value.clone()
            }
        }
        Value::Int(_) | Value::Bool(_) | Value::Float(_) => value.clone(),
        Value::List(_) | Value::Dict(_) | Value::Json(_) => value.clone(),
    }
}