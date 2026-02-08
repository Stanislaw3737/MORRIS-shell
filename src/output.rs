//use std::io;

pub struct Printer {
    pub use_color: bool,
}

impl Printer {
    pub fn new() -> Self {
        // Simple color detection
        #[cfg(windows)]
        let use_color = false; // Windows terminal color support is complex
        
        #[cfg(not(windows))]
        let use_color = std::env::var("TERM")
            .map(|term| term != "dumb")
            .unwrap_or(false);
        
        Self { use_color }
        
    }
    
    pub fn success(&self, message: &str) {
        self.print_prefix("[+]", "green", message);
    }
    
    pub fn error(&self, message: &str) {
        self.print_prefix("[-]", "red", message);
    }
    
    pub fn warning(&self, message: &str) {
        self.print_prefix("[!]", "yellow", message);
    }
    
    pub fn info(&self, message: &str) {
        self.print_prefix("[?]", "cyan", message);
    }
    #[allow(dead_code)]
    pub fn neutral(&self, message: &str) {
        self.print_prefix("[•]", "blue", message);
    }
    
    pub fn header(&self, title: &str) {
        if self.use_color {
            println!("\n\x1b[1;36m{}\x1b[0m", title);  // Bold cyan
            println!("\x1b[90m{}\x1b[0m", "─".repeat(title.len()));  // Dark gray line
        } else {
            println!("\n{}", title);
            println!("{}", "─".repeat(title.len()));
        }
    }
    
    pub fn subheader(&self, title: &str) {
        println!();
        if self.use_color {
            println!("\n\x1b[1m{}\x1b[0m", title);  // Bold
        } else {
            println!("\n{}", title);
        }
    }
    
    pub fn print_prefix(&self, prefix: &str, color: &str, message: &str) {
        println!();
        if self.use_color {
            let color_code = match color {
                "green" => "\x1b[32m",
                "red" => "\x1b[31m",
                "yellow" => "\x1b[33m",
                "cyan" => "\x1b[36m",
                "blue" => "\x1b[34m",
                "magenta" => "\x1b[35m",
                _ => "\x1b[0m",
            };
            println!("{}{}\x1b[0m {}", color_code, prefix, message);
        } else {
            println!("{} {}", prefix, message);
        }
    }
    
    pub fn print_key_value(&self, key: &str, value: &str, indent: usize) {
        let indent_str = " ".repeat(indent);
        if self.use_color {
            println!("{}\x1b[1m{}:\x1b[0m {}", indent_str, key, value);  // Bold key
        } else {
            println!("{}{}: {}", indent_str, key, value);
        }
    }
    #[allow(dead_code)]
    pub fn print_list_item(&self, item: &str, indent: usize) {
        let indent_str = " ".repeat(indent);
        if self.use_color {
            println!("{}\x1b[36m•\x1b[0m {}", indent_str, item);  // Cyan bullet
        } else {
            println!("{}• {}", indent_str, item);
        }
    }
    #[allow(dead_code)]
    pub fn print_indented(&self, text: &str, indent: usize) {
        let indent_str = " ".repeat(indent);
        for line in text.lines() {
            println!("{}{}", indent_str, line);
        }
    }
    #[allow(dead_code)]
    pub fn separator(&self) {
        if self.use_color {
            println!("\x1b[90m{}\x1b[0m", "─".repeat(60));
        } else {
            println!("{}", "─".repeat(60));
        }
    }
}