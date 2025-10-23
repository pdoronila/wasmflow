#!/usr/bin/env bash
#
# WasmFlow Component Scaffolding Script
# Creates a new component with all required files from templates

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
error() {
    echo -e "${RED}Error:${NC} $1" >&2
    exit 1
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

warn() {
    echo -e "${YELLOW}!${NC} $1"
}

# Validate component directory path
if [ -z "$1" ]; then
    error "Usage: $0 <component-name> [output-directory]"
fi

COMPONENT_NAME="$1"
OUTPUT_DIR="${2:-.}"

# Validate component name
if [[ ! "$COMPONENT_NAME" =~ ^[a-z][a-z0-9-]*$ ]]; then
    error "Component name must start with lowercase letter and contain only lowercase letters, numbers, and hyphens"
fi

COMPONENT_DIR="$OUTPUT_DIR/$COMPONENT_NAME"

# Check if directory already exists
if [ -d "$COMPONENT_DIR" ]; then
    warn "Directory '$COMPONENT_DIR' already exists"
    read -p "Overwrite? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        error "Aborted"
    fi
    rm -rf "$COMPONENT_DIR"
fi

info "Creating component '$COMPONENT_NAME' in '$OUTPUT_DIR'"
echo

# Interactive prompts
read -p "Display name (e.g., 'Add Numbers'): " DISPLAY_NAME
read -p "Version [1.0.0]: " VERSION
VERSION="${VERSION:-1.0.0}"
read -p "Description: " DESCRIPTION
read -p "Author: " AUTHOR
read -p "Category (Math/Text/Data/Network/File I/O/Utility/Examples) [Utility]: " CATEGORY
CATEGORY="${CATEGORY:-Utility}"

# Component type selection
echo
echo "Component type:"
echo "  1) Basic (pure computation)"
echo "  2) UI (with custom footer view)"
echo "  3) Network (HTTP/WASI)"
read -p "Select type [1]: " COMPONENT_TYPE
COMPONENT_TYPE="${COMPONENT_TYPE:-1}"

# Create directory structure
mkdir -p "$COMPONENT_DIR/src"
mkdir -p "$COMPONENT_DIR/wit"

# Convert component name to various formats
COMPONENT_SNAKE=$(echo "$COMPONENT_NAME" | tr '-' '_')
PACKAGE_NAME="example-$COMPONENT_NAME"

# Create Cargo.toml based on type
if [ "$COMPONENT_TYPE" = "2" ] || [ "$COMPONENT_TYPE" = "3" ]; then
    # UI or Network component
    cat > "$COMPONENT_DIR/Cargo.toml" <<EOF
[package]
name = "$PACKAGE_NAME"
version = "$VERSION"
edition = "2021"
authors = ["$AUTHOR"]

[workspace]
# This is a standalone crate, not part of the parent workspace

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }

[build-dependencies]
wit-component = "0.215"

[profile.release]
opt-level = "s"  # Optimize for size
lto = true
strip = true
EOF
else
    # Basic component
    cat > "$COMPONENT_DIR/Cargo.toml" <<EOF
[package]
name = "$PACKAGE_NAME"
version = "$VERSION"
edition = "2021"
authors = ["$AUTHOR"]

[workspace]
# This is a standalone crate, not part of the parent workspace

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.30"

[build-dependencies]
wit-component = "0.215"

[profile.release]
opt-level = "s"  # Optimize for size
lto = true
strip = true
EOF
fi

success "Created Cargo.toml"

# Create build.rs
cat > "$COMPONENT_DIR/build.rs" <<'EOF'
fn main() {
    println!("cargo:rerun-if-changed=wit");
}
EOF

success "Created build.rs"

# Create Justfile
cat > "$COMPONENT_DIR/Justfile" <<'EOF'
# Generic WasmFlow Component Builder
# Auto-detects component name from directory

# Auto-detect component name from directory
component_name := replace(file_name(justfile_directory()), "-", "_")
component_display := file_name(justfile_directory())

# Build the component (default)
build: clean
    @echo "Building {{component_display}} component..."
    cargo build --target wasm32-wasip2 --release
    @echo "✓ Component built: target/wasm32-wasip2/release/{{component_name}}.wasm"

# Install the component to WasmFlow components directory
install: build
    @echo "Installing component to ../bin/"
    mkdir -p ../bin
    cp target/wasm32-wasip2/release/{{component_name}}.wasm \
       ../bin/{{component_name}}.wasm
    @echo "✓ Component installed"

# Clean build artifacts
clean:
    cargo clean

# Run tests
test:
    cargo test

# Check without building
check:
    cargo check --target wasm32-wasip2

# Install prerequisites
setup:
    @echo "Installing prerequisites..."
    rustup target add wasm32-wasip2
    @echo "✓ Prerequisites installed"

# Show component information
info:
    @echo "Component name: {{component_name}}"
    @echo "Display name: {{component_display}}"
    @echo "Directory: {{justfile_directory()}}"
EOF

success "Created Justfile"

# Create src/lib.rs based on type
if [ "$COMPONENT_TYPE" = "2" ]; then
    # UI component
    cat > "$COMPONENT_DIR/src/lib.rs" <<EOF
//! $DISPLAY_NAME Component
//!
//! $DESCRIPTION

wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
    ui::{ColoredText, FooterView, Guest as UiGuest, HorizontalLayout,
         KeyValuePair, UiElement, UiElementItem},
};
use wasmflow::node::types::DataType;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "$DISPLAY_NAME".to_string(),
            version: "$VERSION".to_string(),
            description: "$DESCRIPTION".to_string(),
            author: "$AUTHOR".to_string(),
            category: Some("$CATEGORY".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Input value".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Output value".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing $DISPLAY_NAME");

        // Extract input value
        let input = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect an F32 value to the input port".to_string()),
            })?;

        // TODO: Implement computation logic here
        let output = input;

        // Return outputs
        Ok(vec![
            ("output".to_string(), Value::F32Val(output)),
        ])
    }
}

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        let mut elements = Vec::new();

        // Header
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "📊 $DISPLAY_NAME".to_string(),
            r: 100, g: 200, b: 255,
        }));

        elements.push(UiElement::Separator);

        // Display outputs
        for (name, value) in outputs {
            let value_str = match value {
                Value::F32Val(v) => format!("{:.2}", v),
                Value::I32Val(v) => format!("{}", v),
                Value::U32Val(v) => format!("{}", v),
                Value::StringVal(s) => s,
                Value::BinaryVal(_) => "<binary data>".to_string(),
            };

            elements.push(UiElement::KeyValue(KeyValuePair {
                key: name,
                value: value_str,
            }));
        }

        // Status indicator
        elements.push(UiElement::Separator);
        elements.push(UiElement::Horizontal(HorizontalLayout {
            elements: vec![
                UiElementItem::Label("Status:".to_string()),
                UiElementItem::ColoredLabel(ColoredText {
                    text: "✓ Ready".to_string(),
                    r: 100, g: 255, b: 150,
                }),
            ],
        }));

        Some(FooterView { elements })
    }
}

export!(Component);
EOF
elif [ "$COMPONENT_TYPE" = "3" ]; then
    # Network component
    cat > "$COMPONENT_DIR/src/lib.rs" <<EOF
//! $DISPLAY_NAME Component
//!
//! $DESCRIPTION

wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
    with: {
        "wasi:io/error@0.2.0": generate,
        "wasi:io/poll@0.2.0": generate,
        "wasi:io/streams@0.2.0": generate,
        "wasi:http/types@0.2.0": generate,
        "wasi:http/outgoing-handler@0.2.0": generate,
    },
});

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
};
use wasmflow::node::types::DataType;
use wasmflow::node::host;

use wasi::http::types::{Fields, Method, OutgoingRequest, Scheme};
use wasi::http::outgoing_handler;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "$DISPLAY_NAME".to_string(),
            version: "$VERSION".to_string(),
            description: "$DESCRIPTION".to_string(),
            author: "$AUTHOR".to_string(),
            category: Some("$CATEGORY".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "url".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "URL to fetch".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "data".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Fetched data".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        // TODO: Add required network hosts
        Some(vec![
            "network:api.example.com".to_string(),
        ])
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing $DISPLAY_NAME");

        // Extract URL
        let url = inputs
            .iter()
            .find(|(name, _)| name == "url")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'url' input".to_string(),
                input_name: Some("url".to_string()),
                recovery_hint: Some("Provide a valid URL string".to_string()),
            })?;

        // TODO: Implement HTTP request logic here

        let data = "Response data".to_string();

        Ok(vec![
            ("data".to_string(), Value::StringVal(data)),
        ])
    }
}

export!(Component);
EOF
else
    # Basic component
    cat > "$COMPONENT_DIR/src/lib.rs" <<EOF
//! $DISPLAY_NAME Component
//!
//! $DESCRIPTION

wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "$DISPLAY_NAME".to_string(),
            version: "$VERSION".to_string(),
            description: "$DESCRIPTION".to_string(),
            author: "$AUTHOR".to_string(),
            category: Some("$CATEGORY".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Input value".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Output value".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing $DISPLAY_NAME");

        // Extract input value
        let input = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect an F32 value to the input port".to_string()),
            })?;

        // TODO: Implement computation logic here
        let output = input;

        // Return output
        Ok(vec![("output".to_string(), Value::F32Val(output))])
    }
}

export!(Component);
EOF
fi

success "Created src/lib.rs"

# Copy WIT file if in components directory
if [ -f "../wit/node.wit" ]; then
    cp "../wit/node.wit" "$COMPONENT_DIR/wit/"
    success "Copied wit/node.wit from parent directory"
elif [ -f "../../wit/node.wit" ]; then
    cp "../../wit/node.wit" "$COMPONENT_DIR/wit/"
    success "Copied wit/node.wit from grandparent directory"
else
    warn "wit/node.wit not found - you'll need to copy it manually"
    info "Copy from wasmflow/wit/node.wit to $COMPONENT_DIR/wit/"
fi

# Summary
echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Component '$COMPONENT_NAME' created!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
info "Location: $COMPONENT_DIR"
info "Display name: $DISPLAY_NAME"
info "Category: $CATEGORY"
echo
echo "Next steps:"
echo "  1. cd $COMPONENT_DIR"
echo "  2. Implement computation logic in src/lib.rs (search for TODO)"
echo "  3. just build"
echo "  4. just install"
echo "  5. Reload components in wasmflow"
echo
success "Happy coding!"
