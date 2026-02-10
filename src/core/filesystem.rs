use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::{DateTime, Utc, serde::ts_seconds};
use serde::{Serialize, Deserialize};

use crate::core::env::Env;
use crate::core::types::{Value, VariableSource};

// Environment file format
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentFile {
    pub version: String,
    pub morris_version: String,
    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,
    pub variables: HashMap<String, SavedVariable>,
    pub expressions: HashMap<String, String>,
    pub dependencies: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedVariable {
    pub value: Value,
    pub is_constant: bool,
    pub source: VariableSource,
}

pub struct FileSystem;

impl FileSystem {
    pub fn new() -> Self {
        FileSystem
    }
    
    // Save entire environment to file
    pub fn save_env(&self, env: &Env, path: &str) -> Result<String, String> {
        let file = self.create_env_file(env);
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }
        
        // Write with atomic operation
        let temp_path = format!("{}.tmp", path);
        fs::write(&temp_path, &json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        // Atomic rename
        fs::rename(&temp_path, path)
            .map_err(|e| format!("Failed to finalize save: {}", e))?;
        
        let var_count = file.variables.len();
        Ok(format!("Saved {} variables to '{}'", var_count, path))
    }
    
    // Load environment from file
    #[allow(dead_code)]
    pub fn load_env(&self, path: &str, env: &mut Env) -> Result<String, String> {
        // Read file
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
        
        // Deserialize
        let file: EnvironmentFile = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid environment file: {}", e))?;
        
        // Verify version
        if file.version != "1.0" {
            return Err(format!("Unsupported version: {}", file.version));
        }
        
        // Clear current environment
        *env = Env::new();
        
        // Restore variables
        for (name, saved_var) in &file.variables {
            // For now, just set as direct variables
            // TODO: Restore computed variables with expressions
            env.set_direct(name, saved_var.value.clone());
            
            // Restore frozen state
            if saved_var.is_constant {
                env.freeze(name).ok(); // Ignore errors for now
            }
        }
        
        let var_count = file.variables.len();
        Ok(format!("Loaded {} variables from '{}'", var_count, path))
    }
    
    // Read file contents
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path, e))
    }
    
    // Write content to file
    pub fn write_file(&self, path: &str, content: &str) -> Result<String, String> {
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }
        
        // Atomic write
        let temp_path = format!("{}.tmp", path);
        fs::write(&temp_path, content)
            .map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
        
        fs::rename(&temp_path, path)
            .map_err(|e| format!("Failed to finalize write: {}", e))?;
        
        Ok(format!("Wrote {} bytes to '{}'", content.len(), path))
    }
    
    // Append to file
    pub fn append_file(&self, path: &str, content: &str) -> Result<String, String> {
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("Failed to open file '{}': {}", path, e))?;
        
        use io::Write;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to append to file '{}': {}", path, e))?;
        
        Ok(format!("Appended {} bytes to '{}'", content.len(), path))
    }
    
    // Check if file exists
    pub fn file_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }
    
    // Create directory
    pub fn mkdir(&self, path: &str) -> Result<String, String> {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory '{}': {}", path, e))?;
        
        Ok(format!("Created directory: '{}'", path))
    }
    
    // List files in directory
    pub fn list_files(&self, path: &str) -> Result<Vec<String>, String> {
        let dir = Path::new(path);
        if !dir.exists() {
            return Err(format!("Directory '{}' does not exist", path));
        }
        
        if !dir.is_dir() {
            return Err(format!("'{}' is not a directory", path));
        }
        
        let mut files = Vec::new();
        
        for entry in fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory '{}': {}", path, e))?
        {
            let entry = entry
                .map_err(|e| format!("Failed to read directory entry: {}", e))?;
            
            let file_name = entry.file_name().to_string_lossy().to_string();
            files.push(file_name);
        }
        
        Ok(files)
    }
    
    // Get file information
    pub fn file_info(&self, path: &str) -> Result<FileInfo, String> {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to get file info for '{}': {}", path, e))?;
        
        let path_buf = PathBuf::from(path);
        let name = path_buf.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        
        let extension = path_buf.extension()
            .map(|e| e.to_string_lossy().to_string());
        
        let file_type = if metadata.is_dir() {
            "directory"
        } else if metadata.is_file() {
            "file"
        } else {
            "other"
        }.to_string();
        
        let modified = metadata.modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok()
            })
            .map(|d| d.as_secs());
        
        Ok(FileInfo {
            name,
            path: path.to_string(),
            size: metadata.len(),
            file_type,
            extension,
            modified,
        })
    }
    
    // Helper to create environment file structure
    fn create_env_file(&self, env: &Env) -> EnvironmentFile {
        let mut variables = HashMap::new();
        let mut expressions = HashMap::new();
        
        // Extract variables
        for (name, _value) in env.list() {
            if let Some(var) = env.get_variable(&name) {
                let saved_var = SavedVariable {
                    value: var.value.clone(),
                    is_constant: var.is_constant,
                    source: var.source.clone(),
                };
                variables.insert(name.clone(), saved_var);
                
                // Store expression if available
                if let Some(expr_str) = &var.expression {
                    expressions.insert(name.clone(), expr_str.clone());
                }
            }
        }
        
        // TODO: Extract dependency graph properly
        let dependencies = HashMap::new();
        
        EnvironmentFile {
            version: "1.0".to_string(),
            morris_version: "0.5.0".to_string(),
            created: Utc::now(),
            variables,
            expressions,
            dependencies,
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub file_type: String,
    pub extension: Option<String>,
    pub modified: Option<u64>,
}

impl FileInfo {
    #[allow(dead_code)] 
    pub fn display(&self) -> String {
    let size_str = if self.size < 1024 {
        format!("{} B", self.size)
    } else if self.size < 1024 * 1024 {
        format!("{:.1} KB", self.size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
    };
    
    let modified_str = if let Some(ts) = self.modified {
        let dt = chrono::DateTime::from_timestamp(ts as i64, 0);
        dt.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "unknown".to_string()
    };
    
    let type_str = if !self.file_type.is_empty() {
        format!("[{}] ", self.file_type)
    } else {
        String::new()
    };
    
    let ext_str = self.extension
        .as_ref()
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default();
    
    format!("{}{}{}: {} - modified: {} ({})", 
        type_str, self.name, ext_str, size_str, modified_str, self.path)
    }
}
