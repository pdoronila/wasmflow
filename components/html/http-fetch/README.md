# HTTP Fetch Component - WASI HTTP Implementation

A WasmFlow component that performs **real HTTP requests** using WASI HTTP (Preview 0.2.0). This component demonstrates capability-based security and standards-based WebAssembly networking.

## Features

- **Real HTTP/HTTPS Requests**: Uses WASI HTTP Preview 0.2.0 for actual network communication
- **Capability-Based Security**: Declares and enforces allowed domains
- **Configurable Timeouts**: Supports connection and response timeouts (1-300 seconds)
- **Comprehensive Error Handling**: Maps all WASI error codes to user-friendly messages
- **Standards Compliant**: Built with Component Model and WebAssembly Interface Types (WIT)

## Ports

### Inputs
- **url** (String, required): HTTP/HTTPS URL to fetch
  - Must match declared capabilities (httpbin.org or api.example.com)
- **timeout** (U32, optional): Request timeout in seconds (default: 30, range: 1-300)

### Outputs
- **body** (String): Response body as UTF-8 text
- **status** (U32): HTTP status code (e.g., 200, 404, 500)

## Architecture

### Component Model

```
┌─────────────────────────────────────┐
│   HTTP Fetch Component (WASM)       │
│                                      │
│  ┌────────────────────────────────┐ │
│  │ Exports (WasmFlow Interface)   │ │
│  │  • metadata::get-info          │ │
│  │  • metadata::get-capabilities  │ │
│  │  • execution::execute          │ │
│  └────────────────────────────────┘ │
│                                      │
│  ┌────────────────────────────────┐ │
│  │ Imports (WASI HTTP + IO)       │ │
│  │  • wasi:http/types             │ │
│  │  • wasi:http/outgoing-handler  │ │
│  │  • wasi:io/streams             │ │
│  │  • wasi:io/poll                │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
                 ↕
┌─────────────────────────────────────┐
│    Wasmtime Runtime (Host)          │
│  • Provides WASI HTTP implementation│
│  • Enforces capability restrictions │
│  • Manages network access           │
└─────────────────────────────────────┘
```

## Build Instructions

### Prerequisites

```bash
# Add WASI Preview 2 target
rustup target add wasm32-wasip2
```

### Build

```bash
# From the example-http-fetch directory
cargo build --target wasm32-wasip2 --release

# Output: target/wasm32-wasip2/release/example_http_fetch.wasm (89KB)
```

### Build System

This component uses **wit-bindgen** directly (not cargo-component) for maximum control:

- `Cargo.toml`: Configures cdylib library with wit-bindgen dependency
- `build.rs`: Triggers rebuild when WIT files change
- `src/lib.rs`: Uses `wit_bindgen::generate!` macro to generate bindings at compile time

## Capabilities & Security

### Declared Capabilities

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:httpbin.org".to_string(),
        "network:api.example.com".to_string(),
    ])
}
```

### Security Features

- **Whitelist enforcement**: Only httpbin.org and api.example.com domains allowed
- **Subdomain matching**: `api.example.com` permits `*.api.example.com`
- **URL validation**: Invalid URLs and unauthorized domains are rejected before making requests
- **Early validation**: Checks happen before any network activity

### Example URL Validation

```
✓ https://httpbin.org/get
✓ https://api.example.com/data
✓ https://sub.api.example.com/v1/users
✗ https://google.com              (not in capability list)
✗ ftp://httpbin.org/file          (not http/https)
✗ httpbin.org/get                 (missing protocol)
```

## Implementation Details

### HTTP Request Flow

The component performs real HTTP requests using the WASI HTTP resource API:

```rust
// 1. Create request with headers
let headers = Fields::new();
let request = OutgoingRequest::new(headers);

// 2. Configure request (GET method, URL components)
request.set_method(&Method::Get)?;
request.set_scheme(Some(&Scheme::Https))?;
request.set_authority(Some("httpbin.org"))?;
request.set_path_with_query(Some("/get"))?;

// 3. Set timeout options
let options = RequestOptions::new();
let timeout_ns = 30 * 1_000_000_000; // 30 seconds in nanoseconds
options.set_connect_timeout(Some(timeout_ns))?;
options.set_first_byte_timeout(Some(timeout_ns))?;

// 4. Send request (non-blocking, returns future)
let future_response = outgoing_handler::handle(request, Some(options))?;

// 5. Wait for response using pollable
let pollable = future_response.subscribe();
pollable.block();

// 6. Extract response
let incoming_response = future_response.get()??;
let status = incoming_response.status();

// 7. Stream response body in 8KB chunks
let body_stream = incoming_response.consume()?.stream()?;
let mut body_bytes = Vec::new();
loop {
    match body_stream.read(8192) {
        Ok(chunk) if chunk.is_empty() => break,
        Ok(chunk) => body_bytes.extend_from_slice(&chunk),
        Err(StreamError::Closed) => break,
        Err(_) => return Err(...),
    }
}

// 8. Convert to UTF-8 string
let body = String::from_utf8(body_bytes)?;
```

### Error Handling

All 40+ WASI HTTP error codes are mapped to user-friendly `ExecutionError` messages with recovery hints:

| WASI Error | User Message | Recovery Hint |
|------------|--------------|---------------|
| `dns-timeout` | "DNS lookup timed out" | "Check network connectivity and DNS settings" |
| `connection-refused` | "Connection refused" | "The server refused the connection" |
| `connection-timeout` | "Connection timed out" | "Increase timeout or check network speed" |
| `tls-certificate-error` | "TLS certificate error" | "Server certificate is invalid or untrusted" |
| `http-response-timeout` | "HTTP response timed out" | "Increase timeout or try again later" |

See `map_error_code()` in `src/lib.rs:348-513` for the complete mapping.

## WIT Interface Structure

```
examples/example-http-fetch/wit/
├── node.wit                    # WasmFlow component world
└── deps/
    ├── http/
    │   ├── types.wit           # WASI HTTP types (package wasi:http@0.2.0)
    │   └── outgoing-handler.wit# HTTP client handler interface
    └── io/
        ├── error.wit           # WASI IO errors (package wasi:io@0.2.0)
        ├── poll.wit            # Pollable resource for async operations
        └── streams.wit         # Input/output stream interfaces
```

### Key WIT Patterns

1. **Package declarations**: Only one file per package directory has `package ...` declaration
2. **Keyword escaping**: Use `%stream` to escape the `stream` keyword
3. **Resource-oriented API**: WASI HTTP uses resources (`OutgoingRequest`, `IncomingResponse`, etc.) for type safety
4. **Explicit binding generation**: Configure wit-bindgen with `with:` mappings for each WASI package

## Runtime Requirements

The WasmFlow runtime must provide WASI HTTP support. See `src/runtime/wasm_host.rs` for the implementation:

```rust
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

// 1. Add WASI HTTP to HostState
pub struct HostState {
    pub wasi: WasiCtx,
    pub http: WasiHttpCtx,  // WASI HTTP context
    pub table: ResourceTable,
    // ...
}

// 2. Implement WasiHttpView trait
impl WasiHttpView for HostState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

// 3. Add HTTP to linker
wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

// 4. Enable network access in WASI context
let wasi = WasiCtxBuilder::new()
    .inherit_network()  // Enable network access
    .build();
```

## Usage Example

```rust
// Component inputs
let inputs = vec![
    ("url".to_string(), Value::StringVal("https://httpbin.org/get".to_string())),
    ("timeout".to_string(), Value::U32Val(30)),
];

// Execute component
match Component::execute(inputs) {
    Ok(outputs) => {
        // outputs = [
        //   ("body", Value::StringVal("{\"args\": {}, \"headers\": {...}}")),
        //   ("status", Value::U32Val(200)),
        // ]
    },
    Err(err) => {
        println!("Error: {}", err.message);
        if let Some(hint) = err.recovery_hint {
            println!("Hint: {}", hint);
        }
    }
}
```

## Testing

### Unit Tests

```bash
cargo test
```

Tests cover:
- URL validation (valid/invalid protocols)
- Domain extraction from URLs
- Capability enforcement (allowed/denied domains)
- Timeout range validation (1-300 seconds)
- Subdomain matching logic

### Integration Testing

To test real HTTP requests, load the component in the WasmFlow runtime with WASI HTTP support enabled. The component will make actual network requests to httpbin.org.

## Technical Decisions

### Why wit-bindgen instead of cargo-component?

**cargo-component** has limitations with local WIT dependencies. **wit-bindgen** provides:
- Direct control over binding generation
- Support for local WIT packages without complex dependency configuration
- Explicit configuration via `with:` mappings
- Better error messages during development
- Simpler build process for complex WIT structures

### Why WASI HTTP Preview 0.2.0?

This is an **educational/experimental** implementation to explore:
- Standards-based WebAssembly networking
- Component Model resource APIs
- Future-proof architecture for when WASI HTTP becomes stable
- Capability-based security patterns

**Note**: For production use today, consider HTTP libraries that compile to WASM (like reqwest with wasm-bindgen), as WASI HTTP is still in preview stage.

### Resource-Oriented API Benefits

WASI HTTP uses resources (not simple functions) for:
- **Type safety**: Compile-time guarantees about resource usage
- **Resource management**: Proper cleanup via Component Model drop semantics
- **Streaming**: Efficient handling of large responses without buffering
- **Async support**: Future-based API for non-blocking I/O operations

## File Reference

| File | Purpose | Lines |
|------|---------|-------|
| `src/lib.rs` | Main component implementation with HTTP logic | 516 |
| `wit/node.wit` | WasmFlow component world definition | 102 |
| `wit/deps/http/types.wit` | WASI HTTP types interface (full spec) | 209 |
| `wit/deps/http/outgoing-handler.wit` | HTTP client handler interface | 10 |
| `wit/deps/io/streams.wit` | I/O streaming interfaces | 47 |
| `wit/deps/io/poll.wit` | Pollable resource for async | 9 |
| `wit/deps/io/error.wit` | Error resource | 8 |
| `Cargo.toml` | Build configuration | 17 |
| `build.rs` | Build script (rebuild triggers) | 4 |

## Troubleshooting

### Build fails with "stream is a keyword"

**Cause**: `stream` became a reserved keyword in WIT.

**Solution**: Use `%stream` to escape the keyword in WIT interface definitions.

### Build fails with "package 'wasi:http' not found"

**Cause**: WIT files not in expected directory structure.

**Solution**: Ensure:
- `wit/deps/http/` contains HTTP interfaces with one package declaration
- `wit/deps/io/` contains IO interfaces with one package declaration
- Run `cargo clean` and rebuild

### Type mismatch errors with WASI types

**Cause**: Missing or incorrect `with:` mappings in `wit_bindgen::generate!`.

**Solution**: Verify all WASI packages are listed:
```rust
with: {
    "wasi:io/error@0.2.0": generate,
    "wasi:io/poll@0.2.0": generate,
    "wasi:io/streams@0.2.0": generate,
    "wasi:http/types@0.2.0": generate,
    "wasi:http/outgoing-handler@0.2.0": generate,
}
```

### Runtime error: "WASI HTTP not available"

**Cause**: Runtime doesn't have WASI HTTP configured.

**Solution**: Ensure runtime:
1. Has `wasmtime-wasi-http` dependency
2. Calls `add_only_http_to_linker_async`
3. Implements `WasiHttpView` trait
4. Enables network access with `.inherit_network()`

### "Access denied" errors

**Cause**: URL doesn't match declared capabilities.

**Solution**: Either:
- Update URL to use httpbin.org or api.example.com
- Or add domain to `get_capabilities()` and rebuild component

## Future Enhancements

- [ ] Support for POST, PUT, DELETE methods with request bodies
- [ ] Custom HTTP headers (Authorization, Content-Type, etc.)
- [ ] Response header access for content-type detection
- [ ] Streaming response processing (process chunks without buffering)
- [ ] HTTP/2 and HTTP/3 support (when WASI HTTP supports it)
- [ ] Cookie handling and session management
- [ ] Automatic redirect following (with cycle detection)
- [ ] Proxy support (HTTP/HTTPS/SOCKS)
- [ ] Request/response compression (gzip, brotli)
- [ ] Multipart form data uploads

## References

- [WASI HTTP Specification](https://github.com/WebAssembly/wasi-http) - Official specification
- [Component Model](https://github.com/WebAssembly/component-model) - WebAssembly Component Model
- [WIT Format](https://component-model.bytecodealliance.org/design/wit.html) - WIT language specification
- [wit-bindgen Documentation](https://github.com/bytecodealliance/wit-bindgen) - Binding generator
- [wasmtime-wasi-http](https://docs.rs/wasmtime-wasi-http) - Wasmtime WASI HTTP implementation

## License

Part of the WasmFlow project.
