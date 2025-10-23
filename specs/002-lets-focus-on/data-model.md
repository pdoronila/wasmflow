# Data Model: HTTP Fetch Component

**Feature**: HTTP Fetch Component with Real Network Capability  
**Date**: 2025-10-14  
**Status**: Complete

## Overview

This document defines the data entities and their relationships for HTTP request/response handling in WasmFlow. The model bridges the gap between user-facing node inputs/outputs and internal runtime structures.

## Entity Diagram

```
┌─────────────────────┐
│  HTTP Fetch Node    │  (User-visible in graph)
│  ┌────────────────┐ │
│  │ Inputs:        │ │
│  │  - url (str)   │ │
│  │  - timeout(u32)│ │──────┐
│  └────────────────┘ │      │
│  ┌────────────────┐ │      │ Executes with
│  │ Outputs:       │ │      │
│  │  - body (str)  │ │      ▼
│  │  - status(u32) │ │  ┌──────────────────┐
│  │  - headers(opt)│ │  │  HTTP Request    │
│  └────────────────┘ │  │  Entity          │
└─────────────────────┘  └──────────────────┘
                                 │
                                 │ Validated against
                                 ▼
                         ┌──────────────────┐       ┌──────────────────┐
                         │  Network         │◄──────│  Component       │
                         │  Capability      │       │  Metadata        │
                         └──────────────────┘       └──────────────────┘
                                 │
                                 │ If approved
                                 ▼
                         ┌──────────────────┐
                         │  HTTP Client     │
                         │  (reqwest)       │
                         └──────────────────┘
                                 │
                                 │ Returns
                                 ▼
                         ┌──────────────────┐
                         │  HTTP Response   │
                         │  Entity          │
                         └──────────────────┘
                                 │
                                 │ Mapped to
                                 ▼
                         ┌──────────────────┐
                         │  Node Outputs    │
                         │  (WIT values)    │
                         └──────────────────┘
```

## Entities

### 1. HTTP Request (Internal Runtime Entity)

Represents an outbound HTTP GET request before execution.

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `url` | String | Required, valid URL | Target HTTP/HTTPS endpoint |
| `timeout_ms` | u32 | 1000-300000 (1s-5min) | Request timeout in milliseconds |
| `allowed_domains` | Vec<String> | From component capabilities | Approved network domains |

**Validation Rules**:
- `url` MUST start with `http://` or `https://`
- `url` MUST have a valid host component
- `url` host MUST match one of `allowed_domains` (suffix match)
- `timeout_ms` MUST be between 1000 and 300000

**State Transitions**:
```
Created → Validated → Executing → [Completed | Failed | Timeout]
```

**Example**:
```rust
HttpRequest {
    url: "https://api.example.com/data".to_string(),
    timeout_ms: 30000,  // 30 seconds
    allowed_domains: vec!["api.example.com".to_string(), "httpbin.org".to_string()],
}
```

---

### 2. HTTP Response (Internal Runtime Entity)

Contains the result of an HTTP request execution.

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `status` | u16 | 100-599 | HTTP status code |
| `body` | String | Max 10MB | Response body as UTF-8 text |
| `headers` | HashMap<String, String> | Optional | Response headers (key-value pairs) |
| `final_url` | String | Valid URL | Final URL after redirects |

**Validation Rules**:
- `status` MUST be valid HTTP status code (100-599)
- `body` size MUST NOT exceed 10MB (10,485,760 bytes)
- `body` MUST be valid UTF-8 (or return encoding error)
- `headers` keys MUST be valid HTTP header names (case-insensitive)

**Relationships**:
- One `HttpResponse` is produced per `HttpRequest`
- Maps to multiple node outputs (body, status, headers)

**Example**:
```rust
HttpResponse {
    status: 200,
    body: r#"{"message": "Success", "data": [1, 2, 3]}"#.to_string(),
    headers: {
        "content-type": "application/json",
        "content-length": "35",
    }.into_iter().collect(),
    final_url: "https://api.example.com/data".to_string(),  // No redirect
}
```

---

### 3. Network Capability (Security Entity)

Represents approved network access for a component.

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `capability_type` | String | Must be "network" | Type of capability |
| `domain` | String | Valid domain name | Approved domain or host |
| `approved` | bool | - | User approval status |

**Validation Rules**:
- `domain` MUST NOT contain wildcards (explicit domains only)
- `domain` MUST be a valid DNS hostname
- Subdomain matching: `api.example.com` allows `*.api.example.com`

**Capability String Format**:
```
"network:{domain}"

Examples:
  "network:api.example.com"    → Allows api.example.com and subdomains
  "network:httpbin.org"         → Allows httpbin.org and subdomains
```

**Parsing Logic**:
```rust
fn parse_network_capability(cap_string: &str) -> Option<NetworkCapability> {
    if let Some(domain) = cap_string.strip_prefix("network:") {
        Some(NetworkCapability {
            capability_type: "network".to_string(),
            domain: domain.to_string(),
            approved: false,  // Must be approved by user
        })
    } else {
        None
    }
}
```

**Relationships**:
- One `NetworkCapability` per declared domain in component metadata
- Multiple `NetworkCapability` entities per component (multi-domain access)
- Validated against each `HttpRequest` before execution

**Example**:
```rust
vec![
    NetworkCapability {
        capability_type: "network".to_string(),
        domain: "api.example.com".to_string(),
        approved: true,  // User approved
    },
    NetworkCapability {
        capability_type: "network".to_string(),
        domain: "httpbin.org".to_string(),
        approved: true,
    },
]
```

---

### 4. Execution Error (Error Entity)

Represents errors during HTTP request execution.

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `message` | String | Required | Human-readable error description |
| `input_name` | Option<String> | Optional | Which input caused the error |
| `recovery_hint` | Option<String> | Optional | Actionable guidance for user |
| `error_category` | ErrorCategory | Required | Classification for handling |

**Error Categories**:

```rust
enum ErrorCategory {
    UrlValidation,      // Malformed URL
    CapabilityDenied,   // URL not in approved domains
    DnsResolution,      // Domain doesn't exist
    Connection,         // Network unreachable, connection refused
    Timeout,            // Request exceeded timeout
    HttpError,          // 4xx or 5xx status codes
    RedirectBlocked,    // Cross-domain redirect
    EncodingError,      // Response not valid UTF-8
    ResponseTooLarge,   // Response exceeds 10MB
}
```

**Validation Rules**:
- `message` MUST be non-empty
- `recovery_hint` SHOULD provide actionable next step when possible
- `input_name` SHOULD be "url" or "timeout" when error relates to input

**Relationships**:
- One `ExecutionError` per failed `HttpRequest`
- Maps to WIT `execution-error` type for component interface

**Example**:
```rust
ExecutionError {
    message: "Access denied: unauthorized.com not in approved capabilities".to_string(),
    input_name: Some("url".to_string()),
    recovery_hint: Some("This component can only access: api.example.com, httpbin.org".to_string()),
    error_category: ErrorCategory::CapabilityDenied,
}
```

---

### 5. Node Input/Output Ports (WIT Interface)

Maps between node-level data and internal entities.

#### Input Ports

| Port Name | WIT Type | Optional | Maps To |
|-----------|----------|----------|---------|
| `url` | `string` | No | `HttpRequest.url` |
| `timeout` | `u32` | Yes | `HttpRequest.timeout_ms` (× 1000 for ms) |

**Default Values**:
- `timeout`: 30 (seconds) → 30000ms if not provided

#### Output Ports

| Port Name | WIT Type | Optional | Maps From |
|-----------|----------|----------|-----------|
| `body` | `string` | No | `HttpResponse.body` |
| `status` | `u32` | No | `HttpResponse.status` (cast u16 → u32) |
| `headers` | `string` | Yes | `HttpResponse.headers` (JSON-encoded) |

**Encoding Rules**:
- `headers` encoded as JSON string: `{"content-type": "application/json", ...}`
- Empty headers → omit `headers` output (optional port)

**Example Mapping**:

Input:
```rust
vec![
    ("url".to_string(), Value::StringVal("https://api.example.com/data".to_string())),
    ("timeout".to_string(), Value::U32Val(10)),  // 10 seconds
]
```

Output (success):
```rust
vec![
    ("body".to_string(), Value::StringVal(r#"{"result": "ok"}"#.to_string())),
    ("status".to_string(), Value::U32Val(200)),
    ("headers".to_string(), Value::StringVal(r#"{"content-type":"application/json"}"#.to_string())),
]
```

Output (error):
```rust
Err(ExecutionError {
    message: "Request timed out after 10 seconds".to_string(),
    input_name: Some("timeout".to_string()),
    recovery_hint: Some("Server is slow. Try increasing timeout.".to_string()),
    error_category: ErrorCategory::Timeout,
})
```

---

## Entity Relationships

### Component Metadata → Network Capability
- **Cardinality**: 1:N (one component declares N capabilities)
- **Constraint**: All declared capabilities must be validated and approved before execution

### HTTP Request → Network Capability
- **Cardinality**: N:M (request URL validated against M approved capabilities)
- **Constraint**: Request URL host must match at least one approved domain

### HTTP Request → HTTP Response
- **Cardinality**: 1:1 (each request produces exactly one response or error)
- **Constraint**: Response only created if request passes capability validation

### HTTP Response → Node Outputs
- **Cardinality**: 1:N (one response maps to 2-3 output ports)
- **Constraint**: Mandatory outputs (body, status) always present; headers optional

### HTTP Request → Execution Error
- **Cardinality**: 1:0..1 (request either succeeds or produces error)
- **Constraint**: Error returned via WIT `result` type

---

## Data Flow

### Happy Path (Successful Request)

```
1. Node receives inputs (url, optional timeout)
   ↓
2. Create HttpRequest entity with validation
   ↓
3. Extract NetworkCapability list from component metadata
   ↓
4. Validate HttpRequest.url against NetworkCapability.domain list
   ↓ (approved)
5. Execute HTTP request via reqwest (host runtime)
   ↓
6. Receive HttpResponse entity
   ↓
7. Map HttpResponse fields to WIT output values
   ↓
8. Return outputs to graph executor
```

### Error Path (Capability Denied)

```
1. Node receives inputs (url="https://forbidden.com/api")
   ↓
2. Create HttpRequest entity
   ↓
3. Validate url against approved domains
   ↓ (denied)
4. Create ExecutionError with CapabilityDenied category
   ↓
5. Return error to graph executor
   ↓
6. Display error message in UI
```

### Error Path (Network Timeout)

```
1-4. [Same as happy path]
   ↓
5. Execute HTTP request with timeout
   ↓
6. reqwest returns timeout error after 30s
   ↓
7. Map reqwest::Error to ExecutionError with Timeout category
   ↓
8. Return error to graph executor
```

---

## Storage Considerations

**No Persistent Storage Required**:
- All entities are runtime-only (in-memory)
- Network capabilities stored in component metadata (WASM component)
- Approval state stored in graph save file (user's .wfg file)

**Memory Constraints**:
- `HttpResponse.body` max size: 10MB
- `HttpRequest` queue: no queueing (execute immediately)
- Connection pool: managed by reqwest (10 idle connections per host)

---

## Versioning & Evolution

**Current Version**: 1.0.0

**Future Enhancements** (out of scope for this feature):
- Request body support (POST/PUT methods)
- Custom headers (input port for headers map)
- Authentication tokens (secure credential storage)
- Response streaming (for large files >10MB)
- Caching layer (cache responses by URL + TTL)

**Breaking Changes**:
- Adding required input ports (breaks existing graphs)
- Changing output port data types (breaks downstream connections)
- Modifying capability string format (breaks approval persistence)

**Backward Compatibility**:
- Optional ports can be added without breaking changes
- Additional output ports are backward compatible (existing graphs ignore them)
