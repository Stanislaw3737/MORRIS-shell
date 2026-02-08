use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum Verb {
    Set,
    Ensure,
    Writeout,
    Derive,
    Find,
    Analyze,
    Execute,
    Freeze,
    Load,
    Save,
    Read,
    Write,
    Append,
    Mkdir,
    List,
    Info,
    Exists,
    Page,       // Show current page
    Turn,       // Change directory
    Bookmark,   // Create bookmark
    Bookmarks,  // List bookmarks
    Jump,      // Enhanced navigation (supports relative paths)
    Peek,      // Look at history without navigating
    Mark,
    #[allow(dead_code)]      // Create a mark (like bookmark but temporary)
    Goto,      // Alias for jump
    Return,  
    RemoveBookmark, // Remove bookmark
    Volume,     // Define volume
    Volumes,    // List volumes
    Shelve,     // Save position
    Unshelve,   // Restore position
    Annotate,   // Add notes
    ReadAnnotation, // Read annotation
    Index,      // List contents
    Back,
    #[allow(dead_code)]       // Go back in history
    Chapter,
    #[allow(dead_code)]    // Navigate within volume (alias for turn)
    Skim,       // Quick preview (alias for read with preview)
    Library,    // System overview

    // History operations
    History,
    HistorySearch,
    HistoryTag,
    HistoryReplay,
    HistoryClear,
    HistorySave,
    
    // Change Engine operations
    EngineStatus,
    EngineSave,
    EngineLoad,
    EngineValidate,
    EngineDefine,
    EngineRule,
    EngineHook,

    Craft,        // Begin crafting a change (start transaction)
    Forge,        // Finalize and apply crafted changes (commit)
    Smelt,        // Melt down crafted changes (rollback)
    Temper,       // Test changes without applying (dry-run)
    Inspect,      // View current crafted changes
    Anneal,       // Apply changes gradually (staged commit)
    Quench,       // Apply changes immediately (fast commit)
    
    // Transaction verbs (Phase 2 - coming soon)
    Polish,       // Optimize crafted changes before forging
    Alloy,        // Merge multiple crafted changes
    Engrave,      // Add metadata to crafted changes
    Gild,         // Mark changes as important/golden
    Patina,       // Show transaction history for variable
    Transaction,  // Show current transaction status

    WhatIf,

    Collection,    // collection name with item1, item2, item3
    Dictionary,    // dictionary name {key: value, key2: value2}
    //Assign,        // assign array[0] = value
    //Json,          // json var_name """{"key": "value"}"""

}

#[derive(Debug, Clone)]
pub enum Target {
    Variable(String),
    File(String),
    #[allow(dead_code)]
    Service(String),
    Process(String),
    Port(u16),
    Expression(String),
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub left: String,
    pub operator: String,
    pub right: String,
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub verb: Verb,
    pub target: Option<Target>,
    pub condition: Option<Condition>,
    pub parameters: HashMap<String, String>,
    pub context: HashMap<String, String>,
    pub state: IntentState,
    // NEW: Composition fields
    pub is_composition: bool,
    pub composition_name: Option<String>,
    #[allow(dead_code)]
    pub sub_intents: Vec<Uuid>, // IDs of sub-intents
    pub parameter_defs: HashMap<String, String>, // Parameter definitions with defaults
    pub execution_guard: Option<String>, // Condition to check before execution
    pub intent_source: Option<String>, // For defined intents
}

impl Intent {
    pub fn new(verb: Verb) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            verb,
            target: None,
            condition: None,
            parameters: HashMap::new(),
            context: HashMap::new(),
            state: IntentState::Created,
            // New fields with defaults
            is_composition: false,
            composition_name: None,
            sub_intents: Vec::new(),
            parameter_defs: HashMap::new(),
            execution_guard: None,
            intent_source: None,
        }
    }
    
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }
    #[allow(dead_code)]
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }
    
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    #[allow(dead_code)]
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn target_string(&self) -> String {
        match &self.target {
            Some(Target::Variable(name)) => format!("var:{}", name),
            Some(Target::File(path)) => format!("file:{}", path),
            Some(Target::Expression(expr)) => format!("expr:{}", expr),
            Some(Target::Service(name)) => format!("service:{}", name),
            Some(Target::Process(name)) => format!("process:{}", name),
            Some(Target::Port(port)) => format!("port:{}", port),
            None => "none".to_string(),
        }
    }
    #[allow(dead_code)]
    pub fn condition_string(&self) -> Option<String> {
        self.condition.as_ref().map(|c| format!("{} {} {}", c.left, c.operator, c.right))
    }
    
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
    #[allow(dead_code)]
    pub fn display_info(&self) -> String {
        format!("ID: {}, Time: {}", 
            self.id.simple().to_string(),
            self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
    }
    #[allow(dead_code)]
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.timestamp
    }
    
    // NEW: Composition methods
    pub fn mark_as_composition(mut self, name: &str) -> Self {
        self.is_composition = true;
        self.composition_name = Some(name.to_string());
        self
    }
    #[allow(dead_code)]
    pub fn with_sub_intents(mut self, sub_ids: Vec<Uuid>) -> Self {
        self.sub_intents = sub_ids;
        self
    }
    
    pub fn with_parameter_def(mut self, param_name: &str, default_value: &str) -> Self {
        self.parameter_defs.insert(param_name.to_string(), default_value.to_string());
        self
    }
    #[allow(dead_code)]
    pub fn with_execution_guard(mut self, condition: &str) -> Self {
        self.execution_guard = Some(condition.to_string());
        self
    }
    
    pub fn with_source(mut self, source: &str) -> Self {
        self.intent_source = Some(source.to_string());
        self
    }
    
    // NEW: Check if execution is allowed
    pub fn can_execute(&self, env: &crate::core::env::Env) -> Result<bool, String> {
        if let Some(guard) = &self.execution_guard {
            // Simple guard evaluation for now
            // TODO: Implement proper guard evaluation
            if guard.contains(">") {
                let parts: Vec<&str> = guard.split('>').collect();
                if parts.len() == 2 {
                    let left = parts[0].trim();
                    let right = parts[1].trim();
                    
                    // Try to get values from environment
                    let left_val = env.get_value(left).map(|v| v.to_string());
                    let right_val = env.get_value(right).map(|v| v.to_string());
                    
                    if let (Some(l), Some(r)) = (left_val, right_val) {
                        if let (Ok(l_num), Ok(r_num)) = (l.parse::<f64>(), r.parse::<f64>()) {
                            return Ok(l_num > r_num);
                        }
                    }
                }
            }
            Ok(true) // Default to true if can't evaluate
        } else {
            Ok(true)
        }
    }
    
    // NEW: Apply parameters to create concrete intent
    pub fn instantiate_with_params(&self, params: &HashMap<String, String>) -> Self {
        let mut instantiated = self.clone();
        
        // Merge provided parameters with defaults
        let mut all_params = self.parameter_defs.clone();
        for (key, value) in params {
            all_params.insert(key.clone(), value.clone());
        }
        
        // Replace parameter placeholders in target string
        if let Some(Target::Expression(expr)) = &instantiated.target {
            let mut new_expr = expr.clone();
            for (param, value) in &all_params {
                let placeholder = format!("{{{}}}", param);
                new_expr = new_expr.replace(&placeholder, value);
            }
            instantiated.target = Some(Target::Expression(new_expr));
        }
        
        // Also replace in parameters
        for (_key, value) in &mut instantiated.parameters {
            for (param, param_value) in &all_params {
                let placeholder = format!("{{{}}}", param);
                *value = value.replace(&placeholder, param_value);
            }
        }
        
        instantiated
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntentState {
    Created,
    Parsed,
    Executing,
    Succeeded,
    Failed,
    NeedsClarification,
}

// REMOVED THE DUPLICATE impl Intent BLOCK HERE

fn parse_ensure_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("ensure ").trim();

    if content.contains('=') {
        let parts: Vec<&str> = content.splitn(2, '=').map(|s| s.trim()).collect();
        if parts.len() == 2 && !parts[0].is_empty() {
            let var_name = parts[0];
            let value = parts[1];
            
            return Ok(Intent::new(Verb::Ensure)
                .with_target(Target::Variable(var_name.to_string()))
                .with_parameter("value", value));
        }
    }
    
    if content.starts_with("port ") {
        let port_str = content.trim_start_matches("port ").trim();
        
        if let Ok(port) = port_str.parse::<u16>() {
            let intent = Intent::new(Verb::Ensure)
                .with_target(Target::Port(port))
                .with_parameter("state", "open");
            
            Ok(intent)
        } else {
            Err(format!("Invalid port number: {}", port_str))
        }
    } else {
        Err(format!("Unknown ensure format: '{}'", content))
    }
}

fn parse_writeout_intent(input: &str) -> Result<Intent, String> {
    let content = if input.starts_with("writeout(") && input.ends_with(')') {
        &input[9..input.len()-1]
    } else if input.starts_with("writeout ") {
        &input[9..]
    } else {
        return Err("Invalid writeout syntax. Use: writeout(content) or writeout content".to_string());
    };
    
    let intent = Intent::new(Verb::Writeout)
        .with_target(Target::Expression(content.to_string()));
    
    Ok(intent)
}

fn parse_derive_intent(input: &str) -> Result<Intent, String> {
    let var_name = input.trim_start_matches("derive ").trim();
    
    if var_name.is_empty() {
        return Err("Variable name cannot be empty".to_string());
    }
    
    let intent = Intent::new(Verb::Derive)
        .with_target(Target::Variable(var_name.to_string()));
    
    Ok(intent)
}

fn parse_find_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("find ").trim();
    
    if content.is_empty() {
        return Err("Find pattern cannot be empty".to_string());
    }
    
    let intent = if content.starts_with('"') && content.ends_with('"') {
        let pattern = &content[1..content.len()-1];
        Intent::new(Verb::Find)
            .with_parameter("pattern", pattern)
    } else {
        Intent::new(Verb::Find)
            .with_target(Target::Expression(content.to_string()))
    };
    
    Ok(intent)
}

fn parse_analyze_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("analyze ").trim();
    
    if content.is_empty() {
        Ok(Intent::new(Verb::Analyze))
    } else {
        Ok(Intent::new(Verb::Analyze)
            .with_target(Target::Variable(content.to_string())))
    }
}

fn parse_execute_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("execute ").trim();
    
    if content.is_empty() {
        return Err("Execute command cannot be empty".to_string());
    }
    
    if content.starts_with("process ") && content.contains(" monitor") {
        let process_start = content.find('"');
        let process_end = content.rfind('"');
        
        if let (Some(start), Some(end)) = (process_start, process_end) {
            let process = &content[start+1..end];
            return Ok(Intent::new(Verb::Execute)
                .with_target(Target::Process(process.to_string()))
                .with_parameter("action", "monitor"));
        }
    }
    
    let intent = if content.starts_with('"') && content.ends_with('"') {
        let cmd = &content[1..content.len()-1];
        Intent::new(Verb::Execute)
            .with_target(Target::Expression(cmd.to_string()))
    } else {
        Intent::new(Verb::Execute)
            .with_target(Target::Expression(content.to_string()))
    };
    
    Ok(intent)
}

fn parse_freeze_intent(input: &str) -> Result<Intent, String> {
    let var_name = input.trim_start_matches("freeze ").trim();
    
    if var_name.is_empty() {
        return Err("Variable name cannot be empty".to_string());
    }
    
    Ok(Intent::new(Verb::Freeze)
        .with_target(Target::Variable(var_name.to_string())))
}

// NEW: Book metaphor parser functions

fn parse_turn_intent(input: &str) -> Result<Intent, String> {
    let destination = input.trim_start_matches("turn ").trim();
    
    if destination.is_empty() {
        return Err("Turn requires a destination".to_string());
    }
    
    // Check for relative navigation
    if destination.starts_with('-') || destination.starts_with('+') {
        // Validate it's a number after the sign
        let num_part = &destination[1..];
        if num_part.parse::<usize>().is_ok() {
            return Ok(Intent::new(Verb::Turn)
                .with_target(Target::Expression(destination.to_string())));
        }
    }
    
    let cleaned_destination = if destination.starts_with('"') && destination.ends_with('"') {
        &destination[1..destination.len()-1]
    } else {
        destination
    };
    
    Ok(Intent::new(Verb::Turn)
        .with_target(Target::Expression(cleaned_destination.to_string())))
}

fn parse_bookmark_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("bookmark ").trim();
    
    if content.is_empty() {
        return Err("Bookmark command requires arguments".to_string());
    }
    
    let parts: Vec<&str> = content.splitn(2, ' ').collect();
    let action = parts[0];
    
    match action {
        "add" => {
            if parts.len() < 2 {
                return Err("Bookmark add requires: bookmark add \"name\" [path]".to_string());
            }
            
            let rest = parts[1];
            // Try to parse name and optional path
            let mut name_parts = Vec::new();
            let mut in_quotes = false;
            let mut current = String::new();
            
            for ch in rest.chars() {
                if ch == '"' {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current.is_empty() {
                        name_parts.push(current.clone());
                        current.clear();
                    }
                } else if ch == ' ' && !in_quotes {
                    if !current.is_empty() {
                        name_parts.push(current.clone());
                        current.clear();
                    }
                } else {
                    current.push(ch);
                }
            }
            
            if !current.is_empty() {
                name_parts.push(current);
            }
            
            if name_parts.is_empty() {
                return Err("Bookmark name is required".to_string());
            }
            
            let name = &name_parts[0];
            let path = if name_parts.len() > 1 {
                Some(name_parts[1..].join(" ").trim().to_string())
            } else {
                None
            };
            
            let mut intent = Intent::new(Verb::Bookmark)
                .with_parameter("action", "add")
                .with_parameter("name", name);
            
            if let Some(p) = path {
                intent = intent.with_parameter("path", &p);
            }
            
            Ok(intent)
        }
        "remove" => {
            if parts.len() < 2 {
                return Err("Bookmark remove requires: bookmark remove \"name\"".to_string());
            }
            
            let name = parts[1].trim().trim_matches('"');
            
            Ok(Intent::new(Verb::RemoveBookmark)
                .with_parameter("name", name))
        }
        _ => Err("Bookmark command must be 'add' or 'remove'".to_string()),
    }
}

fn parse_volume_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("volume ").trim();
    
    if !content.starts_with("add ") {
        return Err("Volume command must be: volume add \"name\" path [\"description\"]".to_string());
    }
    
    let rest = content[4..].trim();
    
    // Simple parsing for now
    let mut parts = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();
    
    for ch in rest.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            if !in_quotes && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else if ch == ' ' && !in_quotes {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    if parts.len() < 2 {
        return Err("Volume add requires name and path".to_string());
    }
    
    let name = &parts[0];
    let path = &parts[1];
    let description = if parts.len() > 2 {
        Some(parts[2..].join(" ").trim().to_string())
    } else {
        None
    };
    
    let mut intent = Intent::new(Verb::Volume)
        .with_parameter("name", name)
        .with_parameter("path", path);
    
    if let Some(desc) = description {
        intent = intent.with_parameter("description", &desc);
    }
    
    Ok(intent)
}

fn parse_annotate_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("annotate ").trim();
    
    // Find the note (last quoted string)
    let last_quote = content.rfind('"');
    let first_quote = content.find('"');
    
    if let (Some(start), Some(end)) = (first_quote, last_quote) {
        if start >= end {
            return Err("Invalid annotation format".to_string());
        }
        
        let target = content[..start].trim();
        let note = &content[start+1..end];
        
        if target.is_empty() || note.is_empty() {
            return Err("Annotation requires target and note".to_string());
        }
        
        Ok(Intent::new(Verb::Annotate)
            .with_parameter("target", target)
            .with_parameter("note", note))
    } else {
        // Try without quotes
        let parts: Vec<&str> = content.splitn(2, ' ').collect();
        if parts.len() == 2 {
            Ok(Intent::new(Verb::Annotate)
                .with_parameter("target", parts[0])
                .with_parameter("note", parts[1]))
        } else {
            Err("Annotation requires target and note".to_string())
        }
    }
}

fn parse_read_annotation_intent(input: &str) -> Result<Intent, String> {
    let target = input.trim_start_matches("read_annotation ").trim();
    
    if target.is_empty() {
        return Err("read_annotation requires a target".to_string());
    }
    
    let cleaned_target = if target.starts_with('"') && target.ends_with('"') {
        &target[1..target.len()-1]
    } else {
        target
    };
    
    Ok(Intent::new(Verb::ReadAnnotation)
        .with_parameter("target", cleaned_target))
}

fn parse_back_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("back").trim();
    
    let steps = if content.is_empty() {
        "1".to_string()
    } else {
        content.to_string()
    };
    
    Ok(Intent::new(Verb::Back)
        .with_parameter("steps", &steps))
}

fn parse_chapter_intent(input: &str) -> Result<Intent, String> {
    // Chapter is an alias for turn
    let destination = input.trim_start_matches("chapter ").trim();
    
    if destination.is_empty() {
        return Err("Chapter requires a destination".to_string());
    }
    
    let cleaned_destination = if destination.starts_with('"') && destination.ends_with('"') {
        &destination[1..destination.len()-1]
    } else {
        destination
    };
    
    Ok(Intent::new(Verb::Turn)  // Using Turn for chapter
        .with_target(Target::Expression(cleaned_destination.to_string())))
}

fn parse_skim_intent(input: &str) -> Result<Intent, String> {
    // Skim is an alias for read with preview
    let file = input.trim_start_matches("skim ").trim();
    
    if file.is_empty() {
        return Err("Skim requires a file".to_string());
    }
    
    let cleaned_file = if file.starts_with('"') && file.ends_with('"') {
        &file[1..file.len()-1]
    } else {
        file
    };
    
    Ok(Intent::new(Verb::Read)
        .with_target(Target::File(cleaned_file.to_string()))
        .with_parameter("preview", "true"))
}

fn parse_craft_intent(input: &str) -> Result<Intent, String> {
    let name = input.trim_start_matches("craft ").trim();
    
    let mut intent = Intent::new(Verb::Craft);
    
    if !name.is_empty() {
        let cleaned_name = if name.starts_with('"') && name.ends_with('"') {
            &name[1..name.len()-1]
        } else {
            name
        };
        intent = intent.with_parameter("name", cleaned_name);
    }
    
    Ok(intent)
}

fn parse_forge_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Forge))
}

fn parse_smelt_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Smelt))
}

fn parse_temper_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Temper))
}

fn parse_inspect_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Inspect))
}

fn parse_anneal_intent(input: &str) -> Result<Intent, String> {
    let steps = input.trim_start_matches("anneal ").trim();
    
    if steps.is_empty() {
        return Ok(Intent::new(Verb::Anneal)
            .with_parameter("steps", "1"));
    }
    
    // Validate it's a number
    match steps.parse::<usize>() {
        Ok(_) => Ok(Intent::new(Verb::Anneal)
            .with_parameter("steps", steps)),
        Err(_) => Err(format!("Invalid number of steps: {}", steps)),
    }
}

fn parse_quench_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Quench))
}

fn parse_transaction_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Transaction))
}

// Placeholder parsers for Phase 2
fn parse_polish_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Polish))
}

fn parse_alloy_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Alloy))
}

fn parse_engrave_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Engrave))
}

fn parse_gild_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Gild))
}

fn parse_patina_intent(_input: &str) -> Result<Intent, String> {
    Ok(Intent::new(Verb::Patina))
}

pub fn parse_to_intent(input: &str) -> Result<Intent, String> {
    let input = input.trim();
    
    match input {
        "help" | "env" | "clear" => {
            let mut intent = Intent::new(Verb::Set)
                .with_context("system_command", input);
            intent.state = IntentState::NeedsClarification;
            Ok(intent)
        }

        "engine" | "engine status" => Ok(Intent::new(Verb::EngineStatus)),
        "engine save" => Ok(Intent::new(Verb::EngineSave)),
        "engine load" => Ok(Intent::new(Verb::EngineLoad)),
        "engine validate" => Ok(Intent::new(Verb::EngineValidate)),
        
        "history" => Ok(Intent::new(Verb::History)),
        _ if input.starts_with("define intent ") => parse_define_intent(input),
        _ if input.starts_with("forge intent ") => parse_define_intent(&input.replace("forge", "define")),  
        _ if input.starts_with("execute ") => {
            // Check if it's executing a defined intent
            parse_execute_intent(input)
        }
        _ if input.starts_with("set ") => parse_set_intent(input),
        _ if input.starts_with("ensure ") => parse_ensure_intent(input),
        _ if input.starts_with("writeout") => parse_writeout_intent(input),
        _ if input.starts_with("derive ") => parse_derive_intent(input),
        _ if input.starts_with("find ") => parse_find_intent(input),
        _ if input.starts_with("analyze ") => parse_analyze_intent(input),
        _ if input.starts_with("execute ") => parse_execute_intent(input),
        _ if input.starts_with("freeze ") => parse_freeze_intent(input),
        _ if input.starts_with("load ") => parse_load_intent(input),
        _ if input.starts_with("save ") => parse_save_intent(input),
        _ if input.starts_with("read ") => parse_read_intent(input),
        _ if input.starts_with("write ") => parse_write_intent(input),
        _ if input.starts_with("append ") => parse_append_intent(input),
        _ if input.starts_with("mkdir ") => parse_mkdir_intent(input),
        _ if input.starts_with("list ") => parse_list_intent(input),
        _ if input.starts_with("info ") => parse_info_intent(input),
        _ if input.starts_with("exists ") => parse_exists_intent(input),
        _ if input == "page" => Ok(Intent::new(Verb::Page)),
        _ if input.starts_with("turn ") => parse_turn_intent(input),
        _ if input.starts_with("bookmark ") => parse_bookmark_intent(input),
        _ if input == "bookmarks" => Ok(Intent::new(Verb::Bookmarks)),
        _ if input.starts_with("volume ") => parse_volume_intent(input),
        _ if input == "volumes" => Ok(Intent::new(Verb::Volumes)),
        _ if input == "shelve" => Ok(Intent::new(Verb::Shelve)),
        _ if input == "unshelve" => Ok(Intent::new(Verb::Unshelve)),
        _ if input.starts_with("annotate ") => parse_annotate_intent(input),
        _ if input.starts_with("read_annotation ") => parse_read_annotation_intent(input),
        _ if input == "index" => Ok(Intent::new(Verb::Index)),
        _ if input.starts_with("back") => parse_back_intent(input),
        _ if input.starts_with("chapter ") => parse_chapter_intent(input),
        _ if input.starts_with("skim ") => parse_skim_intent(input),
        _ if input == "library" => Ok(Intent::new(Verb::Library)),
        _ if input.starts_with("turn ") => parse_turn_intent(input),
        _ if input.starts_with("jump ") => parse_jump_intent(input),
        _ if input.starts_with("goto ") => parse_jump_intent(&input.replace("goto", "jump")),
        _ if input.starts_with("peek ") => parse_peek_intent(input),
        _ if input == "history" => parse_history_intent(input),
        _ if input.starts_with("history ") => parse_history_intent(input),
        _ if input == "engine" => parse_engine_intent(input),
        _ if input.starts_with("engine ") => parse_engine_intent(input),
        _ if input.starts_with("mark ") => parse_mark_intent(input),
        _ if input.starts_with("goto ") => {
            parse_jump_intent(&input.replace("goto", "jump"))
        }
        _ if input.starts_with("return") => {
            let steps = input.trim_start_matches("return").trim();
            let steps = if steps.is_empty() { "1" } else { steps };
            Ok(Intent::new(Verb::Return)
                .with_parameter("steps", steps))
        }

         // Transaction verbs (Phase 1)
        _ if input == "forge" => parse_forge_intent(input),
        _ if input == "smelt" => parse_smelt_intent(input),
        _ if input == "temper" => parse_temper_intent(input),
        _ if input == "inspect" => parse_inspect_intent(input),
        _ if input == "quench" => parse_quench_intent(input),
        _ if input == "transaction" => parse_transaction_intent(input),
        
        // ... other existing patterns ...
        
        _ if input.starts_with("craft ") => parse_craft_intent(input),
        _ if input.starts_with("anneal ") => parse_anneal_intent(input),
        
        // Transaction verbs (Phase 2 - placeholders)
        _ if input == "polish" => parse_polish_intent(input),
        _ if input == "alloy" => parse_alloy_intent(input),
        _ if input == "engrave" => parse_engrave_intent(input),
        _ if input == "gild" => parse_gild_intent(input),
        _ if input == "patina" => parse_patina_intent(input),
        _ if input.starts_with("what-if ") => parse_what_if_intent(input),
        _ => Err(format!("Unknown intent: '{}'", input)),
        
    }
}
// Add these missing parser functions before parse_to_intent:

fn parse_set_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("set ").trim();
    
    // Simple parser for set command
    // Format: set var = value [as type]
    let parts: Vec<&str> = content.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Set intent requires format: set var = value".to_string());
    }
    
    let var_name = parts[0].trim();
    let value_part = parts[1].trim();
    
    if var_name.is_empty() {
        return Err("Variable name cannot be empty".to_string());
    }
    
    let mut intent = Intent::new(Verb::Set)
        .with_target(Target::Variable(var_name.to_string()))
        .with_parameter("value", value_part);
    
    // Check for type hint
    if value_part.ends_with(" as int") || value_part.ends_with(" as :int") {
        let value = value_part[..value_part.len() - 7].trim();
        intent = intent
            .with_parameter("value", value)
            .with_parameter("type", ":int");
    } else if value_part.ends_with(" as bool") || value_part.ends_with(" as :bool") {
        let value = value_part[..value_part.len() - 8].trim();
        intent = intent
            .with_parameter("value", value)
            .with_parameter("type", ":bool");
    } else if value_part.ends_with(" as string") || value_part.ends_with(" as :string") {
        let value = value_part[..value_part.len() - 10].trim();
        intent = intent
            .with_parameter("value", value)
            .with_parameter("type", ":string");
    }
    
    Ok(intent)
}

fn parse_save_intent(input: &str) -> Result<Intent, String> {
    let path = input.trim_start_matches("save ").trim();
    
    if path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    
    let cleaned_path = if path.starts_with('"') && path.ends_with('"') {
        &path[1..path.len()-1]
    } else {
        path
    };
    
    Ok(Intent::new(Verb::Save)
        .with_target(Target::File(cleaned_path.to_string())))
}

fn parse_read_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("read ").trim();
    
    if content.is_empty() {
        return Err("Read intent requires file and variable".to_string());
    }
    
    // Parse "read \"file.txt\" into var"
    let parts: Vec<&str> = content.split(" into ").collect();
    if parts.len() != 2 {
        return Err("Read intent requires format: read \"file\" into variable".to_string());
    }
    
    let file_path = parts[0].trim();
    let var_name = parts[1].trim();
    
    if file_path.is_empty() || var_name.is_empty() {
        return Err("File path and variable name cannot be empty".to_string());
    }
    
    let cleaned_path = if file_path.starts_with('"') && file_path.ends_with('"') {
        &file_path[1..file_path.len()-1]
    } else {
        file_path
    };
    
    Ok(Intent::new(Verb::Read)
        .with_target(Target::File(cleaned_path.to_string()))
        .with_parameter("variable", var_name))
}

fn parse_write_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("write ").trim();
    
    if content.is_empty() {
        return Err("Write intent requires file and content".to_string());
    }
    
    // Simple parser for write command
    // Format: write "file" "content" or write "file" variable
    let mut parts = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();
    
    for ch in content.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            if !in_quotes && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else if ch == ' ' && !in_quotes {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    if parts.len() < 2 {
        return Err("Write intent requires format: write \"file\" \"content\"".to_string());
    }
    
    let file_path = &parts[0];
    let content_or_var = &parts[1];
    
    let mut intent = Intent::new(Verb::Write)
        .with_target(Target::File(file_path.to_string()));
    
    if content_or_var.starts_with('"') || content_or_var.ends_with('"') {
        // It's a string literal
        let content = if content_or_var.starts_with('"') && content_or_var.ends_with('"') {
            &content_or_var[1..content_or_var.len()-1]
        } else {
            content_or_var
        };
        intent = intent.with_parameter("content", content);
    } else {
        // It's a variable name
        intent = intent.with_parameter("variable", content_or_var);
    }
    
    Ok(intent)
}

fn parse_append_intent(input: &str) -> Result<Intent, String> {
    // Reuse write parser logic
    let write_intent = parse_write_intent(&input.replace("append", "write"))?;
    Ok(Intent {
        verb: Verb::Append,
        ..write_intent
    })
}

fn parse_mkdir_intent(input: &str) -> Result<Intent, String> {
    let path = input.trim_start_matches("mkdir ").trim();
    
    if path.is_empty() {
        return Err("Directory path cannot be empty".to_string());
    }
    
    let cleaned_path = if path.starts_with('"') && path.ends_with('"') {
        &path[1..path.len()-1]
    } else {
        path
    };
    
    Ok(Intent::new(Verb::Mkdir)
        .with_target(Target::File(cleaned_path.to_string())))
}

fn parse_list_intent(input: &str) -> Result<Intent, String> {
    let path = input.trim_start_matches("list ").trim();
    
    let cleaned_path = if path.starts_with('"') && path.ends_with('"') {
        &path[1..path.len()-1]
    } else {
        path
    };
    
    if cleaned_path.is_empty() {
        return Err("Directory path cannot be empty".to_string());
    }
    
    Ok(Intent::new(Verb::List)
        .with_target(Target::File(cleaned_path.to_string())))
}

fn parse_info_intent(input: &str) -> Result<Intent, String> {
    let path = input.trim_start_matches("info ").trim();
    
    if path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    
    let cleaned_path = if path.starts_with('"') && path.ends_with('"') {
        &path[1..path.len()-1]
    } else {
        path
    };
    
    Ok(Intent::new(Verb::Info)
        .with_target(Target::File(cleaned_path.to_string())))
}

fn parse_exists_intent(input: &str) -> Result<Intent, String> {
    let path = input.trim_start_matches("exists ").trim();
    
    if path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    
    let cleaned_path = if path.starts_with('"') && path.ends_with('"') {
        &path[1..path.len()-1]
    } else {
        path
    };
    
    Ok(Intent::new(Verb::Exists)
        .with_target(Target::File(cleaned_path.to_string())))
}

fn parse_load_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("load ").trim();
    
    if content.is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    
    let path = if content.starts_with('"') && content.ends_with('"') {
        &content[1..content.len()-1]
    } else {
        content
    };
    
    Ok(Intent::new(Verb::Load)
        .with_target(Target::File(path.to_string())))
}

// Add these parser functions after existing ones

pub fn parse_define_intent(input: &str) -> Result<Intent, String> {
    // Format: define intent "name" with (param1, param2="default") { expression }
    // OR: define intent "name" composed_of ["intent1", "intent2"]
    
    let content = input.trim_start_matches("define intent ").trim();
    
    // Parse intent name
    let name_end = content.find(' ').ok_or("Expected intent name")?;
    let name = &content[..name_end];
    let rest = content[name_end..].trim();
    
    if rest.starts_with("with") {
        // Parameterized intent definition
        parse_parameterized_intent(name, &rest[4..].trim())
    } else if rest.starts_with("composed_of") {
        // Composition definition
        parse_composition_intent(name, &rest[11..].trim())
    } else {
        Err("Expected 'with' or 'composed_of' after intent name".to_string())
    }
}

fn parse_parameterized_intent(name: &str, input: &str) -> Result<Intent, String> {
    // Find parameters section
    let params_start = input.find('(').ok_or("Expected '(' for parameters")?;
    let params_end = input.find(')').ok_or("Expected ')' after parameters")?;
    
    let params_str = &input[params_start + 1..params_end];
    let after_params = &input[params_end + 1..].trim();
    
    if !after_params.starts_with('{') || !after_params.ends_with('}') {
        return Err("Expected expression in {} after parameters".to_string());
    }
    
    let expression = &after_params[1..after_params.len() - 1].trim();
    
    // Parse parameters
    let mut intent = Intent::new(Verb::Set)  // Using Set as base for expressions
        .mark_as_composition(name)
        .with_source("defined_intent");
    
    for param_part in params_str.split(',') {
        let part = param_part.trim();
        if part.contains('=') {
            let parts: Vec<&str> = part.split('=').collect();
            if parts.len() == 2 {
                let param_name = parts[0].trim();
                let default_value = parts[1].trim().trim_matches('"');
                intent = intent.with_parameter_def(param_name, default_value);
            }
        } else if !part.is_empty() {
            intent = intent.with_parameter_def(part, "");
        }
    }
    
    // Set the expression as target
    intent = intent.with_target(Target::Expression(expression.to_string()));
    
    Ok(intent)
}

fn parse_composition_intent(name: &str, input: &str) -> Result<Intent, String> {
    // Format: ["intent1", "intent2", "intent3"]
    if !input.starts_with('[') || !input.ends_with(']') {
        return Err("Expected list of intents in []".to_string());
    }
    
    let list_str = &input[1..input.len() - 1];
    let intents: Vec<String> = list_str.split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    if intents.is_empty() {
        return Err("Composition must include at least one intent".to_string());
    }
    
    let mut intent = Intent::new(Verb::Execute)  // Execute for compositions
        .mark_as_composition(name)
        .with_source("defined_composition");
    
    // Store intent names as parameter for now (will resolve to IDs later)
    for intent_name in &intents {
        intent = intent.with_parameter("sub_intent", intent_name);
    }
    
    Ok(intent)
}

fn parse_jump_intent(input: &str) -> Result<Intent, String> {
    let destination = input.trim_start_matches("jump ").trim();
    
    if destination.is_empty() {
        return Err("Jump requires a destination".to_string());
    }
    
    Ok(Intent::new(Verb::Jump)
        .with_target(Target::Expression(destination.to_string())))
}

fn parse_peek_intent(input: &str) -> Result<Intent, String> {
    let rest = input.trim_start_matches("peek ").trim();
    
    if rest.is_empty() {
        // Peek back 1 by default
        Ok(Intent::new(Verb::Peek)
            .with_parameter("distance", "-1"))
    } else {
        Ok(Intent::new(Verb::Peek)
            .with_target(Target::Expression(rest.to_string())))
    }
}

fn parse_mark_intent(input: &str) -> Result<Intent, String> {
    // mark "name" [optional description]
    let rest = input.trim_start_matches("mark ").trim();
    
    if rest.is_empty() {
        return Err("Mark requires a name".to_string());
    }
    
    // Simple parsing - find first space or quote
    let mut parts = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();
    
    for ch in rest.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            if !in_quotes && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else if ch == ' ' && !in_quotes {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    if parts.is_empty() {
        return Err("Mark requires a name".to_string());
    }
    
    let name = &parts[0];
    let description = if parts.len() > 1 {
        Some(parts[1..].join(" ").trim().to_string())
    } else {
        None
    };
    
    let mut intent = Intent::new(Verb::Mark)
        .with_parameter("name", name);
    
    if let Some(desc) = description {
        intent = intent.with_parameter("description", &desc);
    }
    
    Ok(intent)
}

// New intents in intent.rs

fn parse_history_intent(input: &str) -> Result<Intent, String> {
    let rest = input.trim_start_matches("history").trim();
    
    if rest.is_empty() {
        return Ok(Intent::new(Verb::History));
    }
    
    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
    match parts[0] {
        "search" if parts.len() > 1 => {
            Ok(Intent::new(Verb::HistorySearch)
                .with_parameter("query", parts[1]))
        }
        "tag" if parts.len() > 1 => {
            Ok(Intent::new(Verb::HistoryTag)
                .with_parameter("tag", parts[1]))
        }
        "replay" if parts.len() > 1 => {
            Ok(Intent::new(Verb::HistoryReplay)
                .with_parameter("id", parts[1]))
        }
        "clear" => Ok(Intent::new(Verb::HistoryClear)),
        "save" => Ok(Intent::new(Verb::HistorySave)),
        _ => Err("Unknown history command".to_string()),
    }
}

fn parse_engine_intent(input: &str) -> Result<Intent, String> {
    let rest = input.trim_start_matches("engine").trim();
    
    if rest.is_empty() {
        return Ok(Intent::new(Verb::EngineStatus));
    }
    
    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
    match parts[0] {
        "save" => Ok(Intent::new(Verb::EngineSave)),
        "load" => Ok(Intent::new(Verb::EngineLoad)),
        "validate" => Ok(Intent::new(Verb::EngineValidate)),
        "define" if parts.len() > 1 => {
            // engine define intent "name" ...
            Ok(Intent::new(Verb::EngineDefine)
                .with_target(Target::Expression(parts[1].to_string())))
        }
        "rule" if parts.len() > 1 => {
            // engine rule add when "x > 5" then "alert high"
            Ok(Intent::new(Verb::EngineRule)
                .with_target(Target::Expression(parts[1].to_string())))
        }
        "hook" if parts.len() > 1 => {
            Ok(Intent::new(Verb::EngineHook)
                .with_target(Target::Expression(parts[1].to_string())))
        }
        _ => Err("Unknown engine command".to_string()),
    }
}

fn parse_what_if_intent(input: &str) -> Result<Intent, String> {
    let content = input.trim_start_matches("what-if ").trim();
    
    if content.is_empty() {
        return Err("What-if requires scenario specification".to_string());
    }
    
    let mut intent = Intent::new(Verb::WhatIf);
    
    // Check for "check" keyword
    if let Some(check_pos) = content.find(" check ") {
        let scenario_part = &content[..check_pos].trim();
        let check_part = &content[check_pos + 6..].trim(); // "check " is 6 chars
        
        intent = intent.with_parameter("check_condition", check_part);
        
        // Parse scenario variables
        for pair in scenario_part.split(',') {
            let parts: Vec<&str> = pair.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                intent = intent.with_parameter(parts[0], parts[1]);
            }
        }
    } else {
        // Parse scenario variables without check
        for pair in content.split(',') {
            let parts: Vec<&str> = pair.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                intent = intent.with_parameter(parts[0], parts[1]);
            }
        }
    }
    
    Ok(intent)
}