# Upload Labs Clone - Technical Specification

**Version**: 1.0.0
**Status**: Draft
**Authors**: Development Team
**Last Updated**: 2024-01-XX

## Executive Summary

This specification defines the architecture and implementation for a native desktop game inspired by Upload Labs, featuring a node-based visual programming system where nodes are WebAssembly components with composable functionality and controlled system access.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Technology Stack](#technology-stack)
4. [Core Components](#core-components)
5. [Implementation Plan](#implementation-plan)
6. [API Design](#api-design)
7. [Security Model](#security-model)
8. [Performance Requirements](#performance-requirements)
9. [Development Workflow](#development-workflow)

## Overview

### Project Goals

- Create a native desktop application with node-based visual programming
- Enable users to compose WASM components into processing pipelines
- Provide secure, capability-based access to system resources
- Support user-created WASM components for extensibility
- Achieve 60 FPS UI performance with near-native execution speed

### Non-Goals

- Web deployment (native desktop only)
- Mobile support
- Real-time multiplayer features
- 3D visualization (unless added in future phases)

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interface Layer                      │
│                    (egui + egui-snarl)                      │
├─────────────────────────────────────────────────────────────┤
│                    Graph Execution Layer                     │
│                 (petgraph + orchestration)                   │
├─────────────────────────────────────────────────────────────┤
│                   Component Runtime Layer                    │
│              (wasmtime + WASI + component model)            │
├─────────────────────────────────────────────────────────────┤
│                      Storage Layer                          │
│                   (serde + file system)                     │
└─────────────────────────────────────────────────────────────┘
```

### Component Architecture

Each node in the graph represents a WASM component with:

- **Inputs**: Typed data received from connected nodes
- **Outputs**: Typed data sent to connected nodes
- **State**: Internal component state (if any)
- **Capabilities**: System access permissions (file I/O, network, etc.)

## Technology Stack

### Core Dependencies

```toml
[dependencies]
# UI Framework
egui = "0.29"
eframe = "0.29"
egui-snarl = "0.3"

# WASM Runtime
wasmtime = { version = "27.0", features = ["component-model", "async"] }
wasmtime-wasi = "27.0"

# Graph Management
petgraph = "0.6"
generational-arena = "0.2"

# Serialization (with BTreeMap for deterministic order)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
crc = "3.0"  # CRC64 checksums for file integrity

# Component Composition
wac-cli = "0.3"

# Utilities
anyhow = "1.0"
thiserror = "1.0"
tokio = { version = "1.40", features = ["full"] }
```

### Development Dependencies

```toml
[dev-dependencies]
cargo-component = "0.8"
wit-bindgen = "0.36"
```

## Core Components

### 1. Node System

```rust
pub struct GraphNode {
    pub id: NodeId,
    pub component_type: ComponentType,
    pub display_name: String,
    pub position: egui::Pos2,
    pub instance: Option<Instance>,
    pub inputs: Vec<Port>,  // Ports stored as Vec for order
    pub outputs: Vec<Port>, // Ports stored as Vec for order
    pub capabilities: CapabilitySet,
    pub metadata: NodeMetadata,
}

pub enum ComponentType {
    Builtin(BuiltinComponent),
    UserDefined(PathBuf),
    Composite(CompositeId),
}

pub enum NodeValue {
    U32(u32),
    I32(i32),
    F32(f32),
    String(String),
    Binary(Vec<u8>),
    List(Vec<NodeValue>),
    Record(BTreeMap<String, NodeValue>), // BTreeMap for deterministic serialization
}
```

### 2. Graph Execution Engine

```rust
pub struct ExecutionEngine {
    store: Store<HostState>,
    linker: Linker<HostState>,
    instances: HashMap<NodeId, InstancePre<HostState>>, // HashMap OK (not serialized)
}

impl ExecutionEngine {
    pub async fn execute_graph(&mut self, graph: &NodeGraph) -> Result<()> {
        // 1. Topological sort
        let execution_order = graph.topological_sort()?;

        // 2. Execute nodes in order
        for node_id in execution_order {
            self.execute_node(node_id, graph).await?;
        }

        Ok(())
    }
}
```

### 3. Component Interface (WIT)

```wit
package upload-labs:node;

interface node-base {
    record node-info {
        name: string,
        version: string,
        description: string,
        author: string,
    }

    record pin-info {
        name: string,
        data-type: data-type,
        optional: bool,
    }

    variant data-type {
        u32,
        i32,
        f32,
        string,
        binary,
        list(data-type),
        record(list<tuple<string, data-type>>),
    }

    get-info: func() -> node-info;
    get-inputs: func() -> list<pin-info>;
    get-outputs: func() -> list<pin-info>;
}

interface node-execute {
    execute: func(inputs: list<tuple<string, value>>) -> result<list<tuple<string, value>>, string>;
}

world node {
    import wasi:filesystem/types@0.2.0;
    import wasi:filesystem/preopens@0.2.0;
    import wasi:http/outgoing-handler@0.2.0;

    export node-base;
    export node-execute;
}
```

### 4. Security Model

```rust
pub enum CapabilitySet {
    None,
    FileRead { paths: Vec<PathBuf> },
    FileWrite { paths: Vec<PathBuf> },
    FileReadWrite { paths: Vec<PathBuf> },
    Network { allowed_hosts: Vec<String> },
    Full,
}

impl CapabilitySet {
    pub fn to_wasi_context(&self) -> Result<WasiCtx> {
        let mut builder = WasiCtxBuilder::new();

        match self {
            CapabilitySet::None => {},
            CapabilitySet::FileRead { paths } => {
                for path in paths {
                    builder = builder.preopened_dir(path, DirPerms::READ)?;
                }
            },
            // ... other variants
        }

        Ok(builder.build())
    }
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1-2)

**Goal**: Basic node editor with simple builtin nodes

- [ ] Setup Rust project with dependencies
- [ ] Implement basic egui application window
- [ ] Integrate egui-snarl for node editing
- [ ] Create simple builtin nodes (Add, Multiply, Constant)
- [ ] Implement graph execution without WASM
- [ ] Add save/load functionality

**Deliverable**: Node editor that can create and execute simple mathematical graphs

### Phase 2: WASM Integration (Week 3-4)

**Goal**: Execute WASM components as nodes

- [ ] Setup wasmtime with component model
- [ ] Define WIT interfaces for nodes
- [ ] Create example WASM component
- [ ] Implement component loading system
- [ ] Add component execution to graph engine
- [ ] Handle data marshaling between Rust and WASM

**Deliverable**: Ability to load and execute WASM components as nodes

### Phase 3: System Capabilities (Week 5-6)

**Goal**: Enable controlled system access for components

- [ ] Implement WASI context builder
- [ ] Add capability permission system
- [ ] Create file I/O nodes
- [ ] Add HTTP request nodes
- [ ] Implement resource limiting
- [ ] Build permission UI in node properties

**Deliverable**: Nodes can safely access files and network with user-granted permissions

### Phase 4: Advanced Features (Week 7-8)

**Goal**: Polish and advanced functionality

- [ ] WAC composition support
- [ ] Component hot-reloading
- [ ] Performance optimization (instance caching)
- [ ] Node categories and search
- [ ] Visual themes and polish
- [ ] Error handling and debugging tools

**Deliverable**: Production-ready application

## API Design

### Host API for Components

```rust
// Trait implemented by host for component callbacks
pub trait HostFunctions {
    fn log(&mut self, level: LogLevel, message: &str);
    fn get_temp_directory(&self) -> Result<PathBuf>;
    fn request_capability(&mut self, cap: CapabilityRequest) -> Result<bool>;
}
```

### Node Registry API

```rust
pub trait NodeRegistry {
    fn register_builtin(&mut self, id: &str, factory: BuiltinNodeFactory);
    fn register_component(&mut self, path: PathBuf) -> Result<ComponentId>;
    fn list_nodes(&self) -> Vec<NodeDescriptor>;
    fn create_instance(&self, id: &NodeId) -> Result<Box<dyn Node>>;
}
```

## Security Model

### Capability-Based Security

1. **Default Deny**: Components have no system access by default
2. **Explicit Grants**: Users must grant specific permissions
3. **Path Scoping**: File access limited to specific directories
4. **Network Allowlisting**: HTTP only to approved domains
5. **Resource Limits**: CPU and memory quotas per component

### Sandboxing Levels

```rust
pub enum SandboxLevel {
    Strict,     // No system access
    Limited,    // Read-only file access to specific paths
    Standard,   // Read/write to user data directory
    Extended,   // Network access + standard file access
    Developer,  // Full access (requires explicit opt-in)
}
```

## Performance Requirements

### UI Performance

- **Target**: 60 FPS during normal operation
- **Maximum**: 100ms response time for user actions
- **Graph Size**: Support 500+ nodes without degradation

### Execution Performance

- **Component Overhead**: < 10ms per component invocation
- **Data Transfer**: < 1ms for typical data sizes (< 1MB)
- **Memory**: < 10MB per component instance

### Optimization Strategies

1. **Instance Pooling**: Reuse component instances
2. **Lazy Compilation**: Compile components on first use
3. **Parallel Execution**: Execute independent subgraphs concurrently
4. **Incremental Updates**: Only re-execute affected nodes

## Development Workflow

### Setting Up Development Environment

```bash
# Clone repository
git clone <repository>
cd upload-labs-clone

# Install Rust toolchain
rustup update stable
rustup target add wasm32-wasip2

# Install development tools
cargo install cargo-component wit-bindgen wac-cli

# Build and run
cargo build --release
cargo run
```

### Creating a New Component

```bash
# Generate component boilerplate
cargo component new my-node --lib

# Edit wit/world.wit to match node interface
# Implement src/lib.rs

# Build component
cargo component build --release

# Test in application
cp target/wasm32-wasip2/release/my_node.wasm components/
```

### Testing Workflow

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*'

# Run with debug logging
RUST_LOG=debug cargo run

# Profile performance
cargo build --release --features profiling
cargo run --release -- --profile
```

## Appendix A: Example Components

### Simple Adder Component

```rust
// src/lib.rs
use upload_labs::node::*;

struct Adder;

impl Guest for Adder {
    fn get_info() -> NodeInfo {
        NodeInfo {
            name: "Adder".to_string(),
            version: "1.0.0".to_string(),
            description: "Adds two numbers".to_string(),
            author: "Example".to_string(),
        }
    }

    fn get_inputs() -> Vec<PinInfo> {
        vec![
            PinInfo { name: "a".to_string(), data_type: DataType::F32, optional: false },
            PinInfo { name: "b".to_string(), data_type: DataType::F32, optional: false },
        ]
    }

    fn get_outputs() -> Vec<PinInfo> {
        vec![
            PinInfo { name: "sum".to_string(), data_type: DataType::F32, optional: false },
        ]
    }

    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, String> {
        let a = inputs.iter().find(|(n, _)| n == "a")
            .ok_or("Missing input 'a'")?
            .1.as_f32()
            .ok_or("Input 'a' must be f32")?;

        let b = inputs.iter().find(|(n, _)| n == "b")
            .ok_or("Missing input 'b'")?
            .1.as_f32()
            .ok_or("Input 'b' must be f32")?;

        Ok(vec![("sum".to_string(), Value::F32(a + b))])
    }
}
```

**End of Specification**

Version History:
- 1.0.0 - Initial specification (current)

## Data Structure Design Decisions

### BTreeMap vs HashMap

**Decision**: Use `BTreeMap` for all serialized data structures, `HashMap` for runtime-only structures.

**Rationale**:
- **Problem**: HashMap has non-deterministic iteration order, causing identical graphs to serialize to different byte sequences
- **Impact**: Breaks CRC64 checksum validation, prevents reliable file integrity checking
- **Solution**: BTreeMap guarantees consistent ordering, enabling deterministic serialization
- **Performance**: O(log n) vs O(1) lookup is negligible for <1000 nodes (~0.1ms difference)
- **Benefits**: Enables robust file corruption detection via CRC64 checksums

**Where Used**:
- `NodeGraph.nodes`: `BTreeMap<Uuid, GraphNode>` (serialized)
- `NodeValue::Record`: `BTreeMap<String, NodeValue>` (serialized)
- `ExecutionEngine.instances`: `HashMap<NodeId, Instance>` (runtime-only, not serialized)

**Implementation Note**: Runtime-only structures can still use `HashMap` when marked with `#[serde(skip)]` for optimal performance.

