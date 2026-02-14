use std::collections::HashSet;
use std::fmt;

impl fmt::Display for crate::core::intent::Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


pub struct SafetyGuard {
    pub blocked_intents: HashSet<String>,
    pub max_recursion_depth: u32,
    pub allowed_sources: HashSet<String>,
    pub resource_limits: ResourceLimits,
    pub current_depth: u32,
}

impl SafetyGuard {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            blocked_intents: HashSet::from([
                "format_system".to_string(),
                "delete_core".to_string(),
                "modify_system_files".to_string(),
            ]),
            max_recursion_depth: 10,
            allowed_sources: HashSet::from([
                "system".to_string(),
                "user".to_string(),
                "library".to_string(),
            ]),
            resource_limits: ResourceLimits::default(),
            current_depth: 0,
        })
    }
    
    pub fn validate_intent(&self, intent: &crate::core::intent::Intent) -> Result<(), String> {
        // Check blocked intents
        let verb_string = format!("{:?}", intent.verb);
        if self.blocked_intents.contains(&verb_string) {
            return Err(format!("Intent '{}' is blocked by safety policy", verb_string));
        }
        
        // Check source
        if !self.allowed_sources.contains(&intent.integrity.created_by) {
            return Err("Intent from unauthorized source".to_string());
        }
        
        // Check recursion depth
        if self.current_depth >= self.max_recursion_depth {
            return Err("Maximum recursion depth exceeded".to_string());
        }
        
        Ok(())
    }
    
    pub fn validate_new_definition(&self, intent: &crate::core::intent::Intent) -> Result<(), String> {
        if intent.integrity.created_by != "user" {
            return Err("Only users can define new intents".to_string());
        }
        
        // Check for dangerous patterns in the definition
        self.validate_definition_safety(intent)
    }
    
    pub fn validate_execution(&self, intent: &crate::core::intent::Intent, env: &crate::core::env::Env) -> Result<(), String> {
        // Check if intent is from allowed source
        if !self.allowed_sources.contains(&intent.integrity.created_by) {
            return Err("Intent from unauthorized source".to_string());
        }
        
        // Check for blocked operations - use blocked_intents instead of blocked_operations
        for blocked in &self.blocked_intents {
            if intent.target_string().contains(blocked) {
                return Err(format!("Operation '{}' is blocked", blocked));
            }
        }
        
        // Resource limit checks
        if env.list().len() > self.resource_limits.max_variables {
            return Err("Resource limit exceeded: too many variables".to_string());
        }
        
        Ok(())
    }
    
    fn validate_definition_safety(&self, intent: &crate::core::intent::Intent) -> Result<(), String> {
        let target_str = intent.target_string();
        
        if target_str.contains("system") || target_str.contains("core") {
            return Err("Intent definition cannot reference system components".to_string());
        }
        
        Ok(())
    }
    
    pub fn create_child_context(&self) -> Self {
        Self {
            current_depth: self.current_depth + 1,
            blocked_intents: self.blocked_intents.clone(),
            max_recursion_depth: self.max_recursion_depth,
            allowed_sources: self.allowed_sources.clone(),
            resource_limits: self.resource_limits.clone(),
        }
    }

    pub fn validate_reflection(&self, intent: &crate::core::intent::Intent) -> Result<(), String> {
        // Only allow reflection in safe contexts
        if intent.integrity.created_by != "system" {
            return Err("Reflection operations restricted to system".to_string());
        }
        
        // Add more safety checks as needed
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_variables: usize,
    pub max_intent_size: usize,
    pub max_execution_time: std::time::Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_variables: 1000,
            max_intent_size: 10_000,
            max_execution_time: std::time::Duration::from_secs(30),
        }
    }
}
#[derive(Debug, Clone)]
pub struct SafetyRules {
    pub max_recursion_depth: u32,
    pub blocked_operations: Vec<String>,
    pub allowed_sources: Vec<String>,
    pub resource_limits: ResourceLimits,
}

impl SafetyRules {
    pub fn load_default_rules() -> Result<Self, String> {
        Ok(Self {
            max_recursion_depth: 10,
            blocked_operations: vec![
                "format_system".to_string(),
                "delete_system_files".to_string(),
                "modify_core_intents".to_string(),
            ],
            allowed_sources: vec![
                "system".to_string(),
                "user".to_string(), 
                "library".to_string(),
            ],
            resource_limits: ResourceLimits::default(),
        })
    }
    
    pub fn validate_user_intent(&self, intent: &crate::core::intent::Intent) -> Result<(), String> {
        // Check if intent is from allowed source
        if !self.allowed_sources.contains(&intent.integrity.created_by) {
            return Err("Intent from unauthorized source".to_string());
        }
        
        // Check for blocked operations
        for blocked in &self.blocked_operations {
            if intent.target_string().contains(blocked) {
                return Err(format!("Operation '{}' is blocked", blocked));
            }
        }
        
        Ok(())
    }
}

impl Clone for SafetyGuard {
    fn clone(&self) -> Self {
        Self {
            blocked_intents: self.blocked_intents.clone(),
            max_recursion_depth: self.max_recursion_depth,
            allowed_sources: self.allowed_sources.clone(),
            resource_limits: self.resource_limits.clone(),
            current_depth: 0,  // Reset depth for new context
        }
    }
}