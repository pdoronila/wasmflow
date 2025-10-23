# HTTP Fetch Component Contracts

This directory contains documentation for WASI HTTP usage in WasmFlow components.

## Files

### `wasi-http-usage.md`

Comprehensive guide to using WASI HTTP (Preview) in WasmFlow components.

**Contents**:
- WASI HTTP interface overview (`wasi:http/types`, `wasi:http/outgoing-handler`)
- Component implementation patterns
- Host runtime integration
- Error handling with WASI error codes
- Capability declaration pattern
- Known limitations and trade-offs

## Approach: WASI HTTP (Standards-Based)

**Decision**: Use standard WASI HTTP interfaces instead of custom host functions

**Why?**
- This app is experimental/educational - exploring WASM component composition
- WASI HTTP is the emerging standard for HTTP in WASM components
- Future-proof as WASI HTTP stabilizes
- No custom WIT interfaces needed

**Trade-offs**:
- ⚠️ Preview API (may change)
- ⚠️ Limited capability enforcement (component-side validation)
- ⚠️ No custom redirect policy (preview limitation)
- ✅ Acceptable for learning and experimentation

## Quick Start

1. **Component imports WASI HTTP**:
```wit
import wasi:http/types@0.2.0;
import wasi:http/outgoing-handler@0.2.0;
```

2. **Component makes HTTP request**:
```rust
use wasi::http::types::OutgoingRequest;
use wasi::http::outgoing_handler;

let request = OutgoingRequest::new(Headers::new());
// ... configure request

let future_response = outgoing_handler::handle(request, Some(timeout_ns))?;
let response = future_response.get()?;
```

3. **Host enables WASI HTTP**:
```rust
use wasmtime_wasi_http::add_only_http_to_linker_async;

add_only_http_to_linker_async(&mut linker)?;
```

See `wasi-http-usage.md` for complete examples.

## Integration with WasmFlow Node Interface

WASI HTTP components still implement the standard WasmFlow node interface:

```
wasmflow:node (base interface)
  ├── metadata (get-info, get-inputs, get-outputs, get-capabilities)
  ├── execution (execute)
  └── host (log, get-temp-dir)

+

wasi:http (standard WASI interface)
  ├── types (OutgoingRequest, IncomingResponse, Headers, etc.)
  └── outgoing-handler (handle function)
```

Components export `wasmflow:node/metadata` and `wasmflow:node/execution`, and import `wasi:http/*`.

## Capability Declaration

Components declare network capabilities in metadata:

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:api.example.com".to_string(),
        "network:httpbin.org".to_string(),
    ])
}
```

**Validation**:
- Component validates URLs against this list before making requests
- Host runtime does NOT enforce (preview limitation)
- Trust model acceptable for experimental project

## Error Handling

WASI HTTP provides `error-code` enum with detailed error types:

```rust
enum ErrorCode {
    DnsTimeout,
    DnsError,
    ConnectionRefused,
    ConnectionTimeout,
    TlsCertificateError,
    HttpResponseTimeout,
    // ... 30+ error variants
}
```

Components map these to `ExecutionError` with user-friendly messages and recovery hints.

## Testing

- **Contract Tests**: Verify WIT interface compliance
- **Integration Tests**: Full wasmtime + WASI HTTP execution
- **Unit Tests**: URL validation, error mapping

See `/tests/contract/` and `/tests/integration/` for examples.

## Versioning

**Current WASI HTTP Version**: 0.2.0 (Preview)

**Compatibility**: Components built against `wasi:http@0.2.0` work with wasmtime 27.0+ with `wasmtime-wasi-http` crate.

**Future Versions**: When WASI HTTP reaches 1.0, may require component updates.

## Resources

- **WASI HTTP Specification**: https://github.com/WebAssembly/wasi-http
- **wasmtime-wasi-http Crate**: https://docs.rs/wasmtime-wasi-http
- **Component Model**: https://component-model.bytecodealliance.org
- **WIT Reference**: https://component-model.bytecodealliance.org/design/wit.html
