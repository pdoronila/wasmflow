# Quick Start: Component Directory Migration

**Feature**: Component Directory Reorganization
**Branch**: `009-reorginize-components-currently`
**Date**: 2025-10-22

## Overview

This guide explains the component directory reorganization and how it affects developers and users of WasmFlow.

## What Changed

### Before (Old Structure)
```
components/
├── README.md
├── example_file_reader.wasm
├── example_footer_view.wasm
├── example_http_fetch.wasm
├── example_json_parser.wasm
├── rust_convert_f32_to_u32.wasm
├── rust_convert_u32_to_f32.wasm
└── rust_string.wasm
```

### After (New Structure)
```
components/
├── README.md
└── bin/
    ├── example_file_reader.wasm
    ├── example_footer_view.wasm
    ├── example_http_fetch.wasm
    ├── example_json_parser.wasm
    ├── rust_convert_f32_to_u32.wasm
    ├── rust_convert_u32_to_f32.wasm
    └── rust_string.wasm
```

**Key Change**: All WASM component binaries now live in `components/bin/` subdirectory.

## Why This Change?

**Benefits**:
- **Clearer organization**: Separates binary artifacts from documentation and configuration
- **Consistent pattern**: Follows common practice of using `bin/` for executables
- **Easier discovery**: All component binaries in one obvious location
- **Room for growth**: Future additions (e.g., `src/`, `lib/`, `examples/`) have logical places

## For Component Developers

### Adding New Components

**Old Way**:
```bash
# Build your component
cd my-component
cargo component build --release

# Copy to WasmFlow components directory
cp target/wasm32-wasip2/release/my_component.wasm \
   /path/to/wasmflow/components/
```

**New Way**:
```bash
# Build your component
cd my-component
cargo component build --release

# Copy to WasmFlow components/bin/ directory
cp target/wasm32-wasip2/release/my_component.wasm \
   /path/to/wasmflow/components/bin/
```

**Only Change**: Add `/bin` to the destination path.

### Using the WASM Creator Node

**No change needed!** The WASM Creator builtin node automatically saves compiled components to `components/bin/`.

### Building Example Components

**No change to build process**:
```bash
cd examples/my-example
cargo component build --release
```

The built component in `target/wasm32-wasip2/release/` can now be copied to `components/bin/` for easy access alongside production components.

## For WasmFlow Users

### Existing Installations

**If you're upgrading from a previous version**:

1. **Check for custom components**: Do you have custom .wasm files in `components/`?
   ```bash
   ls components/*.wasm
   ```

2. **If yes, move them to bin/**:
   ```bash
   mkdir -p components/bin
   mv components/*.wasm components/bin/
   ```

3. **If no custom components**: Just pull the latest code and restart WasmFlow. Example components are already in place.

### Fresh Installations

**No action needed!** The `components/bin/` directory is already set up with example components.

### Saved Graphs

**No migration needed!** Saved graphs reference components by name and ID, not file path. Your existing graphs will load and work without modification.

## For Build Scripts / Automation

**If you have scripts that copy components**, update them:

**Before**:
```bash
# Old script
cp build/*.wasm /path/to/wasmflow/components/
```

**After**:
```bash
# New script
cp build/*.wasm /path/to/wasmflow/components/bin/
```

## Troubleshooting

### "Components directory not found: components/bin/"

**Cause**: The `components/bin/` directory doesn't exist.

**Solution**:
```bash
# Create the directory
mkdir -p components/bin

# If you have components in the old location, move them
mv components/*.wasm components/bin/ 2>/dev/null || true
```

### "No components found in components/bin/ directory"

**Cause**: The directory exists but is empty.

**Solution**:
- Add component files to `components/bin/`
- Or copy example components from the repository
- Or build example components and copy them

### Components Not Loading After Migration

**Verify directory structure**:
```bash
# Check that components are in bin/
ls -la components/bin/*.wasm

# Should show example_file_reader.wasm, etc.
```

**Check working directory**:
```bash
# WasmFlow looks for components/bin/ relative to current directory
# Make sure you're running from the project root
pwd
# Should show: /path/to/wasmflow_cc
```

**Reload components**:
- In WasmFlow, select **File → Reload Components**
- Check status bar for loading message

### Old Components in Wrong Location

**If you still have .wasm files in `components/` (not `components/bin/`)**:

They won't be loaded. Move them:
```bash
mv components/*.wasm components/bin/
```

## Quick Reference

### File Locations

| Item | Old Path | New Path |
|------|----------|----------|
| WASM binaries | `components/*.wasm` | `components/bin/*.wasm` |
| Documentation | `components/README.md` | `components/README.md` (unchanged) |
| Build output | `examples/*/target/.../release/*.wasm` | (unchanged) |

### Commands

| Task | Command |
|------|---------|
| Add new component | `cp my_component.wasm components/bin/` |
| List components | `ls components/bin/*.wasm` |
| Reload in app | **File → Reload Components** |
| Build example | `cd examples/xxx && cargo component build --release` |

## Technical Details

### Code Changes

For developers contributing to WasmFlow:

**Component Loading** (src/ui/app.rs):
```rust
// Before
let components_dir = std::path::Path::new("components");

// After
let components_dir = std::path::Path::new("components/bin");
```

**WASM Creator Output** (src/builtin/wasm_creator.rs):
```rust
// Before
let components_dir = std::env::current_dir()
    .unwrap_or_else(|_| std::path::PathBuf::from("."))
    .join("components");

// After
let components_dir = std::env::current_dir()
    .unwrap_or_else(|_| std::path::PathBuf::from("."))
    .join("components")
    .join("bin");
```

### Backward Compatibility

**Saved Graphs**: No changes needed. Graphs reference components by name/ID, not file path.

**Component Metadata**: Unchanged. ComponentSpec format is identical.

**Runtime Behavior**: Unchanged. Components load and execute exactly as before.

## Summary

**Single Change**: WASM component binaries move from `components/` to `components/bin/`.

**Impact**:
- ✅ Developers: Update copy destination path
- ✅ Users: Move custom components (if any)
- ✅ Build scripts: Update destination path
- ✅ Saved graphs: No change needed
- ✅ Component behavior: No change

**Benefits**: Clearer organization, consistent structure, room for future growth.

**Questions?** See `components/README.md` for full component documentation.
