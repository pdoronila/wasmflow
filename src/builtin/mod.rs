//! Built-in node implementations
//!
//! This module contains built-in nodes (constants, development tools, examples).

pub mod constants;
pub mod continuous_example; // Continuous execution example node
pub mod http_server_listener; // HTTP server listener node
pub mod views;
pub mod wasm_creator; // T040: WASM Component Creator Node

pub use constants::register_constant_nodes;
pub use continuous_example::{register_continuous_example, ContinuousTimerExecutor, ContinuousCombinerExecutor}; // T050: Added ContinuousCombinerExecutor
pub use http_server_listener::{register_http_server_listener, HttpServerListenerExecutor};
pub use views::{HttpFetchFooterView, ConstantNodeFooterView};
pub use wasm_creator::register_wasm_creator_node; // T040: Registration function
