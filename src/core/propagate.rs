// File: src/core/propagate.rs
use std::collections::HashSet;
use crate::core::env::Env;
use crate::core::expr::evaluate;

pub fn propagate_from(env: &mut Env, changed_var: &str) -> Result<Vec<String>, String> {
    let mut updated = Vec::new();
    let mut queue = vec![changed_var.to_string()];
    let mut processed = HashSet::new();
    
    while let Some(var_name) = queue.pop() {
        if processed.contains(&var_name) {
            continue;
        }
        processed.insert(var_name.clone());
        
        println!("DEBUG: Processing variable: {}", var_name); // Debug
        
        let dependents = env.get_dependents(&var_name);
        println!("DEBUG: Dependents of {}: {:?}", var_name, dependents); // Debug
        
        for dependent in &dependents {
            if processed.contains(dependent) {
                println!("DEBUG: Dependent {} already processed, skipping", dependent); // Debug
                continue;
            }
            
            // Check propagation control - ONLY call should_propagate once per dependent
            if let Some(var) = env.get_variable_mut(dependent) {
                println!("DEBUG: Checking propagation for {} before evaluation", dependent);
                if !var.should_propagate() {
                    println!("DEBUG: Propagation suppressed for {} (delay: {}/{}, limit: {}/{})", 
                        dependent, var.delay_counter, var.propagation_delay, 
                        var.limit_counter, var.propagation_limit);
                    processed.insert(dependent.clone());
                    continue;
                }
                
                // If propagation is allowed, proceed with evaluation and update
                if let Some(expr) = env.get_expression(dependent) {
                    println!("DEBUG: Evaluating expression for {}", dependent);
                    match evaluate(expr, env) {
                        Ok(new_value) => {
                            let old_value = env.get_value(dependent);
                            
                            if old_value != Some(&new_value) {
                                // Don't call should_propagate again in update_value
                                match env.update_value_without_propagation_check(dependent, new_value) {
                                    Ok(_) => {
                                        updated.push(dependent.to_string());
                                        if !processed.contains(dependent) {
                                            queue.push(dependent.to_string());
                                            println!("DEBUG: Added {} to queue", dependent);
                                        }
                                        println!("DEBUG: Propagated to {} due to {} change", dependent, var_name);
                                    }
                                    Err(e) => {
                                        println!("DEBUG: Failed to update {}: {}", dependent, e);
                                    }
                                }
                            } else {
                                println!("DEBUG: No change for {}, value unchanged", dependent);
                            }
                        }
                        Err(e) => {
                            println!("DEBUG: Failed to evaluate {}: {}", dependent, e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(updated)
}