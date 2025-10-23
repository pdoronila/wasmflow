# WasmFlow Setup Guide

Quick setup guide for getting WasmFlow running with all multi-language WASM compilation support.

## 🚀 Quick Start with Mise (Recommended)

The easiest way to get started is using [mise](https://mise.jdx.dev/), which automatically manages all dependencies:

```bash
# 1. Install mise
curl https://mise.run | sh

# 2. Activate mise in your shell
echo 'eval "$(mise activate bash)"' >> ~/.bashrc  # or ~/.zshrc for zsh
source ~/.bashrc  # or restart your terminal

# 3. Install all dependencies
cd /path/to/wasmflow_cc
mise install
mise run setup

# 4. Verify installation
mise run verify

# 5. Run WasmFlow
mise run dev
```

✅ This installs everything automatically:
- Rust 1.75+ with cargo-component
- Python 3.11+ with componentize-py
- Node.js 20+ with componentize-js
- All WASM tooling (wasm-tools, wasm32-wasip2)

📖 **Full guide:** See [docs/setup-with-mise.md](docs/setup-with-mise.md)

## 🛠️ Manual Setup

If you prefer manual installation:

### Prerequisites

1. **Rust** (1.75 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-wasip2
   cargo install cargo-component
   cargo install wasm-tools
   ```

2. **Python** (3.11 or later) - *For Python WASM components*
   ```bash
   pip install componentize-py
   ```

3. **Node.js** (20 or later) - *For JavaScript WASM components*
   ```bash
   npm install -g @bytecodealliance/componentize-js
   ```

### Build and Run

```bash
# Build the application
cargo build --release

# Run the application
cargo run
```

## 🎨 Creating WASM Components

Once WasmFlow is running, you can create components in three languages:

### Rust Components
```rust
// @description My Rust component
// @category User-Defined
// @input value:F32 Input number
// @output result:F32 Output number

let result = value * 2.0;
```

### Python Components
```python
# @description My Python component
# @category User-Defined
# @input value:F32 Input number
# @output result:F32 Output number

result = value * 2.0
```

### JavaScript Components
```javascript
// @description My JavaScript component
// @category User-Defined
// @input value:F32 Input number
// @output result:F32 Output number

const result = value * 2.0;
```

## 📋 Mise Commands

If you're using mise, here are helpful commands:

```bash
mise run dev        # Run in development mode
mise run build      # Build release version
mise run test       # Run tests
mise run verify     # Check all tools are installed
mise run clean      # Clean build artifacts
```

## 🐛 Troubleshooting

### "cargo-component not found"
```bash
mise run setup  # Reinstall all tools
# or manually:
cargo install cargo-component
```

### "componentize-py not found"
```bash
mise run install-python-tools
# or manually:
pip install componentize-py
```

### "componentize-js not found"
```bash
mise install  # Reinstall Node.js tools
# or manually:
npm install -g @bytecodealliance/componentize-js
```

### Check What's Installed
```bash
# With mise
mise list
mise run verify

# Manually
rustc --version
python --version
node --version
cargo-component --version
componentize-py --version
componentize-js --version
```

## 📚 Learn More

- **Mise Setup**: [docs/setup-with-mise.md](docs/setup-with-mise.md)
- **Multi-Language Support**: See language dropdown in WASM Creator node
- **Component Model**: https://component-model.bytecodealliance.org/

## 💡 Next Steps

1. **Open WasmFlow**: Run `mise run dev` or `cargo run`
2. **Add WASM Creator Node**: Find it in the "Development" category in the component palette
3. **Select Language**: Use the dropdown to choose Rust, Python, or JavaScript
4. **Write Code**: Use the code editor with language-specific annotations
5. **Compile**: Click "Execute (Compile Component)"
6. **Use**: Reload components (File → Reload Components) and drag your new component onto the canvas!

Happy coding! 🚀
