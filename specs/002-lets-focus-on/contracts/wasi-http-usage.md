# WASI HTTP Usage for WasmFlow Components

This document explains how WasmFlow components use WASI HTTP (Preview) for making HTTP GET requests.

## Overview

**Approach**: Use standard WASI HTTP interfaces instead of custom host functions

**Why WASI HTTP?**
- Standards-based: Components are portable across WASI-compliant runtimes
- Experimental/Educational focus: This app explores WASM component composition
- Future-proof: As WASI HTTP stabilizes, components will remain compatible
- No custom host functions: Leverage wasmtime's built-in HTTP support

**Trade-offs Accepted**:
- ⚠️ Preview API (may change)
- ⚠️ Limited capability enforcement (component-side validation)
- ⚠️ Experimental wasmtime support
- ✅ Acceptable for learning/exploration

---

## WASI HTTP Interfaces

Components import standard WASI HTTP interfaces (no custom WIT needed):

### `wasi:http/types@0.2.0`

Provides HTTP type definitions:

```wit
// Simplified view (see actual WIT spec for complete definitions)

interface types {
    // HTTP methods
    enum method {
        get,
        post,
        // ... etc
    }
    
    // URL scheme
    enum scheme {
        http,
        https,
    }
    
    // HTTP request
    resource outgoing-request {
        constructor(headers: headers);
        set-method: func(method: method) -> result;
        set-scheme: func(scheme: option<scheme>) -> result;
        set-authority: func(authority: option<string>) -> result;
        set-path-with-query: func(path: option<string>) -> result;
    }
    
    // HTTP response
    resource incoming-response {
        status: func() -> status-code;
        headers: func() -> headers;
        consume: func() -> result<incoming-body>;
    }
    
    // Response body stream
    resource incoming-body {
        stream: func() -> result<input-stream>;
    }
    
    // Headers
    resource headers {
        constructor();
        append: func(name: string, value: list<u8>) -> result;
        get: func(name: string) -> list<list<u8>>;
    }
    
    // Error codes
    enum error-code {
        dns-timeout,
        dns-error,
        destination-not-found,
        destination-unavailable,
        destination-ip-prohibited,
        destination-ip-unroutable,
        connection-refused,
        connection-terminated,
        connection-timeout,
        connection-read-timeout,
        connection-write-timeout,
        connection-limit-reached,
        tls-protocol-error,
        tls-certificate-error,
        tls-alert-received,
        http-request-denied,
        http-request-length-required,
        http-request-body-size,
        http-request-method-invalid,
        http-request-uri-invalid,
        http-request-uri-too-long,
        http-request-header-section-size,
        http-request-header-size,
        http-request-trailer-section-size,
        http-request-trailer-size,
        http-response-incomplete,
        http-response-header-section-size,
        http-response-header-size,
        http-response-body-size,
        http-response-trailer-section-size,
        http-response-trailer-size,
        http-response-transfer-coding,
        http-response-content-coding,
        http-response-timeout,
        http-upgrade-failed,
        http-protocol-error,
        loop-detected,
        configuration-error,
        internal-error,
    }
}
```

### `wasi:http/outgoing-handler@0.2.0`

Provides HTTP client functionality:

```wit
interface outgoing-handler {
    use types.{outgoing-request, request-options, future-incoming-response, error-code};
    
    // Make HTTP request
    // request: The outgoing HTTP request
    // options: Optional timeout in nanoseconds
    // Returns: Future that resolves to response or error
    handle: func(
        request: outgoing-request,
        options: option<request-options>
    ) -> result<future-incoming-response, error-code>;
}

// Future response (resolves asynchronously)
resource future-incoming-response {
    // Get the response (blocks until ready)
    get: func() -> result<option<result<incoming-response, error-code>>>;
}

// Request options (timeout configuration)
resource request-options {
    constructor();
    set-connect-timeout: func(duration: duration) -> result;
    set-first-byte-timeout: func(duration: duration) -> result;
    set-between-bytes-timeout: func(duration: duration) -> result;
}
```

---

## Component Implementation Pattern

### 1. WIT World Declaration

```wit
// In component's wit/world.wit

package mycomponent:http-fetch;

world component {
    // Import WASI HTTP
    import wasi:http/types@0.2.0;
    import wasi:http/outgoing-handler@0.2.0;
    
    // Import WasmFlow node interface
    import wasmflow:node/host@1.0.0;
    
    // Export WasmFlow node interface
    export wasmflow:node/metadata@1.0.0;
    export wasmflow:node/execution@1.0.0;
}
```

### 2. Component Code (Rust)

```rust
#[allow(warnings)]
mod bindings;

use bindings::exports::wasmflow::node::metadata::Guest as MetadataGuest;
use bindings::exports::wasmflow::node::execution::Guest as ExecutionGuest;
use bindings::wasmflow::node::types::*;
use bindings::wasmflow::node::host;

// WASI HTTP imports
use bindings::wasi::http::types::{
    OutgoingRequest, Method, Scheme, Headers, IncomingResponse,
};
use bindings::wasi::http::outgoing_handler;

struct Component;

impl MetadataGuest for Component {
    fn get_capabilities() -> Option<Vec<String>> {
        Some(vec![
            "network:api.example.com".to_string(),
            "network:httpbin.org".to_string(),
        ])
    }
    
    // ... other metadata methods
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        let url = extract_string(&inputs, "url")?;
        let timeout_secs = extract_optional_u32(&inputs, "timeout", 30)?;
        
        // Validate URL against allowed capabilities
        validate_url(&url, &["api.example.com", "httpbin.org"])?;
        
        // Parse URL
        let (scheme, authority, path) = parse_url(&url)?;
        
        // Create request
        let headers = Headers::new();
        let request = OutgoingRequest::new(headers);
        
        request.set_method(&Method::Get)
            .map_err(|_| execution_error("Failed to set method"))?;
        
        request.set_scheme(Some(&scheme))
            .map_err(|_| execution_error("Failed to set scheme"))?;
        
        request.set_authority(Some(&authority))
            .map_err(|_| execution_error("Failed to set authority"))?;
        
        request.set_path_with_query(Some(&path))
            .map_err(|_| execution_error("Failed to set path"))?;
        
        // Make request with timeout
        let timeout_ns = (timeout_secs as u64) * 1_000_000_000;
        
        host::log("info", &format!("Making HTTP GET request to {}", url));
        
        let future_response = outgoing_handler::handle(request, Some(timeout_ns))
            .map_err(|e| map_error_code_to_execution_error(e, &url))?;
        
        // Wait for response
        let response_result = future_response.get()
            .map_err(|_| execution_error("Failed to get response"))?;
        
        let response = match response_result {
            Some(Ok(resp)) => resp,
            Some(Err(e)) => return Err(map_error_code_to_execution_error(e, &url)),
            None => return Err(execution_error("Response future not ready")),
        };
        
        // Extract status
        let status = response.status();
        
        // Read body
        let body_stream = response.consume()
            .map_err(|_| execution_error("Failed to consume response body"))?;
        
        let body = read_body_to_string(body_stream)?;
        
        host::log("info", &format!("HTTP request completed: status={}, body_len={}", status, body.len()));
        
        // Return outputs
        Ok(vec![
            ("body".to_string(), Value::StringVal(body)),
            ("status".to_string(), Value::U32Val(status as u32)),
        ])
    }
}

// Helper: Map WASI HTTP error codes to user-friendly messages
fn map_error_code_to_execution_error(error: ErrorCode, url: &str) -> ExecutionError {
    use bindings::wasi::http::types::ErrorCode;
    
    match error {
        ErrorCode::DnsTimeout | ErrorCode::DnsError => ExecutionError {
            message: format!("DNS resolution failed for: {}", url),
            input_name: Some("url".to_string()),
            recovery_hint: Some("Check domain spelling and internet connection".to_string()),
        },
        ErrorCode::ConnectionRefused => ExecutionError {
            message: "Connection refused by server".to_string(),
            input_name: Some("url".to_string()),
            recovery_hint: Some("Server may be down or blocking connections".to_string()),
        },
        ErrorCode::ConnectionTimeout => ExecutionError {
            message: "Connection timeout".to_string(),
            input_name: Some("url".to_string()),
            recovery_hint: Some("Server is unreachable or slow. Check connectivity.".to_string()),
        },
        ErrorCode::HttpResponseTimeout => ExecutionError {
            message: "Request timed out waiting for response".to_string(),
            input_name: Some("timeout".to_string()),
            recovery_hint: Some("Server is slow. Try increasing timeout.".to_string()),
        },
        ErrorCode::TlsCertificateError => ExecutionError {
            message: "TLS certificate validation failed".to_string(),
            input_name: Some("url".to_string()),
            recovery_hint: Some("Server has invalid or expired certificate".to_string()),
        },
        _ => ExecutionError {
            message: format!("HTTP request failed: {:?}", error),
            input_name: Some("url".to_string()),
            recovery_hint: Some("Check URL and network connection".to_string()),
        },
    }
}

// Helper: Read body stream to string
fn read_body_to_string(body: IncomingBody) -> Result<String, ExecutionError> {
    let stream = body.stream()
        .map_err(|_| execution_error("Failed to get body stream"))?;
    
    let mut chunks = Vec::new();
    let mut total_size = 0usize;
    const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB limit
    
    loop {
        match stream.read(65536) {  // Read up to 64KB chunks
            Ok(chunk) if chunk.is_empty() => break,  // EOF
            Ok(chunk) => {
                total_size += chunk.len();
                if total_size > MAX_SIZE {
                    return Err(ExecutionError {
                        message: "Response body too large (>10MB)".to_string(),
                        input_name: None,
                        recovery_hint: Some("Response size exceeded limit".to_string()),
                    });
                }
                chunks.extend_from_slice(&chunk);
            }
            Err(_) => return Err(execution_error("Failed to read body stream")),
        }
    }
    
    String::from_utf8(chunks).map_err(|_| execution_error("Response body is not valid UTF-8"))
}

bindings::export!(Component with_types_in bindings);
```

---

## Host Runtime Integration

### Cargo.toml Setup

```toml
[dependencies]
wasmtime = { version = "27.0", features = ["component-model", "async"] }
wasmtime-wasi = "27.0"
wasmtime-wasi-http = "27.0"
tokio = { version = "1.40", features = ["full"] }
```

### Linker Configuration

```rust
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

// Component state
struct ComponentState {
    wasi: WasiCtx,
    http: WasiHttpCtx,
}

impl WasiView for ComponentState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for ComponentState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
    
    fn table(&mut self) -> &mut ResourceTable {
        self.wasi.table()
    }
}

// Setup linker
let mut linker = Linker::new(&engine);

// Add WASI core
wasmtime_wasi::add_to_linker_async(&mut linker)?;

// Add WASI HTTP
wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

// Create store
let wasi = WasiCtxBuilder::new().build();
let http = WasiHttpCtx::new();

let state = ComponentState { wasi, http };
let mut store = Store::new(&engine, state);

// Instantiate and execute component
let instance = linker.instantiate_async(&mut store, &component).await?;
```

---

## Capability Declaration Pattern

Components MUST declare network capabilities in metadata:

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:api.github.com".to_string(),
        "network:httpbin.org".to_string(),
    ])
}
```

**Validation Responsibility**:
- Component validates URLs before making requests
- Host runtime does NOT enforce (preview limitation)
- Trust model acceptable for experimental/educational use

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_url_validation() {
        let allowed = vec!["api.example.com", "httpbin.org"];
        
        assert!(validate_url("https://api.example.com/data", &allowed).is_ok());
        assert!(validate_url("https://forbidden.com/data", &allowed).is_err());
    }
}
```

### Integration Tests

See `/tests/integration/wasi_http_test.rs` for full wasmtime + WASI HTTP integration tests.

---

## Known Limitations (WASI HTTP Preview)

1. **No host-side capability enforcement**: Relies on component validation
2. **Limited redirect control**: Cannot customize redirect policy from host
3. **API instability**: Preview API may change
4. **Experimental wasmtime support**: May have bugs

**Acceptable for**: Experimental/educational projects  
**Not recommended for**: Production deployments

---

## Resources

- WASI HTTP Spec: https://github.com/WebAssembly/wasi-http
- wasmtime-wasi-http docs: https://docs.rs/wasmtime-wasi-http
- WIT Spec: https://component-model.bytecodealliance.org/design/wit.html
