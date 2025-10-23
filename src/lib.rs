//! WasmFlow - WebAssembly Node-Based Visual Composition System
//!
//! This library provides the core functionality for WasmFlow, including:
//! - Graph data structures and execution engine
//! - Component loading and management
//! - Security and capability enforcement
//! - Serialization and persistence

pub mod builtin;
pub mod graph;
pub mod runtime;
pub mod ui;

/// T086: Initialize the logging framework
///
/// Configures env_logger to support RUST_LOG environment variable.
/// Call this once at application startup.
///
/// # Examples
///
/// ```no_run
/// wasmflow::init_logging();
/// log::info!("Application started");
/// ```
pub fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_millis()
        .init();

    log::info!("WasmFlow logging initialized");
}

/// Re-export commonly used types
pub use graph::{Connection, GraphNode, NodeGraph, NodeValue, Port};

/// Application errors
#[derive(Debug, thiserror::Error)]
pub enum WasmFlowError {
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),

    #[error("Component error: {0}")]
    Component(#[from] ComponentError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    #[error("Continuous execution error: {0}")]
    ContinuousExecution(#[from] ContinuousNodeError),
}

/// Graph-specific errors
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Cycle detected in graph involving nodes: {0:?}")]
    CycleDetected(Vec<uuid::Uuid>),

    #[error("Type mismatch: cannot connect {from:?} to {to:?}")]
    TypeMismatch {
        from: String,
        to: String,
    },

    #[error("Invalid connection: {0}")]
    InvalidConnection(String),

    #[error("Component error: {0}")]
    ComponentError(#[from] ComponentError),
}

/// Component-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("Failed to load component from {path}: {reason}")]
    LoadFailed {
        path: std::path::PathBuf,
        reason: String,
    },

    #[error("Component validation failed: {0}")]
    ValidationFailed(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Permission denied for node {node_id} accessing {capability}")]
    PermissionDenied {
        node_id: uuid::Uuid,
        capability: String,
    },
}

/// Serialization-specific errors
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("Failed to save to {path}: {source}")]
    SaveFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to load from {path}: {reason}")]
    LoadFailed {
        path: std::path::PathBuf,
        reason: String,
    },

    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}

/// Continuous execution errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContinuousNodeError {
    #[error("Execution failed in node {node_name} ({node_id}): {message}")]
    ExecutionFailed {
        node_id: uuid::Uuid,
        node_name: String,
        message: String,
        source_location: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    #[error("Permission denied for node {node_name} ({node_id}): attempted to {attempted_action} but lacks {capability} permission")]
    PermissionDenied {
        node_id: uuid::Uuid,
        node_name: String,
        capability: String,
        attempted_action: String,
    },

    #[error("Node {node_name} ({node_id}) timed out after {duration:?}")]
    Timeout {
        node_id: uuid::Uuid,
        node_name: String,
        duration: std::time::Duration,
    },

    #[error("Network error in node {node_id}: {message}")]
    NetworkError {
        node_id: uuid::Uuid,
        message: String,
        status_code: Option<u16>,
    },

    #[error("Component trapped in node {node_name} ({node_id}): {trap_message}")]
    ComponentTrap {
        node_id: uuid::Uuid,
        node_name: String,
        trap_message: String,
    },
}
