# Data Model

**Feature**: WebAssembly Node-Based Visual Programming System
**Branch**: 001-webassembly-based-node
**Date**: 2025-10-12

## Overview

This document defines the core data entities for the visual programming system. All entities are designed to be serializable (for graph persistence) and to enforce type safety and security constraints at the data model level.

## Core Entities

### 1. NodeValue

**Purpose**: Represents typed data flowing through connections between nodes.

**Attributes**:
- **value_type**: Discriminant indicating which variant is active
- **data**: Actual value storage (enum variant)

**Variants**:
- `U32(u32)`: Unsigned 32-bit integer
- `I32(i32)`: Signed 32-bit integer
- `F32(f32)`: 32-bit floating point number
- `String(String)`: UTF-8 text string
- `Binary(Vec<u8>)`: Raw binary data for efficient large payloads
- `List(Vec<NodeValue>)`: Homogeneous or heterogeneous list of values
- `Record(BTreeMap<String, NodeValue>)`: Key-value structured data (BTreeMap for deterministic serialization)

**Validation Rules**:
- List values must be serializable
- Record keys must be valid UTF-8 strings
- Binary data has no size limit at model level (runtime enforces memory limits)
- F32 values support NaN and Infinity (user-visible error if unexpected)

**Relationships**:
- Stored in Port.current_value (output ports only)
- Passed through Connection during graph execution
- Serialized to/from graph save files

**State Transitions**:
1. **Uninitialized** → **Computed**: Output port receives value from node execution
2. **Computed** → **Stale**: Connected input node changes, invalidates downstream
3. **Stale** → **Computed**: Node re-executes, produces new value

### 2. Port

**Purpose**: Connection point on a node for receiving (input) or providing (output) data.

**Attributes**:
- `id`: Unique identifier (UUID or NodeId + local index)
- `name`: Human-readable label (e.g., "a", "sum", "input_file")
- `data_type`: Expected NodeValue variant (enforces type compatibility)
- `direction`: Input or Output enum
- `optional`: Boolean indicating if connection is required
- `current_value`: Option<NodeValue> (present only for outputs after execution)

**Validation Rules**:
- Input ports cannot have current_value (values come from connections)
- Output ports must have current_value after successful node execution
- Port names must be unique within a node
- data_type must be a valid NodeValue variant or generic "Any" type
- Optional inputs allow unconnected ports without validation errors

**Relationships**:
- Owned by GraphNode (node.inputs, node.outputs)
- Referenced by Connection (connection.source_port, connection.target_port)
- Type compatibility checked when creating Connection

**Example**:
```
Add Node:
  Inputs:
    - Port { name: "a", data_type: F32, direction: Input, optional: false }
    - Port { name: "b", data_type: F32, direction: Input, optional: false }
  Outputs:
    - Port { name: "sum", data_type: F32, direction: Output, current_value: Some(F32(7.5)) }
```

### 3. Connection

**Purpose**: Directed link representing data flow from one node's output to another's input.

**Attributes**:
- `id`: Unique identifier
- `source_node`: NodeId of the producing node
- `source_port`: Port ID on source node (must be Output port)
- `target_node`: NodeId of the consuming node
- `target_port`: Port ID on target node (must be Input port)
- `validated`: Boolean indicating type compatibility has been checked

**Validation Rules**:
- source_port must be an Output port
- target_port must be an Input port
- source_port.data_type must be compatible with target_port.data_type:
  - Exact match (F32 → F32)
  - Any type matches everything (Any → F32, F32 → Any)
  - List compatibility (List<F32> → List<Any>, List<Any> → List<F32>)
- No self-connections (source_node ≠ target_node)
- Each input port can have at most one incoming connection (multiple outputs allowed)

**Relationships**:
- Stored in NodeGraph.connections (adjacency list or edge list)
- Participates in petgraph DiGraph representation (edge between nodes)
- Used during topological sort to determine execution order

**State Transitions**:
1. **Proposed** → **Validated**: Type compatibility check passes
2. **Validated** → **Deleted**: User removes connection or deletes node

**Type Compatibility Matrix**:
```
Source → Target  | Compatible?
─────────────────┼─────────────
F32 → F32        | ✓
F32 → I32        | ✗
F32 → Any        | ✓
Any → F32        | ✓
List<F32> → Any  | ✓
```

### 4. GraphNode

**Purpose**: A computational unit in the visual programming graph.

**Attributes**:
- `id`: Unique node identifier (UUID)
- `component_id`: Reference to ComponentSpec (determines behavior)
- `display_name`: User-visible label on canvas
- `position`: (x: f32, y: f32) canvas coordinates for UI rendering
- `inputs`: Vec<Port> of input ports
- `outputs`: Vec<Port> of output ports
- `metadata`: NodeMetadata struct (author, version, description)
- `capabilities`: CapabilitySet enum (security permissions)
- `execution_state`: ExecutionState enum (Idle, Running, Completed, Failed)

**Validation Rules**:
- Inputs and outputs must match ComponentSpec declaration
- Position coordinates must be finite (no NaN/Infinity)
- At least one input or output required (pure constant nodes have outputs only)
- Component_id must reference a loaded ComponentSpec

**Relationships**:
- Owned by NodeGraph (graph.nodes: BTreeMap<NodeId, GraphNode>)
- Referenced by Connection (source_node, target_node)
- Links to ComponentSpec via component_id (defines behavior)
- Associated with CapabilityGrant instances (user-approved permissions)

**State Transitions**:
1. **Created** → **Configured**: User places node on canvas, UI assigns position
2. **Configured** → **Executable**: All required inputs connected
3. **Executable** → **Running**: Execution engine invokes component
4. **Running** → **Completed**: Component returns successfully
5. **Running** → **Failed**: Component throws error or violates permissions
6. **Completed** → **Stale**: Upstream node changes, invalidates outputs

**Example**:
```
GraphNode {
  id: "node_42",
  component_id: "builtin:math:add",
  display_name: "Add Numbers",
  position: (150.0, 200.0),
  inputs: [
    Port { name: "a", data_type: F32, ... },
    Port { name: "b", data_type: F32, ... }
  ],
  outputs: [
    Port { name: "sum", data_type: F32, current_value: Some(F32(10.0)) }
  ],
  metadata: NodeMetadata { author: "System", version: "1.0.0", ... },
  capabilities: CapabilitySet::None,
  execution_state: ExecutionState::Completed
}
```

### 5. ComponentSpec

**Purpose**: Specification of a node's behavior, interface, and system requirements.

**Attributes**:
- `id`: Unique component identifier (e.g., "builtin:math:add", "user:text-processor")
- `name`: Display name for UI
- `description`: User-visible documentation
- `author`: Creator attribution
- `version`: Semantic version (major.minor.patch)
- `component_type`: Builtin or UserDefined(PathBuf) enum
- `input_spec`: Vec<PortSpec> defining expected inputs
- `output_spec`: Vec<PortSpec> defining produced outputs
- `required_capabilities`: Vec<CapabilityRequest> (file paths, network hosts, etc.)
- `wasm_module`: Option<wasmtime::Module> for user-defined components

**Validation Rules**:
- Semantic versioning must be valid (regex: `\d+\.\d+\.\d+`)
- Input/output specs must have unique port names
- Builtin components must have component_type = Builtin
- UserDefined components must have valid WASM module loaded
- required_capabilities must map to CapabilitySet variants

**Relationships**:
- Referenced by GraphNode.component_id
- Loaded into ComponentRegistry (application-level singleton)
- WIT interface bindings generated from input_spec/output_spec

**Lifecycle**:
1. **Discovered**: Component found in components/ directory or registered as builtin
2. **Loaded**: WASM module validated and compiled
3. **Registered**: Added to ComponentRegistry, available in UI palette
4. **Instantiated**: GraphNode created from ComponentSpec
5. **Unloaded**: Component removed from registry (user deletes .wasm file)

**Example**:
```
ComponentSpec {
  id: "builtin:math:add",
  name: "Add",
  description: "Adds two numbers and returns the sum",
  author: "WasmFlow",
  version: "1.0.0",
  component_type: Builtin,
  input_spec: [
    PortSpec { name: "a", data_type: F32, optional: false },
    PortSpec { name: "b", data_type: F32, optional: false }
  ],
  output_spec: [
    PortSpec { name: "sum", data_type: F32 }
  ],
  required_capabilities: [],
  wasm_module: None
}
```

### 6. NodeGraph

**Purpose**: Container for the complete visual programming graph.

**Attributes**:
- `id`: Unique graph identifier
- `name`: User-assigned graph name
- `nodes`: BTreeMap<NodeId, GraphNode> (all nodes in graph, BTreeMap for deterministic serialization order)
- `connections`: Vec<Connection> (all edges between nodes)
- `metadata`: GraphMetadata struct (created_at, modified_at, author)
- `version`: Schema version for serialization compatibility
- `execution_order_cache`: Option<Vec<NodeId>> (cached topological sort result)

**Validation Rules**:
- All connections must reference valid node IDs in nodes map
- Connection target ports must exist on referenced nodes
- Graph must be acyclic (enforced via petgraph::algo::is_cyclic_directed)
- All required input ports must have connections or default values

**Relationships**:
- Contains GraphNode instances (ownership)
- Contains Connection instances (ownership)
- Serialized to/from graph save files (bincode format)
- Converted to petgraph::DiGraph for execution (borrowed reference)

**Operations**:
- `add_node(spec: ComponentSpec, position: Pos2) -> NodeId`
- `remove_node(id: NodeId) -> Result<()>` (also removes connected edges)
- `add_connection(source, target) -> Result<Connection>` (validates types)
- `remove_connection(id: ConnectionId) -> Result<()>`
- `validate() -> Result<ValidationReport>` (check cycles, types, required inputs)
- `execute() -> Result<ExecutionReport>` (run graph, return outputs)

**State Transitions**:
1. **Empty** → **Edited**: User adds first node
2. **Edited** → **Valid**: All validation rules satisfied
3. **Valid** → **Executing**: User triggers execution
4. **Executing** → **Completed**: All nodes finish successfully
5. **Executing** → **Failed**: Any node execution fails
6. **Completed** → **Edited**: User modifies graph

**Serialization**:
```rust
// Graph save file format (bincode)
GraphSaveFormat {
  version: u32,               // Schema version (e.g., 1)
  graph: NodeGraph,           // Full graph structure
  checksum: u64,              // Optional integrity check
}
```

### 7. CapabilityGrant

**Purpose**: User-approved permission for a node to access system resources.

**Attributes**:
- `node_id`: NodeId receiving the grant
- `capability_set`: CapabilitySet enum defining what's allowed
- `granted_at`: Timestamp of user approval
- `scope`: Specific restrictions (file paths, network hosts)

**Capability Set Variants**:
- `None`: No system access (pure computation)
- `FileRead { paths: Vec<PathBuf> }`: Read-only access to specific directories
- `FileWrite { paths: Vec<PathBuf> }`: Write access to specific directories
- `FileReadWrite { paths: Vec<PathBuf> }`: Combined read/write access
- `Network { allowed_hosts: Vec<String> }`: HTTP access to allowlisted domains
- `Full`: Unrestricted access (requires explicit warning to user)

**Validation Rules**:
- Paths must be absolute (no relative paths like "../")
- Network hosts must be valid domain names or IP addresses
- Full capability requires user confirmation dialog
- Capabilities cannot exceed what component requests in ComponentSpec

**Relationships**:
- Linked to GraphNode.capabilities (one-to-one)
- Stored in graph save file (persisted with graph)
- Used by WASI context builder to configure sandbox

**Lifecycle**:
1. **Requested**: Component metadata declares required_capabilities
2. **Prompted**: UI shows permission dialog when node added to graph
3. **Granted**: User approves, CapabilityGrant created and stored
4. **Enforced**: Runtime configures WASI context based on grant
5. **Revoked**: User modifies permissions, grant updated or removed
6. **Escalation**: Component requests additional capabilities, re-prompt user

**Example**:
```
CapabilityGrant {
  node_id: "node_73",
  capability_set: FileReadWrite {
    paths: ["/Users/alice/Documents/data"]
  },
  granted_at: "2025-10-12T14:30:00Z",
  scope: "Can read/write files in data directory only"
}
```

**Security Enforcement**:
```rust
// WASI context configuration from CapabilityGrant
match capability_grant.capability_set {
  CapabilitySet::FileRead { paths } => {
    for path in paths {
      wasi_ctx.preopened_dir(path, DirPerms::READ)?;
    }
  }
  CapabilitySet::Network { allowed_hosts } => {
    wasi_ctx.inherit_network(); // + allowlist enforcement in host function
  }
  // ... other variants
}
```

## Data Flow Example

**Scenario**: User creates a simple graph: `Constant(5) → Add ← Constant(3) → Output(sum=8)`

1. **Graph Construction**:
   - Create GraphNode for Constant(5) with output port "value"
   - Create GraphNode for Constant(3) with output port "value"
   - Create GraphNode for Add with input ports "a", "b" and output port "sum"
   - Create Connection from Constant(5).value → Add.a
   - Create Connection from Constant(3).value → Add.b

2. **Validation**:
   - Check all connections for type compatibility (F32 → F32) ✓
   - Run petgraph cycle detection ✓
   - Verify all required inputs connected ✓

3. **Execution**:
   - Topological sort: [Constant(5), Constant(3), Add]
   - Execute Constant(5): Set output port "value" = F32(5.0)
   - Execute Constant(3): Set output port "value" = F32(3.0)
   - Execute Add: Read inputs a=5.0, b=3.0, compute sum=8.0, set output port "sum" = F32(8.0)

4. **Display**:
   - UI renders Add node with output port showing "sum: 8.0"
   - User sees result on canvas

## Persistence Format

**Graph Save File** (.wasmflow extension):
```
┌─────────────────────────────┐
│ Magic bytes: "WASMFLOW"     │
│ Version: u32 = 1            │
├─────────────────────────────┤
│ NodeGraph (bincode)         │
│   ├─ nodes: BTreeMap        │
│   ├─ connections: Vec       │
│   ├─ metadata               │
│   └─ version                │
├─────────────────────────────┤
│ CapabilityGrants: Vec       │
├─────────────────────────────┤
│ Checksum: u64 (CRC64)       │
└─────────────────────────────┘
```

**Size Estimates**:
- Small graph (10 nodes, 15 connections): ~5 KB
- Medium graph (50 nodes, 75 connections): ~25 KB
- Large graph (500 nodes, 750 connections): ~250 KB

## Indexing & Performance

**Primary Keys**:
- NodeId: UUID v4 (128-bit, guaranteed unique)
- ConnectionId: UUID v4
- PortId: Composite (NodeId + u8 index) for cache locality

**Indexes**:
- `BTreeMap<NodeId, GraphNode>`: O(log n) node lookup, deterministic iteration order for serialization
- `HashMap<PortId, Vec<ConnectionId>>`: Fast "which connections involve this port" (can use HashMap here as not serialized)
- `cached_execution_order: Vec<NodeId>`: Amortized O(1) topological sort

**Note on BTreeMap vs HashMap**: BTreeMap is used for all serialized data structures to ensure deterministic byte order, enabling reliable checksum validation. The slight performance trade-off (O(log n) vs O(1)) is negligible for graphs with <1000 nodes.

**Optimization Notes**:
- Use generational-arena for stable IDs with reuse (optional)
- Cache petgraph DiGraph construction (rebuild only on structural change)
- Dirty flag tracking for incremental execution

## Validation Summary

**Graph-Level Validations**:
- ✓ No cycles (petgraph::algo::is_cyclic_directed)
- ✓ All connections type-compatible
- ✓ All required inputs have connections or defaults
- ✓ All node IDs in connections exist in nodes map

**Node-Level Validations**:
- ✓ Component spec exists and is loaded
- ✓ Input/output ports match component spec
- ✓ Capabilities granted match component requirements
- ✓ Position coordinates are finite

**Connection-Level Validations**:
- ✓ Source is output port, target is input port
- ✓ Data types compatible
- ✓ No self-connections
- ✓ Each input port has at most one connection

**Runtime Validations**:
- ✓ Component execution doesn't exceed resource limits
- ✓ Returned values match declared output types
- ✓ No capability violations (WASI sandbox enforcement)
