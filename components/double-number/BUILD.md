# Building the Double Number Component

Quick reference for building this example component.

## Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-wasip2

# Install cargo-component
cargo install cargo-component
```

## Build Commands

### Development Build

Fast builds with debug symbols (for testing):

```bash
cargo component build --target wasm32-wasip2
```

Output: `target/wasm32-wasip2/debug/double_number.wasm`

### Release Build

Optimized build for production use:

```bash
cargo component build --target wasm32-wasip2 --release
```

Output: `target/wasm32-wasip2/release/double_number.wasm`

**Note**: The `--target wasm32-wasip2` flag is required to use WASI Preview 2. The `.cargo/config.toml` file sets this as the default target.

### Clean Build

Remove all build artifacts:

```bash
cargo clean
cargo component build --release
```

## Installing the Component

### Option 1: Copy to Components Directory

```bash
# From the double-number directory
cp target/wasm32-wasip2/release/double_number.wasm \
   ../../components/
```

### Option 2: Symlink (for development)

```bash
# From the double-number directory
ln -s $(pwd)/target/wasm32-wasip2/release/double_number.wasm \
      ../../components/double_number.wasm
```

**Note**: Symlinks allow you to rebuild without copying each time.

## Loading in WasmFlow

After building and copying:

1. **Start WasmFlow** (or if already running):
   ```bash
   cd ../..
   cargo run --release
   ```

2. **Reload components**:
   - In WasmFlow: **File → Reload Components**
   - Or restart the application

3. **Find the component**:
   - Look in the palette under "Math" category
   - Component name: "Double"

## Verification

### Check Build Output

```bash
# Verify the file exists
ls -lh target/wasm32-wasip2/release/double_number.wasm

# Should show size around 40-50KB
# Example output:
# -rw-r--r--  1 user  staff   48K Oct 13 17:21 double_number.wasm
```

### Inspect with wasmtime

```bash
# Check if component is valid
wasmtime --version
wasmtime info target/wasm32-wasip2/release/double_number.wasm

# Or run it directly (if it has a main function)
wasmtime run target/wasm32-wasip2/release/double_number.wasm
```

### Test in WasmFlow

Create a simple test graph:

```
1. Add Constant node → Set value to 7.0
2. Add Double node (your component)
3. Connect: Constant.value → Double.input
4. Execute graph (▶ Execute button)
5. Verify: Double.output shows 14.0
```

## Build Optimization

### Minimize Size

The default `Cargo.toml` already includes optimizations:

```toml
[profile.release]
opt-level = "s"  # Optimize for size
lto = true       # Link-time optimization
strip = true     # Strip debug symbols
```

### Further Optimization

For even smaller binaries:

```toml
[profile.release]
opt-level = "z"      # Aggressive size optimization
lto = true
strip = true
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller panic handler
```

Rebuild and check size:

```bash
cargo component build --release
ls -lh target/wasm32-wasip2/release/double_number.wasm
```

## Troubleshooting

### Build Fails: "target not found"

**Error**: `error: target 'wasm32-wasip2' not found`

**Solution**:
```bash
rustup target add wasm32-wasip2
```

### Build Fails: "cargo-component not found"

**Error**: `error: no such subcommand: 'component'`

**Solution**:
```bash
cargo install cargo-component
```

### Build Succeeds but File Not Found

**Error**: `.wasm` file doesn't exist in `target/` directory

**Solution**: Make sure to specify the wasip2 target:
```bash
cargo component build --target wasm32-wasip2 --release
```

Check for build errors:
```bash
cargo component build --target wasm32-wasip2 --release 2>&1 | grep -i error
```

### Component Doesn't Load in WasmFlow

**Check**:
1. File copied to correct location:
   ```bash
   ls ../../components/double_number.wasm
   ```

2. File has `.wasm` extension (exactly)

3. File size is reasonable (<50MB):
   ```bash
   du -h ../../components/double_number.wasm
   ```

4. Reload components in WasmFlow

### Large Binary Size

**If component is >10MB**:

1. Check dependencies in `Cargo.toml`
2. Remove unused dependencies
3. Enable all optimizations (see above)
4. Consider splitting into multiple components

### Linking Errors

**Error**: `error: linking with 'rust-lld' failed`

**Solution**:
```bash
# Update Rust toolchain
rustup update stable

# Clean and rebuild
cargo clean
cargo component build --release
```

## Development Workflow

Efficient workflow for iterating on the component:

1. **Make changes** to `src/lib.rs`

2. **Quick test** (unit tests):
   ```bash
   cargo test
   ```

3. **Build**:
   ```bash
   cargo component build --target wasm32-wasip2 --release
   ```

4. **Copy to components** (if not symlinked):
   ```bash
   cp target/wasm32-wasip2/release/double_number.wasm ../../components/
   ```

5. **Reload in WasmFlow**:
   - File → Reload Components

6. **Test in graph**

7. **Repeat** from step 1

## Continuous Integration

For automated builds:

```yaml
# .github/workflows/build.yml
name: Build Component

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-wasip2
      - name: Install cargo-component
        run: cargo install cargo-component
      - name: Build
        run: cargo component build --release
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: double-number
          path: target/wasm32-wasip2/release/double_number.wasm
```

## Next Steps

- **Modify the component**: Change the multiplier from 2 to another value
- **Add more inputs**: Let users specify the multiplier
- **Add validation**: Check for overflow or invalid inputs
- **Create new component**: Try building a different mathematical operation

See [README.md](README.md) for implementation details and examples.
