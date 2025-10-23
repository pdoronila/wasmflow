# Quick Start: Rectangle Selection & Node Composition

**Feature**: 007-rectangle-selection-tool
**Target Audience**: Developers implementing this feature
**Estimated Reading Time**: 10 minutes

## Overview

This quick start guide provides the essential steps to implement rectangle selection and WebAssembly component composition in the WasmFlow visual node editor.

## Prerequisites

- Rust 1.75+ with wasm32-wasip2 target installed
- Familiarity with egui UI framework
- Understanding of WebAssembly Component Model basics
- WasmFlow codebase checked out on branch `007-rectangle-selection-tool`

## Step 1: Add Dependencies (5 minutes)

Add WAC composition support to `Cargo.toml`:

```toml
[dependencies]
# Add this line to existing dependencies
wac-graph = "0.8"
```

Run `cargo check` to ensure dependencies resolve:

```bash
cargo check
```

## Step 2: Create Selection State Module (15 minutes)

Create `src/ui/selection.rs` to manage rectangle selection state:

```rust
use egui::{Pos2, Rect, Color32};
use std::collections::HashSet;
use crate::graph::NodeId;

/// Manages rectangle selection state
pub struct SelectionState {
    /// Start position of drag (None if not dragging)
    start_pos: Option<Pos2>,

    /// Current mouse position during drag
    current_pos: Option<Pos2>,

    /// Set of currently selected node IDs
    selected_nodes: HashSet<NodeId>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            start_pos: None,
            current_pos: None,
            selected_nodes: HashSet::new(),
        }
    }

    /// Start a new rectangle selection
    pub fn start_drag(&mut self, pos: Pos2) {
        self.start_pos = Some(pos);
        self.current_pos = Some(pos);
    }

    /// Update current drag position
    pub fn update_drag(&mut self, pos: Pos2) {
        if self.is_dragging() {
            self.current_pos = Some(pos);
        }
    }

    /// Finish drag and finalize selection
    pub fn end_drag(&mut self, nodes: HashSet<NodeId>) {
        self.start_pos = None;
        self.current_pos = None;
        self.selected_nodes = nodes;
    }

    /// Cancel current drag
    pub fn cancel_drag(&mut self) {
        self.start_pos = None;
        self.current_pos = None;
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.start_pos.is_some()
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
```

Don't forget to add to `src/ui/mod.rs`:

```rust
pub mod selection;
```

## Step 3: Integrate Selection into Canvas (30 minutes)

Modify `src/ui/canvas.rs` to add selection state and mouse handling:

```rust
use crate::ui::selection::SelectionState;

pub struct Canvas {
    // ... existing fields
    selection: SelectionState,
}

impl Canvas {
    // In the update/render method:
    pub fn update(&mut self, ui: &mut egui::Ui) {
        // Handle mouse input for selection
        let response = ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::click_and_drag());

        // Start drag on empty canvas
        if response.drag_started() && !self.is_over_node(response.interact_pointer_pos()) {
            if let Some(pos) = response.interact_pointer_pos() {
                self.selection.start_drag(pos);
            }
        }

        // Update drag position
        if response.dragged() && self.selection.is_dragging() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.selection.update_drag(pos);
            }
        }

        // End drag
        if response.drag_stopped() && self.selection.is_dragging() {
            if let Some(rect) = self.selection.get_selection_rect() {
                let selected = self.find_nodes_in_rect(rect);
                self.selection.end_drag(selected);
            }
        }

        // Cancel on ESC
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selection.cancel_drag();
        }

        // Render selection rectangle
        if let Some(rect) = self.selection.get_selection_rect() {
            self.render_selection_rectangle(ui, rect);
        }
    }

    fn render_selection_rectangle(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter();

        // Semi-transparent fill
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(100, 150, 200, 50)
        );

        // Border stroke
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.5, Color32::from_rgb(100, 150, 200))
        );
    }

    fn find_nodes_in_rect(&self, rect: Rect) -> HashSet<NodeId> {
        self.graph.nodes.iter()
            .filter(|(_, node)| {
                let node_center = node.position + (node.size / 2.0);
                rect.contains(node_center)
            })
            .map(|(id, _)| *id)
            .collect()
    }
}
```

## Step 4: Create Composition Service (45 minutes)

Create `src/runtime/composer.rs`:

```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wac_graph::{CompositionGraph, EncodeOptions, Package};

pub struct ComponentComposer {
    // Future: add caching
}

impl ComponentComposer {
    pub fn new() -> Self {
        Self {}
    }

    /// Compose multiple components into one
    pub fn compose(&self, socket: &Path, plugs: &[&Path]) -> Result<Vec<u8>> {
        log::info!("Composing {} plugs into socket", plugs.len());

        let mut graph = CompositionGraph::new();

        // Register socket component
        let socket_pkg = Package::from_file(
            "wasmflow:socket",
            None,
            socket,
            graph.types_mut()
        ).context("Failed to load socket component")?;

        let socket_id = graph.register_package(socket_pkg)?;

        // Register and plug dependencies
        for (idx, plug_path) in plugs.iter().enumerate() {
            let plug_pkg = Package::from_file(
                &format!("wasmflow:plug-{}", idx),
                None,
                plug_path,
                graph.types_mut()
            )?;

            let plug_id = graph.register_package(plug_pkg)?;

            wac_graph::plug(&mut graph, socket_id, plug_id)
                .with_context(|| format!("Failed to plug component {}", plug_path.display()))?;
        }

        // Encode with validation
        let bytes = graph.encode(EncodeOptions {
            validate: true,
            ..Default::default()
        })?;

        log::info!("Composition successful: {} bytes", bytes.len());
        Ok(bytes)
    }
}
```

Add to `src/runtime/mod.rs`:

```rust
pub mod composer;
```

## Step 5: Add Graph Validation (20 minutes)

Create `src/graph/validation.rs`:

```rust
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Dfs;
use std::collections::{HashMap, HashSet};
use crate::graph::{NodeId, NodeGraph};
use anyhow::Result;

/// Check if selected nodes form a connected subgraph
pub fn is_connected_subgraph(graph: &NodeGraph, node_ids: &[NodeId]) -> Result<bool> {
    if node_ids.len() < 2 {
        return Ok(false);
    }

    // Build petgraph from node connections
    let mut pg = Graph::<NodeId, ()>::new();
    let mut index_map = HashMap::new();

    // Add nodes
    for id in node_ids {
        let idx = pg.add_node(*id);
        index_map.insert(*id, idx);
    }

    // Add edges (only between selected nodes)
    for edge in &graph.edges {
        if node_ids.contains(&edge.from) && node_ids.contains(&edge.to) {
            let from_idx = index_map[&edge.from];
            let to_idx = index_map[&edge.to];
            pg.add_edge(from_idx, to_idx, ());
        }
    }

    // DFS from first node
    let start_idx = index_map[&node_ids[0]];
    let mut dfs = Dfs::new(&pg, start_idx);
    let mut visited = HashSet::new();

    while let Some(nx) = dfs.next(&pg) {
        visited.insert(nx);
    }

    // All nodes must be reachable
    Ok(visited.len() == node_ids.len())
}
```

## Step 6: Wire Everything Together (30 minutes)

In `src/ui/app.rs`, add composition action:

```rust
use crate::runtime::composer::ComponentComposer;
use crate::graph::validation::is_connected_subgraph;

pub struct App {
    // ... existing fields
    composer: ComponentComposer,
}

impl App {
    pub fn handle_compose_action(&mut self) -> Result<()> {
        let selected: Vec<NodeId> = self.canvas.selection.selected_nodes()
            .iter()
            .copied()
            .collect();

        // Validate selection
        if selected.len() < 2 {
            anyhow::bail!("Select at least 2 nodes to compose");
        }

        if !is_connected_subgraph(&self.graph, &selected)? {
            anyhow::bail!("Selected nodes must form a connected subgraph");
        }

        // Extract component paths
        let socket_node = &self.graph.nodes[&selected[0]];
        let socket_path = self.get_component_path(socket_node)?;

        let plug_paths: Vec<PathBuf> = selected[1..]
            .iter()
            .map(|id| self.get_component_path(&self.graph.nodes[id]))
            .collect::<Result<Vec<_>>>()?;

        let plug_refs: Vec<_> = plug_paths.iter().map(|p| p.as_path()).collect();

        // Compose
        let composed_bytes = self.composer.compose(&socket_path, &plug_refs)?;

        // Create new composite node
        // (Implementation depends on your NodeGraph structure)
        let new_node_id = self.graph.add_composite_node(composed_bytes)?;

        // Remove original nodes
        for id in selected {
            self.graph.remove_node(id);
        }

        Ok(())
    }
}
```

## Step 7: Add UI Button (10 minutes)

In your toolbar rendering:

```rust
// In toolbar render method
let can_compose = self.canvas.selection.selected_nodes().len() >= 2;

ui.add_enabled_ui(can_compose, |ui| {
    if ui.button("🔗 Compose").clicked() {
        if let Err(e) = self.handle_compose_action() {
            self.show_error(&format!("Composition failed: {}", e));
        }
    }
});
```

## Step 8: Testing (20 minutes)

Create `tests/integration/composition_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_composition() {
        let composer = ComponentComposer::new();

        let socket = Path::new("examples/test-socket.wasm");
        let plug = Path::new("examples/test-plug.wasm");

        let result = composer.compose(socket, &[plug]);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_validation_rejects_single_node() {
        let graph = create_test_graph();
        let nodes = vec![NodeId::new()];

        assert!(!is_connected_subgraph(&graph, &nodes).unwrap());
    }

    #[test]
    fn test_validation_rejects_disconnected() {
        let graph = create_test_graph_disconnected();
        let nodes = vec![NodeId::new(), NodeId::new()];

        assert!(!is_connected_subgraph(&graph, &nodes).unwrap());
    }
}
```

Run tests:

```bash
cargo test composition
```

## Step 9: Manual Testing (15 minutes)

1. Build and run WasmFlow:
   ```bash
   cargo run
   ```

2. Load a graph with multiple WASM component nodes

3. Test rectangle selection:
   - Click and drag on empty canvas
   - Verify rectangle appears
   - Release mouse, verify nodes are selected

4. Test composition:
   - Select 2+ connected nodes
   - Click "Compose" button
   - Verify new composite node appears
   - Verify original nodes are removed

5. Test error cases:
   - Select 1 node → should see error
   - Select disconnected nodes → should see error

## Common Issues & Solutions

### Issue: Selection rectangle not appearing

**Solution**: Check that mouse events are being captured. Add debug logging:
```rust
log::debug!("Mouse down at {:?}", pos);
```

### Issue: Composition fails with "NoPlugHappened"

**Solution**: Components don't have matching imports/exports. Verify WIT interfaces are compatible.

### Issue: Performance lag during selection

**Solution**: Ensure hit testing only runs on mouse move, not every frame. Consider spatial indexing for >100 nodes.

## Next Steps

After completing this quick start:

1. **Add drill-down view**: Implement `ViewStack` and `ViewContext` from data-model.md
2. **Add composite node styling**: Distinguish composite nodes visually
3. **Add composition preview**: Show which imports will be satisfied before composing
4. **Add error recovery**: Better error messages and recovery options
5. **Add caching**: Implement composition caching for repeated operations

## Resources

- Full feature spec: `specs/007-rectangle-selection-tool/spec.md`
- Research findings: `specs/007-rectangle-selection-tool/research.md`
- Data model: `specs/007-rectangle-selection-tool/data-model.md`
- WAC documentation: https://github.com/bytecodealliance/wac
- egui documentation: https://docs.rs/egui/0.33.0

## Timeline

Total estimated implementation time: **~3-4 hours** for basic functionality

- Step 1-2: 20 minutes (dependencies + selection state)
- Step 3: 30 minutes (canvas integration)
- Step 4-5: 65 minutes (composition service + validation)
- Step 6-7: 40 minutes (wiring + UI)
- Step 8-9: 35 minutes (testing)

Add 2-3 more hours for drill-down view, polish, and advanced error handling.
