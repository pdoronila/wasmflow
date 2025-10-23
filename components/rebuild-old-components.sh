#!/bin/bash
# Rebuild all old components for wasmflow:node@1.1.0
# Run this script from the components/ directory after network access is restored

set -e  # Exit on error

echo "========================================"
echo "Rebuilding Old Components for v1.1.0"
echo "========================================"
echo ""

OLD_COMPONENTS=(
  "echo"
  "adder"
  "double-number"
  "http-fetch"
  "json-parser"
  "file-reader"
  "footer-view"
)

FAILED=()
SUCCESS=()

for component in "${OLD_COMPONENTS[@]}"; do
  echo "----------------------------------------"
  echo "Building: $component"
  echo "----------------------------------------"

  cd "$component"

  if cargo build --target wasm32-wasip2 --release; then
    # Determine the component binary name
    if [ "$component" = "adder" ]; then
      binary_name="example_adder"
    else
      binary_name=$(echo "$component" | tr '-' '_')
    fi

    # Copy to bin directory
    cp "target/wasm32-wasip2/release/${binary_name}.wasm" "../bin/${binary_name}.wasm"
    echo "✓ $component built and installed"
    SUCCESS+=("$component")
  else
    echo "✗ $component failed to build"
    FAILED+=("$component")
  fi

  cd ..
  echo ""
done

echo "========================================"
echo "Build Summary"
echo "========================================"
echo "Success: ${#SUCCESS[@]}/${#OLD_COMPONENTS[@]}"
echo "Failed:  ${#FAILED[@]}/${#OLD_COMPONENTS[@]}"

if [ ${#SUCCESS[@]} -gt 0 ]; then
  echo ""
  echo "Successfully built:"
  for comp in "${SUCCESS[@]}"; do
    echo "  ✓ $comp"
  done
fi

if [ ${#FAILED[@]} -gt 0 ]; then
  echo ""
  echo "Failed to build:"
  for comp in "${FAILED[@]}"; do
    echo "  ✗ $comp"
  done
  exit 1
fi

echo ""
echo "All components rebuilt successfully!"
echo "Run 'cargo run' to test them in wasmflow."
