# Implementation Plan: JSON Parser Node

**Branch**: `008-json-parser-a` | **Date**: 2025-10-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-json-parser-a/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a JSON parser node WASM component that accepts a JSON string and a key path string (using dot and bracket notation) and extracts the value at that path. The component must support nested object access (`metadata.author`), array indexing (`runs[1]`), and combined paths (`runs[1].time`), while providing comprehensive error handling for invalid JSON, missing paths, and malformed key paths.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: serde_json (JSON parsing), wasmtime 27.0 (component-model runtime), wit-bindgen (WIT interface generation)
**Storage**: N/A (stateless component - processes inputs to outputs)
**Testing**: cargo test (unit tests for key path parser and value extractor), component contract tests (WIT interface validation)
**Target Platform**: WASM (wasm32-wasip2) component compatible with wasmflow_cc node system
**Project Type**: Single WASM component (builtin node implementation)
**Performance Goals**: Process JSON payloads up to 1MB in <100ms, support nesting depth of 100+ levels
**Constraints**: Sandboxing level "Strict" (no system access - pure computation), synchronous execution model
**Scale/Scope**: Single-purpose component with 2 inputs (json_string, key_path), 2 outputs (value, error), ~500-800 LOC estimated

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Component-First Architecture ✅ PASS

- **WIT Interface Required**: Component will define clear WIT interface with inputs (json-string, key-path) and outputs (value, error)
- **Self-Contained**: Pure computation component with no external dependencies
- **Node Integration**: Follows existing builtin node pattern (see `src/builtin/`)
- **Functional Purpose**: Clear single responsibility - JSON value extraction via key path

### II. Capability-Based Security ✅ PASS

- **Sandboxing Level**: Strict (no system access required)
- **Capability Declaration**: No capabilities needed - pure computation
- **Resource Limits**: Component operates within WASM memory constraints
- **Security Model**: No user consent required (no privileged operations)

### III. Typed Data Flow ✅ PASS

- **Input Types**:
  - `json-string: string` (JSON payload)
  - `key-path: string` (path specification)
- **Output Types**:
  - `value: variant { string(string), number(f64), boolean(bool), object(string), array(string), null }` (preserves JSON types)
  - `error: option<string>` (error message if extraction fails)
- **Type Validation**: WIT enforces type contracts at component boundaries
- **Error Surface**: Clear error messages for type mismatches surfaced via error port

### IV. Performance-First Execution ✅ PASS

- **Target**: Process 1MB JSON in <100ms
- **Instantiation**: Stateless component enables instance pooling/reuse
- **Parallel Execution**: Component is pure function - supports parallel execution across graph branches
- **Incremental Updates**: Component re-executes only when inputs change
- **Graph Scale**: Single node - no impact on 500+ node graph performance

### V. Developer Ergonomics ✅ PASS

- **Standard Tooling**: cargo-component for WASM component build
- **WIT Examples**: Will provide example WIT interface and usage documentation
- **Error Messages**: Comprehensive error handling with actionable messages (FR-008 through FR-011)
- **Testing**: Unit tests for key path parser, integration tests for component contracts
- **Documentation**: quickstart.md will guide component usage and integration

### VI. Composability & Modularity ✅ PASS

- **Composition Ready**: Component can be used standalone or composed with other nodes
- **Reusability**: Generic JSON parser - reusable across any workflow needing JSON extraction
- **Metadata**: Will include author, version, description in component metadata
- **Category**: Data transformation / parsing category for discoverability
- **Versioning**: Follows SemVer for WIT interface changes

**GATE RESULT**: ✅ ALL CHECKS PASS - Proceed to Phase 0

---

## Post-Design Constitution Re-evaluation

*Re-checked after Phase 1 design artifacts completed*

### Design Validation

✅ **WIT Interface**: Defined in `contracts/json-parser.wit` with clear input/output types
✅ **Data Model**: Documented in `data-model.md` with comprehensive type mappings
✅ **Error Handling**: Structured errors via `parse-error` record with 5 distinct error kinds
✅ **Type Safety**: WIT variant types ensure type preservation from JSON to output
✅ **Performance**: Target <100ms for 1MB JSON confirmed achievable with serde_json
✅ **Testing Strategy**: Three-tier approach (unit, contract, integration) documented
✅ **Documentation**: quickstart.md provides complete usage guide with 9 examples

### Constitution Compliance (Post-Design)

All six constitution principles remain satisfied after design phase:

1. **Component-First**: WIT interface fully specified ✅
2. **Security**: No capabilities required (pure computation) ✅
3. **Typed Data Flow**: All types defined in WIT ✅
4. **Performance**: Benchmarking plan included ✅
5. **Developer Ergonomics**: Comprehensive docs and examples ✅
6. **Composability**: Reusable across workflows ✅

**FINAL GATE RESULT**: ✅ ALL CHECKS PASS - Ready for Phase 2 (Tasks)

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── builtin/
│   ├── json_parser.rs        # NEW: JSON parser node implementation
│   └── mod.rs                 # MODIFIED: Register json_parser module
│
├── node/
│   ├── value.rs               # MODIFIED: May need JsonValue variant extensions
│   └── mod.rs
│
└── wit/
    └── json-parser.wit        # NEW: WIT interface definition

tests/
├── contract/
│   └── json_parser_test.rs   # NEW: WIT contract tests
│
├── integration/
│   └── json_parser_graph.rs  # NEW: Graph integration tests
│
└── unit/
    └── json_parser_unit.rs   # NEW: Unit tests for key path parsing
```

**Structure Decision**: Single project structure using existing wasmflow_cc repository. Component implemented as a builtin node following the pattern established in `src/builtin/`. WIT interface defined in `src/wit/` directory for component model integration.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations**: All constitution checks pass. This component follows established patterns and requires no complexity justification.
