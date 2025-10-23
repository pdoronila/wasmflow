# Quickstart: HTTP Fetch Component Development

**Feature**: HTTP Fetch Component with Real Network Capability  
**Audience**: Developers building HTTP-enabled WasmFlow components  
**Prerequisites**: Rust 1.75+, cargo-component installed

## Overview

This guide walks through developing a WasmFlow component that makes real HTTP GET requests with capability-based security.

**What you'll build**:
- A component that fetches data from HTTP/HTTPS endpoints
- Capability declaration for network access
- Proper error handling for network failures
- Configurable timeout support

**Time estimate**: 30 minutes

---

## Step 1: Set Up Component Project

Create a new component using cargo-component:

```bash
# Create component directory
mkdir my-http-component
cd my-http-component

# Initialize with cargo-component
cargo component new --lib .

# Add to target
rustup target add wasm32-wasip2
```

**Cargo.toml**:
```toml
[package]
name = "my-http-component"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[package.metadata.component]
package = "wasmflow:node"

[package.metadata.component.target]
path = "wit"

[dependencies]
cargo-component-bindings = "0.6"
wit-bindgen-rt = "0.44"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

---

## Step 2: Set Up WIT Interfaces

Create the WIT directory structure:

```bash
mkdir -p wit/deps/wasmflow-node
mkdir -p wit/deps/wasmflow-http
```

**wit/deps/wasmflow-node/node.wit**:
```wit
// Copy from /wit/node.wit in main repository
// This defines the base interface (metadata, execution, types)
```

**wit/deps/wasmflow-http/http-host.wit**:
```wit
// Copy from /specs/002-lets-focus-on/contracts/http-host.wit
// This defines the HTTP host function interface
```

**wit/world.wit**:
```wit
package my:http-component;

world component {
    import wasmflow:http/http-host;
    import wasmflow:node/host;
    export wasmflow:node/metadata;
    export wasmflow:node/execution;
}
```

---

## Step 3: Implement Component Metadata

**src/lib.rs**:

```rust
#[allow(warnings)]
mod bindings;

use bindings::exports::wasmflow::node::metadata::Guest as MetadataGuest;
use bindings::exports::wasmflow::node::execution::Guest as ExecutionGuest;
use bindings::wasmflow::node::types::*;
use bindings::wasmflow::http::http_host;
use bindings::wasmflow::node::host;

struct Component;

// Step 3a: Define component metadata
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "My HTTP Fetch".to_string(),
            version: "1.0.0".to_string(),
            description: "Fetches data from HTTP endpoints".to_string(),
            author: "Your Name".to_string(),
            category: Some("Network".to_string()),
        }
    }

    // Step 3b: Define input ports
    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "url".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "HTTP URL to fetch".to_string(),
            },
            PortSpec {
                name: "timeout".to_string(),
                data_type: DataType::U32Type,
                optional: true,  // Optional: defaults to 30s
                description: "Request timeout in seconds (default: 30)".to_string(),
            },
        ]
    }

    // Step 3c: Define output ports
    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "body".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Response body as text".to_string(),
            },
            PortSpec {
                name: "status".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "HTTP status code".to_string(),
            },
        ]
    }

    // Step 3d: Declare network capabilities
    fn get_capabilities() -> Option<Vec<String>> {
        Some(vec![
            "network:api.github.com".to_string(),
            "network:httpbin.org".to_string(),
        ])
    }
}

// Export the component
bindings::export!(Component with_types_in bindings);
```

**Key Points**:
- `get_capabilities()`: Declares which domains this component can access
- `timeout` port is optional (has default value)
- Component requests approval for `api.github.com` and `httpbin.org`

---

## Step 4: Implement Execution Logic

Add the execution implementation to `src/lib.rs`:

```rust
// Helper: Extract string input
fn extract_string(inputs: &[(String, Value)], name: &str) -> Result<String, ExecutionError> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| match v {
            Value::StringVal(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| ExecutionError {
            message: format!("Missing or invalid '{}' input", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!("Connect a String value to the '{}' port", name)),
        })
}

// Helper: Extract optional u32 input
fn extract_optional_u32(inputs: &[(String, Value)], name: &str, default: u32) -> u32 {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| match v {
            Value::U32Val(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(default)
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("info", "My HTTP Fetch component executing");

        // Step 4a: Extract inputs
        let url = extract_string(&inputs, "url")?;
        let timeout_secs = extract_optional_u32(&inputs, "timeout", 30);

        // Step 4b: Validate timeout range
        if timeout_secs < 1 || timeout_secs > 300 {
            return Err(ExecutionError {
                message: "Timeout must be between 1 and 300 seconds".to_string(),
                input_name: Some("timeout".to_string()),
                recovery_hint: Some("Use a value between 1 and 300".to_string()),
            });
        }

        let timeout_ms = timeout_secs * 1000;

        host::log("info", &format!("Fetching URL: {} (timeout: {}s)", url, timeout_secs));

        // Step 4c: Call HTTP host function
        let response = http_host::http_get(&url, timeout_ms).map_err(|err_msg| {
            host::log("error", &format!("HTTP request failed: {}", err_msg));
            
            // Map error message to user-friendly ExecutionError
            let (message, recovery_hint) = if err_msg.contains("Access denied") {
                (
                    err_msg,
                    Some("This component can only access: api.github.com, httpbin.org".to_string()),
                )
            } else if err_msg.contains("timed out") {
                (
                    err_msg,
                    Some("Server is slow. Try increasing timeout or check connectivity.".to_string()),
                )
            } else if err_msg.contains("DNS") {
                (
                    err_msg,
                    Some("Check domain spelling and internet connection".to_string()),
                )
            } else {
                (err_msg, Some("Check URL and network connection".to_string()))
            };

            ExecutionError {
                message,
                input_name: Some("url".to_string()),
                recovery_hint,
            }
        })?;

        // Step 4d: Log success
        host::log(
            "info",
            &format!(
                "HTTP request successful: status={}, body_len={}",
                response.status,
                response.body.len()
            ),
        );

        // Step 4e: Return outputs
        Ok(vec![
            ("body".to_string(), Value::StringVal(response.body)),
            ("status".to_string(), Value::U32Val(response.status)),
        ])
    }
}
```

**Key Points**:
- Extract URL and optional timeout from inputs
- Validate timeout is in range (1-300 seconds)
- Call `http_host::http_get()` with URL and timeout
- Map errors to user-friendly messages with recovery hints
- Return body and status as outputs

---

## Step 5: Build the Component

```bash
# Build for WASM target
cargo component build --target wasm32-wasip2 --release

# Component will be at:
# target/wasm32-wasip2/release/my_http_component.wasm
```

**Troubleshooting**:
- Error: "target not found" → Run `rustup target add wasm32-wasip2`
- Error: "cargo-component not found" → Run `cargo install cargo-component`

---

## Step 6: Test the Component

### Unit Tests

Add to `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_string() {
        let inputs = vec![("url".to_string(), Value::StringVal("https://example.com".to_string()))];
        assert_eq!(extract_string(&inputs, "url").unwrap(), "https://example.com");
    }

    #[test]
    fn test_extract_optional_u32_with_default() {
        let inputs = vec![];
        assert_eq!(extract_optional_u32(&inputs, "timeout", 30), 30);
    }

    #[test]
    fn test_extract_optional_u32_with_value() {
        let inputs = vec![("timeout".to_string(), Value::U32Val(10))];
        assert_eq!(extract_optional_u32(&inputs, "timeout", 30), 10);
    }
}
```

Run tests:
```bash
cargo test
```

### Integration Test (Manual)

1. Copy component to WasmFlow:
```bash
cp target/wasm32-wasip2/release/my_http_component.wasm /path/to/wasmflow/components/
```

2. Launch WasmFlow:
```bash
cd /path/to/wasmflow
cargo run --release
```

3. Create test graph:
   - Add a Constant node with value: `"https://httpbin.org/get"`
   - Add your My HTTP Fetch node
   - Connect Constant → url port
   - Execute graph

4. Verify capability prompt:
   - System should prompt: "My HTTP Fetch requests access to: api.github.com, httpbin.org"
   - Approve the request

5. Check outputs:
   - `body` should contain JSON response from httpbin
   - `status` should be `200`

---

## Step 7: Handle Edge Cases

### 7a: Add Headers Output (Optional)

Update `get_outputs()`:
```rust
fn get_outputs() -> Vec<PortSpec> {
    vec![
        PortSpec {
            name: "body".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Response body as text".to_string(),
        },
        PortSpec {
            name: "status".to_string(),
            data_type: DataType::U32Type,
            optional: false,
            description: "HTTP status code".to_string(),
        },
        PortSpec {
            name: "headers".to_string(),
            data_type: DataType::StringType,
            optional: true,  // Optional output
            description: "Response headers as JSON".to_string(),
        },
    ]
}
```

Update `execute()`:
```rust
// Return headers output
Ok(vec![
    ("body".to_string(), Value::StringVal(response.body)),
    ("status".to_string(), Value::U32Val(response.status)),
    ("headers".to_string(), Value::StringVal(response.headers)),  // NEW
])
```

### 7b: Add More Detailed Logging

```rust
// Log input details
host::log(
    "debug",
    &format!(
        "Inputs: url='{}', timeout={}s",
        url, timeout_secs
    ),
);

// Log response details
host::log(
    "debug",
    &format!(
        "Response: status={}, content-type={}, body_preview='{}'",
        response.status,
        extract_content_type(&response.headers),
        &response.body.chars().take(100).collect::<String>()
    ),
);
```

### 7c: Validate URL Format

```rust
// Before calling http_get
if !url.starts_with("http://") && !url.starts_with("https://") {
    return Err(ExecutionError {
        message: "Invalid URL: must start with http:// or https://".to_string(),
        input_name: Some("url".to_string()),
        recovery_hint: Some("Ensure URL includes the protocol (http:// or https://)".to_string()),
    });
}
```

---

## Common Patterns

### Pattern 1: Fetch JSON and Parse

```rust
// In execute()
let response = http_host::http_get(&url, timeout_ms)?;

// Check content-type
if !response.headers.contains("\"content-type\":\"application/json\"") {
    host::log("warn", "Response is not JSON");
}

// Return raw JSON string (downstream nodes can parse it)
Ok(vec![
    ("json".to_string(), Value::StringVal(response.body)),
    ("status".to_string(), Value::U32Val(response.status)),
])
```

### Pattern 2: Retry on Failure

```rust
const MAX_RETRIES: u32 = 3;

for attempt in 1..=MAX_RETRIES {
    match http_host::http_get(&url, timeout_ms) {
        Ok(response) => {
            host::log("info", &format!("Request succeeded on attempt {}", attempt));
            return Ok(vec![
                ("body".to_string(), Value::StringVal(response.body)),
                ("status".to_string(), Value::U32Val(response.status)),
            ]);
        }
        Err(err) if attempt < MAX_RETRIES => {
            host::log("warn", &format!("Attempt {} failed: {}. Retrying...", attempt, err));
            // In real implementation, add delay between retries
            continue;
        }
        Err(err) => {
            return Err(ExecutionError {
                message: format!("All {} attempts failed: {}", MAX_RETRIES, err),
                input_name: Some("url".to_string()),
                recovery_hint: Some("Check server availability and network connection".to_string()),
            });
        }
    }
}

unreachable!()
```

### Pattern 3: Conditional URL Construction

```rust
// Multiple inputs for URL building
fn get_inputs() -> Vec<PortSpec> {
    vec![
        PortSpec {
            name: "base_url".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Base API URL".to_string(),
        },
        PortSpec {
            name: "endpoint".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "API endpoint path".to_string(),
        },
    ]
}

// In execute()
let base_url = extract_string(&inputs, "base_url")?;
let endpoint = extract_string(&inputs, "endpoint")?;

// Construct full URL
let url = if base_url.ends_with('/') {
    format!("{}{}", base_url, endpoint.trim_start_matches('/'))
} else {
    format!("{}/{}", base_url, endpoint.trim_start_matches('/'))
};

host::log("info", &format!("Constructed URL: {}", url));
```

---

## Debugging Tips

### Enable Debug Logging

```bash
# Run WasmFlow with debug logging
RUST_LOG=debug cargo run --release
```

### Common Errors

| Error Message | Cause | Solution |
|---------------|-------|----------|
| "Access denied: X not in approved capabilities" | URL domain not in capability list | Add domain to `get_capabilities()` |
| "Request timed out after Xms" | Server slow or unreachable | Increase timeout or check connectivity |
| "Invalid URL: missing scheme" | URL doesn't start with http:// | Validate URL format before calling http_get |
| "DNS resolution failed" | Domain doesn't exist | Check domain spelling |
| "Cross-domain redirect blocked" | Redirect to unapproved domain | Add redirect target domain to capabilities |

### Component Not Loading

```bash
# Verify component is valid
cargo component check

# Verify WIT interfaces are correct
cargo component wit

# Check component location
ls -la /path/to/wasmflow/components/my_http_component.wasm
```

---

## Next Steps

1. **Add Custom Error Types**: Create enum for error categories
2. **Implement POST Requests**: Wait for POST support in http-host interface
3. **Add Response Caching**: Store responses in temp directory
4. **Build Composite Components**: Combine HTTP Fetch with JSON parser nodes
5. **Publish to Component Registry**: Share your component with others

---

## Resources

- **WIT Specification**: https://component-model.bytecodealliance.org/design/wit.html
- **cargo-component Docs**: https://github.com/bytecodealliance/cargo-component
- **WasmFlow Examples**: `/examples/` directory in repository
- **HTTP Host Contract**: `/specs/002-lets-focus-on/contracts/http-host.wit`
- **Data Model Reference**: `/specs/002-lets-focus-on/data-model.md`

---

## Support

Questions? Check:
- WasmFlow issues: https://github.com/wasmflow/wasmflow/issues
- Component development guide: `/docs/BUILDING_COMPONENTS.md`
- WIT contract tests: `/tests/contract/`
