// File: src/core/propagation/graph.rs
use super::types::*;
use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use petgraph::algo::{toposort, has_path_connecting};
use petgraph::visit::EdgeRef;
use chrono::Utc;
use crate::core::types::Value;
use std::collections::{HashMap, HashSet};

impl PropagationGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            variable_metadata: HashMap::new(),
        }
    }
    
    pub fn add_variable(&mut self, name: &str, value: Value, is_constant: bool) -> Result<(), PropagationError> {
        if self.node_indices.contains_key(name) {
            return Ok(());
        }
        
        let node = VariableNode {
            name: name.to_string(),
            value: value.clone(),
            is_constant,
            last_updated: Utc::now(),
            update_count: 0,
            metadata: HashMap::new(),
        };
        
        let node_index = self.graph.add_node(node);
        self.node_indices.insert(name.to_string(), node_index);
        
        Ok(())
    }
    
    pub fn add_dependency(
        &mut self, 
        source: &str, 
        target: &str, 
        dep_type: DependencyType,
        weight: f64,
        condition: Option<String>,
    ) -> Result<(), PropagationError> {
        let source_idx = *self.node_indices.get(source)
            .ok_or_else(|| PropagationError::VariableNotFound(source.to_string()))?;
        let target_idx = *self.node_indices.get(target)
            .ok_or_else(|| PropagationError::VariableNotFound(target.to_string()))?;
        
        if source == target {
            return Err(PropagationError::InvalidDependency(
                "Cannot add self-dependency".to_string()
            ));
        }
        
        let key = (source.to_string(), target.to_string());
        if self.edge_indices.contains_key(&key) {
            let edge_idx = self.edge_indices[&key];
            let edge = self.graph.edge_weight_mut(edge_idx).unwrap();
            edge.dependency_type = dep_type;
            edge.weight = weight;
            edge.condition = condition.clone();
            return Ok(());
        }
        
        self.check_circular_dependency(source, target)?;
        
        let edge = DependencyEdge {
            source: source.to_string(),
            target: target.to_string(),
            dependency_type: dep_type.clone(),
            weight,
            condition: condition.clone(),
            transform_fn: None,
            created_at: Utc::now(),
        };
        
        let edge_idx = self.graph.add_edge(source_idx, target_idx, edge);
        self.edge_indices.insert(key, edge_idx);
        
        if dep_type == DependencyType::Bidirectional {
            let reverse_key = (target.to_string(), source.to_string());
            let reverse_edge = DependencyEdge {
                source: target.to_string(),
                target: source.to_string(),
                dependency_type: DependencyType::Bidirectional,
                weight,
                condition,
                transform_fn: None,
                created_at: Utc::now(),
            };
            
            let reverse_idx = self.graph.add_edge(target_idx, source_idx, reverse_edge);
            self.edge_indices.insert(reverse_key, reverse_idx);
        }
        
        Ok(())
    }
    
    fn check_circular_dependency(&self, source: &str, target: &str) -> Result<(), PropagationError> {
        let source_idx = self.node_indices.get(source);
        let target_idx = self.node_indices.get(target);
        
        if let (Some(&src_idx), Some(&tgt_idx)) = (source_idx, target_idx) {
            if has_path_connecting(&self.graph, tgt_idx, src_idx, None) {
                let path = self.find_path(tgt_idx, src_idx);
                return Err(PropagationError::CircularDependency(path));
            }
        }
        
        Ok(())
    }
    
    fn find_path(&self, start: NodeIndex, end: NodeIndex) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut stack = vec![(start, vec![])];

        while let Some((node, path)) = stack.pop() {
            if node == end {
                let mut full_path: Vec<String> = Vec::new();
                for &idx in &path {
                    let node_idx: NodeIndex = idx;
                    full_path.push(self.graph[node_idx].name.clone());
                }
                full_path.push(self.graph[end].name.clone());
                return full_path;
            }

            if visited.insert(node) {
                for neighbor in self.graph.neighbors(node) {
                    let mut new_path = path.clone();
                    new_path.push(node);
                    stack.push((neighbor, new_path));
                }
            }
        }

        vec![]
    }
    
    pub fn update_variable(
        &mut self, 
        name: &str, 
        new_value: Value
    ) -> Result<Vec<String>, PropagationError> {
        let node_idx = *self.node_indices.get(name)
            .ok_or_else(|| PropagationError::VariableNotFound(name.to_string()))?;
        
        if self.graph[node_idx].is_constant {
            return Err(PropagationError::ConstraintViolation(
                format!("Variable '{}' is frozen", name)
            ));
        }
        
        self.graph[node_idx].value = new_value;
        self.graph[node_idx].last_updated = Utc::now();
        self.graph[node_idx].update_count += 1;
        
        let affected = self.get_dependents_transitive(name);
        Ok(affected)
    }
    
    pub fn get_direct_dependents(&self, name: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        
        if let Some(&node_idx) = self.node_indices.get(name) {
            for edge in self.graph.edges(node_idx) {
                let target_idx = edge.target();
                let target_name = &self.graph[target_idx].name;
                dependents.push(target_name.clone());
            }
        }
        
        dependents
    }
    
    pub fn get_direct_dependencies(&self, name: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        if let Some(&node_idx) = self.node_indices.get(name) {
            for edge in self.graph.edges_directed(node_idx, petgraph::Direction::Incoming) {
                let source_idx = edge.source();
                let source_name = &self.graph[source_idx].name;
                dependencies.push(source_name.clone());
            }
        }
        
        dependencies
    }
    
    pub fn get_dependents_transitive(&self, name: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut stack = vec![name.to_string()];
        let mut dependents = Vec::new();
        
        while let Some(current) = stack.pop() {
            if visited.insert(current.clone()) {
                let direct = self.get_direct_dependents(&current);
                for dep in direct {
                    if !visited.contains(&dep) {
                        dependents.push(dep.clone());
                        stack.push(dep);
                    }
                }
            }
        }
        
        dependents
    }
    
    pub fn get_topological_order(&self) -> Result<Vec<String>, PropagationError> {
        match toposort(&self.graph, None) {
            Ok(order) => {
                Ok(order.into_iter()
                    .map(|idx| self.graph[idx].name.clone())
                    .collect())
            }
            Err(cycle) => {
                let cycle_node = self.graph[cycle.node_id()].name.clone();
                Err(PropagationError::CircularDependency(vec![cycle_node]))
            }
        }
    }
    
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph PropagationGraph {\n");
        dot.push_str("  rankdir=LR;\n  node [shape=box];\n\n");
        
        for (name, &idx) in &self.node_indices {
            let node = &self.graph[idx];
            let color = if node.is_constant { "lightgray" } else { "white" };
            let style = if node.is_constant { "filled" } else { "solid" };
            
            dot.push_str(&format!(
                "  \"{}\" [label=\"{} = {}\", style={}, fillcolor={}];\n",
                name, name, node.value.display(), style, color
            ));
        }
        
        dot.push_str("\n");
        
        for edge in self.graph.edge_references() {
            let source = &self.graph[edge.source()].name;
            let target = &self.graph[edge.target()].name;
            let dep_edge = edge.weight();
            
            let color = match dep_edge.dependency_type {
                DependencyType::Direct => "black",
                DependencyType::Inverse => "blue",
                DependencyType::Conditional => "orange",
                DependencyType::Weak => "gray",
                DependencyType::Bidirectional => "red",
                _ => "black",
            };
            
            let style = if dep_edge.dependency_type == DependencyType::Weak {
                "dashed"
            } else {
                "solid"
            };
            
            let label = if dep_edge.weight != 1.0 {
                format!(" (w={:.2})", dep_edge.weight)
            } else {
                String::new()
            };
            
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [color={}, style={}, label=\"{}\"];\n",
                source, target, color, style, label
            ));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.node_indices.get(name)
            .map(|&idx| &self.graph[idx].value)
    }
    
    pub fn has_variable(&self, name: &str) -> bool {
        self.node_indices.contains_key(name)
    }
    
    pub fn remove_variable(&mut self, name: &str) -> Result<(), PropagationError> {
        let node_idx = *self.node_indices.get(name)
            .ok_or_else(|| PropagationError::VariableNotFound(name.to_string()))?;
        
        let edges_to_remove: Vec<EdgeIndex> = self.graph.edges(node_idx)
            .chain(self.graph.edges_directed(node_idx, petgraph::Direction::Incoming))
            .map(|edge| edge.id())
            .collect();
        
        for edge_idx in edges_to_remove {
            self.graph.remove_edge(edge_idx);
        }
        
        self.node_indices.remove(name);
        self.graph.remove_node(node_idx);
        
        Ok(())
    }
}