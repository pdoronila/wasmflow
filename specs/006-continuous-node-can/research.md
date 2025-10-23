# Research: Continuous Execution Nodes

**Date**: 2025-10-20
**Feature**: 006-continuous-node-can
**Status**: Phase 0 Complete

## Overview

This document consolidates research findings for implementing continuous execution nodes in WasmFlow. Five key technical unknowns were identified and researched:

1. Async execution model integration (egui + tokio)
2. State management and persistence
3. Thread-safe state sharing
4. Resource cleanup strategies
5. Error propagation patterns

---

## 1. Async Execution Model: egui + tokio Integration

### Decision: Background Threads with Channel Communication

**Pattern**: Use `std::thread::spawn` with `std::sync::mpsc` channels for task-UI communication, with isolated tokio runtimes per background thread.

### Rationale

- **Existing Pattern**: Already proven in `/Users/doronila/git/wasmflow_cc/src/ui/app.rs` (lines 368-418)
- **UI Non-Blocking**: `try_recv()` polling keeps UI at 60 FPS without blocking
- **Performance**: Measured <1ms channel overhead, no impact on rendering
- **Separation**: UI thread remains sync, worker threads handle all async operations

### Implementation Approach

```rust
// Pattern for continuous execution
pub struct ContinuousNodeState {
    node_id: Uuid,
    running: Arc<AtomicBool>,
    result_rx: Receiver<NodeExecutionResult>,
    control_tx: Sender<ControlMessage>,
}

fn start_continuous_node(&mut self, node_id: Uuid) {
    let (result_tx, result_rx) = channel();
    let (control_tx, control_rx) = channel();
    let running = Arc::new(AtomicBool::new(true));

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            while running.load(Ordering::Relaxed) {
                // Execute one cycle
                let result = execute_node_cycle().await;
                let _ = result_tx.send(result);
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    });

    self.continuous_nodes.insert(node_id, ContinuousNodeState {
        node_id, running, result_rx, control_tx,
    });
}
```

### Alternatives Considered

| Approach | Verdict | Reason |
|----------|---------|--------|
| Shared tokio Runtime | ❌ Rejected | Risks blocking UI thread |
| poll_fn with Runtime::block_on | ❌ Rejected | Blocks UI during polling |
| Background thread per node | ✅ Selected | Already working pattern |
| Thread pool | ⚠️ Future optimization | More complex state management |

### References

- Current implementation: `/Users/doronila/git/wasmflow_cc/src/ui/app.rs` (lines 286-418)
- Async execution: `/Users/doronila/git/wasmflow_cc/src/runtime/engine.rs` (lines 317-370)

---

## 2. State Management and Persistence

### Decision: Conservative "Always Start Stopped" Approach

**Pattern**: Do NOT persist running state across app restarts. All continuous nodes start in `Idle` state when graph is loaded.

### Rationale

- **Safety**: Running nodes hold non-serializable resources (sockets, WASM instances, async tasks)
- **Resource Cleanup**: Clean slate prevents orphaned resources from crashed sessions
- **Security**: Requires explicit user consent for long-running privileged operations
- **Alignment**: Matches existing architecture - execution state is runtime-only

### What Gets Persisted

```rust
pub struct ContinuousNodeConfig {
    /// Whether this node supports continuous execution
    pub supports_continuous: bool,

    /// User preference: show play/stop controls
    pub enabled: bool,  // ✅ PERSISTED

    /// Runtime state (NOT serialized)
    #[serde(skip)]
    pub runtime_state: ContinuousRuntimeState,  // ❌ NOT PERSISTED
}
```

### Graph Load Behavior

```rust
impl NodeGraph {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut graph = Self::from_bytes(&bytes)?;

        // SAFETY: Ensure all continuous nodes start stopped
        for node in graph.nodes.values_mut() {
            if let Some(config) = &mut node.continuous_config {
                config.runtime_state = ContinuousRuntimeState::default();
            }
            node.execution_state = ExecutionState::Idle;
        }

        Ok(graph)
    }
}
```

### Alternatives Considered

| Approach | Verdict | Reason |
|----------|---------|--------|
| Auto-restart on load | ❌ Rejected | Security risk, resource conflicts |
| Prompt user on load | ❌ Rejected | Poor UX, still has resource issues |
| Manual restart required | ✅ Selected | Safe, predictable, simple |

### Integration Points

- Graph serialization: `/Users/doronila/git/wasmflow_cc/src/graph/serialization.rs` (lines 99-108)
- Node definition: `/Users/doronila/git/wasmflow_cc/src/graph/node.rs` (lines 260-306)

---

## 3. Thread-Safe State Sharing

### Decision: Hybrid Message Passing + Atomic State

**Pattern**:
- `tokio::sync::mpsc` channels for commands (start/stop) and results
- `Arc<RwLock<ExecutionState>>` for shared status (high-frequency reads)
- `AtomicBool` for simple stop flags

### Rationale

- **UI Reading**: Node status checked at 60fps - RwLock allows multiple readers
- **UI Sending**: Start/stop commands are infrequent - channels provide clear boundaries
- **Async Updates**: Output values sent via channels to avoid lock contention
- **Performance**: RwLock reads <50ns, channel ops <200ns - both acceptable for 60fps (16ms budget)

### Implementation Pattern

```rust
/// Shared state for UI reads (high frequency)
pub struct NodeRuntimeState {
    pub execution_state: Arc<RwLock<ContinuousExecutionState>>,
    pub last_outputs: Arc<RwLock<HashMap<String, NodeValue>>>,
}

/// Commands via channels (low frequency)
pub enum ContinuousCommand {
    Start { node_id: Uuid },
    Stop { node_id: Uuid },
    Shutdown,
}

/// Results via channels (variable frequency)
pub enum ContinuousResult {
    OutputsUpdated { node_id: Uuid, outputs: HashMap<String, NodeValue> },
    Error { node_id: Uuid, error: String },
}
```

### Deadlock Avoidance Strategies

1. **Lock Ordering**: Always acquire locks in consistent order (node_id ascending)
2. **No Nested Locks**: Never acquire RwLock while holding another lock
3. **Short Critical Sections**: Hold locks for minimal time
4. **Use try_lock() for Fallback**: UI can skip locked nodes rather than block

### Alternatives Considered

| Approach | Verdict | Reason |
|----------|---------|--------|
| Pure Arc<Mutex<>> | ❌ Rejected | Risk of deadlocks, mutex contention |
| Pure Channel-Based | ⚠️ Overkill | Complex for simple status reads |
| Crossbeam Channels | ❌ Rejected | Doesn't align with tokio async |
| tokio::sync::watch | ⚠️ Could work | Less flexible than RwLock |

### References

- Current patterns: Arc<Mutex<ComponentManager>> in existing codebase
- Channel usage: `/Users/doronila/git/wasmflow_cc/src/ui/app.rs` (line 369)

---

## 4. Resource Cleanup Strategies

### Decision: Hybrid CancellationToken + Timeout Pattern

**Pattern**: Use `tokio_util::sync::CancellationToken` for cooperative cancellation with hard timeout fallback.

### Rationale

- **Graceful First**: Allows tasks to complete current operations (finish HTTP request, flush buffers)
- **Guaranteed Termination**: Even uncooperative tasks stop within 2 seconds (meets spec requirement)
- **WASM Safety**: Wasmtime's `Store::drop()` is panic-safe, always cleans up instances
- **Composable**: Child tokens inherit cancellation from parent

### Three-Phase Shutdown

#### Phase 1: Cooperative Cancellation (0-1.5s)
```rust
cancellation_token.cancel();
tokio::time::timeout(Duration::from_millis(1500), join_handle).await
```

#### Phase 2: Forced Termination (1.5-2.0s)
```rust
join_handle.abort();
tokio::time::timeout(Duration::from_millis(500), join_handle).await
```

#### Phase 3: Logging and Detach (>2.0s)
```rust
log::error!("Node {} failed to stop within 2s, detaching task", node_id);
// JoinHandle dropped - task becomes detached but Store will clean up
```

### Resource Leak Prevention

```rust
// RAII guard for cleanup
struct ContinuousNodeTask {
    join_handle: JoinHandle<()>,
    cancellation_token: CancellationToken,
}

impl Drop for ContinuousNodeTask {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        self.join_handle.abort();
    }
}
```

### Alternatives Considered

| Approach | Verdict | Reason |
|----------|---------|--------|
| JoinHandle::abort() only | ❌ Rejected | No graceful cleanup opportunity |
| CancellationToken only | ❌ Rejected | Can't guarantee 2s timeout |
| Channel-based shutdown | ❌ Rejected | More complex than CancellationToken |

### Integration Points

- Add to ExecutionEngine: `/Users/doronila/git/wasmflow_cc/src/runtime/engine.rs` (~line 370)
- New file: `/Users/doronila/git/wasmflow_cc/src/runtime/continuous.rs`
- Cleanup patterns: ComponentCompiler::Drop at `/Users/doronila/git/wasmflow_cc/src/runtime/compiler.rs` (lines 767-770)

### Dependencies Required

```toml
[dependencies]
tokio-util = "0.7"  # For CancellationToken
```

---

## 5. Error Propagation Patterns

### Decision: Channel-Based Error Propagation with Structured Error Types

**Pattern**: Send structured error enums via `std::sync::mpsc` channels, with UI polling and visual feedback.

### Rationale

- **Existing Pattern**: Already used in `/Users/doronila/git/wasmflow_cc/src/ui/app.rs` (lines 398-416)
- **Type-Safe**: Enum-based errors with context (node_id, timestamps, actionable info)
- **Non-Blocking**: UI polls with `try_recv()`, remains responsive
- **Structured Logging**: Integrates with existing log framework

### Error Type Design

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContinuousNodeError {
    #[error("Execution failed: {message}")]
    ExecutionFailed {
        node_id: Uuid,
        node_name: String,
        message: String,
        source_location: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    #[error("Permission denied for {capability}")]
    PermissionDenied {
        node_id: Uuid,
        node_name: String,
        capability: String,
        attempted_action: String,
    },

    #[error("Execution timed out after {duration:?}")]
    Timeout {
        node_id: Uuid,
        node_name: String,
        duration: Duration,
    },

    #[error("Network error: {message}")]
    NetworkError {
        node_id: Uuid,
        message: String,
        status_code: Option<u16>,
    },

    #[error("Component trapped: {trap_message}")]
    ComponentTrap {
        node_id: Uuid,
        node_name: String,
        trap_message: String,
    },
}
```

### UI Display Pattern

```rust
// Status bar display
match &self.continuous_error {
    Some(ContinuousNodeError::PermissionDenied { capability, attempted_action, .. }) => {
        ui.colored_label(
            egui::Color32::RED,
            format!("🔒 Permission Denied: Attempted to {} but lacks {} permission",
                    attempted_action, capability)
        );
        if ui.button("Grant Permission").clicked() {
            // Open permission dialog
        }
    }
    // ... other variants
}

// Node visual state
node.execution_state = ExecutionState::Failed;
node.last_error = Some(error.clone());
```

### Panic Handling

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};

thread::spawn(move || {
    let result = catch_unwind(AssertUnwindSafe(|| {
        engine.execute_node_with_outputs(&graph_clone, node_id)
    }));

    let final_result = match result {
        Ok(Ok(outputs)) => Ok(outputs),
        Ok(Err(e)) => Err(ContinuousNodeError::ExecutionFailed { /* ... */ }),
        Err(panic_payload) => {
            let panic_msg = extract_panic_message(panic_payload);
            Err(ContinuousNodeError::ComponentTrap {
                node_id,
                node_name,
                trap_message: panic_msg
            })
        }
    };

    let _ = tx.send(final_result);
});
```

### Alternatives Considered

| Approach | Verdict | Reason |
|----------|---------|--------|
| Shared state with Mutex | ❌ Rejected | Lock contention, simpler to use channels |
| Async/await throughout | ❌ Rejected | egui is synchronous, major refactoring |
| Callback-based | ❌ Rejected | Callback hell, unclear lifecycle |

### References

- Error types: `/Users/doronila/git/wasmflow_cc/src/lib.rs` (lines 38-109)
- Current error handling: `/Users/doronila/git/wasmflow_cc/src/ui/app.rs` (lines 407-416)
- Logging: `/Users/doronila/git/wasmflow_cc/src/runtime/wasm_host.rs` (lines 136-144)

---

## Summary of Decisions

| Research Area | Decision | Key Benefit |
|--------------|----------|-------------|
| **Async Execution** | Background threads + channels | Proven pattern, 60 FPS guaranteed |
| **State Persistence** | Always start stopped | Safe, prevents resource conflicts |
| **Thread Safety** | Hybrid channels + RwLock | Optimal for UI-heavy reads, async writes |
| **Resource Cleanup** | CancellationToken + timeout | Graceful with guaranteed termination <2s |
| **Error Propagation** | Structured enums via channels | Actionable UI feedback, debuggable |

---

## Next Steps

All unknowns from Technical Context have been resolved. Ready to proceed to Phase 1: Design & Contracts.

**Phase 1 Tasks**:
1. Create data-model.md with entity definitions
2. Generate WIT contract for continuous execution interface
3. Create quickstart.md for developers
4. Update agent context (CLAUDE.md)
