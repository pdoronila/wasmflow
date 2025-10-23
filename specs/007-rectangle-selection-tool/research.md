# Research: Rectangle Selection & WAC Composition Integration

**Feature**: Rectangle Selection Tool for Node Composition
**Branch**: 007-rectangle-selection-tool
**Date**: 2025-10-21

## Overview

This document consolidates research findings for implementing rectangle selection and WebAssembly component composition in the WasmFlow visual node editor.

## 1. WAC (WebAssembly Composition) Integration

### Decision: Use `wac-graph` Rust library for programmatic composition

**Rationale:**
- Native Rust API eliminates CLI subprocess overhead
- Type-safe composition with compile-time guarantees
- Fine-grained error handling and validation
- Seamless integration with existing wasmtime 27.0 infrastructure
- Better performance for visual node editor workflow

**Alternatives Considered:**
1. **WAC CLI via std::process::Command** - Rejected due to subprocess overhead (~10-50ms), text parsing complexity, and limited error handling
2. **Manual component composition with wasmtime** - Rejected due to complexity of implementing composition logic from scratch

### Implementation Approach

**Add to Cargo.toml:**
```toml
[dependencies]
wac-graph = "0.8"
```

**Core Composition Pattern:**
```rust
use wac_graph::{CompositionGraph, EncodeOptions, Package};

fn compose_components(socket: &Path, plugs: &[&Path]) -> Result<Vec<u8>> {
    let mut graph = CompositionGraph::new();

    // Register socket (main component)
    let socket_pkg = Package::from_file("socket", None, socket, graph.types_mut())?;
    let socket_id = graph.register_package(socket_pkg)?;

    // Register plugs (dependencies)
    for (idx, plug) in plugs.iter().enumerate() {
        let plug_pkg = Package::from_file(
            &format!("plug{}", idx), None, plug, graph.types_mut()
        )?;
        let plug_id = graph.register_package(plug_pkg)?;

        // Auto-wire compatible exports to imports
        wac_graph::plug(&mut graph, socket_id, plug_id)?;
    }

    // Encode composed component
    graph.encode(EncodeOptions { validate: true, ..Default::default() })
}
```

### Input/Output Formats

**Input Requirements:**
- Valid WebAssembly Component Model binaries (not core modules)
- Components must have WIT interfaces for imports/exports
- Compiled with `cargo component build` or `wasm-tools component new`

**Output:**
- Single composed component binary (.wasm)
- Socket component's exports become composed component's exports
- Plug components are fully encapsulated (not externally visible)
- Unresolved imports (not satisfied by plugs) remain as imports

### Error Handling Strategy

**Key Error Types:**
1. `EncodeError::ValidationFailure` - Component validation failed
2. `EncodeError::GraphContainsCycle` - Circular dependency detected
3. `PlugError::NoPlugHappened` - No compatible exports found
4. `RegisterPackageError` - Invalid component binary

**Validation Pattern:**
```rust
fn compose_with_validation(socket: &Path, plugs: &[&Path]) -> Result<Vec<u8>> {
    // 1. Validate files exist
    // 2. Register components with detailed error context
    // 3. Attempt composition with granular error handling
    // 4. Encode with validation enabled
    // 5. Optional: validate result with wasmtime
}
```

### Integration Architecture

```
┌─────────────────────────────────────┐
│   Visual Node Graph (egui-snarl)   │
│  - User selects nodes via rectangle │
│  - Displays components visually     │
└────────────┬────────────────────────┘
             │ Selection API
             ▼
┌─────────────────────────────────────┐
│   Composition Service               │
│  - Validates selection (subgraph)   │
│  - Calls wac-graph::plug()          │
│  - Generates composed .wasm         │
└────────────┬────────────────────────┘
             │ Binary output
             ▼
┌─────────────────────────────────────┐
│   Runtime Execution (wasmtime)      │
│  - Loads composed component         │
│  - Executes in graph                │
└─────────────────────────────────────┘
```

## 2. Rectangle Selection in egui

### Decision: Implement custom selection logic in canvas rendering layer

**Rationale:**
- egui provides low-level mouse event handling
- Custom rectangle drawing using `Painter` API
- Integration with existing egui-snarl node editor
- Supports real-time visual feedback (< 100ms requirement)

**Alternatives Considered:**
1. **egui-snarl built-in selection** - Currently doesn't support rectangle selection, would require forking library
2. **Third-party selection widget** - No suitable egui widgets available

### Implementation Pattern

**Selection State:**
```rust
pub struct SelectionState {
    /// Start position of rectangle drag
    start_pos: Option<Pos2>,
    /// Current mouse position during drag
    current_pos: Option<Pos2>,
    /// Currently selected node IDs
    selected_nodes: HashSet<NodeId>,
}
```

**Rendering Rectangle:**
```rust
fn render_selection_rectangle(painter: &Painter, rect: Rect) {
    // Semi-transparent fill
    painter.rect_filled(
        rect,
        0.0,
        Color32::from_rgba_unmultiplied(100, 150, 200, 50)
    );

    // Dashed border
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.5, Color32::from_rgb(100, 150, 200))
    );
}
```

**Hit Testing:**
```rust
fn nodes_in_rectangle(rect: Rect, nodes: &HashMap<NodeId, Node>) -> Vec<NodeId> {
    nodes.iter()
        .filter(|(_, node)| {
            let node_center = node.position + (node.size / 2.0);
            rect.contains(node_center)
        })
        .map(|(id, _)| *id)
        .collect()
}
```

## 3. Connected Subgraph Validation

### Decision: Use petgraph for graph connectivity analysis

**Rationale:**
- Already a project dependency (petgraph 0.6)
- Efficient O(V+E) traversal algorithms
- Well-tested graph algorithms library
- Supports both directed and undirected graph analysis

**Algorithm:**
```rust
use petgraph::visit::Dfs;
use petgraph::graph::NodeIndex;

fn is_connected_subgraph(graph: &Graph, nodes: &[NodeId]) -> bool {
    if nodes.is_empty() {
        return false;
    }

    // Create subgraph view
    let subgraph_indices: Vec<NodeIndex> = nodes.iter()
        .map(|id| graph.node_index(*id))
        .collect();

    // DFS from first node
    let mut dfs = Dfs::new(&graph, subgraph_indices[0]);
    let mut visited = HashSet::new();

    while let Some(nx) = dfs.next(&graph) {
        if subgraph_indices.contains(&nx) {
            visited.insert(nx);
        }
    }

    // All nodes must be reachable
    visited.len() == nodes.len()
}
```

## 4. Drill-Down View Context

### Decision: Implement view stack with breadcrumb navigation

**Rationale:**
- Supports future nested composition (if enabled)
- Clear user mental model (similar to folder navigation)
- Easy to implement undo/back navigation
- Maintains performance (only render active view)

**Data Structure:**
```rust
pub enum ViewContext {
    MainCanvas,
    DrillDown {
        composite_node_id: NodeId,
        internal_graph: NodeGraph,
    },
}

pub struct ViewStack {
    stack: Vec<ViewContext>,
}

impl ViewStack {
    pub fn drill_into(&mut self, node_id: NodeId, graph: NodeGraph) {
        self.stack.push(ViewContext::DrillDown {
            composite_node_id: node_id,
            internal_graph: graph,
        });
    }

    pub fn exit_drill_down(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn current(&self) -> &ViewContext {
        self.stack.last().unwrap()
    }
}
```

## 5. Composite Node Serialization

### Decision: Store internal nodes in BTreeMap for deterministic serialization

**Rationale:**
- Aligns with existing CLAUDE.md guideline: "Use BTreeMap for all serialized data structures"
- Enables CRC64 checksum validation
- Deterministic ordering for version control
- Performance difference negligible for <1000 nodes

**Data Model:**
```rust
#[derive(Serialize, Deserialize)]
pub struct CompositeNodeData {
    /// Name of the composite
    pub name: String,

    /// Socket component path
    pub socket_path: PathBuf,

    /// Plug component paths
    pub plug_paths: Vec<PathBuf>,

    /// Internal nodes (preserved for drill-down)
    /// BTreeMap for deterministic serialization
    pub internal_nodes: BTreeMap<NodeId, Node>,

    /// Internal connections
    pub internal_edges: Vec<Edge>,

    /// Exposed inputs (mapped to internal node inputs)
    pub exposed_inputs: BTreeMap<String, (NodeId, String)>,

    /// Exposed outputs (mapped to internal node outputs)
    pub exposed_outputs: BTreeMap<String, (NodeId, String)>,

    /// Cached composed WASM binary (not serialized)
    #[serde(skip)]
    pub cached_composition: Option<Vec<u8>>,
}
```

## 6. UI Visual Feedback

### Decision: Multi-layer rendering with distinct visual states

**Visual States:**
1. **Normal node**: Default theme styling
2. **Hovered (during selection)**: Preview highlight (semi-transparent overlay)
3. **Selected**: Persistent highlight (colored border)
4. **Composite node**: Distinct styling (different color scheme + footer badge)

**Color Palette:**
```rust
pub struct SelectionTheme {
    // During drag
    rectangle_fill: Color32::from_rgba_unmultiplied(100, 150, 200, 50),
    rectangle_stroke: Color32::from_rgb(100, 150, 200),

    // Preview (while dragging over nodes)
    preview_highlight: Color32::from_rgba_unmultiplied(100, 200, 100, 80),

    // Selected state
    selected_border: Color32::from_rgb(100, 200, 255),
    selected_border_width: 2.5,

    // Composite nodes
    composite_background: Color32::from_rgb(70, 60, 90),
    composite_border: Color32::from_rgb(140, 120, 180),
    composite_badge_color: Color32::from_rgb(180, 160, 220),
}
```

## 7. Performance Optimization

### Composition Performance

**Target**: <1 second for up to 10 nodes (per spec)

**Optimizations:**
1. **Async composition**: Offload to tokio blocking thread pool for >5 components
2. **Caching**: Hash-based composition cache (avoid re-composing identical sets)
3. **Validation shortcuts**: Quick type-check before full composition

**Cache Strategy:**
```rust
struct CompositionCache {
    cache: LruCache<CompositionKey, Vec<u8>>,
}

#[derive(Hash, Eq, PartialEq)]
struct CompositionKey {
    socket_hash: u64,
    plug_hashes: Vec<u64>,
}
```

### Selection Rendering Performance

**Target**: <100ms response to mouse movement (per spec)

**Optimizations:**
1. **Spatial indexing**: Use quadtree for large graphs (>100 nodes)
2. **Lazy hit testing**: Only test visible nodes
3. **Dirty rectangles**: Only repaint changed regions

## 8. Testing Strategy

### Unit Tests

1. **Composition validation**
   - Valid composition (connected subgraph)
   - Invalid composition (disconnected nodes)
   - Edge cases (2 nodes minimum, cycle detection)

2. **Selection logic**
   - Rectangle contains nodes
   - Rectangle partially overlaps
   - Empty selection

3. **Graph connectivity**
   - Connected subgraph validation
   - Disconnected subgraph detection

### Integration Tests

1. **End-to-end composition workflow**
   - Select nodes → compose → verify new node created
   - Drill into composite → verify internal structure
   - Execute composite node → verify outputs

2. **UI interaction tests**
   - Mouse drag creates selection rectangle
   - ESC cancels selection
   - Compose button enabled/disabled based on selection

### Performance Tests

1. **Composition benchmarks**
   - Measure composition time for 2, 5, 10 nodes
   - Verify <1 second requirement

2. **Selection rendering benchmarks**
   - Measure frame time during rectangle drag
   - Verify <100ms (equivalent to 10 FPS minimum)

## 9. Dependencies Summary

**New Dependencies:**
- `wac-graph = "0.8"` - WebAssembly composition

**Existing Dependencies (leveraged):**
- `egui 0.33` - UI framework, rectangle rendering
- `petgraph 0.6` - Graph connectivity validation
- `wasmtime 27.0` - WASM component execution
- `serde/bincode` - Composite node serialization
- `tokio` - Async composition for large component sets

## 10. Key Risks & Mitigations

### Risk 1: WAC Composition Complexity
**Impact**: High - Core feature functionality
**Mitigation**: Use well-tested `wac-graph` library, comprehensive error handling, validation before composition

### Risk 2: Performance Degradation
**Impact**: Medium - Could violate 60 FPS requirement
**Mitigation**: Async composition, caching, spatial indexing for large graphs

### Risk 3: Type Compatibility Issues
**Impact**: Medium - Composition failures frustrate users
**Mitigation**: Clear error messages, preview/validation before composition, visual compatibility indicators

### Risk 4: UI/UX Complexity
**Impact**: Low - Users might not understand drill-down
**Mitigation**: Clear breadcrumb navigation, tooltips, documentation

## References

- WAC GitHub: https://github.com/bytecodealliance/wac
- wac-graph docs: https://docs.rs/wac-graph/0.8.0
- Component Model: https://component-model.bytecodealliance.org
- egui documentation: https://docs.rs/egui/0.33.0
- petgraph documentation: https://docs.rs/petgraph/0.6.0
