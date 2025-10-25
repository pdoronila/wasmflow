# WasmFlow Components Library

**Version**: 1.1.0
**Total Components**: 43+
**Categories**: Text, Logic, Math, Collections, Data, HTTP
**Target**: wasm32-wasip2
**WIT Spec**: wasmflow:node@1.1.0

## Overview

The WasmFlow Components Library is a comprehensive collection of pre-built WebAssembly components for common data processing, HTTP handling, and visual programming operations. All components are compiled to wasm32-wasip2 and can be composed together in visual node graphs.

**Key Features**:
- **Pure WASM Components**: Cross-platform execution via WebAssembly
- **Type-Safe Interfaces**: WIT (WebAssembly Interface Types) contracts
- **Comprehensive Testing**: 148+ unit tests across all components
- **Optimized Binaries**: 60KB-1MB per component with LTO optimization
- **Minimal Dependencies**: Mostly standard library only

## Quick Start

### Installing Components

#### Install All Components

```bash
cd components
just install

# With parallel builds (faster, uses 4 threads)
just install "" 4

# With 8 threads
just install "" 8
```

#### Install Single Component

```bash
cd components
just install data/to-string

# Or navigate directly
cd components/data/to-string
just install
```

#### Install All Components in a Category

```bash
# From components directory
cd components/data
just install

# With parallel builds (4 threads)
cd components/data
just install "" 4

# Or from components root
cd components
just install data
```

### Building Components

#### Build All Components

```bash
cd components
just build

# With parallel builds (4 threads)
just build "" 4
```

#### Build Single Component

```bash
cd components
just build html/url-decode

# Or navigate directly
cd components/html/url-decode
just build
```

#### Build Category

```bash
cd components/html
just build

# With 8 threads
just build "" 8
```

### Testing Components

```bash
# Test all components
cd components
just test

# Test with parallel execution
just test "" 4

# Test single component
just test data/json-escape-string

# Test category
cd components/collections
just test
```

### Cleaning Build Artifacts

```bash
# Clean all components
cd components
just clean

# With parallel execution
just clean "" 4

# Clean single component
just clean math/math-power

# Clean category
cd components/math
just clean
```

## Available Commands

All commands support optional threading for `-all` variants:

| Command | Description | Single | Category | All (default) | All (threaded) |
|---------|-------------|--------|----------|---------------|----------------|
| `build` | Compile to WASM | `just build <path>` | `cd <category> && just build` | `just build` | `just build "" 4` |
| `install` | Copy to bin/ | `just install <path>` | `cd <category> && just install` | `just install` | `just install "" 8` |
| `test` | Run unit tests | `just test <path>` | `cd <category> && just test` | `just test` | `just test "" 4` |
| `clean` | Remove artifacts | `just clean <path>` | `cd <category> && just clean` | `just clean` | `just clean "" 4` |

**Threading Notes**:
- Default is 1 thread (sequential)
- Recommended: 4-8 threads for faster builds
- Syntax: `just <command> "" <threads>`
- Example: `just install "" 4` installs all components using 4 threads

## Component Categories

### Text Processing (9 components)

**Location**: `core/` + `text/`

Basic string operations (7): string-concat, string-split, string-length, string-trim, string-case, string-contains, string-substring

Regex pattern matching (2): regex-match, regex-match-any

```bash
# Install all text components
cd components/core && just install
cd components/text && just install
```

### Logic & Validation (7 components)

**Location**: `core/`

compare, boolean-and, boolean-or, boolean-not, boolean-xor, is-null, is-empty

```bash
cd components/core && just install
```

### Mathematical Operations (9 components)

**Location**: `math/`

power, sqrt, abs, min, max, floor, ceil, round, trig

```bash
cd components/math && just install

# Or with parallel builds
cd components/math && just install "" 4
```

### List Manipulation (13 components)

**Location**: `collections/`

Basic list operations (7): list-length, list-get, list-append, list-join, list-slice, list-contains, list-index-of

Advanced with regex (6): list-filter-empty, list-filter-regex, list-filter-regex-any, list-reject-regex, list-count-regex, list-count-regex-any

```bash
cd components/collections && just install "" 8
```

### Data Transformation (5+ components)

**Location**: `data/`

json-stringify, json-extract-each, to-string, parse-number, format-template, parse-key-value-pairs, json-build-object, json-parse-flat-object, json-escape-string, url-decode, url-encode

```bash
cd components/data && just install "" 4
```

### HTTP Components

**Location**: `html/`

HTTP server utilities: body-parser, content-type-header, header-builder, html-escape, http-cookie-parser, http-cors-headers, http-fetch, http-request-parser, http-response-builder, http-set-cookie-builder, json-response-builder, mime-type-detector, path-matcher, query-string-parser, route-dispatcher, simple-template-render, static-file-response, status-code-mapper, url-path-join

```bash
cd components/html && just install "" 8
```

## Directory Structure

```
components/
├── README.md              (this file)
├── LIBRARY.md             (developer guide - detailed API reference)
├── Justfile               (top-level build automation)
├── .templates/            (component templates)
│   ├── node.wit           (standard component WIT)
│   ├── node-with-ui.wit   (component with custom UI WIT)
│   ├── lib.rs             (component template code)
│   └── ...
├── bin/                   (installed .wasm files)
│   ├── string_concat.wasm
│   ├── url_decode.wasm
│   └── ...
├── core/                  (text + logic components)
│   ├── Justfile
│   ├── string-concat/
│   ├── boolean-and/
│   └── ...
├── math/                  (mathematical operations)
│   ├── Justfile
│   ├── math-power/
│   ├── math-sqrt/
│   └── ...
├── collections/           (list manipulation)
│   ├── Justfile
│   ├── list-filter-regex/
│   └── ...
├── data/                  (data transformation)
│   ├── Justfile
│   ├── json-stringify/
│   ├── parse-key-value-pairs/
│   └── ...
├── html/                  (HTTP utilities)
│   ├── Justfile
│   ├── url-decode/
│   ├── http-request-parser/
│   └── ...
└── examples/              (example components)
    ├── Justfile
    └── footer-view/
```

## Prerequisites

```bash
# Rust with wasm32-wasip2 target
rustup target add wasm32-wasip2

# Command runner
cargo install just

# Optional: WASM tools for advanced development
cargo install cargo-component
cargo install wasm-tools
```

## Component Requirements

All `.wasm` files must:

1. **Be valid WebAssembly components** built with the Component Model
2. **Export the wasmflow:node interface** (metadata + execution)
3. **Be compiled for wasm32-wasip2** target
4. **Use the correct WIT template**:
   - `node.wit` for standard components (computation only)
   - `node-with-ui.wit` for components with custom UI rendering

## Creating New Components

### From Template

```bash
# Copy template structure
cp -r components/.templates/my-component components/data/

# Update the component
cd components/data/my-component
# Edit Cargo.toml, src/lib.rs, wit/node.wit

# Build and test
just test
just build
just install
```

### Component Structure

Every component follows this structure:

```
component-name/
├── Cargo.toml          # Package config
├── Justfile            # Build automation
├── wit/
│   └── node.wit        # WIT interface definition
└── src/
    └── lib.rs          # Implementation + tests
```

### Standard Implementation Pattern

See `LIBRARY.md` for detailed implementation patterns, common pitfalls, and best practices.

## Performance

- **Binary sizes**: 60KB-1MB (regex/JSON components larger due to dependencies)
- **Execution time**: <10ms for typical operations
- **Memory**: Stack-allocated, immutable operations
- **Optimization**: LTO enabled, stripped binaries, size-optimized

## Troubleshooting

### Component doesn't build

**Check**:
1. `rustup target add wasm32-wasip2` installed
2. Correct WIT file (use `.templates/node.wit` or `.templates/node-with-ui.wit`)
3. `export!(Component);` macro present in lib.rs
4. Correct DataType variants: `StringType`, `U32Type`, `BoolType`, etc.
5. PortSpec uses `optional` field (not `required`)

### Component builds but doesn't load

**Check**:
1. Built with `--target wasm32-wasip2`
2. File copied to `bin/` directory
3. File has `.wasm` extension
4. WIT interface matches expected schema

### Parallel builds fail

**Fix**:
- Reduce thread count: `just install "" 2`
- Check disk space and memory
- Try sequential build: `just install "" 1`

## Development Workflow

1. **Create** component from template
2. **Implement** WIT interface in `src/lib.rs`
3. **Write tests** (minimum 3: typical, edge, error cases)
4. **Test** with `just test`
5. **Build** with `just build`
6. **Install** with `just install`
7. **Load** in WasmFlow (File → Reload Components)
8. **Test** in node graph
9. **Iterate** based on feedback

## Documentation

- **LIBRARY.md** - Comprehensive developer guide with:
  - Detailed API reference for all 43 components
  - Implementation patterns and best practices
  - Common pitfalls and solutions
  - Performance characteristics
  - Testing strategies

- **Component-specific READMEs** - Each component has its own README with:
  - Purpose and description
  - Input/output specifications
  - Usage examples
  - Edge cases and error handling

- **Integration Tests** - `tests/component_tests/` contains:
  - string_processing.json
  - data_validation.json
  - math_operations.json
  - list_manipulation.json
  - data_transformation.json
  - comprehensive_workflow.json

## Resources

- **Templates**: `.templates/` directory
- **Examples**: `examples/footer-view` and all components
- **WIT Spec**: See `.templates/node.wit` for interface definition
- **Build Scripts**: Justfiles at all levels

## Getting Help

- **Check logs**: `RUST_LOG=debug cargo run`
- **Review LIBRARY.md**: Comprehensive developer guide
- **Examine working components**: See any component in `core/`, `math/`, etc.
- **Run tests**: `just test` to verify component behavior

## Quick Reference Card

```bash
# From components directory:

# Install everything (parallel, 4 threads)
just install "" 4

# Install one category (parallel, 8 threads)
cd math && just install "" 8

# Install one component
just install data/to-string

# Build everything (sequential)
just build

# Test everything (parallel, 4 threads)
just test "" 4

# Clean everything (parallel, 8 threads)
just clean "" 8

# List all component directories
just list
```

---

**Need more details?** See [LIBRARY.md](./LIBRARY.md) for the complete developer guide with API references, implementation patterns, and advanced topics.
