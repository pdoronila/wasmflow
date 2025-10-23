//! Runtime execution and WASM component management
//!
//! This module handles graph execution, component loading, and capability enforcement.

pub mod capabilities;
pub mod compiler;
pub mod continuous;
pub mod engine;
pub mod instance_pool;
pub mod template_generator;
pub mod wac_integration; // T004: WAC composition integration
pub mod wasm_host;

pub use capabilities::CapabilitySet;
pub use compiler::{CompilationConfig, CompilationResult, ComponentCompiler};
pub use continuous::{ControlMessage, ContinuousExecutionManager, ExecutionResult};
pub use template_generator::{ComponentMetadata, PortSpec, TemplateGenerator, TemplateType};
