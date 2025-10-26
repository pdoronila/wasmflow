//! Component loading state management
//!
//! This module provides types and utilities for tracking the progress of
//! asynchronous component loading during application startup.

use crate::graph::node::ComponentRegistry;
use std::sync::{Arc, Mutex};

/// Application loading state
///
/// Tracks the overall state of component loading during application startup.
#[derive(Clone)]
pub enum LoadingState {
    /// Loading has not started yet
    NotStarted,

    /// Components are currently being loaded
    Loading {
        /// Shared progress tracker
        progress: Arc<Mutex<ComponentLoadProgress>>,
        /// Registry being populated by background thread
        registry: Arc<Mutex<ComponentRegistry>>,
    },

    /// Loading completed successfully
    Completed {
        /// Total components loaded
        total: usize,
        /// Errors encountered during loading
        errors: Vec<String>,
    },

    /// Loading failed with a critical error
    Failed {
        /// Error message
        error: String,
    },
}

// Manual Debug implementation since ComponentRegistry might not implement Debug
impl std::fmt::Debug for LoadingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingState::NotStarted => write!(f, "NotStarted"),
            LoadingState::Loading { progress, .. } => f
                .debug_struct("Loading")
                .field("progress", progress)
                .field("registry", &"Arc<Mutex<ComponentRegistry>>")
                .finish(),
            LoadingState::Completed { total, errors } => f
                .debug_struct("Completed")
                .field("total", total)
                .field("errors", errors)
                .finish(),
            LoadingState::Failed { error } => f.debug_struct("Failed").field("error", error).finish(),
        }
    }
}

impl LoadingState {
    /// Check if loading is in progress
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading { .. })
    }

    /// Check if loading is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            LoadingState::Completed { .. } | LoadingState::Failed { .. }
        )
    }
}

impl Default for LoadingState {
    fn default() -> Self {
        LoadingState::NotStarted
    }
}

/// Component loading progress tracker
///
/// Thread-safe progress tracking for background component loading.
/// Shared between the loading thread and the UI thread via Arc<Mutex<>>.
#[derive(Debug, Clone)]
pub struct ComponentLoadProgress {
    /// Total number of components to load
    pub total_components: usize,

    /// Number of components loaded so far
    pub loaded_count: usize,

    /// Name of the component currently being loaded
    pub current_component: Option<String>,

    /// List of errors encountered during loading
    pub errors: Vec<String>,
}

impl ComponentLoadProgress {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            total_components: 0,
            loaded_count: 0,
            current_component: None,
            errors: Vec::new(),
        }
    }

    /// Calculate loading progress as a percentage (0.0 - 1.0)
    pub fn percentage(&self) -> f32 {
        if self.total_components == 0 {
            0.0
        } else {
            self.loaded_count as f32 / self.total_components as f32
        }
    }

    /// Check if loading is complete
    pub fn is_complete(&self) -> bool {
        self.loaded_count >= self.total_components && self.total_components > 0
    }

    /// Get a summary string of current progress
    pub fn summary(&self) -> String {
        format!(
            "{} / {} ({:.0}%)",
            self.loaded_count,
            self.total_components,
            self.percentage() * 100.0
        )
    }

    /// Record a successful component load
    pub fn record_success(&mut self, component_name: String) {
        self.loaded_count += 1;
        log::debug!(
            "Component loaded: {} ({}/{})",
            component_name,
            self.loaded_count,
            self.total_components
        );
    }

    /// Record a failed component load
    pub fn record_error(&mut self, component_name: String, error: String) {
        self.loaded_count += 1;
        let error_msg = format!("{}: {}", component_name, error);
        self.errors.push(error_msg.clone());
        log::warn!(
            "Component load failed: {} ({}/{})",
            error_msg,
            self.loaded_count,
            self.total_components
        );
    }

    /// Set the component currently being loaded
    pub fn set_current(&mut self, component_name: Option<String>) {
        self.current_component = component_name;
    }
}

impl Default for ComponentLoadProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_state_default() {
        let state = LoadingState::default();
        assert!(matches!(state, LoadingState::NotStarted));
        assert!(!state.is_loading());
        assert!(!state.is_complete());
    }

    #[test]
    fn test_loading_state_loading() {
        let progress = Arc::new(Mutex::new(ComponentLoadProgress::new()));
        let registry = Arc::new(Mutex::new(ComponentRegistry::new()));
        let state = LoadingState::Loading {
            progress: progress.clone(),
            registry,
        };

        assert!(state.is_loading());
        assert!(!state.is_complete());
    }

    #[test]
    fn test_loading_state_completed() {
        let state = LoadingState::Completed {
            total: 10,
            errors: vec![],
        };

        assert!(!state.is_loading());
        assert!(state.is_complete());
    }

    #[test]
    fn test_loading_state_failed() {
        let state = LoadingState::Failed {
            error: "Test error".to_string(),
        };

        assert!(!state.is_loading());
        assert!(state.is_complete());
    }

    #[test]
    fn test_progress_percentage() {
        let mut progress = ComponentLoadProgress::new();

        // Empty progress
        assert_eq!(progress.percentage(), 0.0);

        // Set total
        progress.total_components = 10;
        assert_eq!(progress.percentage(), 0.0);

        // Load some components
        progress.loaded_count = 5;
        assert_eq!(progress.percentage(), 0.5);

        // Load all
        progress.loaded_count = 10;
        assert_eq!(progress.percentage(), 1.0);
    }

    #[test]
    fn test_progress_is_complete() {
        let mut progress = ComponentLoadProgress::new();

        assert!(!progress.is_complete());

        progress.total_components = 10;
        assert!(!progress.is_complete());

        progress.loaded_count = 5;
        assert!(!progress.is_complete());

        progress.loaded_count = 10;
        assert!(progress.is_complete());
    }

    #[test]
    fn test_progress_summary() {
        let mut progress = ComponentLoadProgress::new();
        progress.total_components = 76;
        progress.loaded_count = 45;

        let summary = progress.summary();
        assert!(summary.contains("45"));
        assert!(summary.contains("76"));
        assert!(summary.contains("59%")); // 45/76 ≈ 59%
    }

    #[test]
    fn test_record_success() {
        let mut progress = ComponentLoadProgress::new();
        progress.total_components = 10;

        progress.record_success("echo".to_string());
        assert_eq!(progress.loaded_count, 1);
        assert_eq!(progress.errors.len(), 0);

        progress.record_success("trim".to_string());
        assert_eq!(progress.loaded_count, 2);
    }

    #[test]
    fn test_record_error() {
        let mut progress = ComponentLoadProgress::new();
        progress.total_components = 10;

        progress.record_error("bad-component".to_string(), "Parse error".to_string());
        assert_eq!(progress.loaded_count, 1);
        assert_eq!(progress.errors.len(), 1);
        assert!(progress.errors[0].contains("bad-component"));
        assert!(progress.errors[0].contains("Parse error"));
    }

    #[test]
    fn test_thread_safe_progress() {
        let progress = Arc::new(Mutex::new(ComponentLoadProgress::new()));

        {
            let mut p = progress.lock().unwrap();
            p.total_components = 100;
        }

        // Simulate multiple threads updating
        let progress_clone = progress.clone();
        let handle = std::thread::spawn(move || {
            for i in 0..50 {
                let mut p = progress_clone.lock().unwrap();
                p.record_success(format!("component_{}", i));
            }
        });

        handle.join().unwrap();

        let p = progress.lock().unwrap();
        assert_eq!(p.loaded_count, 50);
    }

    #[test]
    fn test_set_current_component() {
        let mut progress = ComponentLoadProgress::new();

        assert_eq!(progress.current_component, None);

        progress.set_current(Some("echo.wasm".to_string()));
        assert_eq!(progress.current_component, Some("echo.wasm".to_string()));

        progress.set_current(None);
        assert_eq!(progress.current_component, None);
    }
}
