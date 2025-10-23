# Setting Up WasmFlow with Mise

This guide explains how to set up the WasmFlow development environment using [mise](https://mise.jdx.dev/), a fast polyglot runtime manager.

## What is Mise?

Mise (formerly rtx) automatically manages:
- Multiple language runtimes (Rust, Python, Node.js)
- Project-specific tool versions
- Development tasks and scripts
- Environment variables

## Quick Start

### 1. Install Mise

**macOS/Linux:**
```bash
curl https://mise.run | sh
```

**Or via Homebrew:**
```bash
brew install mise
```

**Add to your shell:** (choose your shell)
```bash
# For bash
echo 'eval "$(mise activate bash)"' >> ~/.bashrc

# For zsh
echo 'eval "$(mise activate zsh)"' >> ~/.zshrc

# For fish
echo 'mise activate fish | source' >> ~/.config/fish/config.fish
```

Then restart your shell or run `source ~/.bashrc` (or equivalent).

### 2. Install Project Dependencies

Navigate to the project directory and run:

```bash
cd /path/to/wasmflow_cc

# Install all tools and runtimes
mise install

# Run setup to install additional dependencies
mise run setup
```

This will automatically install:
- ✅ Rust 1.75+ with `cargo-component` and `wasm-tools`
- ✅ Python 3.11+ with `componentize-py`
- ✅ Node.js 20+ with `componentize-js`
- ✅ wasm32-wasip2 target for Rust

### 3. Verify Installation

Check that everything is installed correctly:

```bash
mise run verify
```

You should see all tools listed with their versions.

## Available Commands

Mise provides convenient task runners for common operations:

### Development

```bash
# Run WasmFlow in development mode
mise run dev

# Build in release mode
mise run build

# Run tests
mise run test
```

### Verification

```bash
# Verify all tools are installed
mise run verify

# Clean build artifacts
mise run clean
```

### Manual Tool Installation

If you need to install tools separately:

```bash
# Install Python WASM tools
mise run install-python-tools

# Install Rust WASM target
mise run install-rust-target
```

## What Gets Installed

### Core Runtimes

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75 | Main application language |
| Python | 3.11 | For Python WASM components |
| Node.js | 20 | For JavaScript WASM components |

### WASM Compilation Tools

| Tool | Installation | Purpose |
|------|-------------|---------|
| `cargo-component` | Auto (cargo) | Compiles Rust → WASM |
| `componentize-py` | Auto (pip) | Compiles Python → WASM |
| `componentize-js` | Auto (npm) | Compiles JavaScript → WASM |
| `wasm-tools` | Auto (cargo) | WASM manipulation utilities |
| `wasm32-wasip2` | Setup task | Rust WASM compilation target |

## How It Works

### Automatic Version Management

When you `cd` into the project directory, mise automatically:
1. Detects the `.mise.toml` configuration
2. Installs missing runtimes/tools (if configured to do so)
3. Sets up the correct versions in your PATH
4. Activates project-specific environment variables

### Manual vs Automatic Installation

By default, mise will prompt before installing new tools. You can change this behavior:

```bash
# Always install automatically
mise settings set experimental true
mise settings set always_install true
```

## Troubleshooting

### Tools Not Found After Installation

If `cargo-component`, `componentize-py`, or other tools aren't found:

```bash
# Ensure mise is activated in your shell
mise activate

# Verify PATH includes tool directories
echo $PATH | grep -E 'cargo|python|node'

# Reinstall tools
mise run setup
```

### Python Tools Not Installing

```bash
# Manually install with mise's Python
mise exec python -- -m pip install componentize-py

# Or run the setup task again
mise run install-python-tools
```

### Rust Target Missing

```bash
# Manually add wasm32-wasip2 target
rustup target add wasm32-wasip2

# Or run the setup task
mise run install-rust-target
```

### Check Mise Status

```bash
# Show installed tools
mise list

# Show current configuration
mise current

# Show available tasks
mise tasks
```

## Alternative: Manual Installation

If you prefer not to use mise, you can manually install:

1. **Rust** (1.75+): https://rustup.rs/
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-wasip2
   cargo install cargo-component
   cargo install wasm-tools
   ```

2. **Python** (3.11+): https://www.python.org/downloads/
   ```bash
   pip install componentize-py
   ```

3. **Node.js** (20+): https://nodejs.org/
   ```bash
   npm install -g @bytecodealliance/componentize-js
   ```

## Learn More

- Mise Documentation: https://mise.jdx.dev/
- WasmFlow Documentation: See `README.md`
- WASM Component Model: https://component-model.bytecodealliance.org/
