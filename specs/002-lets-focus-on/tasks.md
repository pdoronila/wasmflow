# Tasks: HTTP Fetch Component with WASI HTTP (TDD Approach)

**Input**: Design documents from `/specs/002-lets-focus-on/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
**Approach**: WASI HTTP (Preview) - experimental/educational focus with comprehensive test suite

**Testing Strategy**: TDD - Write tests FIRST for each user story, verify they FAIL, then implement until tests PASS

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- **[TEST]**: Test task - must be written BEFORE implementation
- All file paths are absolute from repository root

## User Story Map

- **US1 (P1)**: Basic HTTP GET Request - MVP, core HTTP functionality
- **US2 (P2)**: Error Handling for Network Failures - Robustness
- **US3 (P2)**: Capability-Based Security Approval - Security validation
- **US4 (P3)**: HTTP Response Headers Access - Advanced feature

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, dependencies, and test infrastructure for WASI HTTP

- [X] T001 [P] Add `wasmtime-wasi-http = "27.0"` dependency to `/Users/doronila/git/wasmflow_cc/Cargo.toml`
- [X] T002 [P] Add test dependencies (`mockito = "1.0"` for HTTP mocking if needed) to `/Users/doronila/git/wasmflow_cc/Cargo.toml` [dev-dependencies]
- [X] T003 [P] Create WASI HTTP WIT files directory at `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/wit/deps/wasi-http/`
- [X] T004 [P] Copy `wasi:http/types@0.2.0` WIT file to `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/wit/deps/wasi-http/types.wit`
- [X] T005 [P] Copy `wasi:http/outgoing-handler@0.2.0` WIT file to `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/wit/deps/wasi-http/outgoing-handler.wit`
- [X] T006 Update component world definition in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/wit/world.wit` to import `wasi:http/types` and `wasi:http/outgoing-handler`
- [X] T007 [P] Create test directory structure: `/Users/doronila/git/wasmflow_cc/tests/contract/`, `/Users/doronila/git/wasmflow_cc/tests/integration/`, `/Users/doronila/git/wasmflow_cc/tests/unit/`

**Checkpoint**: WASI HTTP WIT interfaces and test infrastructure ready

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core wasmtime WASI HTTP integration that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Foundational Tests (Write FIRST)

- [X] T008 [P] [TEST] Create integration test skeleton in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs` that instantiates component with WASI HTTP context
- [X] T009 [P] [TEST] Write test that verifies wasmtime linker has WASI HTTP functions available in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs`

### Foundational Implementation

- [X] T010 Integrate `wasmtime_wasi_http::add_only_http_to_linker_async` in `/Users/doronila/git/wasmflow_cc/src/runtime/wasm_host.rs` linker setup
- [X] T011 Add `WasiHttpCtx` to component state structure in `/Users/doronila/git/wasmflow_cc/src/runtime/wasm_host.rs`
- [X] T012 Implement `WasiHttpView` trait for component state in `/Users/doronila/git/wasmflow_cc/src/runtime/wasm_host.rs`
- [X] T013 Update component instantiation to include HTTP context in `/Users/doronila/git/wasmflow_cc/src/runtime/engine.rs`
- [X] T014 Modify `execute_component_async` to support async WASI HTTP calls in `/Users/doronila/git/wasmflow_cc/src/runtime/engine.rs`
- [X] T015 Run tests T008-T009 and verify they PASS (green)

**Checkpoint**: wasmtime host runtime ready to execute WASI HTTP components, foundational tests passing

---

## Phase 3: User Story 1 - Basic HTTP GET Request (Priority: P1) 🎯 MVP

**Goal**: Enable components to make real HTTP GET requests and return response body + status code

**Independent Test**: Create graph with Constant(URL) → HTTP Fetch → verify real response data returned

### Tests for User Story 1 (Write FIRST - TDD Red Phase)

- [X] T016 [P] [TEST] [US1] Create unit test for `extract_string(inputs, name)` helper in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs` (add `#[cfg(test)]` module)
- [X] T017 [P] [TEST] [US1] Create unit test for `extract_optional_u32(inputs, name, default)` helper in component test module
- [X] T018 [P] [TEST] [US1] Create unit test for `parse_url(url)` that validates URL parsing logic in component test module
- [X] T019 [P] [TEST] [US1] Create unit test for timeout validation (1-300 seconds range) in component test module
- [X] T020 [P] [TEST] [US1] Create WIT contract test in `/Users/doronila/git/wasmflow_cc/tests/contract/wasi_http_component_test.rs` that verifies component exports `metadata` and `execution` interfaces
- [X] T021 [TEST] [US1] Create integration test in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs` that makes real HTTP request to httpbin.org/get and verifies response body + status
- [X] T022 [TEST] [US1] Create integration test for timeout handling (default 30s) in integration test file
- [X] T023 [TEST] [US1] Create integration test for custom timeout (10s) in integration test file
- [X] T024 [TEST] [US1] Create unit test for `read_body_to_string()` with 10MB size limit in component test module
- [X] T025 Run all US1 tests with `cargo test` and verify they FAIL (expected - red phase)

### Implementation for User Story 1 (TDD Green Phase)

- [X] T026 [P] [US1] Update component metadata `get_inputs()` to declare `url` (string, required) and `timeout` (u32, optional) ports in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T027 [P] [US1] Update component metadata `get_outputs()` to declare `body` (string) and `status` (u32) ports in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T028 [P] [US1] Update component metadata `get_capabilities()` to declare `network:httpbin.org` and `network:api.example.com` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T029 [US1] Implement helper function `extract_string(inputs, name)` to extract string inputs in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T030 [US1] Implement helper function `extract_optional_u32(inputs, name, default)` to extract optional timeout in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T031 [US1] Implement helper function `parse_url(url)` to extract scheme, authority, path from URL string in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T032 [US1] Implement `execute()` function: extract URL and timeout inputs in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T033 [US1] Implement timeout validation (1-300 seconds) with clear error messages in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T034 [US1] Create WASI HTTP `OutgoingRequest` with GET method, scheme, authority, path in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T035 [US1] Call `outgoing_handler::handle(request, timeout_ns)` to make HTTP request in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T036 [US1] Wait for `future_response.get()` and extract `IncomingResponse` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T037 [US1] Extract status code from response in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T038 [US1] Implement helper function `read_body_to_string(body_stream)` to consume response body stream in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T039 [US1] Enforce 10MB response size limit during body reading in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs` (references data-model.md)
- [X] T040 [US1] Return outputs: `body` (string) and `status` (u32) in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T041 [US1] Run all US1 tests with `cargo test` and verify they PASS (green phase)
- [X] T042 [US1] Build component with `cargo build --target wasm32-wasip2 --release` and verify WASM output
- [X] T043 [US1] Copy compiled component to `/Users/doronila/git/wasmflow_cc/components/example_http_fetch.wasm`

### Manual Integration Test for US1

- [X] T044 [US1] Launch WasmFlow, add HTTP Fetch node, connect Constant("https://httpbin.org/get") → url, execute, verify response body contains JSON and status is 200

**Checkpoint**: HTTP Fetch component can make real HTTP GET requests, all US1 tests passing

---

## Phase 4: User Story 2 - Error Handling for Network Failures (Priority: P2)

**Goal**: Gracefully handle network errors with clear, actionable error messages

**Independent Test**: Provide invalid URLs (malformed, non-existent domains, timeouts) and verify meaningful errors returned

### Tests for User Story 2 (Write FIRST - TDD Red Phase)

- [X] T045 [P] [TEST] [US2] Create unit test for `map_wasi_error_to_execution_error()` with DNS error in component test module
- [X] T046 [P] [TEST] [US2] Create unit test for error mapping with connection refused in component test module
- [X] T047 [P] [TEST] [US2] Create unit test for error mapping with timeout in component test module
- [X] T048 [P] [TEST] [US2] Create unit test for error mapping with TLS error in component test module
- [X] T049 [P] [TEST] [US2] Create unit test for URL format validation (http/https prefix check) in component test module
- [X] T050 [TEST] [US2] Create integration test for malformed URL error handling in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs`
- [X] T051 [TEST] [US2] Create integration test for non-existent domain error handling in integration test file
- [X] T052 [TEST] [US2] Create integration test for timeout scenario (slow endpoint + 1s timeout) in integration test file
- [X] T053 [TEST] [US2] Create integration test for HTTP 404 error handling in integration test file
- [X] T054 [TEST] [US2] Create integration test for HTTP 500 error handling in integration test file
- [X] T055 Run all US2 tests with `cargo test` and verify they FAIL (expected - red phase)

### Implementation for User Story 2 (TDD Green Phase)

- [X] T056 [P] [US2] Implement helper function `map_error_code()` (maps WASI error codes) in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T057 [US2] Map `ErrorCode::DnsTimeout` and `ErrorCode::DnsError` to DNS resolution error with recovery hint in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T058 [US2] Map `ErrorCode::ConnectionRefused` to connection refused error with recovery hint in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T059 [US2] Map `ErrorCode::ConnectionTimeout` to connection timeout error with recovery hint in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T060 [US2] Map `ErrorCode::HttpResponseTimeout` to request timeout error with recovery hint in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T061 [US2] Map `ErrorCode::TlsCertificateError` to TLS validation error with recovery hint in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T062 [US2] Map HTTP 4xx/5xx status codes to clear error messages in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T063 [US2] Implement URL format validation (http:// or https:// prefix) with clear error in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T064 [US2] Wrap all WASI HTTP operations with error handling that calls error mapping function in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T065 [US2] Add logging via `host::log()` for errors in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T066 [US2] Run all US2 tests with `cargo test` and verify they PASS (green phase)

### Manual Integration Tests for US2

- [ ] T067 [US2] Manual test: Malformed URL ("not-a-url") → verify clear error message
- [ ] T068 [US2] Manual test: Non-existent domain ("https://does-not-exist-12345.invalid") → verify DNS error message
- [ ] T069 [US2] Manual test: Timeout scenario → verify timeout error message
- [X] T070 [US2] Manual test: 404 endpoint (https://httpbin.org/status/404) → verify HTTP error with status code

**Checkpoint**: All network error scenarios return user-friendly error messages, all US2 tests passing

---

## Phase 5: User Story 3 - Capability-Based Security Approval (Priority: P2)

**Goal**: Components validate URLs against declared capabilities before making requests

**Independent Test**: Attempt to fetch from unapproved domain and verify access denied error

### Tests for User Story 3 (Write FIRST - TDD Red Phase)

- [X] T071 [P] [TEST] [US3] Create unit test for `validate_url()` with approved domain in component test module
- [X] T072 [P] [TEST] [US3] Create unit test for validation with unapproved domain (should fail) in component test module
- [X] T073 [P] [TEST] [US3] Create unit test for subdomain matching (`api.example.com` allows `v2.api.example.com`) in component test module
- [X] T074 [P] [TEST] [US3] Create unit test for domain extraction from URL in component test module
- [X] T075 [TEST] [US3] Create integration test for capability approval (httpbin.org allowed) in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs`
- [X] T076 [TEST] [US3] Create integration test for capability denial (unauthorized.com blocked) in integration test file
- [X] T077 [TEST] [US3] Create integration test for subdomain match behavior in integration test file
- [X] T078 Run all US3 tests with `cargo test` and verify they FAIL (expected - red phase)

### Implementation for User Story 3 (TDD Green Phase)

- [X] T079 [P] [US3] Implement helper function `validate_url()` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T080 [US3] Extract domain from URL string (handle scheme, authority parsing) via `extract_domain_from_url()` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T081 [US3] Implement subdomain matching logic: `api.example.com` allows `*.api.example.com` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T082 [US3] Call validation function before making HTTP request in `execute()` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T083 [US3] Return clear "Access denied" error with list of approved domains in recovery hint in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T084 [US3] Add logging for capability checks (approved/denied) in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T085 [US3] Run all US3 tests with `cargo test` and verify they PASS (green phase)

### Manual Integration Tests for US3

- [X] T086 [US3] Manual test: URL to httpbin.org → verify allowed (in capabilities)
- [X] T087 [US3] Manual test: URL to unauthorized.com → verify access denied error with clear message
- [X] T088 [US3] Manual test: URL to subdomain v2.api.example.com → verify allowed (subdomain match)

**Checkpoint**: Component-side capability validation enforces network access control, all US3 tests passing

---

## Phase 6: User Story 4 - HTTP Response Headers Access (Priority: P3)

**Goal**: Provide response headers as optional output for advanced use cases

**Independent Test**: Fetch from endpoint with known headers, verify headers output contains expected key-value pairs

### Tests for User Story 4 (Write FIRST - TDD Red Phase)

- [X] T089 [P] [TEST] [US4] Create unit test for JSON formatting logic (replaces headers resource test - see note) in component test module
- [X] T090 [P] [TEST] [US4] Create unit test for JSON string escaping (standard approach) in component test module
- [X] T091 [P] [TEST] [US4] Create unit test for JSON structure validation in component test module
- [X] T092 [P] [TEST] [US4] Create unit test for empty headers format (should return "{}") in component test module
- [X] T093 [TEST] [US4] Create integration test that fetches from httpbin.org/get and verifies headers output in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs`
- [X] T094 [TEST] [US4] Create integration test that verifies headers JSON can be parsed downstream in integration test file
- [X] T095 Run all US4 tests with `cargo test` and verify they FAIL (expected - red phase)

**Note**: T089-T092 tests were adapted to test JSON formatting logic directly instead of WASI HTTP resources, which require full runtime context. The headers extraction is validated through integration tests.

### Implementation for User Story 4 (TDD Green Phase)

- [X] T096 [P] [US4] Add `headers` (string, optional) output port to `get_outputs()` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T097 [US4] Implement helper function `extract_headers_as_json()` to convert WASI HTTP headers to JSON string in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T098 [US4] Call `response.headers()` to get headers resource in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T099 [US4] Iterate through headers and build JSON map `{"content-type": "application/json", ...}` in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T100 [US4] Add `headers` to return outputs (as third output port) in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T101 [US4] Handle case where headers are empty (return empty JSON object "{}") in `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs`
- [X] T102 [US4] Run all US4 tests with `cargo test` and verify they PASS (green phase)

### Manual Integration Tests for US4

- [X] T103 [US4] Manual test: Fetch from httpbin.org/get → verify headers output contains Content-Type, Content-Length, etc.
- [ ] T104 [US4] Manual test: Connect headers output to downstream node → verify JSON parsing works

**Checkpoint**: Response headers accessible as JSON-encoded string output, all US4 tests passing

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, test refinement, and final quality improvements

- [X] T105 [P] Refactor integration tests: Extract common test helpers into `/Users/doronila/git/wasmflow_cc/tests/common/mod.rs`
- [X] T106 [P] Add test for concurrent HTTP requests (multiple parallel fetches) in `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs`
- [X] T107 [P] Add regression test for redirect behavior (same-domain vs cross-domain) in integration test file
- [X] T108 [P] Add test for response body encoding (UTF-8 validation) in integration test file
- [X] T109 [P] Update `/Users/doronila/git/wasmflow_cc/README.md` with WASI HTTP example usage and test instructions
- [X] T110 [P] Create example graph JSON file in `/Users/doronila/git/wasmflow_cc/examples/graphs/http-fetch-demo.json` demonstrating HTTP Fetch usage (JSON format for documentation)
- [X] T111 [P] Add code coverage configuration to `Cargo.toml` (using cargo-llvm-cov)
- [X] T112 Run `cargo clippy` on component and fix any warnings
- [X] T113 Run `cargo test --all` and ensure all 112 tests pass (42 lib + 70 integration/contract)
- [X] T114 Run full WasmFlow application and execute example graph end-to-end
- [X] T115 Generate test coverage report and document in `/specs/002-lets-focus-on/test-coverage.md`

**Final Checkpoint**: All user stories complete, comprehensive test suite passing (40+ tests), documented

---

## Dependencies & Execution Order

### User Story Dependencies

```
Setup (T001-T007) ← Test infrastructure
    ↓
Foundational (T008-T015) ← BLOCKS ALL USER STORIES
    ↓  [Tests T008-T009 → Implementation T010-T014 → Verify T015]
    ↓
    ├─→ US1: Basic HTTP GET (T016-T044) ← MVP
    │   [Tests T016-T025 → Implementation T026-T043 → Verify T044]
    ↓
    ├─→ US2: Error Handling (T045-T070) ← Depends on US1
    │   [Tests T045-T055 → Implementation T056-T066 → Verify T067-T070]
    ├─→ US3: Capability Validation (T071-T088) ← Can run parallel with US2
    │   [Tests T071-T078 → Implementation T079-T085 → Verify T086-T088]
    ↓
    └─→ US4: Headers Access (T089-T104) ← Depends on US1
        [Tests T089-T095 → Implementation T096-T102 → Verify T103-T104]
    ↓
Polish (T105-T115) ← After all stories complete
```

### Critical Path (TDD Approach)

**Blocking**: Setup (7 tasks) → Foundational (7 tasks) → US1 (29 tasks) = **43 tasks** before first user story works

**MVP Scope**: US1 with tests = **43 tasks total**

### TDD Workflow Per User Story

For each user story:
1. **Red Phase**: Write all tests FIRST (verify they FAIL)
2. **Green Phase**: Implement functionality until tests PASS
3. **Refactor Phase**: Clean up code while keeping tests green
4. **Verify Phase**: Manual integration tests

---

## Test Coverage Summary

### Unit Tests (Component-Level)

**Location**: `/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/src/lib.rs` (`#[cfg(test)]` module)

| User Story | Unit Tests | Description |
|------------|------------|-------------|
| US1 | 8 tests | Helper functions (extract_string, parse_url, timeout validation, body reading) |
| US2 | 6 tests | Error mapping (DNS, connection, timeout, TLS, URL validation) |
| US3 | 4 tests | URL validation (approved, denied, subdomain, domain extraction) |
| US4 | 4 tests | Headers extraction (standard, custom, empty, JSON conversion) |
| **Total** | **22 unit tests** | Component logic validation |

### Integration Tests (Runtime-Level)

**Location**: `/Users/doronila/git/wasmflow_cc/tests/integration/wasi_http_execution_test.rs`

| User Story | Integration Tests | Description |
|------------|-------------------|-------------|
| Foundational | 2 tests | wasmtime linker setup, WASI HTTP context |
| US1 | 3 tests | Real HTTP request, timeout default, timeout custom |
| US2 | 5 tests | Malformed URL, DNS error, timeout, HTTP 404, HTTP 500 |
| US3 | 3 tests | Capability approval, denial, subdomain match |
| US4 | 2 tests | Headers extraction, JSON parsing |
| Polish | 4 tests | Concurrency, redirects, encoding, regression |
| **Total** | **19 integration tests** | End-to-end WASI HTTP execution |

### Contract Tests (WIT Interface)

**Location**: `/Users/doronila/git/wasmflow_cc/tests/contract/wasi_http_component_test.rs`

| Test | Description |
|------|-------------|
| WIT exports | Verify component exports `metadata` and `execution` |
| WIT imports | Verify component imports `wasi:http/types` and `wasi:http/outgoing-handler` |
| Port specifications | Verify inputs (url, timeout) and outputs (body, status, headers) match spec |
| **Total** | **3 contract tests** | WIT interface compliance |

### Manual Tests

| User Story | Manual Tests | Description |
|------------|--------------|-------------|
| US1 | 1 test | End-to-end WasmFlow graph execution |
| US2 | 4 tests | Error scenarios in live UI |
| US3 | 3 tests | Capability validation in live UI |
| US4 | 2 tests | Headers output verification |
| **Total** | **10 manual tests** | User acceptance testing |

### Total Test Count

**Automated Tests**: 44 tests (22 unit + 19 integration + 3 contract)
**Manual Tests**: 10 tests
**Grand Total**: **54 tests**

---

## Parallel Execution Opportunities

**Within Test Writing** (can write in parallel):
- T016-T019 (US1 unit tests) - 4 tests in parallel
- T045-T049 (US2 unit tests) - 5 tests in parallel
- T071-T074 (US3 unit tests) - 4 tests in parallel
- T089-T092 (US4 unit tests) - 4 tests in parallel

**Between User Stories** (after US1 complete):
- US2 tests + implementation (T045-T070) can run parallel with US3 tests + implementation (T071-T088)
- Polish phase tests (T105-T108) can all run in parallel

**Total Parallel Tasks**: ~35 tasks can run in parallel across different developers/agents

---

## Implementation Strategy

### MVP Delivery (User Story 1 with Tests)

**Scope**: 43 tasks (Setup + Foundational + US1 with TDD)

**Outcome**: Working HTTP Fetch component with:
- Real HTTP GET requests
- Response body and status code
- Timeout handling (default 30s, configurable)
- **22 passing unit tests**
- **5 passing integration tests**
- **3 passing contract tests**

**Time Estimate**: 10-12 hours for experienced developer (includes test writing)

### Incremental Delivery (TDD Approach)

1. **Sprint 1**: Setup + Foundational + US1 (43 tasks, 30 tests) → **MVP with tests**
2. **Sprint 2**: US2 + US3 in parallel (47 tasks, 20 tests) → Robust + Secure with tests
3. **Sprint 3**: US4 (16 tasks, 8 tests) → Full-featured with tests
4. **Sprint 4**: Polish (11 tasks, 7 tests) → Production-ready with 54 total tests

### Test-First Development Flow

For each task block:
1. **Write Tests** → Run → **Verify FAIL** (Red)
2. **Implement** → Run Tests → **Verify PASS** (Green)
3. **Refactor** → Run Tests → **Stay Green**
4. **Manual Verify** → End-to-end validation

---

## Task Count Summary

| Phase | Tasks | Tests | Implementation | Verification | Description |
|-------|-------|-------|----------------|--------------|-------------|
| Setup | 7 | 0 | 7 | 0 | WASI HTTP WIT + test infrastructure |
| Foundational | 7 | 2 | 5 | 0 | wasmtime integration (blocking) |
| US1 (P1) | 29 | 10 | 18 | 1 | Basic HTTP GET - MVP with tests |
| US2 (P2) | 26 | 11 | 11 | 4 | Error handling with tests |
| US3 (P2) | 18 | 8 | 7 | 3 | Capability validation with tests |
| US4 (P3) | 16 | 7 | 7 | 2 | Headers access with tests |
| Polish | 11 | 4 | 5 | 2 | Test refinement + documentation |
| **TOTAL** | **114** | **42** | **60** | **12** | Full implementation with test suite |

**Automated Tests**: 44 (38 from user stories + 4 polish + 2 foundational)
**Manual Tests**: 10
**Total Test Coverage**: 54 tests

---

## Notes

**TDD Benefits**:
- Tests document expected behavior
- Catch regressions early
- Design feedback (hard to test = bad design)
- Confidence in refactoring

**WASI HTTP Preview Considerations**:
- Preview API 0.2.0 - experimental but tested
- Component-side validation tested thoroughly
- Integration tests verify wasmtime-wasi-http behavior
- Contract tests ensure WIT compliance

**Test Maintenance**:
- Unit tests are fast (milliseconds)
- Integration tests may be slow (real HTTP)
- Consider mocking for faster test execution (optional)
- CI/CD can run full suite on every commit

**References**:
- Technical context: `/specs/002-lets-focus-on/plan.md`
- User stories: `/specs/002-lets-focus-on/spec.md`
- WASI HTTP usage: `/specs/002-lets-focus-on/contracts/wasi-http-usage.md`
- Data model: `/specs/002-lets-focus-on/data-model.md`
