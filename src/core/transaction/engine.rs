use super::types::*;
//use crate::core::env::Env;
use crate::core::types::Value;
use petgraph::graph::DiGraph;
use petgraph::algo::toposort;   
use std::collections::{HashMap, VecDeque};
use crate::Uuid;
use super::types::{TransactionError, TransactionState};

/*use crate::core::transaction::types::{
    TypeIssue, ConstraintViolation, 
    PropagationAnalysis, PropagationPath, 
    PerformanceEstimate, DetailedChange
};*/

#[derive(Debug)]
pub struct TransactionEngine {
    active_transaction: Option<Transaction>,
    transaction_stack: Vec<Transaction>,
    transaction_log: VecDeque<Transaction>,
    max_log_size: usize,
    nested_transaction_limit: usize,
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            active_transaction: None,
            transaction_stack: Vec::new(),
            transaction_log: VecDeque::new(),
            max_log_size: 1000,
            nested_transaction_limit: 10,
        }
    }
    
    pub fn craft_with_snapshot(&mut self, name: Option<&str>, snapshot: Vec<(String, Value)>) -> Result<Uuid, TransactionError> {
        if self.active_transaction.is_some() {
            return Err(TransactionError::TransactionAlreadyActive);
        }
        
        if self.transaction_stack.len() >= self.nested_transaction_limit {
            return Err(TransactionError::NestedTransactionLimitExceeded(
                self.nested_transaction_limit
            ));
        }
        
        let mut transaction = Transaction::new(name);
        
        // Take snapshot
        for (name, value) in snapshot {
            transaction.snapshot.insert(name, value);
        }
        
        let id = transaction.id;
        self.active_transaction = Some(transaction);
        
        Ok(id)
    }
    
    pub fn take_active_transaction(&mut self) -> Result<Transaction, TransactionError> {
        self.active_transaction.take()
            .ok_or(TransactionError::NoActiveTransaction)
    }
    
    pub fn get_active_transaction_mut(&mut self) -> Result<&mut Transaction, TransactionError> {
        self.active_transaction.as_mut()
            .ok_or(TransactionError::NoActiveTransaction)
    }
    
    pub fn inspect(&self) -> Result<&Transaction, TransactionError> {
        self.active_transaction.as_ref()
            .ok_or(TransactionError::NoActiveTransaction)
    }
    
    pub fn temper(&self, env: &crate::core::env::Env) -> Result<super::types::TransactionPreview, super::types::TransactionError> {
        let transaction = self.active_transaction.as_ref()
            .ok_or(super::types::TransactionError::NoActiveTransaction)?
            .clone();
        
        // Enhanced analysis
        let mut conflicts = Vec::new();
        let mut propagation_paths = Vec::new();
        let mut type_issues = Vec::new();
        let mut constraint_violations = Vec::new();
        let mut direct_propagations = Vec::new();
        let mut computed_propagations = Vec::new();
        let mut detailed_changes = Vec::new();
        
        // Analyze each change
        for (var_name, change) in &transaction.changes {
            // Check if this variable is already frozen
            if let Some(existing_var) = env.get_variable(var_name) {
                if existing_var.is_constant {
                    conflicts.push(format!("{} is frozen", var_name));
                    constraint_violations.push(super::types::ConstraintViolation {
                        variable: var_name.clone(),
                        constraint: "frozen".to_string(),
                        violation: "Attempt to modify frozen variable".to_string(),
                    });
                }
            }

            let dependents = env.get_dependents(var_name);
            for dependent in dependents {
                if let Some(dep_var) = env.get_variable(&dependent) {
                    if dep_var.is_constant {
                        constraint_violations.push(super::types::ConstraintViolation {
                            variable: dependent.clone(),
                            constraint: "frozen".to_string(),
                            violation: format!("Depends on '{}' but is frozen - propagation blocked", var_name),
                        });
                    }
                }
            }
            
            // Type checking (simplified for now)
            self.check_types(var_name, change, env, &mut type_issues);
            
            // Collect propagation paths
            let dependents = env.get_dependents(var_name);
            if !dependents.is_empty() {
                propagation_paths.push(dependents.clone());
                
                // Create detailed propagation info
                for dependent in dependents {
                    if let Some(expr) = env.get_expression(&dependent) {
                        computed_propagations.push(super::types::PropagationPath {
                            source: var_name.clone(),
                            target: dependent.clone(),
                            expression: Some(expr.to_string()),
                            dependencies: env.get_dependencies(&dependent),
                        });
                    } else {
                        direct_propagations.push(super::types::PropagationPath {
                            source: var_name.clone(),
                            target: dependent.clone(),
                            expression: None,
                            dependencies: Vec::new(),
                        });
                    }
                }
            }
            
            // Create detailed change info
            detailed_changes.push(super::types::DetailedChange {
                variable: var_name.clone(),
                old_value: change.old_value.display(),
                new_value: change.new_value.display(),
                change_type: if change.expression.is_some() { 
                    "computed".to_string() 
                } else { 
                    "direct".to_string() 
                },
                propagation_targets: env.get_dependents(var_name),
                safety_notes: self.generate_safety_notes(var_name, change, env),
            });
        }
        
        let safety_analysis = super::types::SafetyAnalysis {
            circular_dependencies: Vec::new(), // Placeholder
            type_issues: type_issues,
            constraint_violations: constraint_violations,
            propagation_analysis: super::types::PropagationAnalysis {
                direct_propagations: direct_propagations,
                computed_propagations: computed_propagations,
                blocked_propagations: Vec::new(), // Placeholder
            },
            performance_estimate: self.estimate_performance(&transaction, env),
            overall_safety_score: self.calculate_safety_score(&transaction, env),
        };
        
        let preview = super::types::TransactionPreview {
            transaction_id: transaction.id,
            changes: transaction.changes.clone(),
            propagation_paths,
            conflicts,
            estimated_affected: transaction.get_affected_variables().len(),
            safety_analysis,
            detailed_changes,
        };
        
        Ok(preview)
    }
    
    pub fn record_transaction(&mut self, transaction: Transaction) {
        self.transaction_log.push_back(transaction);
        if self.transaction_log.len() > self.max_log_size {
            self.transaction_log.pop_front();
        }
    }
    
    pub fn has_active_transaction(&self) -> bool {
        self.active_transaction.is_some()
    }
    
    pub fn active_transaction_info(&self) -> Option<(Uuid, TransactionState, usize)> {
        self.active_transaction.as_ref().map(|t| (
            t.id,
            t.state.clone(),
            t.change_count()
        ))
    }
    
    pub fn get_transaction_history(&self, limit: usize) -> Vec<&Transaction> {
        let start = if self.transaction_log.len() > limit {
            self.transaction_log.len() - limit
        } else {
            0
        };
        
        self.transaction_log.range(start..).collect()
    }

    // File: src/core/transaction/engine.rs - Simplified forge
    pub fn forge(&mut self, env: &mut crate::core::env::Env) -> Result<Vec<String>, TransactionError> {
        let mut transaction = self.take_active_transaction()?;
        
        if transaction.is_empty() {
            transaction.state = TransactionState::Forged;
            self.record_transaction(transaction);
            return Ok(Vec::new());
        }
        
        // First pass: Set all direct values
        let mut applied = Vec::new();
        let mut failures = Vec::new();
        
        for (var_name, change) in &transaction.changes {
            // If it's a direct value (no expression), apply it immediately
            if change.expression.is_none() {
                match env.update_value(var_name, change.new_value.clone()) {
                    Ok(()) => {
                        applied.push(var_name.clone());
                        println!("DEBUG: Set direct value {} = {}", var_name, change.new_value.display());
                    }
                    Err(e) => {
                        failures.push(format!("{}: {}", var_name, e));
                    }
                }
            }
        }
        
        // Second pass: Evaluate expressions
        for (var_name, change) in &transaction.changes {
            if let Some(ref expr) = change.expression {
                match crate::core::expr::evaluate(expr, env) {
                    Ok(value) => {
                        match env.update_value(var_name, value.clone()) {
                            Ok(()) => {
                                applied.push(var_name.clone());
                                println!("DEBUG: Evaluated {} = {} from expr: {:?}", 
                                    var_name, value.display(), expr);
                            }
                            Err(e) => {
                                failures.push(format!("{}: {}", var_name, e));
                            }
                        }
                    }
                    Err(e) => {
                        failures.push(format!("Cannot evaluate {}: {}", var_name, e));
                    }
                }
            }
        }
        
        if !failures.is_empty() {
            // Rollback
            self.rollback_transaction(env, &transaction)?;
            transaction.state = TransactionState::Smelted;
            self.record_transaction(transaction);
            
            return Err(TransactionError::PropagationError(
                format!("Forging failed: {}", failures.join(", "))
            ));
        }
        
        transaction.state = TransactionState::Forged;
        self.record_transaction(transaction);
        
        Ok(applied)
    }
    
    pub fn build_evaluation_order(&self, transaction: &Transaction) -> (Vec<String>, Vec<String>) {
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();
        let mut circular_deps = Vec::new();
        
        // Add nodes for all variables in transaction
        for var_name in transaction.changes.keys() {
            let idx = graph.add_node(var_name.clone());
            node_indices.insert(var_name.clone(), idx);
        }
        
        // Add edges based on dependencies
        for (var_name, change) in &transaction.changes {
            if let Some(source_idx) = node_indices.get(var_name) {
                for dep in &change.dependencies {
                    if let Some(target_idx) = node_indices.get(dep) {
                        // Check if dependency is also in this transaction
                        if transaction.changes.contains_key(dep) {
                            graph.add_edge(*target_idx, *source_idx, ());
                        }
                    }
                }
            }
        }
        
        // Get topological order
        match toposort(&graph, None) {
            Ok(order) => {
                let ordered_vars: Vec<String> = order
                    .into_iter()
                    .map(|idx| graph[idx].clone())
                    .collect();
                (ordered_vars, circular_deps)
            }
            Err(cycle) => {
                // Extract circular dependency path
                let cycle_node = graph[cycle.node_id()].clone();
                circular_deps.push(cycle_node);
                (Vec::new(), circular_deps)
            }
        }
    }
    
    fn evaluate_and_apply_change(
        &self,
        env: &mut crate::core::env::Env,
        change: &ValueChange,
    ) -> Result<(), String> {
        let final_value = if let Some(ref expr) = change.expression {
            // Evaluate expression with current environment
            match crate::core::expr::evaluate(expr, env) {
                Ok(value) => value,
                Err(e) => {
                    // If evaluation fails, try to use the new_value from change
                    // (for direct values or fallback)
                    println!("DEBUG: Evaluation failed for {}: {}, using fallback", change.variable, e);
                    change.new_value.clone()
                }
            }
        } else {
            // Direct value
            change.new_value.clone()
        };
        
        // Apply to environment
        env.update_value(&change.variable, final_value)
            .map_err(|e| format!("Application error: {}", e))
    }
    
    pub fn rollback_transaction(
        &self,
        env: &mut crate::core::env::Env,
        transaction: &Transaction,
    ) -> Result<(), TransactionError> {
        // Restore snapshot
        for (var_name, original_value) in &transaction.snapshot {
            let _ = env.update_value(var_name, original_value.clone());
        }
        
        // Remove variables that were created in this transaction
        for var_name in transaction.changes.keys() {
            if !transaction.snapshot.contains_key(var_name) {
                // This is a bit tricky without access to env's private methods
                // We'll need to add a remove_variable method to Env
            }
        }
        
        Ok(())
    }

    fn check_types(&self, _var_name: &str, _change: &crate::core::transaction::types::ValueChange, _env: &crate::core::env::Env, _issues: &mut Vec<TypeIssue>) {
        // Simple type checking - can be enhanced later
        // For now, this is a placeholder
    }
    
    fn generate_safety_notes(&self, _var_name: &str, _change: &super::types::ValueChange, _env: &crate::core::env::Env) -> Vec<String> {
        // Generate safety-related notes for this change
        Vec::new()
    }

    
    fn estimate_performance(&self, transaction: &super::types::Transaction, _env: &crate::core::env::Env) -> super::types::PerformanceEstimate {
        let variable_count = transaction.changes.len();
        let propagation_steps = transaction.get_affected_variables().len();
        
        // Rough estimation - can be made more sophisticated
        let estimated_time_ms = (variable_count * 10 + propagation_steps * 5) as u64;
        
        let memory_impact = if variable_count > 100 {
            "high".to_string()
        } else if variable_count > 10 {
            "medium".to_string()
        } else {
            "low".to_string()
        };
        
        let bottleneck_variables = transaction.get_affected_variables()
            .into_iter()
            .take(5) // Top 5 most connected variables
            .collect();
        
        super::types::PerformanceEstimate {
            variable_count,
            propagation_steps,
            estimated_time_ms,
            memory_impact,
            bottleneck_variables,
        }
    }
    
    fn calculate_safety_score(&self, transaction: &super::types::Transaction, _env: &crate::core::env::Env) -> f32 {
        // Simple scoring algorithm - can be enhanced
        let base_score = 1.0;
        let mut score = base_score;
        
        // Penalty for conflicts
        let conflict_penalty = transaction.changes.len() as f32 * 0.1;
        score -= conflict_penalty;
        
        // Penalty for frozen variables (placeholder)
        let frozen_count = 0.0;
        score -= frozen_count;
        
        // Ensure score stays between 0 and 1
        score.max(0.0).min(1.0)
    }

    pub fn what_if(&self, scenario: &HashMap<String, crate::core::types::Value>, env: &crate::core::env::Env) -> Result<super::types::ScenarioOutcome, super::types::TransactionError> {
        let transaction = self.active_transaction.as_ref()
            .ok_or(super::types::TransactionError::NoActiveTransaction)?
            .clone();
        
        let mut affected_variables = Vec::new();
        let mut new_conflicts = Vec::new();
        let resolved_mismatches = Vec::new();
        let mut propagation_impacts = Vec::new();
        
        // Add hypothetical changes to affected variables
        for (var_name, hypothetical_value) in scenario {
            affected_variables.push(var_name.clone());
            
            // Record the direct change
            let old_value = if let Some(change) = transaction.changes.get(var_name) {
                change.new_value.display()
            } else if let Some(var) = env.get_variable(var_name) {
                var.value.display()
            } else {
                "undefined".to_string()
            };
            
            propagation_impacts.push(super::types::PropagationImpact {
                variable: var_name.clone(),
                old_value,
                new_value: hypothetical_value.display(),
                reason: "Hypothetical change".to_string(),
            });
            
            // Check for conflicts with frozen variables
            if let Some(var) = env.get_variable(var_name) {
                if var.is_constant {
                    new_conflicts.push(format!("{} would still be frozen", var_name));
                }
            }
        }
        
        // Simulate propagation effects
        for (var_name, _hypothetical_value) in scenario {
            let dependents = env.get_dependents(var_name);
            for dependent in dependents {
                affected_variables.push(dependent.clone());
                
                if let Some(var) = env.get_variable(&dependent) {
                    propagation_impacts.push(super::types::PropagationImpact {
                        variable: dependent.clone(),
                        old_value: var.value.display(),
                        new_value: "would recalculate".to_string(),
                        reason: format!("Depends on {} - would propagate change", var_name),
                    });
                }
            }
        }
        
        // Remove duplicates from affected_variables
        affected_variables.sort();
        affected_variables.dedup();
        
        // Calculate safety delta (simplified)
        let current_safety = self.calculate_safety_score(&transaction, env);
        let hypothetical_safety = current_safety; // No change for now
        let safety_delta = hypothetical_safety - current_safety;
        
        Ok(super::types::ScenarioOutcome {
            affected_variables,
            new_conflicts,
            type_mismatches_resolved: resolved_mismatches,
            propagation_impact: propagation_impacts,
            safety_delta,
        })
    }

}