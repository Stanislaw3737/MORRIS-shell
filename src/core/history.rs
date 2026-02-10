
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, serde::ts_seconds};
use uuid::Uuid;
use dirs;

use crate::core::intent::{Intent, IntentState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: Uuid,
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub intent_string: String,
    pub verb: String,
    pub target: Option<String>,
    pub state: String,
    pub result: Option<String>,
    pub duration_ms: u64,
    pub context: HashMap<String, String>,
    pub tags: Vec<String>,
}

pub struct HistoryManager {
    pub file_path: PathBuf,
    max_entries: usize,
    entries: Vec<HistoryEntry>,
    #[allow(dead_code)]
    session_id: Uuid,
}

impl HistoryManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let morris_dir = home.join(".morris");
        
        // Create .morris directory if it doesn't exist
        if !morris_dir.exists() {
            fs::create_dir_all(&morris_dir).ok();
        }
        
        let file_path = morris_dir.join("history.json");
        
        Self {
            file_path,
            max_entries: 1000,
            entries: Vec::new(),
            session_id: Uuid::new_v4(),
        }
    }
    
    pub fn record(&mut self, intent: &Intent, result: &str, state: IntentState) {
        let end_time = Utc::now();
        let duration_ms = end_time.timestamp_millis() as u64 - 
                         intent.timestamp.timestamp_millis() as u64;
        
        let entry = HistoryEntry {
            id: intent.id,
            timestamp: intent.timestamp,
            intent_string: intent.target_string(),
            verb: format!("{:?}", intent.verb),
            target: intent.target.as_ref().map(|t| match t {
                crate::core::intent::Target::Variable(name) => Some(format!("var:{}", name)),
                crate::core::intent::Target::File(path) => Some(format!("file:{}", path)),
                crate::core::intent::Target::Expression(expr) => Some(format!("expr:{}", expr)),
                crate::core::intent::Target::Process(name) => Some(format!("process:{}", name)),
                crate::core::intent::Target::Port(port) => Some(format!("port:{}", port)),
                crate::core::intent::Target::Service(name) => Some(format!("service:{}", name)),
            }).flatten(),
            state: format!("{:?}", state),
            result: Some(result.to_string()),
            duration_ms,
            context: intent.context.clone(),
            tags: Vec::new(),
        };
        
        self.entries.push(entry);
        
        // Maintain size limit
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        
        // Auto-save periodically (every 10 entries)
        if self.entries.len() % 10 == 0 {
            let _ = self.save(); // Ignore errors for auto-save
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read history file: {}", e))?;
        
        self.entries = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid history format: {}", e))?;
        
        // Keep only most recent entries (respect max_entries)
        if self.entries.len() > self.max_entries {
            let start_idx = self.entries.len() - self.max_entries;
            self.entries = self.entries[start_idx..].to_vec();
        }
        
        Ok(())
    }
    
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;
        
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, &json)
            .map_err(|e| format!("Failed to write history: {}", e))?;
        
        fs::rename(&temp_path, &self.file_path)
            .map_err(|e| format!("Failed to finalize history save: {}", e))?;
        
        Ok(())
    }
    
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries.iter()
            .filter(|entry| 
                entry.intent_string.to_lowercase().contains(&query_lower) ||
                entry.verb.to_lowercase().contains(&query_lower) ||
                entry.result.as_ref().map_or(false, |r| 
                    r.to_lowercase().contains(&query_lower)
                )
            )
            .rev()
            .collect()
    }
    
    #[allow(dead_code)]
    pub fn filter_by_state(&self, state: &str) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|entry| entry.state == state)
            .rev()
            .collect()
    }
    
    pub fn get_last_n(&self, n: usize) -> Vec<&HistoryEntry> {
        let n = n.min(self.entries.len());
        let start_idx = self.entries.len() - n;
        self.entries[start_idx..].iter().collect()
    }
    
    pub fn get_by_id(&self, id: &Uuid) -> Option<&HistoryEntry> {
        self.entries.iter().find(|entry| &entry.id == id)
    }
    #[allow(dead_code)]
    pub fn tag_entry(&mut self, id: &Uuid, tag: &str) -> Result<(), String> {
        if let Some(entry) = self.entries.iter_mut().find(|e| &e.id == id) {
            if !entry.tags.contains(&tag.to_string()) {
                entry.tags.push(tag.to_string());
            }
            return Ok(());
        }
        Err(format!("History entry not found: {}", id))
    }
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    #[allow(dead_code)]
    pub fn stats(&self) -> HistoryStats {
        let total = self.entries.len();
        let succeeded = self.entries.iter()
            .filter(|e| e.state == "Succeeded")
            .count();
        let failed = self.entries.iter()
            .filter(|e| e.state == "Failed")
            .count();
        
        HistoryStats {
            total,
            succeeded,
            failed,
        }
    }
    #[allow(dead_code)]
    pub fn export(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| format!("Failed to serialize for export: {}", e))?;
        
        fs::write(path, json)
            .map_err(|e| format!("Failed to export history: {}", e))?;
        
        Ok(())
    }
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct HistoryStats {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}