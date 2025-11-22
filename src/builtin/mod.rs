//! Built-in node implementations
//!
//! This module contains built-in nodes (constants, development tools, examples).

pub mod constants;
pub mod continuous_example; // Continuous execution example node
pub mod glsl_shader_editor; // GLSL Shader Editor Node
pub mod shader_preview; // Shader Preview Node
pub mod shader_program_linker; // Shader Program Linker Node
pub mod texture_loader; // Texture Loader Node (Phase 3)
pub mod envmap_loader; // Environment Map Loader Node (Phase 4 Step 2)
pub mod http_server_listener; // HTTP server listener node
pub mod views;
pub mod wasm_creator; // T040: WASM Component Creator Node

pub use constants::register_constant_nodes;
pub use continuous_example::{
    register_continuous_example, ContinuousCombinerExecutor, ContinuousTimerExecutor,
}; // T050: Added ContinuousCombinerExecutor
pub use glsl_shader_editor::register_glsl_shader_editor_node; // GLSL shader editor registration
pub use shader_preview::register_shader_preview_node; // Shader preview registration
pub use shader_program_linker::register_shader_program_linker_node; // Shader program linker registration
pub use texture_loader::register_texture_loader_node; // Texture loader registration (Phase 3)
pub use envmap_loader::register_envmap_loader_node; // Environment map loader registration (Phase 4)
pub use http_server_listener::{register_http_server_listener, HttpServerListenerExecutor};
pub use views::{ConstantNodeFooterView, HttpFetchFooterView};
pub use wasm_creator::register_wasm_creator_node; // T040: Registration function
