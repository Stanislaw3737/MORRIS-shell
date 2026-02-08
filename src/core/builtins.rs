use crate::core::types::Value;
use chrono::Utc;
//use std::collections::HashMap;

pub fn count(value: &Value, pattern: &str) -> Result<Value, String> {
    match value {
        Value::Str(s) => {
            let count = s.matches(pattern).count();
            Ok(Value::Int(count as i64))
        }
        _ => Err(format!("Cannot count in type: {}", value.type_name())),
    }
}

pub fn now() -> Value {
    let dt = Utc::now();
    let timestamp = dt.format("%Y%m%d_%H%M%S").to_string();
    Value::Str(timestamp)
}

pub fn len(value: &Value) -> Result<Value, String> {
    match value {
        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
        Value::Int(i) => Ok(Value::Int(i.to_string().len() as i64)),
        Value::Float(f) => Ok(Value::Int(f.to_string().len() as i64)),
        Value::Bool(b) => Ok(Value::Int(if *b { 4 } else { 5 })), // "true" vs "false"
        Value::List(items) => Ok(Value::Int(items.len() as i64)),
        Value::Dict(map) => Ok(Value::Int(map.len() as i64)),
        
    }
}

pub fn upper(value: &Value) -> Result<Value, String> {
    match value {
        Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
        _ => Err(format!("Cannot convert to uppercase: {}", value.type_name())),
    }
}

pub fn lower(value: &Value) -> Result<Value, String> {
    match value {
        Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
        _ => Err(format!("Cannot convert to lowercase: {}", value.type_name())),
    }
}

pub fn trim(value: &Value) -> Result<Value, String> {
    match value {
        Value::Str(s) => Ok(Value::Str(s.trim().to_string())),
        _ => Err(format!("Cannot trim: {}", value.type_name())),
    }
}

pub fn keys(dict_value: &Value) -> Result<Value, String> {
    match dict_value {
        Value::Dict(map) => {
            let keys: Vec<Value> = map.keys()
                .map(|k| Value::Str(k.clone()))
                .collect();
            Ok(Value::List(keys))
        },
        /*Value::Json(json_str) => {
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(serde_json::Value::Object(obj)) => {
                    let keys: Vec<Value> = obj.keys()
                        .map(|k| Value::Str(k.clone()))
                        .collect();
                    Ok(Value::List(keys))
                },
                _ => Err("JSON is not an object".to_string()),
            }
        },*/
        _ => Err(format!("keys() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn get_index(list_value: &Value, index_value: &Value) -> Result<Value, String> {
    let index = match index_value {
        Value::Int(i) => *i as usize,
        _ => return Err("Index must be integer".to_string()),
    };

    match list_value {
        Value::List(items) => {
            if index < items.len() {
                Ok(items[index].clone())
            } else {
                Err(format!("Index {} out of bounds for list of length {}", index, items.len()))
            }
        },
        /*Value::Array(items) => {
            if index < items.len() {
                Ok(items[index].clone())
            } else {
                Err(format!("Index {} out of bounds for array of length {}", index, items.len()))
            }
        },
        Value::Json(json_str) => {
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(json_value) => {
                    match json_value {
                        serde_json::Value::Array(arr) => {
                            if index < arr.len() {
                                // Convert serde_json::Value to our Value type
                                let item = convert_json_value(&arr[index]);
                                Ok(item)
                            } else {
                                Err(format!("Index {} out of bounds for JSON array of length {}", index, arr.len()))
                            }
                        },
                        _ => Err("JSON is not an array".to_string()),
                    }
                },
                Err(_) => Err("Invalid JSON".to_string()),
            }
        },*/
        _ => Err(format!("get_index() requires list/array, got {}", list_value.type_name())),
    }
}

/*fn convert_json_value(json_val: &serde_json::Value) -> Value {
    match json_val {
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Str(n.to_string())
            }
        },
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(convert_json_value).collect();
            Value::List(items)
        },
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Value> = obj.iter()
                .map(|(k, v)| (k.clone(), convert_json_value(v)))
                .collect();
            Value::Dict(map)
        },
        serde_json::Value::Null => Value::Str("null".to_string()),
    }
}*/