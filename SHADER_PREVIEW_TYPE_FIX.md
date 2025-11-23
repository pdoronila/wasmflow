# Shader Preview Type Specification Fix

## Problem

Graph structure validation was failing when loading PBR demo files with the error:
```
incompatible types vec3 (source) -> list<f32> (target)
```

## Root Cause

The shader preview node specification in `create_pbr_demos.rs` declared incorrect input port types:
- `view_matrix`: Declared as `List<F32>`, but `perspective-camera` outputs `Mat4`
- `projection_matrix`: Declared as `List<F32>`, but `perspective-camera` outputs `Mat4`
- `base_color`: Declared as `List<F32>`, but `pbr-material` outputs `Vec3`

## Solution

Updated the shader preview spec in `src/bin/create_pbr_demos.rs` (lines 542-545):

**Before:**
```rust
PortSpec { name: "view_matrix".to_string(), data_type: DataType::List(Box::new(DataType::F32)), ... },
PortSpec { name: "projection_matrix".to_string(), data_type: DataType::List(Box::new(DataType::F32)), ... },
PortSpec { name: "base_color".to_string(), data_type: DataType::List(Box::new(DataType::F32)), ... },
```

**After:**
```rust
PortSpec { name: "view_matrix".to_string(), data_type: DataType::Mat4, ... },
PortSpec { name: "projection_matrix".to_string(), data_type: DataType::Mat4, ... },
PortSpec { name: "base_color".to_string(), data_type: DataType::Vec3, ... },
```

## Impact

- ✅ Graph validation now passes for all PBR demos
- ✅ Type system correctly matches component outputs to shader preview inputs
- ✅ No runtime changes needed - shader preview already handled both `List<F32>` and `Vec3`/`Mat4` via `extract_f32_list()` helper

## Files Changed

1. `src/bin/create_pbr_demos.rs` - Fixed type declarations
2. `examples/basic_pbr.wasmflow` - Regenerated with correct types
3. `examples/multi_light_pbr.wasmflow` - Regenerated with correct types
4. `examples/material_showcase.wasmflow` - Regenerated with correct types

## Testing

Build successful:
```bash
cargo build --release  # ✓ No errors
./target/release/create_pbr_demos  # ✓ All demos generated
```

## Commit

Committed and pushed to `claude/glsl-shader-nodes-01PiuQdjn1DGxaDMUvA1ZUaf`

Commit message:
> fix: Correct shader preview spec to use proper types for Vec3 and Mat4 inputs
>
> Fixed graph structure validation error by updating shader preview port specifications
