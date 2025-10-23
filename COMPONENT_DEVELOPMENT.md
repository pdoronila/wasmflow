# Component Development - Complete Guide

**Complete documentation for building, testing, and deploying custom WasmFlow components.**

## 📚 Documentation Overview

This project includes comprehensive guides for developing custom WebAssembly components:

### **Main Documentation**

1. **[Building Components Guide](docs/BUILDING_COMPONENTS.md)** ⭐ **START HERE**
   - Complete tutorial from setup to deployment
   - WIT interface specification
   - Implementation patterns and best practices
   - Troubleshooting guide
   - Security and capabilities
   - ~500 lines of detailed documentation

2. **[Components Directory README](components/README.md)**
   - Installation and loading guide
   - Validation requirements
   - Security and permissions
   - Troubleshooting tips
   - Quick reference for users

3. **[Main README](README.md)**
   - Updated with component development section
   - Quick start examples
   - Project overview with component features

### **Example Component: Double Number**

Complete, working example in `examples/double-number/`:

1. **[README](examples/double-number/README.md)**
   - Overview and usage
   - Implementation details
   - Testing guide
   - Extension examples

2. **[BUILD.md](examples/double-number/BUILD.md)**
   - Step-by-step build instructions
   - Installation options
   - Verification steps
   - Optimization guide
   - Development workflow

3. **[Source Code](examples/double-number/src/lib.rs)**
   - Complete working implementation
   - Inline documentation
   - Error handling examples
   - Host function usage

4. **[WIT Interface](examples/double-number/wit/world.wit)**
   - Interface definition
   - Type specifications
   - Import/export declarations

5. **[Cargo Configuration](examples/double-number/Cargo.toml)**
   - Proper crate type setup
   - Dependencies
   - Optimization flags

## 🚀 Quick Start for Component Developers

### 1. Read the Guide

Start with [Building Components Guide](docs/BUILDING_COMPONENTS.md) for complete understanding.

### 2. Study the Example

```bash
cd examples/double-number
cat README.md          # Understand the component
cat src/lib.rs         # See implementation
cat wit/world.wit      # Review interface
```

### 3. Build the Example

```bash
# Install prerequisites
rustup target add wasm32-wasip2
cargo install cargo-component

# Build
cargo component build --release

# Verify
ls -lh target/wasm32-wasip2/release/double_number.wasm
```

### 4. Test in WasmFlow

```bash
# Copy to components directory
cp target/wasm32-wasip2/release/double_number.wasm ../../components/

# Run WasmFlow
cd ../..
cargo run --release

# In WasmFlow: File → Reload Components
# Find "Double" in Math category
# Test: Constant(7) → Double → Should output 14.0
```

### 5. Create Your Own

```bash
# Create new component from scratch
cargo component new my-component --lib

# Or copy and modify the example
cp -r examples/double-number examples/my-component
cd examples/my-component

# Modify src/lib.rs, wit/world.wit, Cargo.toml
# Build and test following the same process
```

## 📖 Documentation Structure

```
wasmflow/
├── README.md                          # Main readme with component section
├── COMPONENT_DEVELOPMENT.md           # This file - documentation index
│
├── docs/
│   └── BUILDING_COMPONENTS.md         # ⭐ Complete tutorial
│
├── components/
│   └── README.md                      # Installation and usage guide
│
└── examples/
    └── double-number/
        ├── README.md                  # Component overview
        ├── BUILD.md                   # Build instructions
        ├── Cargo.toml                 # Package configuration
        ├── src/
        │   └── lib.rs                 # Implementation
        └── wit/
            └── world.wit              # Interface definition
```

## 🎯 Learning Path

### **Beginner** (Just want to build a simple component)

1. Read: `examples/double-number/README.md`
2. Study: `examples/double-number/src/lib.rs`
3. Follow: `examples/double-number/BUILD.md`
4. Modify the example to create your own

### **Intermediate** (Understand the full system)

1. Read: `docs/BUILDING_COMPONENTS.md` (complete guide)
2. Study: `examples/double-number/wit/world.wit`
3. Review: `specs/001-webassembly-based-node/contracts/node-interface.wit`
4. Build complex components with multiple inputs/outputs

### **Advanced** (Deep integration)

1. Review: `src/runtime/wasm_host.rs` (host implementation)
2. Study: `src/runtime/capabilities.rs` (security system)
3. Read: Component Model specification (external)
4. Build components with system access and capabilities

## 🔧 Common Tasks

### Build a Component

```bash
cd examples/double-number
cargo component build --release
```

### Install a Component

```bash
cp target/wasm32-wasip2/release/my_component.wasm ../../components/
```

### Reload Components in WasmFlow

In WasmFlow: **File → Reload Components**

### Test a Component

Create test graph: `Constant → YourComponent → Verify Output`

### Debug a Component

```bash
RUST_LOG=debug cargo run
# Or add logging in component:
host::log("info", "Debug message");
```

## 📋 Checklists

### Before Building

- [ ] Rust toolchain installed (1.75+)
- [ ] WASM target added: `rustup target add wasm32-wasip2`
- [ ] cargo-component installed: `cargo install cargo-component`
- [ ] Read relevant documentation

### Component Checklist

- [ ] `Cargo.toml` has `crate-type = ["cdylib"]`
- [ ] WIT interface defined in `wit/world.wit`
- [ ] `get_info()` returns component metadata
- [ ] `get_inputs()` specifies input ports
- [ ] `get_outputs()` specifies output ports
- [ ] `get_capabilities()` declares system access (if needed)
- [ ] `execute()` implements computation logic
- [ ] Error handling provides helpful messages
- [ ] Unit tests cover edge cases
- [ ] Component builds without errors
- [ ] File size is reasonable (<50MB)

### Testing Checklist

- [ ] Unit tests pass: `cargo test`
- [ ] Component builds: `cargo component build --release`
- [ ] File exists: `ls target/wasm32-wasip2/release/*.wasm`
- [ ] Loads in WasmFlow without errors
- [ ] Appears in correct category in palette
- [ ] Executes correctly with test inputs
- [ ] Error handling works for invalid inputs
- [ ] Performance is acceptable (<30s timeout)

## 🎓 Educational Resources

### Internal Documentation

- [Building Components Guide](docs/BUILDING_COMPONENTS.md) - Complete tutorial
- [Example Component](examples/double-number/README.md) - Working reference
- [Components README](components/README.md) - User guide
- [Quickstart Guide](specs/001-webassembly-based-node/quickstart.md) - WasmFlow development
- [WIT Interface](specs/001-webassembly-based-node/contracts/node-interface.wit) - Contract specification

### External Resources

- [Component Model Docs](https://component-model.bytecodealliance.org/)
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
- [Wasmtime Guide](https://docs.wasmtime.dev/)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)

## 🆘 Getting Help

### Quick Troubleshooting

1. **Component won't build**: See `docs/BUILDING_COMPONENTS.md#troubleshooting`
2. **Component won't load**: See `components/README.md#troubleshooting`
3. **Type errors**: Check port data types match in connections
4. **Performance issues**: Review optimization flags in `examples/double-number/BUILD.md`

### Detailed Help

- Review the comprehensive troubleshooting sections in each guide
- Check example component for reference implementation
- Enable debug logging: `RUST_LOG=debug cargo run`
- File an issue with detailed error messages

## ✨ Next Steps

1. **Start Simple**: Build the double-number example
2. **Experiment**: Modify it to do different operations
3. **Build Your Own**: Create a component for your specific use case
4. **Share**: Contribute examples back to the community

## 📦 What's Included

### Complete Example Component

- ✅ Full Rust source code with documentation
- ✅ WIT interface definition
- ✅ Cargo configuration with optimizations
- ✅ Build instructions (multiple methods)
- ✅ Testing guide
- ✅ Integration examples

### Comprehensive Documentation

- ✅ 500+ lines of tutorial content
- ✅ Step-by-step build instructions
- ✅ Troubleshooting guides
- ✅ Best practices and patterns
- ✅ Security and capabilities guide
- ✅ Performance optimization tips

### Developer Tools

- ✅ Example component ready to build
- ✅ Pre-configured Cargo.toml
- ✅ WIT interface template
- ✅ Error handling patterns
- ✅ Host function examples

## 🎉 Success Criteria

You'll know you're successful when you can:

1. ✅ Build the example component without errors
2. ✅ Load it into WasmFlow and see it in the palette
3. ✅ Create a graph using your component
4. ✅ Execute the graph and get correct results
5. ✅ Modify the component and reload it
6. ✅ Create your own component from scratch

---

**Ready to build?** Start with the [Building Components Guide](docs/BUILDING_COMPONENTS.md)! 🚀
