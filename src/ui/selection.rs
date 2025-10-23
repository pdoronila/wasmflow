//! Rectangle selection state management
//!
//! This module handles the state and logic for selecting multiple nodes
//! via click-and-drag rectangle selection on the canvas.

use egui::{Pos2, Rect};
use std::collections::HashSet;

/// Node identifier type (re-export from graph module)
pub type NodeId = uuid::Uuid;

/// Canvas interaction mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasMode {
    /// Normal mode: nodes can be dragged, canvas can be panned
    Normal,
    /// Selection mode: rectangle selection is active, node dragging disabled
    Selection,
}

/// Manages rectangle selection interaction state
pub struct SelectionState {
    /// Initial mouse position when drag started (None if not dragging)
    start_pos: Option<Pos2>,
    /// Current mouse position during drag (None if not dragging)
    current_pos: Option<Pos2>,
    /// Set of currently selected node IDs
    selected_nodes: HashSet<NodeId>,
    /// Whether user is actively dragging selection rectangle
    is_dragging: bool,
}

impl SelectionState {
    /// Create a new empty selection state
    pub fn new() -> Self {
        Self {
            start_pos: None,
            current_pos: None,
            selected_nodes: HashSet::new(),
            is_dragging: false,
        }
    }

    /// Start a new rectangle selection drag
    pub fn start_drag(&mut self, pos: Pos2) {
        self.start_pos = Some(pos);
        self.current_pos = Some(pos);
        self.is_dragging = true;
    }

    /// Update current drag position
    pub fn update_drag(&mut self, pos: Pos2) {
        if self.is_dragging {
            self.current_pos = Some(pos);
        }
    }

    /// Finish drag and finalize selection with given nodes
    pub fn end_drag(&mut self, nodes: HashSet<NodeId>) {
        // Check minimum rectangle size (5x5 pixels) before finalizing selection
        if let Some(rect) = self.get_selection_rect() {
            if rect.width().abs() >= 5.0 && rect.height().abs() >= 5.0 {
                self.selected_nodes = nodes;
            }
        }
        self.start_pos = None;
        self.current_pos = None;
        self.is_dragging = false;
    }

    /// Cancel current drag without updating selection
    pub fn cancel_drag(&mut self) {
        self.start_pos = None;
        self.current_pos = None;
        self.is_dragging = false;
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Get selection rectangle (if dragging)
    pub fn get_selection_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) = (self.start_pos, self.current_pos) {
            Some(Rect::from_two_pos(start, current))
        } else {
            None
        }
    }

    /// Get currently selected nodes
    pub fn selected_nodes(&self) -> &HashSet<NodeId> {
        &self.selected_nodes
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }
}

impl Default for SelectionState {
    fn default() -> Self {
        Self::new()
    }
}
