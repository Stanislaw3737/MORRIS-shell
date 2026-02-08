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
        match self.editor.readline(prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    self.editor.add_history_entry(&line)?;
                }
                Ok(Some(line))
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - clear line and continue
                println!("^C");
                Ok(None)
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - exit
                println!("exit");
                Ok(None)
            }
            Err(err) => Err(err),
        }
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