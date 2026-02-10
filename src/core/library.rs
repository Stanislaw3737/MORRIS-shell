use std::collections::{HashMap, VecDeque};
use std::env;
use std::path::{Path, PathBuf};
use shellexpand;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
    #[allow(dead_code)]
    pub created: chrono::DateTime<chrono::Utc>,
    pub annotation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Volume {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
}

pub struct Library {
    current_page: PathBuf,
    bookmarks: HashMap<String, Bookmark>,
    volumes: HashMap<String, Volume>,
    annotations: HashMap<PathBuf, String>,
    history: VecDeque<PathBuf>,
    shelf: Option<PathBuf>,
    max_history: usize,
}

impl Library {
    pub fn new() -> Self {
        let current_page = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        
        Self {
            current_page,
            bookmarks: HashMap::new(),
            volumes: HashMap::new(),
            annotations: HashMap::new(),
            history: VecDeque::new(),
            shelf: None,
            max_history: 100,
        }
    }
    
    pub fn page(&self) -> &Path {
        &self.current_page
    }
    
    pub fn turn(&mut self, destination: &str) -> Result<String, String> {
        // Save current page to history
        if !self.history.is_empty() && 
           self.history.back() != Some(&self.current_page) {
            self.history.push_back(self.current_page.clone());
            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
        } else if self.history.is_empty() {
            self.history.push_back(self.current_page.clone());
        }
        
        // Expand and validate path
        let new_page = self.expand_path(destination)?;
        
        // Change directory
        env::set_current_dir(&new_page)
            .map_err(|e| format!("Cannot turn page: {}", e))?;
        
        let _old_page = self.current_page.clone();
        self.current_page = new_page.canonicalize()
            .map_err(|e| format!("Cannot resolve path: {}", e))?;
        
        let page_name = self.get_page_name(&self.current_page);
        Ok(format!("Turned to page: {}", page_name))
    }
    
    pub fn bookmark(&mut self, name: &str, path: Option<&str>) -> Result<String, String> {
        let target_path = match path {
            Some(p) => self.expand_path(p)?,
            None => self.current_page.clone(),
        };
        
        let bookmark = Bookmark {
            name: name.to_string(),
            path: target_path.clone(),
            created: chrono::Utc::now(),
            annotation: None,
        };
        
        self.bookmarks.insert(name.to_string(), bookmark);
        
        let page_name = self.get_page_name(&target_path);
        Ok(format!("📑 Bookmarked '{}' → {}", name, page_name))
    }
    
    pub fn remove_bookmark(&mut self, name: &str) -> Result<String, String> {
        if self.bookmarks.remove(name).is_some() {
            Ok(format!("Removed bookmark '{}'", name))
        } else {
            Err(format!("Bookmark '{}' not found", name))
        }
    }
    #[allow(dead_code)]
    pub fn open_bookmark(&mut self, name: &str) -> Result<String, String> {
        // Clone the path first to avoid borrowing issues
        let bookmark_path = self.bookmarks.get(name)
            .ok_or_else(|| format!("Bookmark '{}' not found", name))?
            .path
            .clone();
    
        let _result = self.turn(bookmark_path.to_str().unwrap())?;
        let page_name = self.get_page_name(&bookmark_path);
        Ok(format!("📖 Opened bookmark '{}' → {}", name, page_name))
    }

    
    pub fn list_bookmarks(&self) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<&Bookmark> = self.bookmarks.values().collect();
        bookmarks.sort_by_key(|b| &b.name);
        bookmarks
    }
    
    pub fn volume(&mut self, name: &str, path: &str, description: Option<&str>) -> Result<String, String> {
        let expanded = self.expand_path(path)?;
        
        let volume = Volume {
            name: name.to_string(),
            path: expanded.clone(),
            description: description.map(|s| s.to_string()),
        };
        
        self.volumes.insert(name.to_string(), volume);
        
        let page_name = self.get_page_name(&expanded);
        Ok(format!("📚 Mounted volume '{}' → {}", name, page_name))
    }
    
    pub fn list_volumes(&self) -> Vec<&Volume> {
        let mut volumes: Vec<&Volume> = self.volumes.values().collect();
        volumes.sort_by_key(|v| &v.name);
        volumes
    }
    
    pub fn shelve(&mut self) -> String {
        self.shelf = Some(self.current_page.clone());
        let page_name = self.get_page_name(&self.current_page);
        format!("📚 Shelved current page: {}", page_name)
    }
    
    pub fn unshelve(&mut self) -> Result<String, String> {
        let shelved_path = self.shelf.clone()
            .ok_or("Nothing shelved".to_string())?;
    
        let _result = self.turn(shelved_path.to_str().unwrap())?;
        let page_name = self.get_page_name(&shelved_path);
        Ok(format!("📚 Retrieved shelved page: {}", page_name))
    }
    
    pub fn annotate(&mut self, target: &str, note: &str) -> Result<String, String> {
        let path = if target == "." {
            self.current_page.clone()
        } else {
            self.expand_path(target)?
        };
        
        self.annotations.insert(path.clone(), note.to_string());
        
        let target_name = if path == self.current_page {
            "current page".to_string()
        } else {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };
        
        Ok(format!("📝 Annotated '{}': {}", target_name, note))
    }
    
    pub fn get_annotation(&self, target: &str) -> Option<String> {
        let path = if target == "." {
            self.current_page.clone()
        } else {
            match self.expand_path(target) {
                Ok(p) => p,
                Err(_) => return None,
            }
        };
    
        self.annotations.get(&path).cloned()
    }
    
    pub fn back(&mut self, steps: usize) -> Result<String, String> {
        if steps == 0 || steps > self.history.len() {
            return Err("Invalid number of steps".to_string());
        }
        
        for _ in 0..steps {
            if let Some(previous) = self.history.pop_back() {
                let _result = self.turn(previous.to_str().unwrap())?;
            }
        }
        
        let page_name = self.get_page_name(&self.current_page);
        Ok(format!("↩ Returned {} page(s) to: {}", steps, page_name))
    }
    
    pub fn index(&self) -> Result<Vec<String>, String> {
        let entries: Vec<String> = std::fs::read_dir(&self.current_page)
            .map_err(|e| format!("Cannot read directory: {}", e))?
            .filter_map(|entry| {
                entry.ok().map(|e| {
                    let path = e.path();
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string();
                    
                    let metadata = e.metadata().ok();
                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    
                    let icon = if is_dir { "📁" } else { "📄" };
                    let dir_marker = if is_dir { "/" } else { "" };
                    
                    format!("{} {}{} ({})", icon, name, dir_marker, Self::format_size(size))
                })
            })
            .collect();
        
        Ok(entries)
    }
    pub fn go_back(&mut self, steps: usize) -> Result<String, String> {
        if steps == 0 || steps > self.history.len() {
            return Err("Invalid number of steps".to_string());
        }
        
        for _ in 0..steps {
            if let Some(previous) = self.history.pop_back() {
                let _ = self.turn_internal(&previous)?;
            }
        }
        
        let page_name = self.get_page_name(&self.current_page);
        Ok(format!("↩ Went back {} page(s) to: {}", steps, page_name))
    }
    
    pub fn go_forward(&mut self, _steps: usize) -> Result<String, String> {
        // For forward navigation, we'd need a forward history stack
        // For now, we can implement if needed
        Ok("[?] Forward navigation not yet implemented".to_string())
    }
    
    pub fn jump_to(&mut self, destination: &str) -> Result<String, String> {
        // Support relative jumps like "-1", "-2", "+1", etc.
        if destination.starts_with('-') {
            if let Ok(steps) = destination[1..].parse::<usize>() {
                return self.go_back(steps);
            }
        } else if destination.starts_with('+') {
            if let Ok(steps) = destination[1..].parse::<usize>() {
                return self.go_forward(steps);
            }
        }
        
        // Regular turn
        self.turn(destination)
    }
    
    // Helper method for internal turn without adding to history
    fn turn_internal(&mut self, path: &Path) -> Result<(), String> {
        env::set_current_dir(path)
            .map_err(|e| format!("Cannot change directory: {}", e))?;
        
        self.current_page = path.canonicalize()
            .map_err(|e| format!("Cannot resolve path: {}", e))?;
        
        Ok(())
    }
    
    pub fn peek(&self, distance: isize) -> Option<String> {
        // Peek ahead or behind in history
        let history_len = self.history.len();
        if distance < 0 {
            let abs_dist = (-distance) as usize;
            if abs_dist <= history_len {
                let idx = history_len - abs_dist;
                return self.history.get(idx)
                    .map(|p| self.get_page_name(p));
            }
        }
        None
    }
    
    fn expand_path(&self, path: &str) -> Result<PathBuf, String> {
        // Handle special cases
        match path {
            "." => Ok(self.current_page.clone()),
            ".." => Ok(self.current_page.parent()
                .unwrap_or(&self.current_page)
                .to_path_buf()),
            "~" => dirs::home_dir()
                .ok_or_else(|| "No home directory".to_string()),
            _ => {
                // Check if it's a bookmark
                if let Some(bookmark) = self.bookmarks.get(path) {
                    return Ok(bookmark.path.clone());
                }
                
                // Check if it's a volume
                if let Some(volume) = self.volumes.get(path) {
                    return Ok(volume.path.clone());
                }
                
                // Regular path
                let expanded = shellexpand::full(path)
                    .map_err(|e| format!("Invalid path: {}", e))?;
                
                let full_path = if Path::new(&*expanded).is_absolute() {
                    PathBuf::from(&*expanded)
                } else {
                    self.current_page.join(&*expanded)
                };
                
                // Check if path exists
                if !full_path.exists() {
                    return Err(format!("Page not found: {}", full_path.display()));
                }
                
                Ok(full_path)
            }
        }
    }
    
    fn get_page_name(&self, path: &Path) -> String {
        // Try to get a friendly name
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.is_empty() || name_str == "/" {
                    return "Root Volume".to_string();
                }
                return name_str.to_string();
            }
        }
        
        // Fall back to full path
        path.display().to_string()
    }
    
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        
        if size >= GB {
            format!("{:.1} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}