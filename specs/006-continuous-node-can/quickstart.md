# Quickstart: Building Continuous Execution Nodes

**Feature**: 006-continuous-node-can
**Audience**: Component developers
**Prerequisites**: Familiarity with WASM Component Model, Rust, and WIT interfaces

## Overview

This guide shows you how to create a WASM component that runs continuously in WasmFlow, like an HTTP server or file watcher. Continuous nodes differ from regular nodes by running indefinitely until explicitly stopped by the user.

---

## Quick Comparison

| Aspect | Single Execution Node | Continuous Execution Node |
|--------|----------------------|---------------------------|
| **Execution** | Runs once when triggered | Runs repeatedly until stopped |
| **UI Control** | Execute button | Play/Stop buttons |
| **Use Cases** | Data transformation, calculations | Servers, listeners, monitors |
| **Lifecycle** | Execute → Complete | Initialize → Loop → Cleanup |
| **Example** | Parse JSON | HTTP server |

---

## Step 1: Set Up Your Component Project

```bash
# Create new component project
cargo component new my-continuous-node --lib

cd my-continuous-node
```

### Add Dependencies

**Cargo.toml**:
```toml
[package]
name = "my-continuous-node"
version = "0.1.0"
edition = "2021"

[dependencies]
wit-bindgen = "0.30"

[lib]
crate-type = ["cdylib"]

[package.metadata.component]
package = "my-org:my-continuous-node"
```

---

## Step 2: Define WIT Interface

**wit/world.wit**:
```wit
package my-org:my-continuous-node@1.0.0;

interface types {
    use wasmflow:types@1.0.0.{node-value};
    use wasmflow:continuous@1.0.0.{continuous-error, execution-stats};
}

world continuous-node {
    import wasmflow:continuous/continuous-host@1.0.0;
    export wasmflow:continuous/continuous-component@1.0.0;
}
```

---

## Step 3: Implement the Component

**src/lib.rs**:
```rust
use wasmflow_continuous::{
    ContinuousComponent, ContinuousError, ExecutionStats,
    ContinuousHost, LogLevel,
};

// Component state (persisted across iterations)
struct MyComponent {
    iteration_count: u64,
    last_value: Option<String>,
}

impl ContinuousComponent for MyComponent {
    /// Declare continuous execution support
    fn supports_continuous() -> bool {
        true
    }

    /// Initialize when continuous execution starts
    fn initialize_continuous(
        inputs: Vec<(String, NodeValue)>
    ) -> Result<Self, ContinuousError> {
        // Parse configuration from inputs
        let config = parse_config(&inputs)?;

        // Set up long-lived resources (e.g., open connection)
        let state = MyComponent {
            iteration_count: 0,
            last_value: None,
        };

        ContinuousHost::log(LogLevel::Info, "Initialized continuous execution");

        Ok(state)
    }

    /// Execute one iteration
    fn execute_iteration(
        &mut self,
        inputs: Vec<(String, NodeValue)>
    ) -> Result<Vec<(String, NodeValue)>, ContinuousError> {
        self.iteration_count += 1;

        // Process inputs
        let result = process_inputs(&inputs)?;
        self.last_value = Some(result.clone());

        // Update progress indicator
        ContinuousHost::update_progress(
            &format!("Processed {} iterations", self.iteration_count)
        );

        // Return outputs
        Ok(vec![
            ("output".to_string(), NodeValue::String(result)),
            ("count".to_string(), NodeValue::U64(self.iteration_count)),
        ])
    }

    /// Check if should continue running
    fn should_continue(&self) -> bool {
        // Could check for shutdown signals, error conditions, etc.
        // Return false to stop gracefully
        true
    }

    /// Cleanup when execution stops
    fn cleanup_continuous(self) -> Result<(), ContinuousError> {
        ContinuousHost::log(
            LogLevel::Info,
            &format!("Stopping after {} iterations", self.iteration_count)
        );

        // Release resources (close connections, flush buffers)
        // Must complete within 2 seconds

        Ok(())
    }

    /// Get execution statistics (optional)
    fn get_stats(&self) -> Option<ExecutionStats> {
        Some(ExecutionStats {
            iterations: self.iteration_count,
            total_duration_ms: 0, // Tracked by host
            avg_iteration_ms: None,
            last_iteration_ms: None,
        })
    }
}
```

---

## Step 4: Build and Test

```bash
# Build the component
cargo component build --release

# Component will be at: target/wasm32-wasip2/release/my_continuous_node.wasm
```

### Test Locally

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let inputs = vec![
            ("config".to_string(), NodeValue::String("test".into())),
        ];

        let result = MyComponent::initialize_continuous(inputs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_iteration() {
        let mut component = MyComponent {
            iteration_count: 0,
            last_value: None,
        };

        let inputs = vec![
            ("input".to_string(), NodeValue::String("data".into())),
        ];

        let result = component.execute_iteration(inputs);
        assert!(result.is_ok());
        assert_eq!(component.iteration_count, 1);
    }

    #[test]
    fn test_cleanup() {
        let component = MyComponent {
            iteration_count: 100,
            last_value: Some("test".into()),
        };

        let result = component.cleanup_continuous();
        assert!(result.is_ok());
    }
}
```

---

## Step 5: Use in WasmFlow

### Load Component

1. Open WasmFlow
2. Add new component: **File → Add Component**
3. Select `my_continuous_node.wasm`
4. Component appears in node palette with play/stop icons

### Add to Graph

1. Drag component from palette to canvas
2. Notice play/stop buttons (instead of execute button)
3. Connect inputs/outputs as usual
4. Click play to start continuous execution

### Monitor Execution

- **Running indicator**: Green pulsing border
- **Status bar**: Iteration count, progress updates
- **Error display**: Red border + error message
- **Logs**: Check console for component logs

### Stop Execution

- Click stop button
- Node shows "Stopping..." state
- Cleanup completes within 2 seconds
- Node returns to idle state

---

## Example: HTTP Server Component

Here's a more complex example showing an HTTP server:

```rust
use wasmflow_continuous::*;
use std::collections::HashMap;

struct HttpServer {
    port: u16,
    request_count: u64,
    // Note: Actual server would use WASI HTTP imports
}

impl ContinuousComponent for HttpServer {
    fn supports_continuous() -> bool {
        true
    }

    fn initialize_continuous(
        inputs: Vec<(String, NodeValue)>
    ) -> Result<Self, ContinuousError> {
        // Parse port from inputs
        let port = inputs.iter()
            .find(|(k, _)| k == "port")
            .and_then(|(_, v)| {
                if let NodeValue::U32(p) = v {
                    Some(*p as u16)
                } else {
                    None
                }
            })
            .ok_or_else(|| ContinuousError {
                message: "Missing 'port' input".into(),
                source_location: None,
                timestamp: current_timestamp(),
                category: ErrorCategory::ExecutionFailed,
            })?;

        // Bind to port (requires network capability)
        // let listener = bind_to_port(port)?;

        ContinuousHost::log(
            LogLevel::Info,
            &format!("HTTP server listening on port {}", port)
        );

        Ok(HttpServer {
            port,
            request_count: 0,
        })
    }

    fn execute_iteration(
        &mut self,
        _inputs: Vec<(String, NodeValue)>
    ) -> Result<Vec<(String, NodeValue)>, ContinuousError> {
        // Check for new HTTP requests (non-blocking)
        // let requests = check_for_requests()?;

        // Process requests
        // for req in requests {
        //     handle_request(req)?;
        //     self.request_count += 1;
        // }

        // Simulate processing
        self.request_count += 1;

        // Return stats as outputs
        Ok(vec![
            ("request_count".to_string(), NodeValue::U64(self.request_count)),
            ("port".to_string(), NodeValue::U32(self.port as u32)),
        ])
    }

    fn should_continue(&self) -> bool {
        // Could check for shutdown signal from HTTP request
        true
    }

    fn cleanup_continuous(self) -> Result<(), ContinuousError> {
        ContinuousHost::log(
            LogLevel::Info,
            &format!("Shutting down server after {} requests", self.request_count)
        );

        // Stop accepting connections
        // Wait for active requests to complete
        // Close listener

        Ok(())
    }

    fn get_stats(&self) -> Option<ExecutionStats> {
        Some(ExecutionStats {
            iterations: self.request_count,
            total_duration_ms: 0,
            avg_iteration_ms: None,
            last_iteration_ms: None,
        })
    }
}
```

---

## Best Practices

### 1. Keep Iterations Short

Execute iterations should complete quickly (<100ms) to allow responsive shutdown:

```rust
// ❌ BAD: Long-running iteration
fn execute_iteration(&mut self, inputs: Vec<...>) -> Result<...> {
    std::thread::sleep(Duration::from_secs(10)); // Blocks shutdown!
    Ok(vec![])
}

// ✅ GOOD: Quick iteration with non-blocking checks
fn execute_iteration(&mut self, inputs: Vec<...>) -> Result<...> {
    // Check for new data (non-blocking)
    if let Some(data) = check_for_data() {
        process_data(data)?;
    }
    Ok(vec![])
}
```

### 2. Handle Cleanup Gracefully

Cleanup must complete within 2 seconds or will be force-terminated:

```rust
fn cleanup_continuous(self) -> Result<(), ContinuousError> {
    // ✅ Set shutdown flag
    self.set_shutdown_flag(true);

    // ✅ Wait briefly for operations to complete
    self.wait_for_completion(Duration::from_millis(500));

    // ✅ Force close remaining resources
    self.force_close_all();

    Ok(())
}
```

### 3. Use Host Logging

Use `ContinuousHost::log()` instead of println/eprintln:

```rust
// ❌ BAD: Won't appear in WasmFlow logs
println!("Processing request");

// ✅ GOOD: Appears in WasmFlow log panel
ContinuousHost::log(LogLevel::Info, "Processing request");
```

### 4. Return Meaningful Outputs

Update outputs each iteration to show progress:

```rust
fn execute_iteration(&mut self, inputs: Vec<...>) -> Result<Vec<...>> {
    // ... processing ...

    Ok(vec![
        ("status".to_string(), NodeValue::String("running".into())),
        ("processed".to_string(), NodeValue::U64(self.processed_count)),
        ("last_update".to_string(), NodeValue::String(current_timestamp())),
    ])
}
```

### 5. Declare Required Capabilities

In component metadata, declare what permissions you need:

```toml
[package.metadata.component.capabilities]
network = ["127.0.0.1:8080"]
files = ["read:/data/input"]
```

---

## Troubleshooting

### Component Won't Start

**Problem**: Play button is grayed out

**Solutions**:
- Check that `supports_continuous()` returns `true`
- Verify component implements all required interface methods
- Ensure capabilities are granted in WasmFlow

### Execution Stops Immediately

**Problem**: Node stops right after starting

**Solutions**:
- Check `should_continue()` returns `true`
- Look for errors in `initialize_continuous()` or `execute_iteration()`
- Check WasmFlow error log for exception details

### Can't Stop Node

**Problem**: Stop button doesn't work

**Solutions**:
- Ensure `execute_iteration()` completes quickly (<100ms)
- Avoid blocking operations in iteration
- Check `cleanup_continuous()` doesn't hang

### Memory Leak

**Problem**: Memory usage grows over time

**Solutions**:
- Clear temporary data each iteration
- Use bounded collections (limit size)
- Profile with `cargo instrument` to find leaks

---

## Advanced Topics

### Input Change Detection

```rust
struct MyComponent {
    last_inputs: HashMap<String, NodeValue>,
}

fn execute_iteration(&mut self, inputs: Vec<...>) -> Result<...> {
    let inputs_map: HashMap<_, _> = inputs.into_iter().collect();

    // Only process if inputs changed
    if inputs_map != self.last_inputs {
        process_changed_inputs(&inputs_map)?;
        self.last_inputs = inputs_map;
    }

    Ok(vec![])
}
```

### Rate Limiting

```rust
use std::time::{Duration, Instant};

struct MyComponent {
    last_execution: Instant,
    min_interval: Duration,
}

fn execute_iteration(&mut self, inputs: Vec<...>) -> Result<...> {
    // Skip iteration if too soon
    if self.last_execution.elapsed() < self.min_interval {
        return Ok(vec![]); // No output this iteration
    }

    // ... actual work ...

    self.last_execution = Instant::now();
    Ok(vec![...])
}
```

### Graceful Shutdown on Signal

```rust
fn should_continue(&self) -> bool {
    // Check for shutdown signal from external source
    if self.received_shutdown_signal() {
        ContinuousHost::request_stop();
        return false;
    }
    true
}
```

---

## Next Steps

- **See Example Components**: `/components/examples/http-server/`
- **Read WIT Spec**: `/specs/006-continuous-node-can/contracts/continuous-execution.wit`
- **API Reference**: `/docs/api/continuous-nodes.md`
- **Component Development Guide**: `/docs/component-development.md`

---

## Support

- **Issues**: https://github.com/your-org/wasmflow/issues
- **Discussions**: https://github.com/your-org/wasmflow/discussions
- **Examples**: https://github.com/your-org/wasmflow-components
