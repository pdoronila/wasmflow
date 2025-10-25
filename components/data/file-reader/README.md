# Example File Reader Component

A WasmFlow component that reads text files from the filesystem.

## Overview

This component demonstrates:
- **File system access** with capability declaration
- **Permission prompts** for user approval
- **Path validation** to enforce security boundaries
- **Multiple outputs** (content and size)
- **Error handling** for file I/O operations

## Ports

### Inputs
- **path** (String): File path to read (must be in /tmp directory)

### Outputs
- **content** (String): File contents as text
- **size** (U32): File size in bytes

## Capabilities

**Required**: `file-read:/tmp`

This component requires permission to read files from the `/tmp` directory. Users will see a permission dialog when loading this component for the first time.

## Security

- **Scoped access**: Only files in `/tmp` can be read
- **User approval**: Explicit permission required
- **Path validation**: Attempts to read outside `/tmp` are rejected with clear error messages

## Building

```bash
cd examples/example-file-reader
cargo component build --release
cp target/wasm32-wasip1/release/example_file_reader.wasm ../../components/
```

## Usage in WasmFlow

1. **Load the component**: File → Reload Components
2. **Approve permission**: When prompted, review and approve the file-read:/tmp capability
3. **Add to canvas**: Find "Read File" in the Files category
4. **Create test file**:
   ```bash
   echo "Hello from WasmFlow!" > /tmp/test.txt
   ```
5. **Connect input**: Provide path "/tmp/test.txt" to the 'path' port
6. **Execute**: Click Execute to run the graph
7. **View results**: Content and size appear on output ports

## Example Graph

```
[Constant: "/tmp/test.txt"] → [path]
                                     [Read File] → [content] → [Display]
                                                 → [size] → [Display]

Results:
  content: "Hello from WasmFlow!"
  size: 21
```

## Error Handling

The component handles several error cases:

1. **Path outside scope**: Clear error if path isn't in /tmp
2. **File not found**: Helpful message to check file existence
3. **Permission denied**: Indicates file isn't readable
4. **Invalid input**: Guides user to provide a String path

## Source Code

See `src/lib.rs` for the complete implementation.

Key features:
- Capability declaration in `get_capabilities()`
- Path validation before file access
- Proper error handling with recovery hints
- Metadata extraction (file size)
