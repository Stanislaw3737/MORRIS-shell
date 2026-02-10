use crate::core::types::Value;
use crate::core::expr::Expr;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SafetyAnalysis {
    pub circular_dependencies: Vec<CircularDependency>,
    pub type_issues: Vec<TypeIssue>,
    pub constraint_violations: Vec<ConstraintViolation>,
    pub propagation_analysis: PropagationAnalysis,
    pub performance_estimate: PerformanceEstimate,
    pub overall_safety_score: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct CircularDependency {
    pub path: Vec<String>,
    pub severity: String, // "warning", "error"
}

#[derive(Debug, Clone)]
pub struct TypeIssue {
    pub variable: String,
    pub expected_type: String,
    pub actual_type: String,
    pub issue: String,
}

#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub variable: String,
    pub constraint: String,
    pub violation: String,
}

#[derive(Debug, Clone)]
pub struct PropagationAnalysis {
    pub direct_propagations: Vec<PropagationPath>,
    pub computed_propagations: Vec<PropagationPath>,
    pub blocked_propagations: Vec<BlockedPropagation>,
}

#[derive(Debug, Clone)]
pub struct PropagationPath {
    pub source: String,
    pub target: String,
    pub expression: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BlockedPropagation {
    pub source: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct PerformanceEstimate {
    pub variable_count: usize,
    pub propagation_steps: usize,
    pub estimated_time_ms: u64,
    pub memory_impact: String, // "low", "medium", "high"
    pub bottleneck_variables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetailedChange {
    pub variable: String,
    pub old_value: String,
    pub new_value: String,
    pub change_type: String, // "direct", "computed", "propagated"
    pub propagation_targets: Vec<String>,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Crafting,
    Tempered,
    Forged,
    Smelted,
    Quenched,
    Annealing(usize),
    Polishing,
}

#[derive(Debug, Clone)]
pub struct ValueChange {
    pub variable: String,
    pub old_value: Value,
    pub new_value: Value,
    pub expression: Option<Expr>,
    pub raw_expression: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl ValueChange {
    pub fn new(
        variable: String,
        old_value: Value,
        new_value: Value,
        expression: Option<Expr>,
        raw_expression: Option<String>,
        dependencies: Vec<String>,
    ) -> Self {
        Self {
            variable,
            old_value,
            new_value,
            expression,
            raw_expression,
            dependencies,
            metadata: HashMap::new(),
        }
    }
    
    pub fn simple(
        variable: String,
        old_value: Value,
        new_value: Value,
        expression: Option<Expr>,
        dependencies: Vec<String>,
    ) -> Self {
        let raw_expr = expression.as_ref().map(|e| e.to_string());
        
        Self::new(
            variable,
            old_value,
            new_value,
            expression,
            raw_expr,
            dependencies,
        )
    }
    
    pub fn add_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
    }
    
    pub fn extract_dependencies(&mut self) -> Vec<String> {
        if let Some(ref raw_expr) = self.raw_expression {
            use crate::core::expr::extract_variables;
            
            if let Ok(expr) = crate::core::expr::parse_expression(raw_expr) {
                self.dependencies = extract_variables(&expr);
                self.dependencies.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub name: Option<String>,
    pub state: TransactionState,
    pub changes: HashMap<String, ValueChange>,
    pub snapshot: HashMap<String, Value>,
    pub metadata: HashMap<String, Value>,
    pub parent_transaction: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub propagation_paths: Vec<Vec<String>>,
    pub failed_propagations: Vec<String>,
}

impl Transaction {
    pub fn new(name: Option<&str>) -> Self {  // Fixed typo: "lf" -> "Self"
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        Self {
            id,
            name: name.map(|s| s.to_string()),
            state: TransactionState::Crafting,
            changes: HashMap::new(),
            snapshot: HashMap::new(),
            metadata: HashMap::new(),
            parent_transaction: None,
            created_at: now,
            modified_at: now,
            propagation_paths: Vec::new(),
            failed_propagations: Vec::new(),
        }
    }
    
    pub fn add_change(
        &mut self,
        variable: String,
        old_value: Value,
        new_value: Value,
        expression: Option<Expr>,
        dependencies: Vec<String>,
    ) {
        let change = ValueChange::simple(
            variable.clone(),
            old_value,
            new_value,
            expression,
            dependencies,
        );
        
        self.changes.insert(variable, change);
        self.modified_at = Utc::now();
    }
    
    pub fn add_change_with_raw_expr(
        &mut self,
        variable: String,
        old_value: Value,
        new_value: Value,
        expression: Option<Expr>,
        raw_expression: Option<String>,
        dependencies: Vec<String>,
    ) {
        let change = ValueChange::new(
            variable.clone(),
            old_value,
            new_value,
            expression,
            raw_expression,
            dependencies,
        );
        
        self.changes.insert(variable, change);
        self.modified_at = Utc::now();
    }
    
    pub fn add_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
        self.modified_at = Utc::now();
    }
    
    pub fn get_affected_variables(&self) -> Vec<String> {
        let mut affected = Vec::new();
        affected.extend(self.changes.keys().cloned());
        for path in &self.propagation_paths {
            affected.extend(path.clone());
        }
        affected.sort();
        affected.dedup();
        affected
    }
    
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
    
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionPreview {
    pub transaction_id: Uuid,
    pub changes: HashMap<String, ValueChange>,
    pub propagation_paths: Vec<Vec<String>>,
    pub conflicts: Vec<String>,
    pub estimated_affected: usize,
    // NEW ENHANCEMENTS:
    pub safety_analysis: SafetyAnalysis,
    pub detailed_changes: Vec<DetailedChange>,
}

#[derive(Debug)]
pub enum TransactionError {
    NoActiveTransaction,
    TransactionAlreadyActive,
    VariableNotFound(String),
    ConstraintViolation(String),
    PropagationError(String),
    CircularDependency(Vec<String>),
    MergeConflict(String),
    InvalidState(TransactionState, &'static str),
    NestedTransactionLimitExceeded(usize),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::NoActiveTransaction => 
                write!(f, "No active transaction"),
            TransactionError::TransactionAlreadyActive => 
                write!(f, "Transaction already active"),
            TransactionError::VariableNotFound(name) => 
                write!(f, "Variable not found: {}", name),
            TransactionError::ConstraintViolation(msg) => 
                write!(f, "Constraint violation: {}", msg),
            TransactionError::PropagationError(msg) => 
                write!(f, "Propagation error: {}", msg),
            TransactionError::CircularDependency(path) => 
                write!(f, "Circular dependency: {}", path.join(" -> ")),
            TransactionError::MergeConflict(msg) => 
                write!(f, "Merge conflict: {}", msg),
            TransactionError::InvalidState(state, expected) => 
                write!(f, "Invalid transaction state: {:?} (expected {})", state, expected),
            TransactionError::NestedTransactionLimitExceeded(limit) => 
                write!(f, "Nested transaction limit exceeded: {}", limit),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WhatIfScenario {
    pub scenario_name: String,
    pub hypothetical_changes: HashMap<String, Value>,
    pub predicted_outcome: ScenarioOutcome,
}

#[derive(Debug, Clone)]
pub struct ScenarioOutcome {
    pub affected_variables: Vec<String>,
    pub new_conflicts: Vec<String>,
    pub type_mismatches_resolved: Vec<String>,
    pub propagation_impact: Vec<PropagationImpact>,
    pub safety_delta: f32, // Change in safety score
}

#[derive(Debug, Clone)]
pub struct PropagationImpact {
    pub variable: String,
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
}