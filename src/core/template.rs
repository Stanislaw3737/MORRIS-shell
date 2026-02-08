// src/core/template.rs
use crate::core::env::Env;
use crate::core::expr;
use std::collections::HashMap;
#[allow(dead_code)]
pub fn render(template: &str, params: &HashMap<String, String>) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Handle escape sequence
            if let Some(&next) = chars.peek() {
                if next == '{' {
                    // Double {{ means literal {
                    result.push('{');
                    chars.next(); // Consume second {
                    continue;
                }
            }
            
            // Parse variable/expression inside {}
            let mut content = String::new();
            let mut brace_depth = 1;
            
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        chars.next(); // Consume closing }
                        break;
                    }
                } else if next == '{' {
                    brace_depth += 1;
                }
                
                content.push(chars.next().unwrap());
            }
            
            if brace_depth > 0 {
                return Err("Unclosed template variable".to_string());
            }
            
            // Trim and check if empty
            let content = content.trim();
            if content.is_empty() {
                return Err("Empty template variable".to_string());
            }
            
            // Try to get from parameters first, then try as expression
            if let Some(value) = params.get(content) {
                result.push_str(value);
            } else {
                // Could be a complex expression or nested template
                // For now, treat as literal with error marker
                result.push_str(&format!("{{{}?}}", content));
            }
        } else if ch == '\\' {
            // Handle escape sequences
            if let Some(&next) = chars.peek() {
                match next {
                    '{' | '}' | '\\' => {
                        result.push(next);
                        chars.next(); // Consume escaped character
                    }
                    _ => {
                        // Not a special escape, keep backslash
                        result.push('\\');
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}
#[allow(dead_code)]
pub fn render_with_env(
    template: &str, 
    params: &HashMap<String, String>,
    env: &Env
) -> Result<String, String> {
    render_with_params(template, params, env)
}

#[allow(dead_code)]
pub fn render_with_env_original(
    template: &str, 
    params: &HashMap<String, String>,
    env: &Env
) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Handle escape sequence
            if let Some(&next) = chars.peek() {
                if next == '{' {
                    result.push('{');
                    chars.next(); // Consume second {
                    continue;
                }
            }
            
            // Parse variable/expression inside {}
            let mut content = String::new();
            let mut brace_depth = 1;
            
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        chars.next(); // Consume closing }
                        break;
                    }
                } else if next == '{' {
                    brace_depth += 1;
                }
                
                content.push(chars.next().unwrap());
            }
            
            if brace_depth > 0 {
                return Err("Unclosed template variable".to_string());
            }
            
            let content = content.trim();
            if content.is_empty() {
                return Err("Empty template variable".to_string());
            }
            
            // Try parameters first
            if let Some(value) = params.get(content) {
                result.push_str(value);
            } else {
                // Try as expression with environment
                match expr::parse_expression(content) {
                    Ok(expr) => {
                        match expr::evaluate(&expr, env) {
                            Ok(value) => result.push_str(&value.to_string()),
                            Err(e) => {
                                // If evaluation fails, check if it's a simple variable
                                if env.get_value(content).is_some() {
                                    // It's a variable name, try to get it directly
                                    if let Some(value) = env.get_value(content) {
                                        result.push_str(&value.to_string());
                                    } else {
                                        result.push_str(&format!("{{{}?}}", content));
                                    }
                                } else {
                                    return Err(format!("Cannot evaluate '{}': {}", content, e));
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Not a valid expression, try as literal variable name
                        if let Some(value) = env.get_value(content) {
                            result.push_str(&value.to_string());
                        } else {
                            result.push_str(&format!("{{{}?}}", content));
                        }
                    }
                }
            }
        } else if ch == '\\' {
            // Handle escape sequences
            if let Some(&next) = chars.peek() {
                match next {
                    '{' | '}' | '\\' => {
                        result.push(next);
                        chars.next(); // Consume escaped character
                    }
                    _ => {
                        result.push('\\');
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}
#[allow(dead_code)]
pub fn extract_variables(template: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Skip escaped {{
            if let Some(&next) = chars.peek() {
                if next == '{' {
                    chars.next(); // Consume second {
                    continue;
                }
            }
            
            // Parse variable name
            let mut var_name = String::new();
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next(); // Consume }
                    break;
                }
                var_name.push(chars.next().unwrap());
            }
            
            let var_name = var_name.trim().to_string();
            if !var_name.is_empty() {
                variables.push(var_name);
            }
        } else if ch == '\\' {
            // Skip next character if it's a special escape
            if let Some(&next) = chars.peek() {
                if next == '{' || next == '}' || next == '\\' {
                    chars.next();
                }
            }
        }
    }
    
    variables
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_simple() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "Alice".to_string());
        params.insert("age".to_string(), "30".to_string());
        
        assert_eq!(
            render("Hello {name}! You are {age} years old.", &params).unwrap(),
            "Hello Alice! You are 30 years old."
        );
    }
    
    #[test]
    fn test_render_escaped() {
        let params = HashMap::new();
        
        assert_eq!(
            render("Literal {{brace}} and \\{escaped}", &params).unwrap(),
            "Literal {brace} and {escaped}"
        );
    }
    
    #[test]
    fn test_extract_variables() {
        let result = extract_variables("Hello {name} from {city}!");
        assert_eq!(result, vec!["name".to_string(), "city".to_string()]);
    }
}

#[allow(dead_code)]
pub fn render_with_params(
    template: &str,
    params: &HashMap<String, String>,
    env: &Env
) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '$' => {
                // Handle $parameter or ${parameter}
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        // ${parameter} syntax
                        chars.next(); // Consume '{'
                        let mut param_name = String::new();
                        
                        while let Some(&next) = chars.peek() {
                            if next == '}' {
                                chars.next(); // Consume '}'
                                break;
                            }
                            param_name.push(chars.next().unwrap());
                        }
                        
                        let param_name = param_name.trim();
                        if param_name.is_empty() {
                            return Err("Empty parameter name in ${}".to_string());
                        }
                        
                        // Try parameters first, then environment
                        if let Some(value) = params.get(param_name) {
                            result.push_str(value);
                        } else if let Some(value) = env.get_value(param_name) {
                            result.push_str(&value.to_string());
                        } else {
                            return Err(format!("Parameter '{}' not provided", param_name));
                        }
                    } else if next.is_alphabetic() || next == '_' {
                        // $parameter syntax (no braces)
                        let mut param_name = String::new();
                        
                        while let Some(&next) = chars.peek() {
                            if next.is_alphanumeric() || next == '_' {
                                param_name.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        
                        if param_name.is_empty() {
                            result.push('$'); // Lone $
                        } else {
                            // Try parameters first, then environment
                            if let Some(value) = params.get(&param_name) {
                                result.push_str(value);
                            } else if let Some(value) = env.get_value(&param_name) {
                                result.push_str(&value.to_string());
                            } else {
                                return Err(format!("Parameter '{}' not provided", param_name));
                            }
                        }
                    } else {
                        // $ followed by non-alphabetic character
                        result.push('$');
                    }
                } else {
                    // $ at end of string
                    result.push('$');
                }
            }
            '{' => {
                // Regular {expression} interpolation (environment only)
                // Handle escape sequence
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        result.push('{');
                        chars.next(); // Consume second {
                        continue;
                    }
                }
                
                let mut expr = String::new();
                let mut brace_depth = 1;
                
                while let Some(&next) = chars.peek() {
                    if next == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            chars.next(); // Consume }
                            break;
                        }
                    } else if next == '{' {
                        brace_depth += 1;
                    }
                    expr.push(chars.next().unwrap());
                }
                
                if brace_depth > 0 {
                    return Err("Unclosed { in template".to_string());
                }
                
                let expr = expr.trim();
                if expr.is_empty() {
                    return Err("Empty expression in {}".to_string());
                }
                
                // Evaluate as expression with environment
                match expr::parse_expression(expr) {
                    Ok(parsed_expr) => {
                        match expr::evaluate(&parsed_expr, env) {
                            Ok(value) => result.push_str(&value.to_string()),
                            Err(e) => return Err(format!("Cannot evaluate '{}': {}", expr, e)),
                        }
                    }
                    Err(e) => return Err(format!("Invalid expression '{}': {}", expr, e)),
                }
            }
            '\\' => {
                // Handle escape sequences
                if let Some(&next) = chars.peek() {
                    match next {
                        '$' | '{' | '}' | '\\' => {
                            result.push(next);
                            chars.next(); // Consume escaped character
                        }
                        _ => {
                            result.push('\\');
                        }
                    }
                } else {
                    result.push('\\');
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }
    
    Ok(result)
}

#[allow(dead_code)]
pub fn extract_dollar_params(template: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '$' {
            if let Some(&next) = chars.peek() {
                if next == '{' {
                    // ${parameter} syntax
                    chars.next(); // Consume '{'
                    let mut param_name = String::new();
                    
                    while let Some(&next) = chars.peek() {
                        if next == '}' {
                            chars.next(); // Consume '}'
                            break;
                        }
                        param_name.push(chars.next().unwrap());
                    }
                    
                    let param_name = param_name.trim().to_string();
                    if !param_name.is_empty() {
                        params.push(param_name);
                    }
                } else if next.is_alphabetic() || next == '_' {
                    // $parameter syntax
                    let mut param_name = String::new();
                    
                    while let Some(&next) = chars.peek() {
                        if next.is_alphanumeric() || next == '_' {
                            param_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    
                    if !param_name.is_empty() {
                        params.push(param_name);
                    }
                }
            }
        } else if ch == '\\' {
            // Skip escaped character
            if chars.peek().is_some() {
                chars.next();
            }
        }
    }
    
    // Remove duplicates while preserving order
    let mut unique_params = Vec::new();
    for param in params {
        if !unique_params.contains(&param) {
            unique_params.push(param);
        }
    }
    
    unique_params
}