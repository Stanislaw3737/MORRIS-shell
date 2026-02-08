// File: src/core/propagation/mod.rs
mod types;
mod graph;
mod engine;

// Export everything public from types module
pub use types::*;

// Export engine
pub use engine::PropagationEngine;

// The graph methods are already available on PropagationGraph