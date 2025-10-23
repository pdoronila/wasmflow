# Data Model: Continuous Execution Nodes

**Feature**: 006-continuous-node-can
**Date**: 2025-10-20
**Status**: Phase 1 Design

## Overview

This document defines the data structures and state management for continuous execution nodes. These definitions are technology-agnostic but will be implemented in Rust with WIT interfaces for WASM components.

---

## Core Entities

### 1. ContinuousNodeConfig

**Purpose**: Configuration metadata for nodes that support continuous execution.

**Attributes**:
- `supports_continuous` (boolean): Whether this node type can run continuously
- `enabled` (boolean): User preference for showing play/stop controls
- `runtime_state` (ContinuousRuntimeState): Current execution state (transient, not persisted)

**Persistence**: `supports_continuous` and `enabled` are saved with graph. `runtime_state` is always reset to default on load.

**Validation Rules**:
- If `supports_continuous` is false, `enabled` must be false
- On graph load, `runtime_state.is_running` must be false

**State Transitions**:
```
[enabled: false] --user enables--> [enabled: true]
[enabled: true] --user disables--> [enabled: false]
(Only when node is stopped)
```

---

### 2. ContinuousRuntimeState

**Purpose**: Runtime-only state tracking for active continuous nodes.

**Attributes**:
- `is_running` (boolean): Whether execution is currently active
- `started_at` (timestamp, optional): When execution began
- `iterations` (integer): Number of execution cycles completed
- `last_error` (string, optional): Most recent error message
- `task_handle` (opaque reference): Handle for stopping async task

**Persistence**: NOT persisted. Always defaults to stopped state on app start.

**Lifecycle**:
1. **Created**: When node is added to graph (if `supports_continuous` is true)
2. **Updated**: Each execution cycle, error occurrence, start/stop
3. **Destroyed**: When node is removed from graph or app shuts down

**State Transitions**:
```
[Idle] --play clicked--> [Starting] --task spawned--> [Running]
[Running] --stop clicked--> [Stopping] --task cancelled--> [Idle]
[Running] --error occurs--> [Error] --user resets--> [Idle]
```

---

### 3. ContinuousExecutionState

**Purpose**: Enumeration of possible execution states for continuous nodes.

**Values**:
- `Idle`: Not running, waiting for user action
- `Starting`: Play button clicked, spawning async task
- `Running`: Actively executing, processing inputs
- `Stopping`: Stop button clicked, graceful shutdown in progress
- `Stopped`: Successfully stopped by user
- `Error`: Execution failed, error details available

**Visual Indicators** (UI mapping):
- `Idle`: Gray/neutral color, play button visible
- `Starting`: Yellow/orange, spinner animation
- `Running`: Green, pulsing indicator, stop button visible
- `Stopping`: Orange, fading animation
- `Stopped`: Gray, play button visible
- `Error`: Red, error icon, details on hover

**Transitions**:
```
Idle → Starting → Running → Stopping → Stopped → Idle
              ↓              ↓
            Error ← ← ← ← Error
```

---

### 4. ControlMessage

**Purpose**: Commands sent from UI thread to execution manager.

**Variants**:
- `Start { node_id }`: Begin continuous execution
- `Stop { node_id }`: Request graceful shutdown
- `Pause { node_id }`: Pause execution (optional, future)
- `Resume { node_id }`: Resume from pause (optional, future)
- `Shutdown`: Stop all continuous nodes (app shutdown)

**Delivery**: Via bounded channel (capacity: 100 messages)

**Handling**:
- Commands are processed in order
- If node is already in target state, command is ignored
- Invalid transitions log warning but don't error

---

### 5. ExecutionResult

**Purpose**: Results and updates sent from execution tasks to UI thread.

**Variants**:
- `Started { node_id, timestamp }`: Execution began successfully
- `Stopped { node_id, iterations, duration }`: Execution stopped cleanly
- `OutputsUpdated { node_id, outputs }`: New output values available
- `Error { node_id, error }`: Execution failed
- `IterationComplete { node_id, iteration, duration }`: One cycle finished

**Delivery**: Via unbounded channel (results should never block execution)

**Handling**:
- UI polls channel every frame (non-blocking)
- Multiple results may be batched per frame
- Results update both graph state and visual indicators

---

### 6. ContinuousNodeError

**Purpose**: Structured error information for continuous execution failures.

**Variants**:
- `ExecutionFailed { node_id, node_name, message, source_location, timestamp }`
- `PermissionDenied { node_id, node_name, capability, attempted_action }`
- `Timeout { node_id, node_name, duration }`
- `NetworkError { node_id, message, status_code }`
- `ComponentTrap { node_id, node_name, trap_message }`

**Attributes** (common to all variants):
- `node_id` (UUID): Identifies which node failed
- `node_name` (string): Human-readable node identifier
- Additional context varies by variant

**Display**:
- Status bar: One-line summary with icon
- Node visual: Red border/background
- Error panel: Full details with timestamp
- Logs: Structured logging with all context

---

## Relationships

```
GraphNode (1) --has-a--> (0..1) ContinuousNodeConfig
ContinuousNodeConfig (1) --contains--> (1) ContinuousRuntimeState
ContinuousRuntimeState (1) --references--> (1) ContinuousExecutionState

ExecutionManager (1) --manages--> (0..N) ContinuousNodeTask
ContinuousNodeTask (1) --tracks--> (1) GraphNode
ContinuousNodeTask (1) --sends--> (N) ExecutionResult
ContinuousNodeTask (1) --receives--> (N) ControlMessage
```

---

## Data Flow

### Start Continuous Node
```
[User clicks play]
  ↓
[UI creates ControlMessage::Start{node_id}]
  ↓
[ExecutionManager receives command]
  ↓
[Spawns async task with cancellation token]
  ↓
[Task sends ExecutionResult::Started]
  ↓
[UI updates node state to Running]
```

### Execution Cycle
```
[Async task executes WASM component]
  ↓
[Component produces outputs]
  ↓
[Task sends ExecutionResult::OutputsUpdated]
  ↓
[UI receives result via channel poll]
  ↓
[Updates node output values in graph]
  ↓
[Triggers canvas repaint]
  ↓
[Loop continues until stopped or error]
```

### Stop Continuous Node
```
[User clicks stop]
  ↓
[UI creates ControlMessage::Stop{node_id}]
  ↓
[ExecutionManager cancels task token]
  ↓
[Task detects cancellation, cleans up]
  ↓
[Task sends ExecutionResult::Stopped]
  ↓
[UI updates node state to Idle]
```

### Error Handling
```
[WASM component throws error]
  ↓
[Task catches error/panic]
  ↓
[Task creates ContinuousNodeError]
  ↓
[Task sends ExecutionResult::Error]
  ↓
[UI displays error in status bar + node visual]
  ↓
[Task terminates, state → Error]
```

---

## Persistence Strategy

### Saved to Disk (Graph Serialization)
- `GraphNode.continuous_config.supports_continuous` ✅
- `GraphNode.continuous_config.enabled` ✅
- Node metadata, ports, connections ✅
- Capability grants ✅

### NOT Saved (Runtime Only)
- `ContinuousRuntimeState.is_running` ❌
- `ContinuousRuntimeState.started_at` ❌
- `ContinuousRuntimeState.iterations` ❌
- `ContinuousRuntimeState.task_handle` ❌
- `ContinuousRuntimeState.last_error` ❌
- Execution manager state ❌
- Channel contents ❌

**Rationale**: Running state depends on system resources (threads, WASM instances, network sockets) that cannot be serialized. Always starting stopped ensures clean state and explicit user control.

---

## Validation Rules

### Node Configuration
1. Only nodes with WIT interface supporting continuous execution can have `supports_continuous = true`
2. Play/stop controls only shown if `enabled = true`
3. Cannot enable continuous mode while node is executing (single or continuous)
4. Cannot modify node configuration while running

### Execution State
1. Cannot start node that is already running
2. Cannot stop node that is not running
3. Graceful stop must complete within 2 seconds or forced abort occurs
4. Maximum 10 concurrent continuous nodes (configurable)

### Capability Grants
1. Continuous nodes must re-validate capability grants on each start
2. Network/file capabilities require explicit user approval
3. Capability escalation (requesting new permissions) stops running node

### Graph Operations
1. Cannot delete node while running (must stop first)
2. Cannot disconnect inputs to running node (must stop first)
3. Graph save warns if continuous nodes are running
4. Graph load always resets all nodes to stopped

---

## Performance Characteristics

### Memory Usage (per continuous node)
- `ContinuousNodeConfig`: ~16 bytes
- `ContinuousRuntimeState`: ~128 bytes
- Channel buffers: ~2-3 KB
- Total per node: ~3-4 KB

### Latency
- Start/stop command: <100ms (spec requirement)
- Execution cycle: <10ms overhead target
- Error propagation: <5ms to UI
- State read (UI): <100ns (RwLock read)

### Throughput
- Supported concurrent nodes: 10+ (spec requirement)
- Execution cycles per second: 100+ (depends on component)
- Channel message rate: 1000+ msgs/sec

---

## Example Scenarios

### Scenario 1: HTTP Server Node
- **Configuration**: `supports_continuous = true`, `enabled = true`
- **Capabilities**: Network access (port 8080), no file access
- **Execution**: Continuous loop listening for HTTP requests
- **Inputs**: Configuration (port number, routes)
- **Outputs**: Request count, last request timestamp
- **Stop Behavior**: Complete current request, close listener, send final stats

### Scenario 2: File Watcher Node
- **Configuration**: `supports_continuous = true`, `enabled = true`
- **Capabilities**: Read-only file access (specific directory)
- **Execution**: Poll directory for changes every 1 second
- **Inputs**: Directory path, file pattern
- **Outputs**: Changed files list
- **Stop Behavior**: Complete current scan, release file handles

### Scenario 3: Timer Node
- **Configuration**: `supports_continuous = true`, `enabled = true`
- **Capabilities**: None (pure computation)
- **Execution**: Emit timestamp every N seconds
- **Inputs**: Interval (seconds)
- **Outputs**: Current timestamp
- **Stop Behavior**: Immediate, no cleanup needed

---

## Open Questions

None - all questions resolved in research phase.

---

## Next Steps

1. Generate WIT interface contracts for continuous execution
2. Create quickstart guide for component developers
3. Update implementation plan with concrete file changes
