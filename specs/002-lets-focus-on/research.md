# Research: HTTP Fetch Component Implementation

**Feature**: HTTP Fetch Component with Real Network Capability  
**Date**: 2025-10-14  
**Status**: Complete

## Overview

This document captures technical research and decisions for implementing real HTTP GET requests in WasmFlow components. The primary challenge is bridging WASM components (sandboxed, no direct network access) with native HTTP clients while maintaining capability-based security.

## Key Research Areas

### 1. WASM Component HTTP Access Patterns

**Question**: How can a WASM component make HTTP requests given WASM's sandboxed nature?

**Research Findings**:

Three approaches were evaluated:

1. **WASI HTTP (wasi:http)** - Proposed WASI standard for HTTP client/server
   - Status: Preview, not stable in WASI Preview 2
   - Support: Limited wasmtime support, experimental
   - Verdict: Too unstable for production use

2. **Host Functions (Custom Import)** - Component imports HTTP functions from host
   - Implementation: Host provides custom WIT interface with HTTP methods
   - Component calls host function with URL, host performs request, returns result
   - Verdict: Most flexible, full control over security model

3. **Embedded HTTP Client in WASM** - Compile reqwest/hyper to WASM
   - Challenge: Requires WASI socket support, large binary size
   - Complexity: Network stack in WASM, TLS certificate handling
   - Verdict: Not feasible with current WASI Preview 2 limitations

**Decision**: **WASI HTTP (Option 1)** - ⚠️ Experimental

**Rationale** (Updated based on project goals):
- **Experimental focus**: This application is meant to explore WASM components and composition, not production deployment
- **Standards-based approach**: WASI HTTP is the emerging standard for HTTP in WASM components
- **Learning opportunity**: Experimenting with preview features provides valuable insights into component model evolution
- **True component isolation**: Components have direct HTTP access without custom host functions
- **Future-proof**: As WASI HTTP stabilizes, this implementation will be compatible

**Implementation Approach**:
- Components import `wasi:http/outgoing-handler` interface
- Use `wasi:http/types` for request/response types
- wasmtime 27.0+ has experimental support via `wasi-http` feature
- Capability validation handled by WASI runtime configuration

**Trade-offs Accepted**:
- ⚠️ Preview/unstable API - may change in future WASI versions
- ⚠️ Limited wasmtime support - requires experimental features enabled
- ⚠️ Potential bugs - preview features may have issues
- ✅ Acceptable for experimental/educational project

**Implementation Notes**:
- Enable `wasi-http` feature in wasmtime dependency
- Components use standard WASI HTTP imports (no custom host functions needed)
- Capability control via wasmtime `WasiHttpCtx` configuration
- Redirect handling via WASI HTTP redirect policy (if available)

**Alternatives Considered**:
- Custom host functions: Would work but defeats purpose of exploring standard WASI interfaces
- Embedded client: Still rejected due to WASI socket limitations and binary size

---

### 2. WASI HTTP Runtime Integration

**Question**: How does wasmtime provide WASI HTTP support to components?

**Research Findings**:

WASI HTTP is provided through the `wasmtime-wasi-http` crate:

| Aspect | Implementation | Notes |
|--------|---------------|-------|
| **HTTP Client** | Built into wasmtime | Components import `wasi:http/outgoing-handler` |
| **TLS Support** | Native (via rustls/native-tls) | Handled by wasmtime runtime |
| **Async/Sync** | Async-first | Integrates with wasmtime's async runtime |
| **Redirects** | Automatic (configurable) | Part of WASI HTTP spec |
| **Timeout** | Per-request | Components specify timeout in request |

**Decision**: **Use wasmtime-wasi-http crate**

**Rationale**:
- Standard WASI interface - components are portable across WASI-compliant runtimes
- No custom host function implementation needed
- HTTP functionality is built into the component model
- Reduces wasmflow-specific code (leverage standard libraries)

**Configuration**:
```rust
// In wasmtime host setup
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

let mut linker = Linker::new(&engine);

// Add WASI HTTP support
wasmtime_wasi_http::add_to_linker_async(&mut linker)?;

// Configure HTTP context with capabilities
let http_ctx = WasiHttpCtx::new();
// Capability control happens via wasmtime's resource limits
```

**Component Side**:
```wit
// Component imports WASI HTTP
import wasi:http/outgoing-handler@0.2.0;
import wasi:http/types@0.2.0;

// In component code:
let request = wasi::http::types::OutgoingRequest::new(...);
let response = wasi::http::outgoing_handler::handle(request, None)?;
```

**Capability Validation**:
- WASI HTTP doesn't have built-in domain allowlisting (preview limitation)
- Will need to implement custom validation layer:
  - Option A: Wrap wasi-http with validation interceptor
  - Option B: Validate URLs in component before making request (trust-based)
  - **Chosen**: Option A - Add validation in wasmtime host via custom resource limiter

**Alternatives Considered**:
- reqwest in host: Would require custom host functions (less standards-aligned)
- Pure component-side validation: Insufficient security (components could bypass)

---

### 3. Async Execution Model

**Question**: How does async HTTP execution integrate with WasmFlow's component execution engine?

**Research Findings**:

Current WasmFlow execution is synchronous:
- `engine.rs` executes components in topological order
- Each component blocks until completion
- Works for fast pure computation (<10ms)

HTTP requests are slow (100ms - 30s):
- Blocking execution would freeze UI
- Need async component invocation

**Decision**: **Async Component Execution with wasmtime + WASI HTTP**

**Rationale**:
- WASI HTTP is inherently async (uses wasi:io/poll for async operations)
- wasmtime's async runtime integrates seamlessly with WASI HTTP
- WasmFlow already depends on tokio - wasmtime will use it
- Allows UI thread to remain responsive during network I/O

**Implementation Approach**:

1. **Component Side** (uses WASI HTTP async):
```rust
// Component imports WASI HTTP
use wasi::http::outgoing_handler;
use wasi::http::types::{OutgoingRequest, Method, Scheme};

// Create request
let request = OutgoingRequest::new(Headers::new());
request.set_method(&Method::Get)?;
request.set_scheme(Some(&Scheme::Https))?;
request.set_authority(Some("api.example.com"))?;
request.set_path_with_query(Some("/data"))?;

// Make async HTTP call (WASI handles async internally)
let future_response = outgoing_handler::handle(request, None)?;
let response = future_response.get()?; // Blocks component, but wasmtime handles async
```

2. **Host Side** (wasmtime async integration):
```rust
// In engine.rs - component execution is async
async fn execute_component_async(
    mut store: Store<ComponentState>, 
    instance: &ComponentInstance,
    inputs: Vec<Input>
) -> Result<Vec<Output>> {
    // Call component's execute function
    // WASI HTTP operations inside component become async operations here
    let execute = instance.get_typed_func::<(Vec<Input>,), Result<Vec<Output>>>(&mut store, "execute")?;
    
    execute.call_async(&mut store, (inputs,)).await?
}
```

3. **Graph Execution** (parallel independent nodes):
```rust
// Execute independent nodes concurrently
let tasks: Vec<_> = independent_nodes.iter()
    .map(|node| {
        let store = create_store_for_node(node);
        tokio::spawn(execute_component_async(store, node.instance, node.inputs))
    })
    .collect();

let results = futures::future::join_all(tasks).await;
```

**Benefits**:
- Non-blocking network I/O via WASI async model
- UI remains responsive (60 FPS maintained)  
- Parallel execution of independent HTTP nodes
- Standard WASI async patterns (portable across runtimes)

**Alternatives Considered**:
- Thread pool: Rejected due to overhead, harder to manage than tokio tasks
- Blocking with progress indicator: Rejected as it still freezes graph execution

---

### 4. Capability Validation & Redirect Handling

**Question**: How to enforce capability-based network access with redirect support?

**Research Findings**:

Capability declaration format (existing):
```rust
// Component metadata
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:api.example.com".to_string(),
        "network:httpbin.org".to_string(),
    ])
}
```

Validation points:
1. **Initial URL** - Before making request
2. **Redirect targets** - During redirect chain

**Decision**: **Component-Side Validation + wasmtime Resource Limiter**

**Rationale**:
- WASI HTTP doesn't provide built-in domain allowlisting (preview limitation)
- Components validate URLs before making requests (first line of defense)
- wasmtime resource limiter provides second validation layer (enforcement)
- Meets FR-013: allow same-domain redirects, block cross-domain

**Implementation Approach**:

1. **Component-Side Validation** (before making request):
```rust
// In component execute() function
fn validate_url(url: &str, allowed_domains: &[&str]) -> Result<(), ExecutionError> {
    // Parse URL
    let parsed_url = url.parse::<&str>().map_err(|_| ExecutionError {
        message: format!("Invalid URL: {}", url),
        input_name: Some("url".to_string()),
        recovery_hint: Some("URL must be valid HTTP/HTTPS format".to_string()),
    })?;
    
    // Extract domain (simplified - real impl would parse properly)
    let domain = extract_domain(url)?;
    
    // Check against allowed domains
    let allowed = allowed_domains.iter().any(|d| domain.ends_with(d));
    
    if !allowed {
        return Err(ExecutionError {
            message: format!("Access denied: {} not in approved capabilities", domain),
            input_name: Some("url".to_string()),
            recovery_hint: Some(format!("This component can only access: {:?}", allowed_domains)),
        });
    }
    
    Ok(())
}

// Usage in execute:
let allowed_domains = vec!["api.example.com", "httpbin.org"];
validate_url(&url, &allowed_domains)?;

// Now safe to make WASI HTTP request
let request = OutgoingRequest::new(Headers::new());
// ... make request
```

2. **Host-Side Enforcement** (wasmtime resource limiter):
```rust
// In wasmtime host setup
use wasmtime::ResourceLimiter;

struct ComponentResourceLimiter {
    allowed_domains: Vec<String>,
}

impl ResourceLimiter for ComponentResourceLimiter {
    // Implement memory/table limits (existing)
    // ... 
    
    // Custom: Intercept network requests (if wasmtime provides hook)
    // Note: This may not be available in preview - fallback to component validation
}
```

**Redirect Handling**:
- WASI HTTP handles redirects automatically (part of spec)
- Redirect policy configuration: **NOT AVAILABLE in preview**
- **Workaround**: Components should specify `redirect: false` in request options (if supported)
- Or accept that redirects follow default behavior for experimental use

**Edge Case Handling**:
- Subdomain matching: `api.example.com` allows `*.api.example.com`
- Redirect loops: WASI HTTP has max redirects (typically 10)
- HTTP -> HTTPS upgrade: Allowed (same domain)
- Cross-domain redirects: **Cannot block in preview** - trust component validation

**Accepted Limitations** (preview trade-off):
- ⚠️ No host-side redirect policy enforcement (preview limitation)
- ⚠️ Relies on component-side validation (trust model)
- ✅ Acceptable for experimental/educational project

**Alternatives Considered**:
- reqwest custom redirect policy: Requires custom host functions (defeats WASI HTTP purpose)
- Component-only validation: Chosen as best available option for WASI HTTP preview

---

### 5. Timeout Configuration

**Question**: How to implement configurable timeout with 30s default?

**Research Findings**:

Timeout options:
1. **Global client timeout** - Set once on reqwest::Client creation
2. **Per-request timeout** - Override on individual requests
3. **Component input parameter** - User provides timeout value

**Decision**: **Per-Request Timeout in WASI HTTP Request**

**Rationale**:
- WASI HTTP requests support timeout configuration per-request
- Meets FR-012: "30 seconds with optional input parameter to override"
- Component specifies timeout when creating OutgoingRequest
- No global client needed (WASI HTTP is request-based, not client-based)

**Implementation Approach**:

1. **Component Input Port** (optional):
```rust
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
            optional: true,  // Optional!
            description: "Request timeout in seconds (default: 30)".to_string(),
        },
    ]
}
```

2. **Component Logic with WASI HTTP**:
```rust
use wasi::http::types::{OutgoingRequest, Method, Scheme, Headers};
use wasi::http::outgoing_handler;

fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
    let url = extract_string_input(&inputs, "url")?;
    
    // Optional timeout, default to 30s
    let timeout_secs = inputs.iter()
        .find(|(name, _)| name == "timeout")
        .and_then(|(_, val)| match val {
            Value::U32Val(t) => Some(*t),
            _ => None,
        })
        .unwrap_or(30);  // Default: 30 seconds
    
    // Validate timeout range (1s - 300s / 5 minutes)
    if timeout_secs < 1 || timeout_secs > 300 {
        return Err(ExecutionError {
            message: "Timeout must be between 1 and 300 seconds".to_string(),
            input_name: Some("timeout".to_string()),
            recovery_hint: Some("Use a value between 1 and 300".to_string()),
        });
    }
    
    // Create WASI HTTP request with timeout
    let request = OutgoingRequest::new(Headers::new());
    request.set_method(&Method::Get)?;
    // Parse URL and set scheme, authority, path
    // ... (URL parsing logic)
    
    // Make request with timeout (in nanoseconds for WASI)
    let timeout_ns = (timeout_secs as u64) * 1_000_000_000;
    let future_response = outgoing_handler::handle(request, Some(timeout_ns))?;
    
    // Wait for response
    let incoming_response = future_response.get()?;
    let status = incoming_response.status();
    
    // Read body
    let body_stream = incoming_response.consume()?;
    let body = read_body_to_string(body_stream)?;
    
    // Return outputs
    Ok(vec![
        ("body".to_string(), Value::StringVal(body)),
        ("status".to_string(), Value::U32Val(status as u32)),
    ])
}
```

**Validation Rules**:
- Minimum: 1 second (prevent abuse with 0ms timeout)
- Maximum: 300 seconds / 5 minutes (prevent indefinite blocking)
- Invalid values: Return clear error message via ExecutionError

**Alternatives Considered**:
- Fixed timeout only: Rejected as doesn't meet FR-012 requirement
- Infinite timeout option: Rejected due to UI blocking concerns

---

### 6. Error Handling & User Feedback

**Question**: What errors can occur and how should they be communicated?

**Research Findings**:

Error categories:
1. **URL validation errors** - Malformed URL, missing scheme, invalid characters
2. **DNS resolution errors** - Domain doesn't exist, DNS server unreachable
3. **Connection errors** - Host unreachable, connection refused, network down
4. **Timeout errors** - Request exceeded timeout duration
5. **HTTP errors** - 4xx client errors, 5xx server errors
6. **Redirect errors** - Cross-domain redirect blocked, too many redirects
7. **Response errors** - Invalid encoding, response too large

**Decision**: **Structured Error Messages with Recovery Hints**

**Rationale**:
- Meets FR-006, FR-007: clear error messages for network/HTTP failures
- ExecutionError has `recovery_hint` field for actionable guidance
- Users can debug graph issues without examining logs

**Implementation Approach**:

Error message patterns:
```rust
// URL validation error
ExecutionError {
    message: "Invalid URL format: missing scheme".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("URL must start with http:// or https://".to_string()),
}

// DNS resolution error
ExecutionError {
    message: "Could not resolve domain: api.example.com".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("Check domain spelling and internet connection".to_string()),
}

// Connection refused
ExecutionError {
    message: "Connection refused by server".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("Server may be down or blocking connections".to_string()),
}

// Timeout error
ExecutionError {
    message: "Request timed out after 30 seconds".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("Server may be slow. Try increasing timeout or check connectivity.".to_string()),
}

// HTTP error
ExecutionError {
    message: "HTTP 404 Not Found".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("The requested resource does not exist on the server".to_string()),
}

// Cross-domain redirect
ExecutionError {
    message: "Cross-domain redirect blocked: Attempted redirect to unauthorized.com".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("This component can only access approved domains. Check capability settings.".to_string()),
}

// Capability violation
ExecutionError {
    message: "Access denied: api.forbidden.com not in approved capabilities".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("This component is only authorized to access: api.example.com, httpbin.org".to_string()),
}
```

**Error Categorization**:
- Map WASI HTTP errors to user-friendly messages
- Preserve technical details in logs (via `host::log`)
- Surface actionable guidance in UI via `recovery_hint`

**WASI HTTP Error Types**:
- `wasi:http/types/error-code` enum provides standard error categories
- Timeout, DNS, connection refused, TLS errors all mapped
- Components catch and translate to ExecutionError

**Alternatives Considered**:
- Generic error messages: Rejected as doesn't meet FR-006 "clear error messages"
- Stack traces in UI: Rejected as too technical for end users

---

## Summary of Key Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| **HTTP Access Pattern** | WASI HTTP (Preview) ⚠️ | Standards-based, experimental focus, future-proof |
| **HTTP Runtime** | wasmtime-wasi-http | Built into wasmtime, no custom host functions needed |
| **Execution Model** | Async with wasmtime + tokio | WASI HTTP is async, integrates with existing runtime |
| **Capability Validation** | Component-side + trust model | Preview limitation, acceptable for experimental use |
| **Timeout Configuration** | Per-request in WASI HTTP | 30s default + optional override, native WASI support |
| **Error Handling** | Structured messages with recovery hints | User-friendly, actionable guidance |

## Implementation Dependencies

### Cargo.toml Additions

**Host runtime** (`wasmflow/Cargo.toml`):
```toml
[dependencies]
wasmtime = { version = "27.0", features = ["component-model", "async"] }
wasmtime-wasi = "27.0"
wasmtime-wasi-http = "27.0"  # NEW: WASI HTTP support

# Note: reqwest NOT needed - WASI HTTP handles HTTP client
```

**Component** (`example-http-fetch/Cargo.toml`):
```toml
[dependencies]
# WASI HTTP bindings auto-generated from WIT
# No additional HTTP client dependencies
wit-bindgen = "0.44"
```

**WIT Dependencies** (component side):
- `wasi:http/types@0.2.0` - HTTP types (request, response, headers)
- `wasi:http/outgoing-handler@0.2.0` - HTTP client interface

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| WASI HTTP API changes (preview instability) | High | Medium | Acceptable for experimental project; document API version |
| wasmtime-wasi-http bugs | Medium | Medium | Test thoroughly; report issues to wasmtime project |
| Limited redirect control (preview limitation) | High | Low | Component-side URL validation; accept for experimental use |
| No host-side capability enforcement | High | Medium | Trust component validation; acceptable for educational project |
| TLS certificate validation failure | Low | Medium | wasmtime uses system cert store |
| Memory exhaustion from large responses | Medium | Medium | Component should limit response size (10MB) |

## Next Steps

Phase 1 will use these decisions to:
1. Define data model for HTTP Request/Response entities
2. Create WIT interface extensions for HTTP host functions
3. Write quickstart guide for HTTP-enabled component development
