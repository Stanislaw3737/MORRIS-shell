use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::core::intent::{Verb, Target, parse_to_intent};
use crate::core::intent::{Intent, SafetyLevel};

pub struct LibraryManager {
    base_path: PathBuf,
    integrity_store: IntegrityStore,
}

impl LibraryManager {
    pub fn new() -> Result<Self, String> {
        let home = dirs::home_dir()
            .ok_or("Cannot determine home directory")?;
        let base_path = home.join(".morris");
        
        // Create directory structure with correct permissions
        let directories = vec![
            "system/intents",      // For future system intent files
            "user/intents",        // For user intent files  
            "user/validated", 
            "user/pending",
            "quarantine",
            "backups",
            "integrity"
        ];
        
        for dir in directories {
            let dir_path = base_path.join(dir);
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create {}: {}", dir, e))?;
        }
        
        let integrity_store = IntegrityStore::new(base_path.join("integrity"))?;
        
        Ok(Self {
            base_path,
            integrity_store,
        })
    }

    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    
    pub fn load_library(&mut self) -> Result<LibraryState, String> {
        let mut state = LibraryState::new();
        
        // Load system intents first (immutable)
        self.load_system_intents(&mut state)?;
        
        // Validate user intents against system integrity
        self.validate_user_intents(&mut state)?;
        
        // Check for tampering
        self.check_tampering(&state)?;
        
        Ok(state)
    }
    
    fn load_system_intents(&self, state: &mut LibraryState) -> Result<(), String> {
        // Look for system files in base_path, not subdirectories
        let core_msh_path = self.base_path.join("core.msh");
        let safety_msh_path = self.base_path.join("safety.msh");
        
        if core_msh_path.exists() {
            let content = fs::read_to_string(&core_msh_path)
                .map_err(|e| format!("Cannot read core.msh: {}", e))?;
            let intents = self.parse_intent_file(&content, &core_msh_path)?;
            let intents_vec: Vec<Intent> = intents.into_values().collect();
            state.add_system_intents(intents_vec);
        }
        
        if safety_msh_path.exists() {
            // Load safety-specific intents if needed
            let content = fs::read_to_string(&safety_msh_path)
                .map_err(|e| format!("Cannot read safety.msh: {}", e))?;
            let intents = self.parse_intent_file(&content, &safety_msh_path)?;
            let intents_vec: Vec<Intent> = intents.into_values().collect();
            state.add_system_intents(intents_vec);
        }
        
        Ok(())
    }
    
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn validate_user_intents(&self, state: &mut LibraryState) -> Result<(), String> {
        // Simple validation - check that user intents don't override system intents
        for (name, intent) in &state.user_intents {
            if state.system_intents.contains_key(name) {
                return Err(format!("User intent '{}' conflicts with system intent", name));
            }
        }
        Ok(())
    }
    
    pub fn check_tampering(&self, _state: &LibraryState) -> Result<(), String> {
        // Check modification dates, suspicious patterns, etc.
        // For now, simple check
        let integrity_file = self.base_path.join("integrity/system_hashes.json");
        if !integrity_file.exists() {
            return Err("System integrity database missing - possible tampering".to_string());
        }
        Ok(())
    }
    
    fn parse_intent_file(&self, content: &str, file_path: &Path) -> Result<HashMap<String, Intent>, String> {
        let mut intents = HashMap::new();
        let mut current_intent_name: Option<String> = None;  // Fix: Use String instead of str
        let mut current_intent_lines = Vec::new();
        
        for (_line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Check for intent definition (name = ...)
            if line.contains('=') && !line.starts_with(' ') {
                // Save previous intent if we have one
                if let Some(name) = current_intent_name.take() {
                    if let Ok(intent) = self.parse_intent_definition(&name, &current_intent_lines.join(" ")) {
                        intents.insert(name.clone(), intent);
                    }
                    current_intent_lines.clear();
                }
                
                // Parse new intent definition
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    current_intent_name = Some(parts[0].trim().to_string());
                    current_intent_lines.push(parts[1].trim());
                }
            } else if current_intent_name.is_some() {
                // Continuation line for current intent
                current_intent_lines.push(line);
            }
        }
        
        // Don't forget the last intent
        if let Some(name) = current_intent_name {
            if let Ok(intent) = self.parse_intent_definition(&name, &current_intent_lines.join(" ")) {
                intents.insert(name.clone(), intent);
            }
        }
        
        Ok(intents)
    }
    
    pub fn load_validated_library(&self) -> Result<LibraryState, String> {
        let mut state = LibraryState::new();
        
        // Load system intents
        self.load_system_intents(&mut state)?;
        
        // Validate user intents
        self.validate_user_intents(&mut state)?;
        
        // Check for tampering
        self.check_tampering(&state)?;
        
        Ok(state)
    }

    pub fn load_intent_files(&self) -> Result<HashMap<String, Intent>, String> {
        let mut loaded_intents = HashMap::new();
        
        // Load system intents
        let system_intents_path = self.base_path.join("system/intents");
        if !system_intents_path.exists() {
            self.create_default_system_intents(&system_intents_path)?;
        }
        
        self.load_intents_from_directory(&system_intents_path, &mut loaded_intents)?;
        
        // Load user intents
        let user_intents_path = self.base_path.join("user/intents");
        if user_intents_path.exists() {
            self.load_intents_from_directory(&user_intents_path, &mut loaded_intents)?;
        }
        
        Ok(loaded_intents)
    }
    
    fn load_intents_from_directory(
        &self, 
        dir_path: &PathBuf, 
        intents: &mut HashMap<String, Intent>
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?
        {
            let entry = entry.map_err(|e| format!("Invalid directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "msh") {
                match self.load_intent_file(&path) {
                    Ok(new_intents) => {
                        intents.extend(new_intents);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load intent file {}: {}", path.display(), e);
                    }
                }
            }
        }
        Ok(())
    }
    
    fn load_intent_file(&self, file_path: &PathBuf) -> Result<HashMap<String, Intent>, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read intent file: {}", e))?;
        
        self.parse_intent_file(&content, file_path)
    }
    
    fn parse_intent_definition(&self, name: &str, definition: &str) -> Result<Intent, String> {
        // Handle quoted content properly
        let clean_definition = if definition.contains('\"') {
            // Extract content within quotes
            let mut in_quotes = false;
            let mut clean = String::new();
            
            for ch in definition.chars() {
                if ch == '"' {
                    in_quotes = !in_quotes;
                } else if !in_quotes && ch.is_whitespace() {
                    clean.push(' ');
                } else {
                    clean.push(ch);
                }
            }
            clean
        } else {
            definition.to_string()
        };
        
        match parse_to_intent(&clean_definition) {
            Ok(mut intent) => {
                Ok(intent.mark_as_composition(name).with_source("file"))
            }
            Err(e) => {
                // Fallback for complex expressions
                Ok(Intent::new(Verb::Set)
                    .mark_as_composition(name)
                    .with_source("file")
                    .with_target(Target::Expression(definition.to_string())))
            }
        }
    }
    
    fn parse_set_intent_definition(&self, name: &str, body: &str) -> Result<Intent, String> {
        // Parse patterns like: {var1} = {var2}
        let mut intent = Intent::new(Verb::Set)
            .mark_as_composition(name)
            .with_source("file");
            
        // Simple parsing - you can enhance this
        if body.contains('=') {
            let parts: Vec<&str> = body.splitn(2, '=').collect();
            if parts.len() == 2 {
                let var_part = parts[0].trim().trim_matches('{').trim_matches('}');
                let value_part = parts[1].trim();
                
                intent = intent.with_target(Target::Variable(var_part.to_string()))
                    .with_parameter("value", value_part);
            }
        }
        
        Ok(intent)
    }
    
    fn parse_ensure_intent_definition(&self, name: &str, body: &str) -> Result<Intent, String> {
        // Parse ensure patterns
        let mut intent = Intent::new(Verb::Ensure)
            .mark_as_composition(name)
            .with_source("file");
            
        // Add parsing logic based on the body content
        if body.starts_with("port ") {
            let port_str = body.trim_start_matches("port ").trim();
            if let Ok(port) = port_str.parse::<u16>() {
                intent = intent.with_target(Target::Port(port))
                    .with_parameter("state", "open");
            }
        }
        
        Ok(intent)
    }

    fn create_default_system_intents(&self, path: &PathBuf) -> Result<(), String> {
        // Create directory
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create system intents directory: {}", e))?;
        
        // Create core.msh with essential intents
        let core_intents = r#"# Core Morris system intents - Bootstrapping foundation
set = set {variable} = {value}
ensure = ensure {condition}
derive = derive {variable}
analyze = analyze {target}
writeout = writeout {content}
find = find {pattern}
freeze = freeze {variable}

# File operations  
save = save {path}
load = load {path}
read = read {path} into {variable}
write = write {path} {content}
append = append {path} {content}
mkdir = mkdir {path}
list = list {path}
info = info {path}
exists = exists {path}

# Book navigation
page = page
turn = turn {destination}
bookmark = bookmark add {name} {path}
bookmarks = bookmarks
volume = volume add {name} {path} {description}
volumes = volumes
shelve = shelve
unshelve = unshelve
annotate = annotate {target} {note}
read_annotation = read_annotation {target}
index = index
back = back {steps}
library = library

# Change engine operations
engine = engine status
engine_status = engine status
engine_save = engine save
engine_load = engine load
engine_validate = engine validate

# Transaction system
craft = craft {name}
forge = forge
smelt = smelt
temper = temper
inspect = inspect
anneal = anneal {steps}
quench = quench
transaction = transaction
what_if = what-if {scenario}"#;

        let core_path = path.join("core.msh");
        fs::write(&core_path, core_intents)
            .map_err(|e| format!("Failed to create core.msh: {}", e))?;
        
        // Create navigation.msh  
        let navigation_intents = r#"# Navigation enhancements
jump = jump {target}
goto = goto {target}
peek = peek {distance}
return = return {steps}
mark = mark {name} {description}
chapter = chapter {path}
skim = skim {file}

# History operations
history = history
history_search = history search {query}
history_tag = history tag {name}
history_replay = history replay {id}
history_clear = history clear
history_save = history save"#;

        let nav_path = path.join("navigation.msh");
        fs::write(&nav_path, navigation_intents)
            .map_err(|e| format!("Failed to create navigation.msh: {}", e))?;

        // Create advanced.msh
        let advanced_intents = r#"# Advanced operations  
collection = collection {name} with {items}
dictionary = dictionary {name} {content}
parse_json = parse-json {json_string}
to_json = to-json {variable}
from_json = from-json {json} into {variable}
json_get = json-get {variable}.{path}
json_set = json-set {variable}.{path} = {value}

# Meta-programming
examine = examine {target}
construct = construct intent {name} with {params} {expression}
evolve = evolve {intent_name} {action} {params}
grow = grow {new_intent} from {base_intent}
reflect = reflect {expression}
test = test {intent} with {params}
adopt = adopt {intent_name}"#;

        let advanced_path = path.join("advanced.msh");
        fs::write(&advanced_path, advanced_intents)
            .map_err(|e| format!("Failed to create advanced.msh: {}", e))?;

        Ok(())
    }
}

pub struct LibraryState {
    pub system_intents: HashMap<String, crate::core::intent::Intent>,
    pub user_intents: HashMap<String, crate::core::intent::Intent>,
    pub quarantined_files: Vec<PathBuf>,
    pub validation_errors: Vec<String>,
}

impl LibraryState {
    pub fn new() -> Self {
        Self {
            system_intents: HashMap::new(),
            user_intents: HashMap::new(),
            quarantined_files: Vec::new(),
            validation_errors: Vec::new(),
        }
    }
    
    pub fn add_system_intents(&mut self, intents: Vec<Intent>) {
        for intent in intents {
            self.system_intents.insert(
                intent.get_name().unwrap_or("unknown".to_string()),
                intent
            );
        }
    }
}

pub struct IntegrityStore {
    storage_path: PathBuf,
    system_hashes: HashMap<String, String>, // file_path -> expected_hash
}

impl IntegrityStore {
    pub fn new(storage_path: PathBuf) -> Result<Self, String> {
        let mut store = Self {
            storage_path: storage_path.clone(),
            system_hashes: HashMap::new(),
        };
        
        // Load or initialize system integrity database
        store.load_system_hashes()?;
        
        Ok(store)
    }
    
    pub fn get_system_hash(&self, file_path: &Path) -> Result<String, String> {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file path")?;
            
        self.system_hashes.get(file_name)
            .cloned()
            .ok_or_else(|| format!("No integrity record for {}", file_name))
    }
    
    fn load_system_hashes(&mut self) -> Result<(), String> {
        let integrity_file = self.storage_path.join("system_hashes.json");
        
        if !integrity_file.exists() {
            // First run - initialize with current system files
            return self.initialize_system_hashes();
        }
        
        let content = fs::read_to_string(&integrity_file)
            .map_err(|e| format!("Cannot read integrity file: {}", e))?;
        
        self.system_hashes = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid integrity file: {}", e))?;
            
        Ok(())
    }
    
    fn initialize_system_hashes(&mut self) -> Result<(), String> {
        // This would hash all system files and create the initial integrity database
        // For now, create empty database
        self.save_integrity_database()?;
        Ok(())
    }
    
    fn save_integrity_database(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.system_hashes)
            .map_err(|e| format!("Cannot serialize integrity data: {}", e))?;
            
        fs::write(&self.storage_path.join("system_hashes.json"), &content)
            .map_err(|e| format!("Cannot save integrity database: {}", e))?;
            
        Ok(())
    }
}

fn create_fallback_intent(name: &str, definition: &str) -> Intent {
    Intent::new(Verb::Set)
        .mark_as_composition(name)
        .with_source("file")
        .with_target(Target::Expression(definition.to_string()))
}

impl Intent {
    pub fn to_string(&self) -> String {
        format!("{:?} -> {}", self.verb, self.target_string())
    }
}