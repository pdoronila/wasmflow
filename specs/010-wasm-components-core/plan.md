# Implementation Plan: WASM Components Core Library

**Branch**: `010-wasm-components-core` | **Date**: 2025-10-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/home/user/wasmflow/specs/010-wasm-components-core/spec.md`

## Summary

Build a comprehensive core library of 35+ WebAssembly components organized into foundational categories (string operations, comparison/logic, math extensions, list operations, and data transformation). Components follow the established wasmflow pattern using WIT interfaces, Rust implementations, and the Just/Nushell build system. All components integrate with the existing component loading system and node editor UI, providing immediate value for data processing workflows.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: wit-bindgen 0.30, serde (for list/data serialization), standard library (no external crates for core operations)
**Storage**: N/A (stateless components - all data flows through inputs/outputs)
**Testing**: cargo test (unit tests embedded in each component's src/lib.rs)
**Target Platform**: WASM Component Model (wasm32-wasip2), loaded by wasmtime 27.0 runtime
**Project Type**: Component library (35+ independent WASM components in shared directory structure)
**Performance Goals**: <10ms execution per operation for typical inputs (strings <1MB, lists <1000 elements), <1 second end-to-end for user workflows
**Constraints**: Component size <200KB per .wasm file, zero heap allocations for simple operations where possible, IEEE 754 compliance for math operations
**Scale/Scope**: 35+ components across 5 categories, all following identical build/interface patterns established by existing json-parser component

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Component-First Architecture ✅

**Compliance**: FULL
- All 35+ components implemented as WASM components with WIT interfaces
- Each component is self-contained and independently testable
- Components integrate with node-based visual composition via existing component loading system
- All components use the standard `world component` WIT interface (metadata + execution)
- Clear functional purpose for each component (string-concat, math-power, list-get, etc.)

**Evidence**: Following established pattern from json-parser component with identical WIT structure.

### II. Capability-Based Security (NON-NEGOTIABLE) ✅

**Compliance**: FULL
- All core library components are **pure computation** - no system access required
- Capability declaration: `None` for all 35+ components (no file I/O, no network)
- Default deny enforced by wasmtime runtime context (no WASI imports beyond memory/allocation)
- Zero security risk from these components - sandboxed execution only

**Evidence**: String, math, logic, list, and type conversion operations require no capabilities. Follows same pattern as existing adder/double-number components.

### III. Typed Data Flow ✅

**Compliance**: FULL
- All components use WIT-defined types: u32, i32, f32, string, list, any
- Type validation at graph construction via existing node editor
- Input/output type mismatches surfaced to users in UI
- List operations use `list<string>` for string lists, `list<any>` for mixed types
- Compare component handles type checking internally, returns errors for incompatible types

**Evidence**: Using established WIT type system. Compare component includes type compatibility logic.

### IV. Performance-First Execution ✅

**Compliance**: FULL
- Components leverage existing graph execution engine's topological sorting
- Component instance pooling/reuse handled by existing ComponentManager
- Each component execution target: <10ms for typical inputs
- Zero-copy string operations where possible (string views, slicing)
- Math operations use native WASM instructions (optimal performance)
- List operations use efficient Vec<T> internally

**Evidence**: Core operations are simple, single-purpose functions. No complex algorithms or external I/O.

### V. Developer Ergonomics ✅

**Compliance**: FULL
- Standard cargo-component tooling via Just + Nushell build system
- WIT interface follows established template (copy from json-parser)
- Components auto-discovered from `components/bin/` directory (existing mechanism)
- Error messages include input_name and recovery_hint for actionable context
- Unit tests required in each component's src/lib.rs
- Comprehensive documentation in component READMEs

**Evidence**: Build system already established. All components use identical Justfile and build.rs.

### VI. Composability & Modularity ✅

**Compliance**: FULL
- Components organized by category: components/core/, components/math/, components/collections/
- Component metadata includes category field for palette organization
- Each component is independently reusable and composable
- Components can be chained in node graphs to create complex operations
- Import/export via existing graph save/load mechanism

**Evidence**: Category organization enables discoverability. Visual composition via node editor.

**GATE RESULT**: ✅ **PASSED** - No violations. All core library components comply with constitution.

## Project Structure

### Documentation (this feature)

```
specs/010-wasm-components-core/
├── plan.md              # This file
├── research.md          # Phase 0 output (technology choices, patterns)
├── data-model.md        # Phase 1 output (component specifications)
├── quickstart.md        # Phase 1 output (development guide)
├── contracts/           # Phase 1 output (WIT interface templates)
│   ├── string-operations.wit
│   ├── comparison-logic.wit
│   ├── math-operations.wit
│   ├── list-operations.wit
│   └── data-transformation.wit
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```
components/
├── core/                         # String and logic operations (P1-P2)
│   ├── string-concat/
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── Justfile
│   │   ├── wit/
│   │   │   └── node.wit
│   │   └── src/
│   │       └── lib.rs
│   ├── string-split/
│   ├── string-length/
│   ├── string-trim/
│   ├── string-case/
│   ├── string-contains/
│   ├── string-substring/
│   ├── compare/
│   ├── boolean-and/
│   ├── boolean-or/
│   ├── boolean-not/
│   ├── boolean-xor/
│   ├── is-null/
│   └── is-empty/
│
├── math/                         # Mathematical operations (P3)
│   ├── power/
│   ├── sqrt/
│   ├── abs/
│   ├── min/
│   ├── max/
│   ├── floor/
│   ├── ceil/
│   ├── round/
│   └── trig/
│
├── collections/                  # List operations (P4)
│   ├── list-length/
│   ├── list-get/
│   ├── list-append/
│   ├── list-join/
│   ├── list-slice/
│   ├── list-contains/
│   └── list-index-of/
│
├── data/                         # Data transformation (P5)
│   ├── json-stringify/
│   ├── to-string/
│   ├── parse-number/
│   └── format-template/
│
├── bin/                          # Compiled .wasm files (build output)
│   ├── string_concat.wasm
│   ├── string_split.wasm
│   └── ... (35+ total)
│
└── Justfile                      # Top-level build automation

tests/
├── integration/
│   └── core_library_test.rs     # Integration tests for component loading
└── component_tests/              # Individual component test graphs
    ├── string_processing.json
    ├── math_operations.json
    └── list_manipulation.json
```

**Structure Decision**: Using category-based directory structure as proposed in spec. Each category (core, math, collections, data) contains related components. This structure:
- Improves discoverability (developers can find related operations quickly)
- Enables batch operations (build all math components, test all string components)
- Maps to UI component palette categories
- Follows modular design principles (separation by functional domain)

All 35+ components follow identical internal structure (Cargo.toml, build.rs, Justfile, wit/, src/) established by existing components like json-parser.

## Complexity Tracking

*No constitutional violations - this section intentionally left empty.*

All components comply with wasmflow constitution. No complexity exceptions required.
