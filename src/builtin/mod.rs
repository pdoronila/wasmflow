//! Built-in node implementations
//!
//! This module contains built-in computational nodes (math, constants, text operations).

pub mod constants;
pub mod continuous_example; // Continuous execution example node
pub mod math;
pub mod views;
pub mod wasm_creator; // T040: WASM Component Creator Node

pub use constants::register_constant_nodes;
pub use continuous_example::{register_continuous_example, ContinuousTimerExecutor, ContinuousCombinerExecutor}; // T050: Added ContinuousCombinerExecutor
pub use math::register_math_nodes;
pub use views::{HttpFetchFooterView, ConstantNodeFooterView, MathNodeFooterView};
pub use wasm_creator::register_wasm_creator_node; // T040: Registration function
