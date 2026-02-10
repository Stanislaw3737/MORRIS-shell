mod core;

use std::io::{self, Write};
use crate::core::env::Env;
use crate::core::intent::{parse_to_intent, Verb, Target};

fn main() -> io::Result<()> {
    println!("Morris v0.3 - Testing...");
    
    let mut env = Env::new();
    
    // Simple test
    println!("Testing intent parsing...");
    
    let test_commands = [
        "set x = 5",
        "ensure x = 10",
        "writeout(test)",
        "derive x",
    ];
    
    for cmd in test_commands {
        match parse_to_intent(cmd) {
            Ok(intent) => {
                println!("✅ Parsed '{}' as {:?}", cmd, intent.verb);
            }
            Err(e) => {
                println!("❌ Failed to parse '{}': {}", cmd, e);
            }
        }
    }
    
    println!("\nTest complete!");
    Ok(())
}
END_OF_FILE