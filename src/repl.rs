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
        let mut input_lines = Vec::<String>::new();
        let mut in_multiline = false;
        let mut accumulated_statement = String::new();
        
        loop {
            let current_prompt = if in_multiline { "... " } else { prompt };
            
            match self.editor.readline(current_prompt) {
                Ok(line) => {
                    let line_trimmed = line.trim();
                    
                    // Handle Ctrl+C cancellation
                    if line_trimmed.is_empty() && in_multiline {
                        println!("^C - Cancelled multi-line input");
                        in_multiline = false;
                        accumulated_statement.clear();
                        continue;
                    }
                    
                    // Check for semicolon termination (complete statement)
                    if line_trimmed.ends_with(';') {
                        let statement = if accumulated_statement.is_empty() {
                            // Single line statement
                            line_trimmed[..line_trimmed.len()-1].to_string() // Remove semicolon
                        } else {
                            // Complete the multiline statement
                            accumulated_statement.push_str(&line);
                            let full_statement = accumulated_statement.trim();
                            let result = full_statement[..full_statement.len()-1].to_string(); // Remove semicolon
                            accumulated_statement.clear();
                            result
                        };
                        
                        if !statement.trim().is_empty() {
                            self.editor.add_history_entry(statement.trim())?;
                            return Ok(Some(statement.trim().to_string()));
                        }
                        return Ok(None);
                    }
                    
                    // Check if we should enter multiline mode
                    if !in_multiline && (self.should_enter_multiline(&line) || !accumulated_statement.is_empty()) {
                        in_multiline = true;
                        accumulated_statement.push_str(&line);
                        accumulated_statement.push('\n');
                        continue;
                    }
                    
                    // In multiline mode, accumulate
                    if in_multiline {
                        accumulated_statement.push_str(&line);
                        accumulated_statement.push('\n');
                        continue;
                    }
                    
                    // Single line commands (system commands, etc.)
                    // These don't require semicolons for backward compatibility
                    if !line_trimmed.is_empty() {
                        self.editor.add_history_entry(line_trimmed)?;
                        return Ok(Some(line_trimmed.to_string()));
                    }
                    
                    return Ok(None);
                }
                Err(ReadlineError::Interrupted) => {
                    if in_multiline || !accumulated_statement.is_empty() {
                        println!("^C - Cancelled multi-line input");
                        in_multiline = false;
                        accumulated_statement.clear();
                        continue;
                    } else {
                        println!("^C");
                        return Ok(None);
                    }
                }
                Err(ReadlineError::Eof) => {
                    if in_multiline || !accumulated_statement.is_empty() {
                        // Process what we have
                        if !accumulated_statement.is_empty() {
                            self.editor.add_history_entry(accumulated_statement.as_str())?;
                            let result = accumulated_statement.clone();
                            accumulated_statement.clear();
                            return Ok(Some(result));
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
        
        // Don't enter multiline for system commands
        match line {
            "env" | "history" | "clear" => return false,
            _ => {}
        }
        
        // Enter multiline for complex expressions
        line.ends_with('{') || 
        line.starts_with("define intent") ||
        (line.contains('|') && (line.contains("when") || line.contains("otherwise"))) ||
        line.trim_end().ends_with("when") ||
        line.trim_end().ends_with("|") ||
        line.starts_with("match ")
    }

    fn should_exit_multiline(&self, line: &str) -> bool {
        let line = line.trim();
        // Exit multiline when we see a semicolon
        line.ends_with(';')
    }

    fn is_statement_complete(line: &str) -> bool {
        line.trim_end().ends_with(';')
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