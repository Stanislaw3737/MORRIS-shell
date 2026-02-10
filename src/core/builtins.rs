use crate::core::types::Value;
use chrono::Utc;
use std::collections::HashMap;
//use serde_json;

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
        Value::Json(json_str) => Ok(Value::Int(json_str.len() as i64)),
        
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


pub fn parse_json(json_str: &str) -> Result<Value, String> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(serde_value) => Ok(convert_json_value(&serde_value)),
        Err(e) => Err(format!("Invalid JSON: {}", e)),
    }
}


pub fn to_json(value: &Value) -> Result<String, String> {
    let serde_value = convert_to_json_value(value);
    serde_json::to_string(&serde_value)
        .map_err(|e| format!("Cannot serialize to JSON: {}", e))
}


fn convert_json_value(json_val: &serde_json::Value) -> Value {
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
}

fn convert_to_json_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::String(f.to_string()))
        },
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::List(items) => {
            serde_json::Value::Array(
                items.iter().map(convert_to_json_value).collect()
            )
        },
        Value::Dict(map) => {
            serde_json::Value::Object(
                map.iter().map(|(k, v)| (k.clone(), convert_to_json_value(v))).collect()
            )
        },
        Value::Json(json_str) => {
            serde_json::from_str(json_str)
                .unwrap_or(serde_json::Value::String(json_str.clone()))
        },
    }
}

#[derive(Debug)]
pub struct JsonPath {
    segments: Vec<JsonPathSegment>,
}

#[derive(Debug)]
pub enum JsonPathSegment {
    Root,
    Key(String),
    Index(usize),
    Wildcard,
}

impl JsonPath {
    pub fn parse(path: &str) -> Result<Self, String> {
        let mut segments = vec![JsonPathSegment::Root];
        
        if path.is_empty() || path == "$" {
            return Ok(JsonPath { segments });
        }
        
        // Simple path parsing: $.key[0].nested or key.nested[0]
        let path = if path.starts_with("$") { &path[1..] } else { path };
        
        for segment in path.split('.') {
            if segment.is_empty() {
                continue;
            }
            
            if segment.contains('[') && segment.contains(']') {
                // Handle array indexing like key[0]
                let bracket_start = segment.find('[').unwrap();
                let key_part = &segment[..bracket_start];
                let index_part = &segment[bracket_start+1..segment.len()-1];
                
                if !key_part.is_empty() {
                    segments.push(JsonPathSegment::Key(key_part.to_string()));
                }
                
                if index_part == "*" {
                    segments.push(JsonPathSegment::Wildcard);
                } else {
                    let index = index_part.parse::<usize>()
                        .map_err(|_| format!("Invalid array index: {}", index_part))?;
                    segments.push(JsonPathSegment::Index(index));
                }
            } else if segment == "*" {
                segments.push(JsonPathSegment::Wildcard);
            } else {
                segments.push(JsonPathSegment::Key(segment.to_string()));
            }
        }
        
        Ok(JsonPath { segments })
    }
    
    pub fn get(&self, value: &Value) -> Result<Value, String> {
        let mut current = value.clone();  // Clone once at the start
        
        for segment in &self.segments {
            match segment {
                JsonPathSegment::Root => continue,
                JsonPathSegment::Key(key) => {
                    match &current {
                        Value::Dict(map) => {
                            if let Some(val) = map.get(key) {
                                current = val.clone();
                            } else {
                                return Err(format!("Key '{}' not found", key));
                            }
                        }
                        Value::Json(json_str) => {
                            match parse_json(json_str) {
                                Ok(parsed) => {
                                    if let Value::Dict(ref map) = parsed {
                                        if let Some(val) = map.get(key) {
                                            current = val.clone();
                                        } else {
                                            return Err(format!("Key '{}' not found in JSON", key));
                                        }
                                    } else {
                                        return Err("JSON is not an object".to_string());
                                    }
                                }
                                Err(e) => return Err(format!("Cannot parse JSON: {}", e)),
                            }
                        }
                        _ => return Err(format!("Cannot access key '{}' on non-object type", key)),
                    }
                }
                JsonPathSegment::Index(index) => {
                    match &current {
                        Value::List(items) => {
                            if *index < items.len() {
                                current = items[*index].clone();
                            } else {
                                return Err(format!("Index {} out of bounds for array of length {}", index, items.len()));
                            }
                        }
                        _ => return Err(format!("Cannot access index {} on non-array type", index)),
                    }
                }
                JsonPathSegment::Wildcard => {
                    break;
                }
            }
        }
        
        Ok(current)
    }

    
    /*pub fn set(&self, value: &Value, new_value: Value) -> Result<Value, String> {
        // This is a simplified setter - in reality you'd want to modify in place
        // For now, we'll just demonstrate the concept
        Err("JSON path setting not yet implemented".to_string())
    }*/
}