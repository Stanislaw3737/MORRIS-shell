use std::collections::HashSet;
use crate::core::types::Value;
use crate::core::env::Env;
use crate::core::builtins;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Variable(String),

    List(Vec<Expr>),
    Dict(HashMap<String, Expr>),
    IndexAccess(Box<Expr>, Box<Expr>), // list[index] or dict["key"]
    MethodCall(Box<Expr>, String, Vec<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
    Conditional(Vec<ConditionalBranch>),
    // NEW: Comparison operators
    GreaterThan(Box<Expr>, Box<Expr>),
    GreaterThanOrEqual(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    LessThanOrEqual(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    // NEW: Logical operators
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct ConditionalBranch {
    pub value: Box<Expr>,
    pub condition: Option<Box<Expr>>,
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(Value::Str(s)) => write!(f, "\"{}\"", s),
            Expr::Literal(Value::Int(i)) => write!(f, "{}", i),
            Expr::Literal(Value::Float(fl)) => write!(f, "{}", fl),
            Expr::Literal(Value::Bool(b)) => write!(f, "{}", b),
            Expr::Literal(Value::List(items)) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            Expr::Literal(Value::Dict(map)) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            },
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Add(left, right) => write!(f, "({} + {})", left, right),
            Expr::Subtract(left, right) => write!(f, "({} - {})", left, right),
            Expr::Multiply(left, right) => write!(f, "({} * {})", left, right),
            Expr::Divide(left, right) => write!(f, "({} / {})", left, right),
            Expr::GreaterThan(left, right) => write!(f, "({} > {})", left, right),
            Expr::GreaterThanOrEqual(left, right) => write!(f, "({} >= {})", left, right),
            Expr::LessThan(left, right) => write!(f, "({} < {})", left, right),
            Expr::LessThanOrEqual(left, right) => write!(f, "({} <= {})", left, right),
            Expr::Equal(left, right) => write!(f, "({} == {})", left, right),
            Expr::NotEqual(left, right) => write!(f, "({} != {})", left, right),
            Expr::And(left, right) => write!(f, "({} and {})", left, right),
            Expr::Or(left, right) => write!(f, "({} or {})", left, right),
            Expr::Not(expr) => write!(f, "(not {})", expr),
            Expr::FunctionCall(name, args) => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Conditional(branches) => {
                for (i, branch) in branches.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", branch)?;
                }
                Ok(())
            }
            Expr::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Expr::Dict(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            Expr::IndexAccess(container, index) => {
                write!(f, "{}[{}]", container, index)
            }
            Expr::MethodCall(obj, method, args) => {
                write!(f, "{}.{}(", obj, method)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            _ => write!(f, "<?>"),
        }
    }
}

impl std::fmt::Display for ConditionalBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)?;
        if let Some(cond) = &self.condition {
            write!(f, " when {}", cond)?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for ch in input.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
        } else if (ch == '+' || ch == '-' || ch == '*' || ch == '/' || ch == '(' || ch == ')' || ch == ',') && !in_quotes {
            if !current.trim().is_empty() {
                tokens.push(current.trim().to_string());
                current.clear();
            }
            tokens.push(ch.to_string());
        } else if ch.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    
    if !current.is_empty() {
        tokens.push(current);
    }
    
    tokens
}

fn parse_token(token: &str) -> Expr {
    let token = token.trim();
    if token.starts_with('"') && token.ends_with('"') {
        let content = token[1..token.len()-1].to_string();
        return Expr::Literal(Value::Str(content));
    }
    
    // Try integer first
    if let Ok(n) = token.parse::<i64>() {
        return Expr::Literal(Value::Int(n));
    }
    // Try float (must contain a dot and parse as f64, but not as int)
    if token.contains('.') {
        if let Ok(f) = token.parse::<f64>() {
            return Expr::Literal(Value::Float(f));
        }
    }
    
    if token == "true" {
        return Expr::Literal(Value::Bool(true));
    }
    if token == "false" {
        return Expr::Literal(Value::Bool(false));
    }
    
    // Check for function calls
    if token.contains('(') && token.ends_with(')') {
        let name_end = token.find('(').unwrap();
        let func_name = &token[..name_end];
        let args_str = &token[name_end+1..token.len()-1];
        
        // Parse arguments
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut paren_depth = 0;
        let mut in_quotes = false;
        
        for ch in args_str.chars() {
            match ch {
                '"' => in_quotes = !in_quotes,
                '(' if !in_quotes => paren_depth += 1,
                ')' if !in_quotes => paren_depth -= 1,
                ',' if !in_quotes && paren_depth == 0 => {
                    if !current_arg.trim().is_empty() {
                        args.push(parse_token(current_arg.trim()));
                    }
                    current_arg.clear();
                }
                _ => {
                    current_arg.push(ch);
                }
            }
        }
        
        if !current_arg.trim().is_empty() {
            args.push(parse_token(current_arg.trim()));
        }
        
        return Expr::FunctionCall(func_name.to_string(), args);
    }

    
    
    // Handle list literals: [1, 2, 3]
    if token.starts_with('[') && token.ends_with(']') {
        let content = &token[1..token.len()-1].trim();
        if content.is_empty() {
            return Expr::List(Vec::new());
        }
        // Very simple parser for now - just split by comma
        let items: Vec<Expr> = content.split(',')
            .map(|item| parse_token(item.trim()))
            .collect();
        return Expr::List(items);
    }
    
    // Handle dict literals: {"key": "value"} - very simple
    if token.starts_with('{') && token.ends_with('}') {
        // For now, return empty dict - we'll enhance this later
        return Expr::Dict(HashMap::new());
    }
    
    Expr::Variable(token.to_string())
}

fn is_conditional_expression(s: &str) -> bool {
    let mut paren_depth = 0;
    let mut in_quotes = false;
    let mut in_braces = 0;
    
    for ch in s.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '(' if !in_quotes => paren_depth += 1,
            ')' if !in_quotes => paren_depth -= 1,
            '{' if !in_quotes => in_braces += 1,
            '}' if !in_quotes => in_braces -= 1,
            '|' if !in_quotes && paren_depth == 0 && in_braces == 0 => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn split_conditional_branches(s: &str) -> Result<Vec<&str>, String> {
    let mut branches = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0;
    let mut in_quotes = false;
    let mut in_braces = 0;
    
    for (i, ch) in s.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '(' if !in_quotes => paren_depth += 1,
            ')' if !in_quotes => paren_depth -= 1,
            '{' if !in_quotes => in_braces += 1,
            '}' if !in_quotes => in_braces -= 1,
            '|' if !in_quotes && paren_depth == 0 && in_braces == 0 => {
                let branch = &s[start..i];
                if branch.trim().is_empty() {
                    return Err("Empty branch in conditional expression".to_string());
                }
                branches.push(branch);
                start = i + 1;
            }
            _ => {}
        }
    }
    
    let last_branch = &s[start..];
    if last_branch.trim().is_empty() {
        return Err("Empty branch in conditional expression".to_string());
    }
    branches.push(last_branch);
    
    Ok(branches)
}

fn find_when_keyword(s: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut paren_depth = 0;
    let mut in_braces = 0;
    
    let chars: Vec<char> = s.chars().collect();
    
    for i in 0..chars.len() {
        match chars[i] {
            '"' => in_quotes = !in_quotes,
            '(' if !in_quotes => paren_depth += 1,
            ')' if !in_quotes => paren_depth -= 1,
            '{' if !in_quotes => in_braces += 1,
            '}' if !in_quotes => in_braces -= 1,
            _ => {}
        }
        
        if !in_quotes && paren_depth == 0 && in_braces == 0 {
            if i + 4 <= chars.len() {
                let word: String = chars[i..i+4].iter().collect();
                if word.to_lowercase() == "when" {
                    let prev_char = if i > 0 { Some(chars[i-1]) } else { None };
                    let next_char = if i + 4 < chars.len() { Some(chars[i+4]) } else { None };
                    
                    let is_word_start = prev_char.map(|c| c.is_whitespace() || c == '|').unwrap_or(true);
                    let is_word_end = next_char.map(|c| c.is_whitespace() || c == '|').unwrap_or(true);
                    
                    if is_word_start && is_word_end {
                        return Some(i);
                    }
                }
            }
        }
    }
    
    None
}

fn parse_conditional_branch(s: &str) -> Result<ConditionalBranch, String> {
    if let Some(when_pos) = find_when_keyword(s) {
        let value_str = s[..when_pos].trim();
        let condition_str = s[when_pos + 4..].trim(); // "when" is 4 chars
        
        if value_str.is_empty() {
            return Err("Missing value before 'when'".to_string());
        }
        if condition_str.is_empty() {
            return Err("Missing condition after 'when'".to_string());
        }
        
        let value_expr = parse_operator_expression(value_str)?;
        let condition_expr = parse_condition_expression(condition_str)?; // CHANGED
        
        Ok(ConditionalBranch {
            value: Box::new(value_expr),
            condition: Some(Box::new(condition_expr)),
        })
    } else {
        let value_expr = parse_operator_expression(s)?;
        Ok(ConditionalBranch {
            value: Box::new(value_expr),
            condition: None,
        })
    }
}

fn parse_conditional_expression(s: &str) -> Result<Expr, String> {
    let branches_str = split_conditional_branches(s)?;
    let mut branches = Vec::new();
    
    for branch_str in branches_str {
        let branch = parse_conditional_branch(branch_str)?;
        branches.push(branch);
    }
    
    Ok(Expr::Conditional(branches))
}

fn parse_condition_expression(s: &str) -> Result<Expr, String> {
    let s = s.trim();
    
    // Handle "not" operator
    if s.starts_with("not ") {
        let rest = s[4..].trim(); // Skip "not" and space
        if rest.is_empty() {
            return Err("Missing operand after 'not'".to_string());
        }
        let expr = parse_condition_expression(rest)?;
        return Ok(Expr::Not(Box::new(expr)));
    }
    
    // Handle parentheses at the beginning
    if s.starts_with('(') && s.ends_with(')') {
        // Check if it's properly matched outermost parentheses
        let mut depth = 0;
        let mut is_outermost = true;
        
        for (i, ch) in s.char_indices() {
            match ch {
                '(' => {
                    depth += 1;
                    if depth == 1 && i != 0 {
                        is_outermost = false;
                    }
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 && i != s.len() - 1 {
                        is_outermost = false;
                    }
                }
                _ => {}
            }
        }
        
        if is_outermost {
            // Parse inside the parentheses
            return parse_condition_expression(&s[1..s.len()-1].trim());
        }
    }
    
    // First, try to split on "and" (lowest precedence)
    if let Some(pos) = find_logical_operator(s, "and") {
        let left = &s[..pos].trim();
        let right = &s[pos + 3..].trim(); // "and" is 3 chars
        
        if left.is_empty() || right.is_empty() {
            return Err("Incomplete 'and' expression".to_string());
        }
        
        let left_expr = parse_condition_expression(left)?;
        let right_expr = parse_condition_expression(right)?;
        
        return Ok(Expr::And(Box::new(left_expr), Box::new(right_expr)));
    }
    
    // Then try "or"
    if let Some(pos) = find_logical_operator(s, "or") {
        let left = &s[..pos].trim();
        let right = &s[pos + 2..].trim(); // "or" is 2 chars
        
        if left.is_empty() || right.is_empty() {
            return Err("Incomplete 'or' expression".to_string());
        }
        
        let left_expr = parse_condition_expression(left)?;
        let right_expr = parse_condition_expression(right)?;
        
        return Ok(Expr::Or(Box::new(left_expr), Box::new(right_expr)));
    }
    
    // Then try comparison operators
    let comparisons = [">=", "<=", "==", "!=", ">", "<"];
    
    for &op in &comparisons {
        // Try to find the operator, ignoring those inside parentheses
        let mut search_pos = 0;
        while let Some(pos) = s[search_pos..].find(op) {
            let actual_pos = search_pos + pos;
        
            // Check if this operator is at top level (not inside parentheses)
            let before = &s[..actual_pos];
            let after = &s[actual_pos + op.len()..];
        
            // Simple check: count parentheses
            let open_parens = before.chars().filter(|&c| c == '(').count();
            let close_parens = before.chars().filter(|&c| c == ')').count();
        
            if open_parens == close_parens {
                // At top level
                let left = before.trim();
                let right = after.trim();
            
                if !left.is_empty() && !right.is_empty() {
                    let left_expr = parse_operator_expression(left)?;
                    let right_expr = parse_operator_expression(right)?;
                
                    return match op {
                        ">" => Ok(Expr::GreaterThan(Box::new(left_expr), Box::new(right_expr))),
                        ">=" => Ok(Expr::GreaterThanOrEqual(Box::new(left_expr), Box::new(right_expr))),
                        "<" => Ok(Expr::LessThan(Box::new(left_expr), Box::new(right_expr))),
                        "<=" => Ok(Expr::LessThanOrEqual(Box::new(left_expr), Box::new(right_expr))),
                        "==" => Ok(Expr::Equal(Box::new(left_expr), Box::new(right_expr))),
                        "!=" => Ok(Expr::NotEqual(Box::new(left_expr), Box::new(right_expr))),
                        _ => unreachable!(),
                    };
                }
            }
        
            search_pos = actual_pos + 1;
        }
    }
    
    // If no comparison operator found, parse as arithmetic expression
    parse_operator_expression(s)
}

fn find_logical_operator(s: &str, op: &str) -> Option<usize> {
    // Find "and" or "or" as whole words (not inside other words)
    let mut in_quotes = false;
    let mut in_paren = 0;
    
    for (i, _) in s.char_indices() {
        if i + op.len() <= s.len() && s[i..].starts_with(op) {
            // Check if it's a whole word
            let prev_ok = i == 0 || !s[i-1..].chars().next().unwrap().is_alphanumeric();
            let next_ok = i + op.len() >= s.len() || !s[i+op.len()..].chars().next().unwrap().is_alphanumeric();
            
            if prev_ok && next_ok && !in_quotes && in_paren == 0 {
                return Some(i);
            }
        }
        
        // Update quote/paren state
        // (Simplified - would need proper handling for nested quotes/parens)
        if s.chars().nth(i) == Some('"') {
            in_quotes = !in_quotes;
        } else if s.chars().nth(i) == Some('(') && !in_quotes {
            in_paren += 1;
        } else if s.chars().nth(i) == Some(')') && !in_quotes && in_paren > 0 {
            in_paren -= 1;
        }
    }
    
    None
}

fn parse_operator_expression(s: &str) -> Result<Expr, String> {
    let mut paren_depth = 0;
    let mut best_pos = None;
    let mut best_op = None;
    let mut best_precedence = 3;

    // Handle negative numbers
    if s.starts_with('-') {
        let rest = &s[1..].trim();
        if !rest.is_empty() {
            // Check if it's a number
            if let Ok(n) = rest.parse::<i64>() {
                return Ok(Expr::Literal(Value::Int(-n)));
            }
            // Or it could be a parenthesized expression
            // For simplicity, we'll parse it as 0 - expression
            let expr = parse_operator_expression(rest)?;
            return Ok(Expr::Subtract(Box::new(Expr::Literal(Value::Int(0))), Box::new(expr)));
        }
    }
    
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '+' | '-' if paren_depth == 0 => {
                best_pos = Some(i);
                best_op = Some(ch);
                best_precedence = 1;
            }
            '*' | '/' if paren_depth == 0 && best_precedence > 1 => {
                best_pos = Some(i);
                best_op = Some(ch);
                best_precedence = 2;
            }
            _ => {}
        }
    }
    
    if let (Some(pos), Some(op)) = (best_pos, best_op) {
        let left = s[..pos].trim();
        let right = s[pos+1..].trim();
        
        if left.is_empty() || right.is_empty() {
            return Err(format!("Incomplete expression around '{}'", op));
        }
        
        let left_expr = parse_operator_expression(left)?;
        let right_expr = parse_operator_expression(right)?;
        
        match op {
            '+' => Ok(Expr::Add(Box::new(left_expr), Box::new(right_expr))),
            '-' => Ok(Expr::Subtract(Box::new(left_expr), Box::new(right_expr))),
            '*' => Ok(Expr::Multiply(Box::new(left_expr), Box::new(right_expr))),
            '/' => Ok(Expr::Divide(Box::new(left_expr), Box::new(right_expr))),
            _ => unreachable!(),
        }
    } else {
        if s.starts_with('(') && s.ends_with(')') {
            let mut depth = 0;
            let mut is_outermost = true;
            
            for (i, ch) in s.char_indices() {
                match ch {
                    '(' => {
                        depth += 1;
                        if depth == 1 && i != 0 {
                            is_outermost = false;
                        }
                    }
                    ')' => {
                        depth -= 1;
                        if depth == 0 && i != s.len() - 1 {
                            is_outermost = false;
                        }
                    }
                    _ => {}
                }
            }
            
            if is_outermost {
                parse_operator_expression(&s[1..s.len()-1].trim())
            } else {
                Ok(parse_token(s))
            }
        } else {
            Ok(parse_token(s))
        }
    }
}

fn parse_expr_with_precedence(s: &str) -> Result<Expr, String> {
    if is_conditional_expression(s) {
        parse_conditional_expression(s)
    } else {
        parse_operator_expression(s)
    }
}

pub fn parse_expression(input: &str) -> Result<Expr, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    parse_expr_with_precedence(trimmed)
}

pub fn evaluate(expr: &Expr, env: &Env) -> Result<Value, String> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Variable(name) => {
            env.get_value(name)
                .cloned()
                .ok_or_else(|| format!("Variable not found: {}", name))
        }
        Expr::Add(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64) + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + (*b as f64))),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Str(a), Value::Int(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Int(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Str(a), Value::Float(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Float(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                _ => Err(format!("Cannot add {} and {} - use explicit types for math", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::Subtract(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64) - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - (*b as f64))),
                _ => Err(format!("Cannot subtract {} from {} - must be int or float", right_val.type_name(), left_val.type_name())),
            }
        }
        Expr::Multiply(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64) * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * (*b as f64))),
                _ => Err(format!("Cannot multiply {} and {} - must be int or float", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::Divide(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Float((*a as f64) / (*b as f64)))
                    }
                }
                (Value::Float(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Float(a / b))
                    }
                }
                (Value::Int(a), Value::Float(b)) => {
                    if *b == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Float((*a as f64) / *b))
                    }
                }
                (Value::Float(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Float(*a / (*b as f64)))
                    }
                }
                _ => Err(format!("Cannot divide {} by {} - must be int or float", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::FunctionCall(name, args) => {
            let evaluated_args: Result<Vec<Value>, String> = 
                args.iter().map(|arg| evaluate(arg, env)).collect();
            let args_values = evaluated_args?;
            
            match name.as_str() {
                "count" if args_values.len() == 2 => {
                    let value = &args_values[0];
                    let pattern = match &args_values[1] {
                        Value::Str(s) => s.as_str(),
                        _ => return Err("Pattern must be a string".to_string()),
                    };
                    builtins::count(value, pattern)
                }
                "now" if args_values.is_empty() => Ok(builtins::now()),
                "len" if args_values.len() == 1 => {
                    builtins::len(&args_values[0])
                }
                "upper" if args_values.len() == 1 => {
                    builtins::upper(&args_values[0])
                }
                "lower" if args_values.len() == 1 => {
                    builtins::lower(&args_values[0])
                }
                "trim" if args_values.len() == 1 => {
                    builtins::trim(&args_values[0])
                }
                _ => Err(format!("Unknown function or wrong arity: {}/{}", name, args.len())),
            }
        }
        Expr::Conditional(branches) => {
            for branch in branches {
                match &branch.condition {
                    Some(condition) => {
                        match evaluate(condition, env)? {
                            Value::Bool(true) => {
                                return evaluate(&branch.value, env);
                            }
                            Value::Bool(false) => {
                                continue;
                            }
                            _ => return Err("Condition must evaluate to boolean".to_string()),
                        }
                    }
                    None => {
                        return evaluate(&branch.value, env);
                    }
                }
            }
            Err("No matching condition in conditional expression".to_string())
        }
        Expr::GreaterThan(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > (*b as f64))),
                _ => Err(format!("Cannot compare {} and {}", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::GreaterThanOrEqual(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a >= (*b as f64))),
                _ => Err(format!("Cannot compare {} and {}", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::LessThan(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < (*b as f64))),
                _ => Err(format!("Cannot compare {} and {}", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::LessThanOrEqual(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            match (&left_val, &right_val) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a <= (*b as f64))),
                _ => Err(format!("Cannot compare {} and {}", left_val.type_name(), right_val.type_name())),
            }
        }
        Expr::Equal(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            Ok(Value::Bool(left_val == right_val))
        }
        Expr::NotEqual(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            Ok(Value::Bool(left_val != right_val))
        }
        Expr::And(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            
            match (&left_val, &right_val) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                _ => Err("Logical 'and' requires boolean operands".to_string()),
            }
        }
        Expr::Or(left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            
            match (&left_val, &right_val) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                _ => Err("Logical 'or' requires boolean operands".to_string()),
            }
        }
        Expr::Not(expr) => {
            let val = evaluate(expr, env)?;
            match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err("Logical 'not' requires boolean operand".to_string()),
            }
        }
        Expr::List(items) => {
            let mut evaluated_items = Vec::new();
            for item in items {
                evaluated_items.push(evaluate(item, env)?);
            }
            Ok(Value::List(evaluated_items))
        }
        Expr::Dict(map) => {
            let mut evaluated_map = HashMap::new();
            for (key, value_expr) in map {
                let value = evaluate(value_expr, env)?;
                evaluated_map.insert(key.clone(), value);
            }
            Ok(Value::Dict(evaluated_map))
        }
        Expr::IndexAccess(container_expr, index_expr) => {
            let container = evaluate(container_expr, env)?;
            let index = evaluate(index_expr, env)?;
            
            match (&container, &index) {
                (Value::List(items), Value::Int(i)) => {
                    let idx = *i as usize;
                    if idx < items.len() {
                        Ok(items[idx].clone())
                    } else {
                        Err(format!("Index {} out of bounds for list of length {}", idx, items.len()))
                    }
                }
                (Value::Dict(map), Value::Str(key)) => {
                    if let Some(value) = map.get(key) {
                        Ok(value.clone())
                    } else {
                        Err(format!("Key '{}' not found in dictionary", key))
                    }
                }
                _ => Err(format!("Cannot index {} with {}", container.type_name(), index.type_name())),
            }
        }
        Expr::MethodCall(obj_expr, method_name, args) => {
            let obj = evaluate(obj_expr, env)?;
            
            // Evaluate all arguments first
            let mut evaluated_args = Vec::new();
            for arg in args {
                evaluated_args.push(evaluate(arg, env)?);
            }
            
            match (obj, method_name.as_str()) {
                // String methods
                (Value::Str(s), "len") => Ok(Value::Int(s.len() as i64)),
                (Value::Str(s), "upper") => Ok(Value::Str(s.to_uppercase())),
                (Value::Str(s), "lower") => Ok(Value::Str(s.to_lowercase())),
                (Value::Str(s), "trim") => Ok(Value::Str(s.trim().to_string())),
                // List methods
                (Value::List(items), "len") => Ok(Value::Int(items.len() as i64)),
                (Value::List(mut items), "push") if !evaluated_args.is_empty() => {
                    items.push(evaluated_args[0].clone());
                    Ok(Value::List(items))
                }
                (Value::List(items), "pop") => {
                    let mut items = items;
                    if let Some(_last) = items.pop() {
                        Ok(Value::List(items))
                    } else {
                        Err("Cannot pop from empty list".to_string())
                    }
                }
                // Add more method implementations as needed
                (obj_val, method) => Err(format!("Method '{}' not implemented for {}", method, obj_val.type_name())),
            }
        }
    }
}


pub fn extract_variables(expr: &Expr) -> Vec<String> {
    let mut vars = HashSet::new();
    extract_variables_recursive(expr, &mut vars);
    vars.into_iter().collect()
}

    fn extract_variables_recursive(expr: &Expr, vars: &mut HashSet<String>) {
        match expr {
            Expr::Variable(name) => {
                vars.insert(name.clone());
            }
            Expr::Add(left, right)
            | Expr::Subtract(left, right)
            | Expr::Multiply(left, right)
            | Expr::Divide(left, right) => {
                extract_variables_recursive(left, vars);
                extract_variables_recursive(right, vars);
            }
            Expr::GreaterThan(left, right)
            | Expr::GreaterThanOrEqual(left, right)
            | Expr::LessThan(left, right)
            | Expr::LessThanOrEqual(left, right)
            | Expr::Equal(left, right)
            | Expr::NotEqual(left, right)
            | Expr::And(left, right)
            | Expr::Or(left, right) => {
                extract_variables_recursive(left, vars);
                extract_variables_recursive(right, vars);
            }
            Expr::Not(expr) => {
                extract_variables_recursive(expr, vars);
            }
            Expr::FunctionCall(_, args) => {
                for arg in args {
                    extract_variables_recursive(arg, vars);
                }
            }
            Expr::Conditional(branches) => {
                for branch in branches {
                    extract_variables_recursive(&branch.value, vars);
                    if let Some(condition) = &branch.condition {
                        extract_variables_recursive(condition, vars);
                    }
                }
            }
            
            Expr::List(items) => {
                for item in items {
                    extract_variables_recursive(item, vars);
                }
            }
            Expr::Dict(map) => {
                for value in map.values() {
                    extract_variables_recursive(value, vars);
                }
            }
            Expr::IndexAccess(container, index) => {
                extract_variables_recursive(container, vars);
                extract_variables_recursive(index, vars);
            }
            Expr::MethodCall(obj, _, args) => {
                extract_variables_recursive(obj, vars);
                for arg in args {
                    extract_variables_recursive(arg, vars);
                }
            }
            Expr::Literal(_) => {}
        }
    }

/*fn parse_index_access(expr_str: &str) -> Result<Expr, String> {
    // Find opening bracket
    if let Some(open_pos) = expr_str.find('[') {
        let var_name = &expr_str[..open_pos];
        let rest = &expr_str[open_pos..];
        
        if rest.ends_with(']') && rest.len() > 2 {
            let index_str = &rest[1..rest.len()-1];
            let var_expr = parse_token(var_name);
            let index_expr = parse_token(index_str);
            return Ok(Expr::IndexAccess(Box::new(var_expr), Box::new(index_expr)));
        }
    }
    Err("Invalid index access".to_string())
}*/