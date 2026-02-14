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
        
        
        
        let dependents = env.get_dependents(&var_name);
        
        
        for dependent in &dependents {
            if processed.contains(dependent) {
                
                continue;
            }
            
            // Check propagation control - ONLY call should_propagate once per dependent
            if let Some(var) = env.get_variable_mut(dependent) {
                
                if !var.should_propagate() {
                    
                    processed.insert(dependent.clone());
                    continue;
                }
                
                // If propagation is allowed, proceed with evaluation and update
                if let Some(expr) = env.get_expression(dependent) {
                    
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
                                            
                                        }
                                        
                                    }
                                    Err(e) => {
                                        
                                    }
                                }
                            } else {
                                
                            }
                        }
                        Err(e) => {
                            
                        }
                    }
                }
            }
        }
    }
    
    Ok(updated)
}