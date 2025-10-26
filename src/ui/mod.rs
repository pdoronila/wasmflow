//! User interface components
//!
//! This module contains the egui-based UI implementation.

mod app;
pub(crate) mod canvas;
pub mod code_editor; // T029: Rust code editor for WASM Creator Node
pub mod component_view;
mod dialogs;
pub(crate) mod execution_status; // T031: Visual indicators for continuous execution states
pub mod loading_state; // T011: Component loading progress tracking
mod palette;
pub mod selection; // T002: Rectangle selection state management
pub mod splash_screen; // T011: Splash screen for component loading
mod spotlight; // Spotlight search for quick node creation
mod theme;
pub mod wit_ui_renderer;

pub use app::WasmFlowApp;
pub use loading_state::{ComponentLoadProgress, LoadingState};
pub use splash_screen::SplashScreen;
