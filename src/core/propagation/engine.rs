// File: src/core/propagation/engine.rs
use super::types::*;
use crate::core::types::Value;
use crate::core::expr::{Expr, evaluate};
use std::collections::{HashMap, VecDeque};
use chrono::Utc;

#[derive(Debug)]
pub struct PropagationEngine {
    graph: PropagationGraph,
    expressions: HashMap<String, Expr>,
    strategy: PropagationStrategy,
    propagation_history: Vec<PropagationEvent>,
    pending_changes: VecDeque<PendingChange>,
}

#[derive(Debug)]
struct PendingChange {
    variable: String,
    new_value: Value,
    timestamp: chrono::DateTime<Utc>,
}

impl PropagationEngine {
    pub fn new() -> Self {
        Self {
            graph: PropagationGraph::new(),
            expressions: HashMap::new(),
            strategy: PropagationStrategy::Immediate,
            propagation_history: Vec::new(),
            pending_changes: VecDeque::new(),
        }
    }
    
    pub fn set_strategy(&mut self, strategy: PropagationStrategy) {
        self.strategy = strategy;
    }
    
    pub fn register_computed_variable(
        &mut self,
        name: &str,
        initial_value: Value,
        expr: &Expr,
    ) -> Result<(), PropagationError> {
        self.graph.add_variable(name, initial_value.clone(), false)?;
        self.expressions.insert(name.to_string(), expr.clone());
        
        let dependencies = self.extract_dependencies(expr);
        
        for dep in dependencies {
            if !self.graph.has_variable(&dep) {
                self.graph.add_variable(&dep, Value::Str("".to_string()), false)?;
            }
            
            self.graph.add_dependency(
                &dep,
                name,
                DependencyType::Direct,
                1.0,
                None,
            )?;
        }
        
        Ok(())
    }
    
    pub fn register_direct_variable(
        &mut self,
        name: &str,
        value: Value,
        is_constant: bool,
    ) -> Result<(), PropagationError> {
        self.graph.add_variable(name, value, is_constant)
    }
    
    fn extract_dependencies(&self, expr: &Expr) -> Vec<String> {
        use crate::core::expr::extract_variables;
        extract_variables(expr)
    }
    
    pub fn set_variable(
        &mut self,
        name: &str,
        new_value: Value,
    ) -> Result<PropagationResult, PropagationError> {
        let old_value = self.graph.get_value(name)
            .cloned()
            .unwrap_or(Value::Str("".to_string()));
        
        match self.strategy {
            PropagationStrategy::Immediate => {
                self.propagate_immediate(name, old_value, new_value)
            }
            PropagationStrategy::Debounced(duration) => {
                self.queue_change(name, new_value, duration)
            }
            PropagationStrategy::Batched(batch_size) => {
                self.queue_change(name, new_value, std::time::Duration::from_millis(0));
                if self.pending_changes.len() >= batch_size {
                    self.process_batched_changes()
                } else {
                    Ok(PropagationResult {
                        changed_variables: vec![name.to_string()],
                        propagation_paths: vec![],
                        time_taken: std::time::Duration::from_secs(0),
                        conflicts_resolved: 0,
                        failed_propagations: vec![],
                    })
                }
            }
            _ => {
                self.propagate_immediate(name, old_value, new_value)
            }
        }
    }
    
    fn propagate_immediate(
        &mut self,
        name: &str,
        old_value: Value,
        new_value: Value,
    ) -> Result<PropagationResult, PropagationError> {
        let start_time = std::time::Instant::now();
        
        let affected = self.graph.update_variable(name, new_value.clone())?;
        
        let to_update: Vec<String> = affected.clone(); // CHANGED: removed mut
        let mut updated = vec![name.to_string()];
        let mut failed = Vec::new();
        
        if let Ok(order) = self.graph.get_topological_order() {
            for var in order {
                if to_update.contains(&var) && var != name {
                    if let Some(expr) = self.expressions.get(&var) {
                        let temp_env = self.create_evaluation_environment();
                        match evaluate(expr, &temp_env) {
                            Ok(new_val) => {
                                if let Err(_e) = self.graph.update_variable(&var, new_val) {
                                    failed.push(var.clone());
                                } else {
                                    updated.push(var.clone());
                                }
                            }
                            Err(_) => {
                                failed.push(var.clone());
                            }
                        }
                    }
                }
            }
        }
        
        let time_taken = start_time.elapsed();
        
        self.propagation_history.push(PropagationEvent {
            timestamp: Utc::now(),
            variable: name.to_string(),
            old_value,
            new_value,
            affected_variables: affected.clone(),
        });
        
        if self.propagation_history.len() > 1000 {
            self.propagation_history.remove(0);
        }
        
        Ok(PropagationResult {
            changed_variables: updated,
            propagation_paths: vec![affected],
            time_taken,
            conflicts_resolved: 0,
            failed_propagations: failed,
        })
    }
    
    fn queue_change(
        &mut self,
        name: &str,
        new_value: Value,
        _delay: std::time::Duration,
    ) -> Result<PropagationResult, PropagationError> {
        self.pending_changes.push_back(PendingChange {
            variable: name.to_string(),
            new_value,
            timestamp: Utc::now(),
        });
        
        Ok(PropagationResult {
            changed_variables: vec![name.to_string()],
            propagation_paths: vec![],
            time_taken: std::time::Duration::from_secs(0),
            conflicts_resolved: 0,
            failed_propagations: vec![],
        })
    }
    
    fn process_batched_changes(&mut self) -> Result<PropagationResult, PropagationError> {
        let mut all_results = PropagationResult {
            changed_variables: Vec::new(),
            propagation_paths: Vec::new(),
            time_taken: std::time::Duration::from_secs(0),
            conflicts_resolved: 0,
            failed_propagations: Vec::new(),
        };
        
        let start_time = std::time::Instant::now();
        
        let mut changes: HashMap<String, Value> = HashMap::new();
        while let Some(change) = self.pending_changes.pop_front() {
            changes.insert(change.variable, change.new_value);
        }
        
        for (var, value) in changes {
            match self.propagate_immediate(&var, Value::Str("".to_string()), value) {
                Ok(result) => {
                    all_results.changed_variables.extend(result.changed_variables);
                    all_results.propagation_paths.extend(result.propagation_paths);
                    all_results.failed_propagations.extend(result.failed_propagations);
                }
                Err(_e) => {
                    all_results.failed_propagations.push(var);
                }
            }
        }
        
        all_results.time_taken = start_time.elapsed();
        Ok(all_results)
    }
    
    fn create_evaluation_environment(&self) -> crate::core::env::Env {
        let mut env = crate::core::env::Env::new();
        
        for (name, &idx) in &self.graph.node_indices {
            let node = &self.graph.graph[idx];
            env.set_direct(name, node.value.clone());
        }
        
        env
    }
    
    pub fn visualize(&self) -> String {
        self.graph.to_dot()
    }
    
    pub fn get_history(&self, limit: usize) -> Vec<&PropagationEvent> {
        let start = if self.propagation_history.len() > limit {
            self.propagation_history.len() - limit
        } else {
            0
        };
        
        self.propagation_history[start..].iter().collect()
    }
    
    pub fn find_propagation_path(&self, _from: &str, _to: &str) -> Option<Vec<String>> {
        None
    }
    
    pub fn get_value(&self, name: &str) -> Option<Value> {
        self.graph.get_value(name).cloned()
    }
    
    pub fn freeze_variable(&mut self, name: &str) -> Result<(), PropagationError> {
        if let Some(&idx) = self.graph.node_indices.get(name) {
            self.graph.graph[idx].is_constant = true;
            Ok(())
        } else {
            Err(PropagationError::VariableNotFound(name.to_string()))
        }
    }
    
    pub fn unfreeze_variable(&mut self, name: &str) -> Result<(), PropagationError> {
        if let Some(&idx) = self.graph.node_indices.get(name) {
            self.graph.graph[idx].is_constant = false;
            Ok(())
        } else {
            Err(PropagationError::VariableNotFound(name.to_string()))
        }
    }
    
    pub fn clear(&mut self) {
        self.graph = PropagationGraph::new();
        self.expressions.clear();
        self.propagation_history.clear();
        self.pending_changes.clear();
    }
    
    pub fn graph_mut(&mut self) -> &mut PropagationGraph {
        &mut self.graph
    }
    
    pub fn graph(&self) -> &PropagationGraph {
        &self.graph
    }
    
    pub fn has_variable(&self, name: &str) -> bool {
        self.graph.has_variable(name)
    }
    
    pub fn variable_names(&self) -> Vec<String> {
        self.graph.node_indices.keys().cloned().collect()
    }
    
    pub fn bulk_register_variables(
        &mut self,
        variables: Vec<(String, Value, bool)>,
    ) -> Result<(), PropagationError> {
        for (name, value, is_constant) in variables {
            self.graph.add_variable(&name, value, is_constant)?;
        }
        Ok(())
    }
    
    pub fn bulk_add_dependencies(
        &mut self,
        dependencies: Vec<(String, String, DependencyType)>,
    ) -> Result<(), PropagationError> {
        for (source, target, dep_type) in dependencies {
            self.graph.add_dependency(&source, &target, dep_type, 1.0, None)?;
        }
        Ok(())
    }
}