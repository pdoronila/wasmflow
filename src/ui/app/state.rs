//! State management for WasmFlow application
//!
//! This module handles file operations, undo/redo, and graph lifecycle management.

use super::WasmFlowApp;
use crate::graph::graph::NodeGraph;
use std::path::PathBuf;

impl WasmFlowApp {
    /// Undo the last command
    pub(super) fn undo(&mut self) {
        match self.history.undo(&mut self.graph) {
            Ok(()) => {
                self.status_message = "Undone".to_string();
                self.error_message = None;
                self.dirty = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Cannot undo: {}", e));
            }
        }
    }

    /// Redo the last undone command
    pub(super) fn redo(&mut self) {
        match self.history.redo(&mut self.graph) {
            Ok(()) => {
                self.status_message = "Redone".to_string();
                self.error_message = None;
                self.dirty = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Cannot redo: {}", e));
            }
        }
    }

    /// Save the current graph to a file
    pub(super) fn save_graph(&mut self) {
        if let Some(path) = &self.current_file {
            // Save to existing file
            match self.graph.save_to_file(path) {
                Ok(()) => {
                    self.status_message = format!("Saved to {}", path.display());
                    self.error_message = None;
                    self.dirty = false;
                    self.add_recent_file(path.clone());
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save: {}", e));
                }
            }
        } else {
            // No current file, show save dialog
            self.save_graph_as();
        }
    }

    /// Save the graph to a new file (Save As)
    pub(super) fn save_graph_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WasmFlow Graph", &["wasmflow"])
            .set_file_name("graph.wasmflow")
            .save_file()
        {
            match self.graph.save_to_file(&path) {
                Ok(()) => {
                    self.status_message = format!("Saved to {}", path.display());
                    self.error_message = None;
                    self.current_file = Some(path.clone());
                    self.dirty = false;
                    self.add_recent_file(path);
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save: {}", e));
                }
            }
        }
    }

    /// Load a graph from a file
    pub(super) fn load_graph(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WasmFlow Graph", &["wasmflow"])
            .pick_file()
        {
            match NodeGraph::load_from_file(&path) {
                Ok(graph) => {
                    log::info!("Successfully deserialized graph from {}", path.display());
                    log::info!(
                        "Graph has {} nodes and {} connections",
                        graph.nodes.len(),
                        graph.connections.len()
                    );

                    self.graph = graph;
                    self.current_file = Some(path.clone());
                    self.dirty = false;
                    self.history = crate::graph::command::CommandHistory::new();
                    self.status_message = format!("Loaded {}", path.display());
                    self.error_message = None;
                    self.add_recent_file(path.clone());

                    // Mark canvas dirty to force re-sync with loaded graph
                    self.canvas.mark_dirty();

                    log::info!("Graph loaded successfully from {}", path.display());
                }
                Err(e) => {
                    // Log full error chain for debugging
                    log::error!("Failed to load graph from {}", path.display());
                    log::error!("Error chain: {:#}", e);

                    // Provide user-friendly error messages for common issues
                    let error_msg = if e.to_string().contains("magic bytes") {
                        "Invalid file format: This is not a WasmFlow graph file.".to_string()
                    } else if e.to_string().contains("version") {
                        format!("Incompatible file version: {}. Please upgrade WasmFlow.", e)
                    } else if e.to_string().contains("Checksum mismatch") {
                        "File appears to be corrupted. Please try another file.".to_string()
                    } else {
                        format!("Failed to load graph: {:#}", e)
                    };
                    self.error_message = Some(error_msg);
                }
            }
        }
    }

    /// Create a new graph (with confirmation if current graph has unsaved changes)
    pub(super) fn new_graph(&mut self) {
        if self.dirty {
            // Show unsaved changes dialog
            self.unsaved_changes_dialog.open();
            self.pending_action = Some(super::PendingAction::NewGraph);
            return;
        }

        // Create new graph immediately if no unsaved changes
        self.create_new_graph();
    }

    /// Actually create a new graph (called after confirmation or if no unsaved changes)
    pub(super) fn create_new_graph(&mut self) {
        self.graph = NodeGraph::new("Untitled Graph".to_string(), "User".to_string());
        self.current_file = None;
        self.dirty = false;
        self.history = crate::graph::command::CommandHistory::new();
        self.status_message = "New graph created".to_string();
        self.error_message = None;
    }

    /// Get the path to the recent files configuration file
    fn recent_files_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("wasmflow");
            path.push("recent_files.json");
            path
        })
    }

    /// Load recent files from config
    pub(super) fn load_recent_files() -> Vec<PathBuf> {
        if let Some(path) = Self::recent_files_path() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(files) = serde_json::from_str::<Vec<PathBuf>>(&contents) {
                    // Filter out files that no longer exist
                    return files.into_iter().filter(|f| f.exists()).collect();
                }
            }
        }
        Vec::new()
    }

    /// Save recent files to config
    fn save_recent_files(&self) {
        if let Some(path) = Self::recent_files_path() {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            // Save recent files
            if let Ok(contents) = serde_json::to_string_pretty(&self.recent_files) {
                let _ = std::fs::write(&path, contents);
            }
        }
    }

    /// Add a file to the recent files list
    pub(super) fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);

        // Add to front
        self.recent_files.insert(0, path);

        // Keep only last 10
        self.recent_files.truncate(10);

        // Save to config
        self.save_recent_files();
    }

    /// Open a file from the recent files list
    pub(super) fn open_recent_file(&mut self, path: PathBuf) {
        if !path.exists() {
            self.error_message = Some(format!("File not found: {}", path.display()));
            // Remove from recent files
            self.recent_files.retain(|p| p != &path);
            self.save_recent_files();
            return;
        }

        match NodeGraph::load_from_file(&path) {
            Ok(graph) => {
                self.graph = graph;
                self.current_file = Some(path.clone());
                self.dirty = false;
                self.history = crate::graph::command::CommandHistory::new();
                self.status_message = format!("Loaded {}", path.display());
                self.error_message = None;
                self.add_recent_file(path.clone());

                // Mark canvas dirty to force re-sync with loaded graph
                self.canvas.mark_dirty();

                log::info!("Graph loaded successfully from {}", path.display());
            }
            Err(e) => {
                // Provide user-friendly error messages for common issues
                let error_msg = if e.to_string().contains("magic bytes") {
                    "Invalid file format: This is not a WasmFlow graph file.".to_string()
                } else if e.to_string().contains("version") {
                    format!("Incompatible file version: {}. Please upgrade WasmFlow.", e)
                } else if e.to_string().contains("Checksum mismatch") {
                    "File appears to be corrupted. Please try another file.".to_string()
                } else {
                    format!("Failed to load graph: {}", e)
                };
                self.error_message = Some(error_msg);
            }
        }
    }

    /// T099: Load a graph from a specific path (for CLI support)
    pub fn load_graph_from_path(&mut self, path: PathBuf) {
        match NodeGraph::load_from_file(&path) {
            Ok(graph) => {
                self.graph = graph;
                self.current_file = Some(path.clone());
                self.dirty = false;
                self.history = crate::graph::command::CommandHistory::new();
                self.status_message = format!("Loaded {}", path.display());
                self.error_message = None;
                self.add_recent_file(path.clone());

                // Mark canvas dirty to force re-sync with loaded graph
                self.canvas.mark_dirty();

                log::info!("Graph loaded successfully from {}", path.display());
            }
            Err(e) => {
                let error_msg = format!("Failed to load graph: {:#}", e);
                self.error_message = Some(error_msg.clone());
                log::error!("Failed to load graph from {}: {:#}", path.display(), e);
            }
        }
    }
}
