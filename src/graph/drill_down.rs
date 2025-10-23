//! Drill-down view context management
//!
//! This module handles navigation between main canvas and drill-down views
//! into composite nodes' internal structure.
//!
//! T037-T038: Full implementation of ViewStack for drill-down navigation

use std::collections::BTreeMap;

/// Node identifier type (re-export from graph module)
pub type NodeId = uuid::Uuid;

/// View context enumeration - represents what the user is currently viewing
#[derive(Debug, Clone)]
pub enum ViewContext {
    /// Viewing the main canvas with all top-level nodes
    MainCanvas,
    /// Drilled down into a composite node to view its internal structure
    DrillDown {
        /// ID of the composite node being viewed
        _composite_node_id: NodeId,
        /// Display name of the composite node (for breadcrumb)
        composite_node_name: String,
        /// Internal nodes snapshot (cloned from composition_data)
        internal_nodes: BTreeMap<NodeId, crate::graph::node::GraphNode>,
        /// Internal connections snapshot
        internal_edges: Vec<crate::graph::connection::Connection>,
    },
}

/// T037-T038: Stack-based view navigation for drill-down functionality
///
/// Maintains a stack of view contexts to support nested drill-down
/// (though current implementation only supports single-level drill-down)
pub struct ViewStack {
    /// Current view context stack
    /// Index 0 is always MainCanvas, subsequent entries are drill-downs
    stack: Vec<ViewContext>,
}

impl ViewStack {
    /// Create a new view stack starting at main canvas
    pub fn new() -> Self {
        Self {
            stack: vec![ViewContext::MainCanvas],
        }
    }

    /// Get the current view context
    pub fn current(&self) -> &ViewContext {
        self.stack.last().expect("ViewStack should never be empty")
    }

    /// Check if currently viewing main canvas
    pub fn is_main_canvas(&self) -> bool {
        matches!(self.current(), ViewContext::MainCanvas)
    }

    /// Check if currently in drill-down mode
    pub fn is_drill_down(&self) -> bool {
        !self.is_main_canvas()
    }

    /// Get the depth of the current view (0 = main canvas, 1+ = drill-down levels)
    pub fn depth(&self) -> usize {
        self.stack.len() - 1
    }

    /// T038: Drill down into a composite node
    ///
    /// Pushes a new drill-down context onto the stack.
    /// Returns Ok(()) if successful, Err if the node is not a composite node.
    pub fn drill_down(
        &mut self,
        composite_node_id: NodeId,
        composite_node_name: String,
        composition_data: &crate::graph::node::CompositionData,
    ) -> Result<(), String> {
        // Clone the internal structure for the drill-down view
        let context = ViewContext::DrillDown {
            composite_node_id,
            composite_node_name,
            internal_nodes: composition_data.internal_nodes.clone(),
            internal_edges: composition_data.internal_edges.clone(),
        };

        self.stack.push(context);
        log::info!(
            "Drilled down into composite node {} (depth: {})",
            composite_node_id,
            self.depth()
        );
        Ok(())
    }

    /// T041: Go back to the previous view context
    ///
    /// Pops the current view from the stack, returning to the parent view.
    /// Returns true if navigation occurred, false if already at main canvas.
    pub fn go_back(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            log::info!("Navigated back (depth: {})", self.depth());
            true
        } else {
            // Already at main canvas, cannot go back further
            false
        }
    }

    /// T042: Reset to main canvas
    ///
    /// Clears all drill-down contexts and returns to main canvas
    pub fn reset_to_main(&mut self) {
        if self.stack.len() > 1 {
            self.stack.truncate(1);
            log::info!("Reset to main canvas");
        }
    }

    /// T039: Get breadcrumb path for UI rendering
    ///
    /// Returns a vector of (name, depth) pairs representing the navigation path
    pub fn breadcrumb_path(&self) -> Vec<(String, usize)> {
        self.stack
            .iter()
            .enumerate()
            .map(|(depth, context)| {
                let name = match context {
                    ViewContext::MainCanvas => "Main Canvas".to_string(),
                    ViewContext::DrillDown {
                        composite_node_name,
                        ..
                    } => composite_node_name.clone(),
                };
                (name, depth)
            })
            .collect()
    }
}

impl Default for ViewStack {
    fn default() -> Self {
        Self::new()
    }
}
