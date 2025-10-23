# JSON Parser Component

A WasmFlow component that extracts values from JSON strings using key path notation.

## Features

- **Dot notation**: Extract nested properties (e.g., `metadata.author`)
- **Bracket notation**: Access array elements (e.g., `runs[1]`)
- **Combined paths**: Mix both notations (e.g., `runs[1].time`)
- **Type preservation**: Outputs values as strings while preserving the original type

## Inputs

- **json_string** (String): JSON string to parse
- **key_path** (String): Key path using dot and/or bracket notation

## Output

- **value** (String): Extracted value as a string representation

## Examples

```
Input: {"version": 1}
Path: "version"
Output: "1"

Input: {"metadata": {"author": "Alice"}}
Path: "metadata.author"
Output: "Alice"

Input: {"values": [10, 20, 30]}
Path: "values[1]"
Output: "20"

Input: {"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}
Path: "runs[1].time"
Output: "1000"
```

## Building

```bash
cargo build --target wasm32-wasip2 --release
```

The compiled component will be at:
`target/wasm32-wasip2/release/example_json_parser.wasm`

## Testing

```bash
cargo test
```

## Error Handling

The component provides detailed error messages for:
- Invalid JSON syntax
- Missing keys in objects
- Out-of-bounds array access
- Invalid key path syntax
- Type mismatches (e.g., indexing into non-arrays)
