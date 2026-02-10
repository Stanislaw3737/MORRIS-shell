use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::path::PathBuf;
use dirs;

use crate::output::Printer;

pub struct Repl {
    editor: DefaultEditor,
    history_file: PathBuf,
    printer: Printer,  // Make this owned, not referenced
}

impl Repl {
    pub fn new() -> Result<Self, String> {
        let mut editor = DefaultEditor::new()
            .map_err(|e| format!("Failed to initialize line editor: {}", e))?;
        
        // Create .morris directory if it doesn't exist
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let morris_dir = home.join(".morris");
        if !morris_dir.exists() {
            std::fs::create_dir_all(&morris_dir)
                .map_err(|e| format!("Failed to create .morris directory: {}", e))?;
        }
        
        let history_file = morris_dir.join("repl_history.txt");
        
        // Load REPL command history
        if history_file.exists() {
            editor.load_history(&history_file).ok();
        }
        
        Ok(Self {
            editor,
            history_file,
            printer: Printer::new(),  // Create a new Printer instance
        })
    }
    
    

    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>, ReadlineError> {
        let mut input_lines = Vec::new();
        let mut in_multiline = false;
        
        loop {
            let current_prompt = if in_multiline { "... " } else { prompt };
            
            match self.editor.readline(current_prompt) {
                Ok(line) => {
                    let line_trimmed = line.trim_end();
                    
                    // Check if we're entering multi-line mode
                    if !in_multiline && self.should_enter_multiline(&line) {
                        in_multiline = true;
                        input_lines.push(line);
                        continue;
                    }
                    
                    // Check for multi-line end with ;;
                    if in_multiline && line_trimmed == ";;" {
                        // Remove the ;; line and process the multi-line input
                        let full_input = input_lines.join("\n");
                        if !full_input.trim().is_empty() {
                            self.editor.add_history_entry(&full_input)?;
                            return Ok(Some(full_input));
                        }
                        return Ok(None);
                    }
                    
                    // Continue collecting multi-line input
                    if in_multiline {
                        input_lines.push(line);
                        continue;
                    }
                    
                    // Single line command
                    if !line_trimmed.is_empty() {
                        self.editor.add_history_entry(&line)?;
                        return Ok(Some(line));
                    }
                    return Ok(None);
                }
                Err(ReadlineError::Interrupted) => {
                    if in_multiline {
                        println!("^C - Cancelled multi-line input");
                        in_multiline = false;
                        input_lines.clear();
                        continue;
                    } else {
                        println!("^C");
                        return Ok(None);
                    }
                }
                Err(ReadlineError::Eof) => {
                    if in_multiline {
                        // Process what we have so far
                        let full_input = input_lines.join("\n");
                        if !full_input.trim().is_empty() {
                            self.editor.add_history_entry(&full_input)?;
                            return Ok(Some(full_input));
                        }
                    }
                    println!("exit");
                    return Ok(None);
                }
                Err(err) => return Err(err),
            }
        }
    }

    fn should_enter_multiline(&self, line: &str) -> bool {
        let line = line.trim();
        
        // Existing conditions
        line.ends_with('{') || 
        line.starts_with("define intent") ||
        line.starts_with("define ") && line.contains('{') ||
        (line.starts_with('{') && line.contains(":")) ||
        
        // NEW: Enhanced match detection
        line.starts_with("match ") ||
        
        // NEW: Enhanced conditional detection
        (line.contains('|') && line.contains("when")) ||
        line.trim_end().ends_with("when") ||
        line.trim_end().ends_with("|") ||
        
        // NEW: Likely continuation patterns
        line.trim_end().ends_with("and") ||
        line.trim_end().ends_with("or") ||
        (line.contains(" | ") && !line.contains(";;"))
    }

    fn should_exit_multiline(&self, line: &str) -> bool {
        let line = line.trim();
        line == "}" || 
        (line.ends_with('}') && !line.contains('{')) || // JSON object end
        (line.starts_with('}') && line.len() <= 3)
    }
    pub fn save_history(&mut self) -> Result<(), String> {
        self.editor.save_history(&self.history_file)
            .map_err(|e| format!("Failed to save REPL history: {}", e))
    }
    
    // Return a reference to printer
    pub fn printer(&self) -> &Printer {
        &self.printer
    }
    
    // Return a mutable reference to printer
    #[allow(dead_code)]
    pub fn printer_mut(&mut self) -> &mut Printer {
        &mut self.printer
    }
}