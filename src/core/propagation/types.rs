// File: src/core/propagation/types.rs
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use crate::core::types::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyType {
    Direct,
    Inverse,
    Statistical,
    Temporal,
    Conditional,
    Weak,
    Bidirectional,
}

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub name: String,
    pub value: Value,
    pub is_constant: bool,
    pub last_updated: DateTime<Utc>,
    pub update_count: u64,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub source: String,
    pub target: String,
    pub dependency_type: DependencyType,
    pub weight: f64,
    pub condition: Option<String>,
    pub transform_fn: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct PropagationGraph {
    pub graph: DiGraph<VariableNode, DependencyEdge>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub edge_indices: HashMap<(String, String), EdgeIndex>,
    pub variable_metadata: HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug, Clone)]
pub struct PropagationResult {
    pub changed_variables: Vec<String>,
    pub propagation_paths: Vec<Vec<String>>,
    pub time_taken: std::time::Duration,
    pub conflicts_resolved: usize,
    pub failed_propagations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PropagationStrategy {
    Immediate,
    Debounced(std::time::Duration),
    Batched(usize),
    Lazy,
    Conditional,
    Transactional,
}

#[derive(Debug)]
pub enum PropagationError {
    VariableNotFound(String),
    CircularDependency(Vec<String>),
    ConstraintViolation(String),
    Timeout,
    InvalidDependency(String),
    TransactionConflict,
}

#[derive(Debug, Clone)]
pub struct PropagationEvent {
    pub timestamp: DateTime<Utc>,
    pub variable: String,
    pub old_value: Value,
    pub new_value: Value,
    pub affected_variables: Vec<String>,
}

impl PropagationEvent {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
    
    pub fn variable(&self) -> &str {
        &self.variable
    }
    
    pub fn new_value(&self) -> &Value {
        &self.new_value
    }
    
    pub fn affected_count(&self) -> usize {
        self.affected_variables.len()
    }
}