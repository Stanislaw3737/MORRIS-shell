
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, serde::ts_seconds};
use dirs;
use std::fmt;

use crate::core::types::{Value, VariableSource};
use crate::core::env::Env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeEngine {
    pub version: String,
    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub last_modified: DateTime<Utc>,
    
    // Variables with metadata
    pub variables: HashMap<String, EngineVariable>,
    
    // Computed expressions
    pub computed_expressions: HashMap<String, ComputedExpression>,
    
    // Intent definitions
    pub intent_definitions: HashMap<String, IntentDefinition>,
    
    // Propagation rules
    pub propagation_rules: Vec<PropagationRule>,
    
    // Hooks
    pub hooks: HashMap<String, Hook>,
    
    // Metadata
    pub tags: HashMap<String, Vec<String>>,
    pub annotations: HashMap<String, String>,
    
    // Session state
    pub current_session: Option<SessionInfo>,
    pub recent_sessions: Vec<SessionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineVariable {
    pub value: Value,
    pub source: VariableSource,
    pub computed_from: Option<String>,
    pub metadata: VariableMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariableMetadata {
    pub description: Option<String>,
    pub units: Option<String>,
    pub confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated: Option<i64>,  // Store as timestamp
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComputedExpression {
    pub id: String,
    pub expression: String,
    pub dependencies: Vec<String>,
    pub triggers: Vec<Trigger>,
    pub cache_result: Option<Value>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntentDefinition {
    pub name: String,
    pub template: String,
    pub parameters: HashMap<String, ParameterDef>,
    pub guard_conditions: Vec<String>,
    pub examples: Vec<String>,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropagationRule {
    pub id: String,
    pub when: String,
    pub then: String,
    pub priority: i32,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hook {
    pub event: String,
    pub condition: Option<String>,
    pub action: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub started: i64,  // Store as timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended: Option<i64>,  // Store as timestamp
    pub intents_executed: usize,
    pub tags: Vec<String>,
}

impl SessionInfo {
    #[allow(dead_code)]
    pub fn started_dt(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.started, 0)
            .unwrap_or_else(|| Utc::now())
    }
    #[allow(dead_code)]
    pub fn ended_dt(&self) -> Option<DateTime<Utc>> {
        self.ended.and_then(|ts| DateTime::from_timestamp(ts, 0))
    }
    
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            started: Utc::now().timestamp(),
            ended: None,
            intents_executed: 0,
            tags: vec!["auto".to_string()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParameterDef {
    pub default: Option<String>,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trigger {
    pub event: String,
    pub condition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationRule {
    pub condition: String,
    pub message: String,
    pub severity: String, // "error", "warning", "info"
}

pub struct ChangeEngineManager {
    pub engine: ChangeEngine,
    pub file_path: PathBuf,
    auto_save: bool,
}

impl ChangeEngineManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let morris_dir = home.join(".morris");
        
        // Create .morris directory if it doesn't exist
        if !morris_dir.exists() {
            fs::create_dir_all(&morris_dir).ok();
        }
        
        let file_path = morris_dir.join("change_engine.json");
        
        let engine = ChangeEngine {
            version: "1.0".to_string(),
            created: Utc::now(),
            last_modified: Utc::now(),
            variables: HashMap::new(),
            computed_expressions: HashMap::new(),
            intent_definitions: HashMap::new(),
            propagation_rules: Vec::new(),
            hooks: HashMap::new(),
            tags: HashMap::new(),
            annotations: HashMap::new(),
            current_session: None,
            recent_sessions: Vec::new(),
        };
        
        Self {
            engine,
            file_path,
            auto_save: true,
        }
    }
    
    pub fn load(&mut self) -> Result<(), String> {
        if !self.file_path.exists() {
            self.initialize_defaults();
            return self.save();
        }
        
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read change engine: {}", e))?;
        
        if content.is_empty() {
            self.initialize_defaults();
            return self.save();
        }
        
        self.engine = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid change engine format: {}", e))?;
        
        // Update last modified and start new session
        self.engine.last_modified = Utc::now();
        self.start_session();
        
        Ok(())
    }
    
    pub fn save(&mut self) -> Result<(), String> {
        self.engine.last_modified = Utc::now();
        
        let json = serde_json::to_string_pretty(&self.engine)
            .map_err(|e| format!("Failed to serialize change engine: {}", e))?;
        
        // Atomic write
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, &json)
            .map_err(|e| format!("Failed to write change engine: {}", e))?;
        
        fs::rename(&temp_path, &self.file_path)
            .map_err(|e| format!("Failed to finalize change engine save: {}", e))?;
        
        Ok(())
    }
    
    fn initialize_defaults(&mut self) {
        // Add default intent definitions
        self.define_intent(IntentDefinition {
            name: "set".to_string(),
            template: "set {variable} = {value} [as {type}]".to_string(),
            parameters: HashMap::from([
                ("variable".to_string(), ParameterDef {
                    default: None,
                    description: Some("Variable name".to_string()),
                    required: true,
                }),
                ("value".to_string(), ParameterDef {
                    default: None,
                    description: Some("Value to assign".to_string()),
                    required: true,
                }),
                ("type".to_string(), ParameterDef {
                    default: Some("auto".to_string()),
                    description: Some("Type hint: int, bool, string, auto".to_string()),
                    required: false,
                }),
            ]),
            guard_conditions: vec![],
            examples: vec![
                "set x = 42".to_string(),
                "set name = \"Alice\" as string".to_string(),
                "set enabled = true as bool".to_string(),
            ],
            category: "core".to_string(),
        });
        
        // Add default propagation rule
        self.add_propagation_rule(PropagationRule {
            id: "default_propagation".to_string(),
            when: "variable changes".to_string(),
            then: "recompute dependents".to_string(),
            priority: 100,
            enabled: true,
            description: Some("Default propagation when variables change".to_string()),
        });
        
        // Add startup hook
        self.add_hook(Hook {
            event: "startup".to_string(),
            condition: None,
            action: "echo \"Morris Change Engine initialized\"".to_string(),
            enabled: true,
        });
        
        // Add shutdown hook
        self.add_hook(Hook {
            event: "shutdown".to_string(),
            condition: None,
            action: "engine save".to_string(),
            enabled: true,
        });
    }
    
    fn start_session(&mut self) {
        let session = SessionInfo::new();  // Use the constructor
        self.engine.current_session = Some(session);
    }
    
    pub fn end_session(&mut self) {
        if let Some(ref mut session) = self.engine.current_session {
            session.ended = Some(Utc::now().timestamp());  // Store timestamp
        
            // Move to recent sessions
            self.engine.recent_sessions.push(session.clone());
            if self.engine.recent_sessions.len() > 10 {
                self.engine.recent_sessions.remove(0);
            }
        
            self.engine.current_session = None;
        }
    }
    
    pub fn record_intent(&mut self) {
        if let Some(ref mut session) = self.engine.current_session {
            session.intents_executed += 1;
        }
    }
    
    // Engine operations
    pub fn define_intent(&mut self, definition: IntentDefinition) {
        self.engine.intent_definitions.insert(definition.name.clone(), definition);
        if self.auto_save {
            self.save().ok();
        }
    }
    
    pub fn add_propagation_rule(&mut self, rule: PropagationRule) {
        self.engine.propagation_rules.push(rule);
        self.engine.propagation_rules.sort_by_key(|r| -r.priority);
        if self.auto_save {
            self.save().ok();
        }
    }
    
    pub fn add_hook(&mut self, hook: Hook) {
        self.engine.hooks.insert(hook.event.clone(), hook);
        if self.auto_save {
            self.save().ok();
        }
    }
    #[allow(dead_code)]
    pub fn tag_variable(&mut self, var_name: &str, tag: &str) {
        self.engine.tags
            .entry(var_name.to_string())
            .or_insert_with(Vec::new)
            .push(tag.to_string());
        if self.auto_save {
            self.save().ok();
        }
    }
    #[allow(dead_code)]
    pub fn annotate(&mut self, target: &str, annotation: &str) {
        self.engine.annotations.insert(target.to_string(), annotation.to_string());
        if self.auto_save {
            self.save().ok();
        }
    }
    
    pub fn capture_env_state(&mut self, env: &Env) {
        // Capture all variables from environment
        for (name, value) in env.list() {
            if let Some(var) = env.get_variable(&name) {
                let engine_var = EngineVariable {
                    value: value.clone(),
                    source: var.source.clone(),
                    computed_from: var.expression.clone(),
                    metadata: VariableMetadata {
                        description: None,
                        units: None,
                        confidence: 1.0,
                        last_validated: Some(Utc::now().timestamp()),                        
                        tags: Vec::new(),
                    },
                };
                self.engine.variables.insert(name, engine_var);
            }
        }
    }
    #[allow(dead_code)]
    pub fn restore_env_state(&self, env: &mut Env) -> Result<(), String> {
        // Restore variables to environment
        for (name, engine_var) in &self.engine.variables {
            env.set_direct(name, engine_var.value.clone());
        }
        Ok(())
    }
    
    // Query methods
    #[allow(dead_code)]
    pub fn find_intent(&self, name: &str) -> Option<&IntentDefinition> {
        self.engine.intent_definitions.get(name)
    }
    #[allow(dead_code)]
    pub fn get_variables_by_tag(&self, tag: &str) -> Vec<(&String, &EngineVariable)> {
        self.engine.variables.iter()
            .filter(|(name, _)| self.engine.tags.get(*name)
                .map_or(false, |tags| tags.contains(&tag.to_string())))
            .collect()
    }
    
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        // Check for undefined variables in computed expressions
        for (expr_id, expr) in &self.engine.computed_expressions {
            for dep in &expr.dependencies {
                if !self.engine.variables.contains_key(dep) {
                    errors.push(ValidationError {
                        severity: "warning".to_string(),
                        message: format!("Computed expression '{}' depends on undefined variable '{}'", expr_id, dep),
                        location: format!("expression:{}", expr_id),
                    });
                }
            }
        }
        
        // Check propagation rule syntax (basic check)
        for rule in &self.engine.propagation_rules {
            if rule.when.is_empty() || rule.then.is_empty() {
                errors.push(ValidationError {
                    severity: "error".to_string(),
                    message: format!("Propagation rule '{}' has empty when/then clause", rule.id),
                    location: format!("rule:{}", rule.id),
                });
            }
        }
        
        errors
    }
    
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            variables: self.engine.variables.len(),
            intent_definitions: self.engine.intent_definitions.len(),
            propagation_rules: self.engine.propagation_rules.len(),
            hooks: self.engine.hooks.len(),
            sessions: self.engine.recent_sessions.len() + 
                     if self.engine.current_session.is_some() { 1 } else { 0 },
            last_modified: self.engine.last_modified,
        }
    }
    #[allow(dead_code)]
    pub fn backup(&self, backup_path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.engine)
            .map_err(|e| format!("Failed to serialize for backup: {}", e))?;
        
        fs::write(backup_path, json)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct ValidationError {
    pub severity: String,
    pub message: String,
    pub location: String,
}

#[derive(Debug)]
pub struct EngineStats {
    pub variables: usize,
    pub intent_definitions: usize,
    pub propagation_rules: usize,
    pub hooks: usize,
    #[allow(dead_code)]
    pub sessions: usize,
    #[allow(dead_code)]
    pub last_modified: DateTime<Utc>,
}

impl VariableMetadata {
    #[allow(dead_code)]
    pub fn last_validated_dt(&self) -> Option<DateTime<Utc>> {
        self.last_validated.map(|ts| 
            DateTime::from_timestamp(ts, 0).unwrap_or(Utc::now())
        )
    }
    #[allow(dead_code)]
    pub fn set_last_validated(&mut self, dt: DateTime<Utc>) {
        self.last_validated = Some(dt.timestamp());
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} at {}", self.severity, self.message, self.location)
    }
}
