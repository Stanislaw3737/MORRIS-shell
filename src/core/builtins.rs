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

}

pub fn push(list_value: &Value, item: &Value) -> Result<Value, String> {
    match list_value {
        Value::List(items) => {
            let mut new_list = items.clone();
            new_list.push(item.clone());
            Ok(Value::List(new_list))
        }
        _ => Err(format!("push() requires list, got {}", list_value.type_name())),
    }
}

pub fn pop(list_value: &Value) -> Result<Value, String> {
    match list_value {
        Value::List(items) => {
            let mut new_list = items.clone();
            if let Some(last_item) = new_list.pop() {
                Ok(last_item)
            } else {
                Err("Cannot pop from empty list".to_string())
            }
        }
        _ => Err(format!("pop() requires list, got {}", list_value.type_name())),
    }
}

pub fn contains(container: &Value, item: &Value) -> Result<Value, String> {
    match container {
        Value::List(items) => {
            let found = items.contains(item);
            Ok(Value::Bool(found))
        }
        Value::Str(s) => {
            let item_str = match item {
                Value::Str(search) => search,
                _ => return Err("String contains requires string search term".to_string())
            };
            Ok(Value::Bool(s.contains(item_str)))
        }
        _ => Err(format!("contains() requires list or string, got {}", container.type_name())),
    }
}

pub fn filter(list_value: &Value, condition_expr: &str, env: &crate::core::env::Env) -> Result<Value, String> {
    match list_value {
        Value::List(items) => {
            let mut filtered = Vec::new();
            
            for item in items {
                // Create a temporary environment with the item
                // This is a simplified approach - you might want a more sophisticated way
                // to evaluate conditions with item context
                match crate::core::expr::parse_expression(condition_expr) {
                    Ok(expr) => {
                        match crate::core::expr::evaluate(&expr, env) {
                            Ok(Value::Bool(true)) => filtered.push(item.clone()),
                            Ok(Value::Bool(false)) => {}, // Skip
                            Ok(other) => return Err(format!("Filter condition must return boolean, got {}", other.type_name())),
                            Err(e) => return Err(format!("Filter condition evaluation failed: {}", e)),
                        }
                    }
                    Err(e) => return Err(format!("Invalid filter condition: {}", e)),
                }
            }
            
            Ok(Value::List(filtered))
        }
        _ => Err(format!("filter() requires list, got {}", list_value.type_name())),
    }
}

pub fn sort(list_value: &Value) -> Result<Value, String> {
    match list_value {
        Value::List(items) => {
            let mut sorted_items = items.clone();
            
            // Simple sorting for homogeneous lists
            // This is basic - you might want more sophisticated sorting
            sorted_items.sort_by(|a, b| {
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => a.cmp(b),
                    (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                    (Value::Str(a), Value::Str(b)) => a.cmp(b),
                    // Mixed types - convert to string for comparison
                    _ => a.to_string().cmp(&b.to_string()),
                }
            });
            
            Ok(Value::List(sorted_items))
        }
        _ => Err(format!("sort() requires list, got {}", list_value.type_name())),
    }
}

pub fn split(string_value: &Value, delimiter: &Value) -> Result<Value, String> {
    match (string_value, delimiter) {
        (Value::Str(s), Value::Str(delim)) => {
            let parts: Vec<Value> = s.split(delim)
                .map(|part| Value::Str(part.to_string()))
                .collect();
            Ok(Value::List(parts))
        }
        (Value::Str(_), _) => Err("split() delimiter must be a string".to_string()),
        _ => Err(format!("split() requires string, got {}", string_value.type_name())),
    }
}

pub fn join(list_value: &Value, separator: &Value) -> Result<Value, String> {
    match (list_value, separator) {
        (Value::List(items), Value::Str(sep)) => {
            let string_parts: Vec<String> = items.iter()
                .map(|item| item.to_string())
                .collect();
            Ok(Value::Str(string_parts.join(sep)))
        }
        (Value::List(_), _) => Err("join() separator must be a string".to_string()),
        _ => Err(format!("join() requires list, got {}", list_value.type_name())),
    }
}

pub fn replace(string_value: &Value, old: &Value, new: &Value) -> Result<Value, String> {
    match (string_value, old, new) {
        (Value::Str(s), Value::Str(old_str), Value::Str(new_str)) => {
            Ok(Value::Str(s.replace(old_str, new_str)))
        }
        (Value::Str(_), _, _) => Err("replace() old and new values must be strings".to_string()),
        _ => Err(format!("replace() requires string, got {}", string_value.type_name())),
    }
}

pub fn substring(string_value: &Value, start: &Value, end: &Value) -> Result<Value, String> {
    match (string_value, start, end) {
        (Value::Str(s), Value::Int(start_idx), Value::Int(end_idx)) => {
            let start_usize = *start_idx as usize;
            let end_usize = *end_idx as usize;
            
            if start_usize > s.len() || end_usize > s.len() {
                return Err("Substring indices out of bounds".to_string());
            }
            
            if start_usize > end_usize {
                return Err("Start index must be less than or equal to end index".to_string());
            }
            
            Ok(Value::Str(s[start_usize..end_usize].to_string()))
        }
        (Value::Str(_), _, _) => Err("substring() indices must be integers".to_string()),
        _ => Err(format!("substring() requires string, got {}", string_value.type_name())),
    }
}

pub fn starts_with(string_value: &Value, prefix: &Value) -> Result<Value, String> {
    match (string_value, prefix) {
        (Value::Str(s), Value::Str(prefix_str)) => {
            Ok(Value::Bool(s.starts_with(prefix_str)))
        }
        (Value::Str(_), _) => Err("starts_with() prefix must be a string".to_string()),
        _ => Err(format!("starts_with() requires string, got {}", string_value.type_name())),
    }
}

pub fn ends_with(string_value: &Value, suffix: &Value) -> Result<Value, String> {
    match (string_value, suffix) {
        (Value::Str(s), Value::Str(suffix_str)) => {
            Ok(Value::Bool(s.ends_with(suffix_str)))
        }
        (Value::Str(_), _) => Err("ends_with() suffix must be a string".to_string()),
        _ => Err(format!("ends_with() requires string, got {}", string_value.type_name())),
    }
}

pub fn char_at(string_value: &Value, index_value: &Value) -> Result<Value, String> {
    match (string_value, index_value) {
        (Value::Str(s), Value::Int(index)) => {
            let idx = *index as usize;
            if idx < s.len() {
                let ch = s.chars().nth(idx).unwrap();
                Ok(Value::Str(ch.to_string()))
            } else {
                Err(format!("Index {} out of bounds for string of length {}", idx, s.len()))
            }
        }
        (Value::Str(_), _) => Err("char_at() index must be an integer".to_string()),
        _ => Err(format!("char_at() requires string, got {}", string_value.type_name())),
    }
}

pub fn substring_index(string_value: &Value, start_index: &Value, length: &Value) -> Result<Value, String> {
    match (string_value, start_index, length) {
        (Value::Str(s), Value::Int(start_idx), Value::Int(len)) => {
            let start_usize = *start_idx as usize;
            let length_usize = *len as usize;
            
            if start_usize > s.len() {
                return Err("Start index out of bounds".to_string());
            }
            
            let end_index = std::cmp::min(start_usize + length_usize, s.len());
            Ok(Value::Str(s[start_usize..end_index].to_string()))
        }
        (Value::Str(_), _, _) => Err("substring_index() indices and length must be integers".to_string()),
        _ => Err(format!("substring_index() requires string, got {}", string_value.type_name())),
    }
}

pub fn find_index(string_value: &Value, search_value: &Value) -> Result<Value, String> {
    match (string_value, search_value) {
        (Value::Str(s), Value::Str(search)) => {
            if let Some(pos) = s.find(search) {
                Ok(Value::Int(pos as i64))
            } else {
                Ok(Value::Int(-1)) // Not found
            }
        }
        (Value::Str(_), _) => Err("find_index() search term must be a string".to_string()),
        _ => Err(format!("find_index() requires string, got {}", string_value.type_name())),
    }
}

pub fn replace_at(string_value: &Value, start_index: &Value, length: &Value, replacement: &Value) -> Result<Value, String> {
    match (string_value, start_index, length, replacement) {
        (Value::Str(s), Value::Int(start_idx), Value::Int(len), Value::Str(repl)) => {
            let start_usize = *start_idx as usize;
            let length_usize = *len as usize;
            
            if start_usize > s.len() {
                return Err("Start index out of bounds".to_string());
            }
            
            let end_index = std::cmp::min(start_usize + length_usize, s.len());
            let mut result = s[..start_usize].to_string();
            result.push_str(repl);
            result.push_str(&s[end_index..]);
            
            Ok(Value::Str(result))
        }
        (Value::Str(_), _, _, _) => Err("replace_at() requires integer indices and string replacement".to_string()),
        _ => Err(format!("replace_at() requires string, got {}", string_value.type_name())),
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
        }
        _ => Err(format!("get_index() requires list, got {}", list_value.type_name())),
    }
}

pub fn put_index(list_value: &Value, index_value: &Value, item_value: &Value) -> Result<Value, String> {
    let index = match index_value {
        Value::Int(i) => *i as usize,
        _ => return Err("Index must be integer".to_string()),
    };

    match list_value {
        Value::List(items) => {
            if index < items.len() {
                let mut new_items = items.clone(); // Clone instead of move
                new_items[index] = item_value.clone();
                Ok(Value::List(new_items))
            } else {
                Err(format!("Index {} out of bounds for list of length {}", index, items.len()))
            }
        }
        _ => Err(format!("put_index() requires list, got {}", list_value.type_name())),
    }
}

pub fn insert(list_value: &Value, index_value: &Value, item_value: &Value) -> Result<Value, String> {
    let index = match index_value {
        Value::Int(i) => *i as usize,
        _ => return Err("Index must be integer".to_string()),
    };

    match list_value {
        Value::List(items) => {
            if index <= items.len() {
                let mut new_items = items.clone(); // Clone instead of move
                new_items.insert(index, item_value.clone());
                Ok(Value::List(new_items))
            } else {
                Err(format!("Index {} out of bounds for list of length {}", index, items.len()))
            }
        }
        _ => Err(format!("insert() requires list, got {}", list_value.type_name())),
    }
}

pub fn remove_index(list_value: &Value, index_value: &Value) -> Result<Value, String> {
    let index = match index_value {
        Value::Int(i) => *i as usize,
        _ => return Err("Index must be integer".to_string()),
    };

    match list_value {
        Value::List(items) => {
            if index < items.len() {
                let mut new_items = items.clone(); // Clone instead of move
                let removed_item = new_items.remove(index);
                Ok(Value::List(new_items)) // Return the modified list
            } else {
                Err(format!("Index {} out of bounds for list of length {}", index, items.len()))
            }
        }
        _ => Err(format!("remove_index() requires list, got {}", list_value.type_name())),
    }
}
// Enhanced sort with reverse option
pub fn sort_with_direction(list_value: &Value, reverse_value: &Value) -> Result<Value, String> {
    let reverse = match reverse_value {
        Value::Bool(b) => *b,
        _ => return Err("sort() reverse parameter must be boolean".to_string()),
    };

    match list_value {
        Value::List(items) => {
            let mut sorted_items = items.clone();
            
            if reverse {
                // Descending sort
                sorted_items.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Int(a), Value::Int(b)) => b.cmp(a),
                        (Value::Float(a), Value::Float(b)) => b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal),
                        (Value::Str(a), Value::Str(b)) => b.cmp(a),
                        _ => b.to_string().cmp(&a.to_string()),
                    }
                });
            } else {
                // Ascending sort
                sorted_items.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Int(a), Value::Int(b)) => a.cmp(b),
                        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                        (Value::Str(a), Value::Str(b)) => a.cmp(b),
                        _ => a.to_string().cmp(&b.to_string()),
                    }
                });
            }
            
            Ok(Value::List(sorted_items))
        }
        _ => Err(format!("sort() requires list, got {}", list_value.type_name())),
    }
}

pub fn keys(dict_value: &Value) -> Result<Value, String> {
    match dict_value {
        Value::Dict(map) => {
            let keys: Vec<Value> = map.keys()
                .map(|k| Value::Str(k.clone()))
                .collect();
            Ok(Value::List(keys))
        }
        Value::Json(json_str) => {
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(serde_json::Value::Object(obj)) => {
                    let keys: Vec<Value> = obj.keys()
                        .map(|k| Value::Str(k.clone()))
                        .collect();
                    Ok(Value::List(keys))
                }
                _ => Err("JSON is not an object".to_string()),
            }
        }
        _ => Err(format!("keys() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn values(dict_value: &Value) -> Result<Value, String> {
    match dict_value {
        Value::Dict(map) => {
            let values: Vec<Value> = map.values().cloned().collect();
            Ok(Value::List(values))
        }
        Value::Json(json_str) => {
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(serde_json::Value::Object(obj)) => {
                    let values: Vec<Value> = obj.values()
                        .map(|v| convert_json_value(v))
                        .collect();
                    Ok(Value::List(values))
                }
                _ => Err("JSON is not an object".to_string()),
            }
        }
        _ => Err(format!("values() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn get(dict_value: &Value, key_value: &Value) -> Result<Value, String> {
    let key = match key_value {
        Value::Str(s) => s,
        _ => return Err("Dictionary key must be a string".to_string()),
    };
    
    match dict_value {
        Value::Dict(map) => {
            if let Some(value) = map.get(key) {
                Ok(value.clone())
            } else {
                Err(format!("Key '{}' not found in dictionary", key))
            }
        }
        Value::Json(json_str) => {
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(serde_json::Value::Object(obj)) => {
                    if let Some(value) = obj.get(key) {
                        Ok(convert_json_value(value))
                    } else {
                        Err(format!("Key '{}' not found in JSON object", key))
                    }
                }
                _ => Err("JSON is not an object".to_string()),
            }
        }
        _ => Err(format!("get() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn put(dict_value: &Value, key_value: &Value, val_value: &Value) -> Result<Value, String> {
    let key = match key_value {
        Value::Str(s) => s,
        _ => return Err("Dictionary key must be a string".to_string()),
    };
    
    match dict_value {
        Value::Dict(map) => {
            let mut new_map = map.clone();
            new_map.insert(key.clone(), val_value.clone());
            Ok(Value::Dict(new_map))
        }
        _ => Err(format!("put() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn has_key(dict_value: &Value, key_value: &Value) -> Result<Value, String> {
    let key = match key_value {
        Value::Str(s) => s,
        _ => return Err("Dictionary key must be a string".to_string()),
    };
    
    match dict_value {
        Value::Dict(map) => {
            Ok(Value::Bool(map.contains_key(key)))
        }
        Value::Json(json_str) => {
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(serde_json::Value::Object(obj)) => {
                    Ok(Value::Bool(obj.contains_key(key)))
                }
                _ => Err("JSON is not an object".to_string()),
            }
        }
        _ => Err(format!("has_key() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn remove(dict_value: &Value, key_value: &Value) -> Result<Value, String> {
    let key = match key_value {
        Value::Str(s) => s,
        _ => return Err("Dictionary key must be a string".to_string()),
    };
    
    match dict_value {
        Value::Dict(map) => {
            let mut new_map = map.clone();
            if new_map.remove(key).is_some() {
                Ok(Value::Dict(new_map))
            } else {
                Err(format!("Key '{}' not found in dictionary", key))
            }
        }
        _ => Err(format!("remove() requires dictionary, got {}", dict_value.type_name())),
    }
}

pub fn merge(dict1_value: &Value, dict2_value: &Value) -> Result<Value, String> {
    match (dict1_value, dict2_value) {
        (Value::Dict(map1), Value::Dict(map2)) => {
            let mut merged = map1.clone();
            for (key, value) in map2 {
                merged.insert(key.clone(), value.clone());
            }
            Ok(Value::Dict(merged))
        }
        _ => Err(format!("merge() requires two dictionaries, got {}/{}", 
                        dict1_value.type_name(), dict2_value.type_name())),
    }
}