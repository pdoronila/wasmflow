# Data Model: Rectangle Selection & Node Composition

**Feature**: 007-rectangle-selection-tool
**Date**: 2025-10-21

## Overview

This document defines the data structures and entities required for rectangle selection and WebAssembly component composition in the WasmFlow visual node editor.

## Core Entities

### 1. SelectionState

**Purpose**: Tracks the current rectangle selection interaction state

**Fields**:
- `start_position: Option<Pos2>` - Initial mouse position when drag started (None if not dragging)
- `current_position: Option<Pos2>` - Current mouse position during drag (None if not dragging)
- `selected_nodes: HashSet<NodeId>` - Set of currently selected node IDs
- `is_dragging: bool` - Whether user is actively dragging selection rectangle

**State Transitions**:
```
Idle (selected_nodes may contain previous selection)
  ↓ Mouse down on empty canvas
Dragging (start_position and current_position set)
  ↓ Mouse move (updates current_position)
Dragging (rectangle grows/shrinks)
  ↓ Mouse up
Idle (selected_nodes updated with nodes in rectangle)
  ↓ ESC key or click outside
Idle (selected_nodes cleared)
```

**Validation Rules**:
- `start_position` and `current_position` must both be Some or both be None
- `selected_nodes` can be empty (no selection)
- Minimum rectangle size for selection: 5×5 pixels (prevents accidental selection on click)

**Relationships**:
- References NodeId values from NodeGraph
- Used by Canvas rendering to draw selection rectangle
- Queried by composition service to determine which nodes to compose

---

### 2. CompositeNode

**Purpose**: Represents a composed WebAssembly component created from multiple nodes

**Fields**:
- `id: NodeId` - Unique identifier for this composite node
- `name: String` - User-visible name of the composite
- `socket_path: PathBuf` - Path to the socket (main) component file
- `plug_paths: Vec<PathBuf>` - Paths to plug (dependency) component files
- `internal_nodes: BTreeMap<NodeId, Node>` - Internal nodes preserved for drill-down (deterministic order)
- `internal_edges: Vec<Edge>` - Connections between internal nodes
- `exposed_inputs: BTreeMap<String, PortMapping>` - External inputs mapped to internal node inputs
- `exposed_outputs: BTreeMap<String, PortMapping>` - External outputs mapped to internal node outputs
- `component_metadata: CompositionMetadata` - Metadata about the composition
- `cached_composition: Option<Vec<u8>>` - Cached WASM binary (not serialized, runtime only)

**Validation Rules**:
- `internal_nodes` must contain at least 2 nodes (minimum for composition)
- All `NodeId` values in `internal_edges` must exist in `internal_nodes`
- `exposed_inputs` and `exposed_outputs` must map to valid internal node ports
- `socket_path` and all `plug_paths` must point to valid WASM component files
- Internal nodes must form a connected subgraph (validated before composition)

**Relationships**:
- Contains snapshot of Node entities
- Contains Edge entities for internal connections
- References file paths for component sources
- Parent-child relationship: CompositeNode contains internal Nodes

---

### 3. PortMapping

**Purpose**: Maps an external port (on composite node) to an internal node's port

**Fields**:
- `external_name: String` - Name of the port as exposed on the composite node
- `internal_node_id: NodeId` - ID of the internal node this port connects to
- `internal_port_name: String` - Name of the port on the internal node
- `port_type: PortType` - Data type of the port (for validation)

**Validation Rules**:
- `internal_node_id` must exist in parent CompositeNode's `internal_nodes`
- `internal_port_name` must be a valid port on the referenced internal node
- `port_type` must match the type of the internal port

**Relationships**:
- Owned by CompositeNode (in `exposed_inputs` or `exposed_outputs`)
- References Node (via `internal_node_id`)
- References Port (via `internal_port_name`)

---

### 4. CompositionMetadata

**Purpose**: Stores metadata about a composition for display and debugging

**Fields**:
- `created_at: DateTime<Utc>` - When the composition was created
- `component_count: usize` - Number of components composed (for quick display)
- `component_names: Vec<String>` - Names of all composed components (for footer display)
- `composition_size_bytes: usize` - Size of the composed WASM binary
- `composition_hash: u64` - Hash of the composition inputs (for cache invalidation)

**Validation Rules**:
- `component_count` must match `component_names.len()`
- `component_count` must be ≥ 2

**Relationships**:
- Owned by CompositeNode
- No references to other entities

---

### 5. ViewContext

**Purpose**: Manages the current view state (main canvas or drill-down into composite)

**Variants**:

```rust
pub enum ViewContext {
    /// Viewing the main canvas with all top-level nodes
    MainCanvas,

    /// Drilled down into a composite node
    DrillDown {
        /// ID of the composite node being viewed
        composite_node_id: NodeId,

        /// Reference to the composite node's internal graph
        internal_graph: NodeGraphView,
    },
}
```

**State Transitions**:
```
MainCanvas
  ↓ Double-click composite node
DrillDown(composite_node_id)
  ↓ Click "Exit" button or breadcrumb
MainCanvas
```

**Validation Rules**:
- In `DrillDown` state, `composite_node_id` must reference a valid CompositeNode
- `internal_graph` must be non-empty (composite must have internal nodes)

**Relationships**:
- References NodeId of CompositeNode when in DrillDown state
- Contains view-specific rendering state

---

### 6. ViewStack

**Purpose**: Maintains navigation history for drill-down views (supports future nested composition)

**Fields**:
- `stack: Vec<ViewContext>` - Stack of view contexts (last = current view)

**Operations**:
- `push(context: ViewContext)` - Enter a new view level
- `pop() -> Option<ViewContext>` - Exit current view level
- `current() -> &ViewContext` - Get current view without popping
- `clear()` - Return to main canvas (pop all)

**Validation Rules**:
- `stack` must always contain at least one element (MainCanvas as base)
- Cannot pop when `stack.len() == 1`

**Relationships**:
- Owns ViewContext entities
- Maintains chronological view history

---

### 7. NodeGraphView

**Purpose**: Read-only view of a node graph for drill-down display

**Fields**:
- `nodes: &BTreeMap<NodeId, Node>` - Reference to nodes (deterministic order)
- `edges: &[Edge]` - Reference to edges
- `viewport_center: Pos2` - Center of view (for panning)
- `zoom_level: f32` - Zoom level (1.0 = 100%)

**Validation Rules**:
- All `edges` must reference valid `nodes`
- `zoom_level` must be > 0.0

**Relationships**:
- References (not owns) Node and Edge data from CompositeNode
- Temporary view constructed for rendering

---

## Extended Entities (Modifications to Existing)

### Node (Extended)

**New Field**:
- `composition_spec: Option<CompositeNodeData>` - If this is a composite node, contains composition data

**Validation Rule**:
- If `composition_spec` is Some, the node is a composite and has different rendering

---

### ComponentSpec (New Variant)

**Purpose**: Extend existing ComponentSpec enum to support composite components

**New Variant**:
```rust
Composed {
    /// Reference to composite node data
    composite_data: Arc<CompositeNodeData>,

    /// Quick access to exposed interface
    exposed_inputs: Vec<PortDefinition>,
    exposed_outputs: Vec<PortDefinition>,
}
```

**Validation Rules**:
- `exposed_inputs` and `exposed_outputs` must match `composite_data` port mappings
- `composite_data` must contain valid composition

---

## Data Flow Diagrams

### Selection → Composition Flow

```
┌─────────────────┐
│ User drag mouse │
│  on canvas      │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ SelectionState updated  │
│  - start_position       │
│  - current_position     │
│  - is_dragging = true   │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Canvas renders          │
│ selection rectangle     │
└────────┬────────────────┘
         │
         ▼ Mouse up
┌─────────────────────────┐
│ SelectionState.         │
│ selected_nodes updated  │
│ (nodes in rectangle)    │
└────────┬────────────────┘
         │
         ▼ User clicks "Compose"
┌─────────────────────────┐
│ Validate selection:     │
│  - ≥2 nodes selected    │
│  - Forms connected      │
│    subgraph             │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Extract component paths │
│ from selected nodes     │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ WAC composition:        │
│  - wac_graph::plug()    │
│  - Generate binary      │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Create CompositeNode:   │
│  - Store internal nodes │
│  - Map exposed ports    │
│  - Cache composition    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Add to NodeGraph        │
│ Remove original nodes   │
└─────────────────────────┘
```

### Drill-Down Flow

```
┌──────────────────────┐
│ User double-clicks   │
│ composite node       │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────────┐
│ ViewStack.push(          │
│   DrillDown {            │
│     composite_node_id,   │
│     internal_graph       │
│   }                      │
│ )                        │
└──────────┬───────────────┘
           │
           ▼
┌──────────────────────────┐
│ Canvas switches to       │
│ render internal_graph    │
│ - Only internal nodes    │
│ - Only internal edges    │
│ - Hide external nodes    │
└──────────┬───────────────┘
           │
           ▼ User clicks "Exit"
┌──────────────────────────┐
│ ViewStack.pop()          │
│ Return to MainCanvas     │
└──────────────────────────┘
```

## Serialization Format

### CompositeNode JSON Structure

```json
{
  "id": "node-123",
  "name": "Data Processor Pipeline",
  "socket_path": "/path/to/socket.wasm",
  "plug_paths": [
    "/path/to/plug1.wasm",
    "/path/to/plug2.wasm"
  ],
  "internal_nodes": {
    "node-100": {
      "id": "node-100",
      "name": "Filter",
      "position": {"x": 100.0, "y": 100.0},
      "component_spec": { /* ... */ }
    },
    "node-101": {
      "id": "node-101",
      "name": "Map",
      "position": {"x": 300.0, "y": 100.0},
      "component_spec": { /* ... */ }
    }
  },
  "internal_edges": [
    {
      "from": "node-100",
      "from_port": "output",
      "to": "node-101",
      "to_port": "input"
    }
  ],
  "exposed_inputs": {
    "data": {
      "external_name": "data",
      "internal_node_id": "node-100",
      "internal_port_name": "input",
      "port_type": "list<u8>"
    }
  },
  "exposed_outputs": {
    "result": {
      "external_name": "result",
      "internal_node_id": "node-101",
      "internal_port_name": "output",
      "port_type": "list<u8>"
    }
  },
  "component_metadata": {
    "created_at": "2025-10-21T12:00:00Z",
    "component_count": 2,
    "component_names": ["Filter", "Map"],
    "composition_size_bytes": 45678,
    "composition_hash": 12345678901234567890
  }
}
```

**Note**: BTreeMap serializes to JSON object with sorted keys for deterministic output.

## Database/Storage Considerations

**File-based Storage**:
- Composite nodes are saved as part of the main graph JSON file
- `cached_composition` field is marked `#[serde(skip)]` - not persisted
- On load, composition is regenerated from `socket_path` and `plug_paths` if needed

**Performance**:
- BTreeMap ensures deterministic serialization order
- CRC64 checksum can be computed on stable JSON output
- Composition cache invalidation via `composition_hash` comparison

## Relationships Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     NodeGraph                           │
│                                                         │
│  ┌─────────────────────────────────────────────┐       │
│  │            nodes: BTreeMap<NodeId, Node>    │       │
│  │                                             │       │
│  │  ┌──────────────────────────────────────┐  │       │
│  │  │  Node                               │  │       │
│  │  │   - id: NodeId                      │  │       │
│  │  │   - composition_spec: Option<...>   │  │       │
│  │  │                                     │  │       │
│  │  │   If Some:                          │  │       │
│  │  │   ┌──────────────────────────────┐ │  │       │
│  │  │   │ CompositeNode                │ │  │       │
│  │  │   │  - internal_nodes: BTreeMap  │ │  │       │
│  │  │   │  - internal_edges: Vec       │ │  │       │
│  │  │   │  - exposed_inputs: BTreeMap  │ │  │       │
│  │  │   │  - exposed_outputs: BTreeMap │ │  │       │
│  │  │   │  - metadata: Metadata        │ │  │       │
│  │  │   └──────────────────────────────┘ │  │       │
│  │  └──────────────────────────────────┘  │       │
│  └─────────────────────────────────────────────┘       │
│                                                         │
│  ┌─────────────────────────────────────────────┐       │
│  │        edges: Vec<Edge>                     │       │
│  └─────────────────────────────────────────────┘       │
│                                                         │
│  Used by:                                               │
│  ┌─────────────────────────────────────────────┐       │
│  │   ViewStack                                 │       │
│  │    - stack: Vec<ViewContext>                │       │
│  │                                             │       │
│  │    ViewContext::MainCanvas                  │       │
│  │      (renders full NodeGraph)               │       │
│  │                                             │       │
│  │    ViewContext::DrillDown                   │       │
│  │      (renders CompositeNode.internal_nodes) │       │
│  └─────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                  SelectionState                         │
│   - selected_nodes: HashSet<NodeId> ────────────────┐   │
│                                                     │   │
│   References nodes in NodeGraph                    │   │
└─────────────────────────────────────────────────────┼───┘
                                                      │
                                                      │
                                                      ▼
                                            Used by Composition
                                            Service to create
                                            CompositeNode
```

## Index Strategy

For efficient selection and rendering:

**Spatial Index** (for large graphs >100 nodes):
- Quadtree or R-tree for fast rectangle hit testing
- Rebuild when nodes move
- Query: `find_nodes_in_rect(rect: Rect) -> Vec<NodeId>`

**Node Lookup**:
- Primary: `BTreeMap<NodeId, Node>` (existing, deterministic)
- No additional index needed (BTreeMap is O(log n) lookup)

**Composite Node Lookup**:
- Filter nodes by `composition_spec.is_some()`
- Build runtime index: `HashMap<NodeId, &CompositeNode>` if needed

## Constraints & Invariants

### System Invariants

1. **Selection Integrity**: All NodeIds in SelectionState.selected_nodes must exist in NodeGraph
2. **Composition Integrity**: CompositeNode.internal_nodes must form a connected subgraph
3. **View Stack Integrity**: ViewStack must always have at least one context (MainCanvas)
4. **Port Mapping Integrity**: All PortMappings must reference valid internal nodes and ports
5. **Deterministic Serialization**: All serialized collections use BTreeMap (not HashMap)

### Performance Constraints

1. **Selection Response Time**: <100ms from mouse move to rectangle render
2. **Composition Time**: <1 second for up to 10 nodes
3. **Drill-Down Transition**: <500ms to switch views
4. **Memory**: Composite nodes should not duplicate large binary data (use caching)

### Capacity Limits

1. **Nodes per Composition**: Recommended max 10, hard limit 50
2. **Total Nodes per Graph**: Support up to 500 nodes
3. **View Stack Depth**: Max 5 levels (for future nested composition)
4. **Selection Size**: No hard limit, but composition requires ≥2 nodes
