# Foundational Components Implementation Plan

**Status**: 🚧 In Progress
**Goal**: Build 9 atomic, composable components for advanced data processing workflows
**Primary Use Case**: Kernel message filtering engine (JSONL → extract paths → filter → regex match → count)

---

## Quick Reference

| Phase | Category | Components | Status |
|-------|----------|------------|--------|
| 1 | Core | 3 | ⬜ Not Started |
| 2 | Filtering | 2 | ⬜ Not Started |
| 3 | Analysis | 2 | ⬜ Not Started |
| 4 | Advanced | 2 | ⬜ Not Started |
| **Total** | | **9** | **0/9 Complete** |

---

## Phase 1: Core Components (Essential)

### ✅ Component 1: `regex-match`

**Category**: Text / Regex
**Location**: `components/text/regex-match/`
**Status**: ⬜ Not Started

**Description**: Test a single string against a single regex pattern.

**Specification**:
```rust
Component Info:
  name: "Regex Match"
  version: "1.0.0"
  category: "Text"
  author: "WasmFlow Core Library"

Inputs:
  - text: string (required) - "Text to test against pattern"
  - pattern: string (required) - "Regular expression pattern"

Outputs:
  - matches: bool - "True if text matches pattern"
  - error: string (optional) - "Error message if pattern is invalid"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Valid pattern match (text: "hello.rs", pattern: ".*\\.rs$" → true)
- ✅ Valid pattern no match (text: "hello.txt", pattern: ".*\\.rs$" → false)
- ✅ Invalid regex pattern (pattern: "[invalid(" → error)
- ✅ Empty string match (text: "", pattern: "^$" → true)
- ✅ Unicode text (text: "test_файл.rs", pattern: ".*\\.rs$" → true)

**Build Commands**:
```bash
cd components/text/regex-match
just test
just build
just install
```

**Composition Use Cases**:
- Validate single file path
- Conditional logic in workflows
- Input validation nodes

---

### ✅ Component 2: `list-filter-empty`

**Category**: Collections
**Location**: `components/collections/list-filter-empty/`
**Status**: ⬜ Not Started

**Description**: Remove empty strings and whitespace-only strings from a list.

**Specification**:
```rust
Component Info:
  name: "List Filter Empty"
  version: "1.0.0"
  category: "Collections"
  author: "WasmFlow Core Library"

Inputs:
  - list: StringListVal (required) - "List to filter"

Outputs:
  - filtered: StringListVal - "List with empty/whitespace items removed"
  - removed_count: u32 - "Number of items removed"

Dependencies: None (standard library only)
```

**Behavior**:
- Removes: empty strings (`""`), whitespace-only (`" "`, `"\t"`, `"\n"`)
- Keeps: any string with non-whitespace content
- Uses: `s.trim().is_empty()` for detection

**Test Cases**:
- ✅ All valid items (["a", "b"] → ["a", "b"], removed: 0)
- ✅ Some empty items (["a", "", "b", " "] → ["a", "b"], removed: 2)
- ✅ All empty items (["", " ", "\t"] → [], removed: 3)
- ✅ Empty input list ([] → [], removed: 0)
- ✅ Whitespace variations (["a", "  ", "\n\t", "b"] → ["a", "b"], removed: 2)

**Build Commands**:
```bash
cd components/collections/list-filter-empty
just test
just build
just install
```

**Composition Use Cases**:
- Clean extracted JSONL fields
- Remove blank lines from text
- Data validation pipelines

---

### ✅ Component 3: `json-extract-each`

**Category**: Data
**Location**: `components/data/json-extract-each/`
**Status**: ⬜ Not Started

**Description**: Extract a field from each JSON string in a list (JSONL batch processing).

**Specification**:
```rust
Component Info:
  name: "JSON Extract Each"
  version: "1.0.0"
  category: "Data"
  author: "WasmFlow Core Library"

Inputs:
  - json_strings: StringListVal (required) - "List of JSON strings to parse"
  - field_path: string (required) - "Key path to extract (e.g., 'path', 'event.file', 'data[0]')"

Outputs:
  - values: StringListVal - "Extracted values (skips failed parses)"
  - error_count: u32 - "Number of items that failed to parse"
  - success_count: u32 - "Number of successful extractions"

Dependencies:
  - serde_json = "1.0"
```

**Behavior**:
- Parses each JSON string individually
- Extracts field using same logic as existing `json-parser` component
- Skips items that fail to parse (doesn't abort entire operation)
- Returns only successfully extracted values
- Supports dot notation (`metadata.author`) and bracket notation (`runs[1].time`)

**Test Cases**:
- ✅ All valid JSON (3 objects, field exists → 3 values, 0 errors)
- ✅ Some invalid JSON (3 objects, 1 malformed → 2 values, 1 error)
- ✅ Field missing in some (3 objects, field in 2 → 2 values, 1 error)
- ✅ Nested field extraction (field: "event.path" → extracts nested value)
- ✅ Array index extraction (field: "files[0]" → extracts array element)
- ✅ Empty input list ([] → [], 0 errors)

**Build Commands**:
```bash
cd components/data/json-extract-each
just test
just build
just install
```

**Composition Use Cases**:
- JSONL log file processing
- Kernel message parsing (primary use case)
- Batch API response extraction

---

## Phase 2: Filtering Components

### ✅ Component 4: `list-filter-regex`

**Category**: Collections
**Location**: `components/collections/list-filter-regex/`
**Status**: ⬜ Not Started

**Description**: Keep only list items matching a regex pattern.

**Specification**:
```rust
Component Info:
  name: "List Filter Regex"
  version: "1.0.0"
  category: "Collections"
  author: "WasmFlow Core Library"

Inputs:
  - list: StringListVal (required) - "List to filter"
  - pattern: string (required) - "Regular expression pattern"

Outputs:
  - matched: StringListVal - "Items that matched the pattern"
  - removed_count: u32 - "Number of items removed"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Some matches (["a.rs", "b.txt", "c.rs"] + pattern ".*\\.rs$" → ["a.rs", "c.rs"], removed: 1)
- ✅ No matches (["a.txt", "b.md"] + pattern ".*\\.rs$" → [], removed: 2)
- ✅ All match (["a.rs", "b.rs"] + pattern ".*\\.rs$" → ["a.rs", "b.rs"], removed: 0)
- ✅ Empty list ([] → [], removed: 0)
- ✅ Invalid pattern (pattern: "[invalid(" → error)

**Composition Use Cases**:
- Filter files by extension
- Select log lines by pattern
- Allowlist filtering

---

### ✅ Component 5: `list-filter-regex-any`

**Category**: Collections
**Location**: `components/collections/list-filter-regex-any/`
**Status**: ⬜ Not Started

**Description**: Keep items matching ANY of multiple regex patterns (OR logic).

**Specification**:
```rust
Component Info:
  name: "List Filter Regex Any"
  version: "1.0.0"
  category: "Collections"
  author: "WasmFlow Core Library"

Inputs:
  - list: StringListVal (required) - "List to filter"
  - patterns: StringListVal (required) - "Regular expression patterns (OR logic)"

Outputs:
  - matched: StringListVal - "Items that matched at least one pattern"
  - removed_count: u32 - "Number of items removed"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Multiple patterns, some match (["a.rs", "b.txt", "c.md"] + patterns [".*\\.rs$", ".*\\.toml$"] → ["a.rs"], removed: 2)
- ✅ Item matches multiple patterns (["a.rs"] + patterns [".*\\.rs$", "a.*"] → ["a.rs"], removed: 0)
- ✅ No patterns provided (list + [] → error: "At least one pattern required")
- ✅ All patterns match different items (["a.rs", "b.toml"] + [".*\\.rs$", ".*\\.toml$"] → ["a.rs", "b.toml"], removed: 0)
- ✅ One invalid pattern (patterns: [".*\\.rs$", "[invalid("] → error)

**Composition Use Cases**:
- Multi-extension file filtering
- Complex allowlists
- Log pattern matching

---

## Phase 3: Analysis Components

### ✅ Component 6: `list-count-regex`

**Category**: Collections
**Location**: `components/collections/list-count-regex/`
**Status**: ⬜ Not Started

**Description**: Count how many list items match a regex pattern.

**Specification**:
```rust
Component Info:
  name: "List Count Regex"
  version: "1.0.0"
  category: "Collections"
  author: "WasmFlow Core Library"

Inputs:
  - list: StringListVal (required) - "List to analyze"
  - pattern: string (required) - "Regular expression pattern"

Outputs:
  - count: u32 - "Number of items matching pattern"
  - percentage: f32 - "Percentage of items matching (count/total * 100)"
  - total: u32 - "Total items in input list"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Some matches (["a.rs", "b.txt", "c.rs"] + ".*\\.rs$" → count: 2, percentage: 66.67, total: 3)
- ✅ No matches (["a.txt"] + ".*\\.rs$" → count: 0, percentage: 0.0, total: 1)
- ✅ All match (["a.rs", "b.rs"] + ".*\\.rs$" → count: 2, percentage: 100.0, total: 2)
- ✅ Empty list ([] → count: 0, percentage: 0.0, total: 0)
- ✅ Invalid pattern → error

**Composition Use Cases**:
- Pattern frequency metrics
- Code statistics (e.g., "% of files that are tests")
- Quality metrics

---

### ✅ Component 7: `list-count-regex-any`

**Category**: Collections
**Location**: `components/collections/list-count-regex-any/`
**Status**: ⬜ Not Started

**Description**: Count items matching ANY of multiple patterns (Kernel engine use case).

**Specification**:
```rust
Component Info:
  name: "List Count Regex Any"
  version: "1.0.0"
  category: "Collections"
  author: "WasmFlow Core Library"

Inputs:
  - list: StringListVal (required) - "List to analyze"
  - patterns: StringListVal (required) - "Regular expression patterns (OR logic)"

Outputs:
  - count: u32 - "Number of items matching at least one pattern"
  - percentage: f32 - "Percentage of items matching"
  - total: u32 - "Total items in input list"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Multiple patterns (["a.rs", "b.toml", "c.txt"] + [".*\\.rs$", ".*\\.toml$"] → count: 2, percentage: 66.67)
- ✅ Overlapping patterns (["a.rs"] + [".*\\.rs$", "a.*"] → count: 1, not 2)
- ✅ No patterns → error
- ✅ No matches → count: 0, percentage: 0.0
- ✅ Empty list → count: 0, percentage: 0.0, total: 0

**Composition Use Cases**:
- **Kernel message engine** (final counting step)
- Multi-criteria metrics
- Aggregate pattern analysis

---

## Phase 4: Advanced Components

### ✅ Component 8: `regex-match-any`

**Category**: Text / Regex
**Location**: `components/text/regex-match-any/`
**Status**: ⬜ Not Started

**Description**: Test single string against multiple patterns (returns true if ANY match).

**Specification**:
```rust
Component Info:
  name: "Regex Match Any"
  version: "1.0.0"
  category: "Text"
  author: "WasmFlow Core Library"

Inputs:
  - text: string (required) - "Text to test"
  - patterns: StringListVal (required) - "Regular expression patterns (OR logic)"

Outputs:
  - matches: bool - "True if text matches at least one pattern"
  - matched_pattern: string - "First pattern that matched (empty if none)"
  - match_count: u32 - "How many patterns matched"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Matches one pattern (text: "file.rs", patterns: [".*\\.rs$", ".*\\.txt$"] → true, matched: ".*\\.rs$", count: 1)
- ✅ Matches multiple patterns (text: "a.rs", patterns: [".*\\.rs$", "^a.*"] → true, count: 2)
- ✅ No match → false, matched: "", count: 0
- ✅ Empty patterns list → error
- ✅ One invalid pattern → error

**Composition Use Cases**:
- Single file validation against rules
- Complex conditional routing
- Multi-criteria validation

---

### ✅ Component 9: `list-reject-regex`

**Category**: Collections
**Location**: `components/collections/list-reject-regex/`
**Status**: ⬜ Not Started

**Description**: Remove items matching a pattern (inverse of filter - blocklist).

**Specification**:
```rust
Component Info:
  name: "List Reject Regex"
  version: "1.0.0"
  category: "Collections"
  author: "WasmFlow Core Library"

Inputs:
  - list: StringListVal (required) - "List to filter"
  - pattern: string (required) - "Regular expression pattern to reject"

Outputs:
  - kept: StringListVal - "Items that did NOT match the pattern"
  - removed_count: u32 - "Number of items removed"

Dependencies:
  - regex = "1.10"
```

**Test Cases**:
- ✅ Some matches removed (["a.rs", "b.txt", "c.rs"] + ".*\\.rs$" → ["b.txt"], removed: 2)
- ✅ No matches (["a.txt"] + ".*\\.rs$" → ["a.txt"], removed: 0)
- ✅ All match (["a.rs", "b.rs"] + ".*\\.rs$" → [], removed: 2)
- ✅ Common blocklist (["src/main.rs", ".git/config", "target/debug/app"] + "(\\.git|target)/" → ["src/main.rs"], removed: 2)
- ✅ Invalid pattern → error

**Composition Use Cases**:
- Exclude `.git/`, `node_modules/`, `target/`
- Blocklist filtering
- Remove sensitive paths

---

## Kernel Message Engine Pipeline

**Goal**: Parse JSONL Kernel messages, extract file paths, filter empties, match against patterns, count results.

**Node Graph**:
```
[JSONL Input: String]
    ↓
[string-split] (delimiter: "\n")                    [existing component]
    ↓
[json-extract-each] (field_path: "path")           [Phase 1, Component 3]
    ↓
[list-filter-empty]                                 [Phase 1, Component 2]
    ↓
[list-count-regex-any] (patterns: [".*\\.rs$", "Cargo\\.toml"])  [Phase 3, Component 7]
    ↓
[Output: count (u32), percentage (f32)]
```

**Test Data Example**:
```jsonl
{"event": "create", "path": "src/main.rs", "timestamp": 1234567890}
{"event": "modify", "path": "Cargo.toml", "timestamp": 1234567891}
{"event": "delete", "path": "", "timestamp": 1234567892}
{"event": "create", "path": "target/debug/app", "timestamp": 1234567893}
{"event": "modify", "path": "README.md", "timestamp": 1234567894}
```

**Expected Result**:
- After split: 5 JSON strings
- After extract: ["src/main.rs", "Cargo.toml", "", "target/debug/app", "README.md"]
- After filter-empty: ["src/main.rs", "Cargo.toml", "target/debug/app", "README.md"]
- After count-regex-any (patterns: [".*\\.rs$", "Cargo\\.toml"]): count: 2, percentage: 50.0

---

## Alternative Composition Examples

### Email Validation Pipeline
```
[CSV String] → string-split (",") → list-filter-empty →
list-filter-regex ("^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$") →
list-length
```

### Log Error Analysis
```
[Log File] → string-split ("\n") →
list-filter-regex-any (["ERROR", "FATAL", "CRITICAL"]) →
list-count-regex ("database") → [Count]
```

### Clean Code Metrics
```
[File Paths] → list-filter-empty →
list-reject-regex ("(node_modules|\\.git|target)/") →
list-count-regex-any ([".*\\.rs$", ".*\\.toml$"]) → [Rust File Count]
```

---

## Build Strategy

### Setup Once
```bash
cd components

# Create category directories if needed
mkdir -p text collections data

# Ensure templates are ready
ls .templates/node.wit
```

### Per-Component Process (45 minutes each)

1. **Create structure** (5 min)
   ```bash
   cd components/<category>/<component-name>
   # Copy Cargo.toml, Justfile, wit/ from similar component
   ```

2. **Implement** (25 min)
   - Copy lib.rs from similar component
   - Update metadata (name, version, inputs, outputs)
   - Implement execute() logic
   - Add dependencies to Cargo.toml if needed

3. **Test** (10 min)
   ```bash
   cargo test
   ```

4. **Build** (5 min)
   ```bash
   just build
   just install
   ```

### Batch Operations
```bash
# Test all new components in a category
cd components/collections
just test-all

# Build all new components
cd components
just build-all

# Install all to bin/
cd components
just install-all
```

---

## Dependencies Summary

| Component | Crates Needed | Size Impact |
|-----------|---------------|-------------|
| regex-match | regex = "1.10" | +20KB |
| list-filter-empty | None | ~100KB |
| json-extract-each | serde_json = "1.0" | +50KB (~150KB total) |
| list-filter-regex | regex = "1.10" | +20KB |
| list-filter-regex-any | regex = "1.10" | +20KB |
| list-count-regex | regex = "1.10" | +20KB |
| list-count-regex-any | regex = "1.10" | +20KB |
| regex-match-any | regex = "1.10" | +20KB |
| list-reject-regex | regex = "1.10" | +20KB |

**Note**: All regex-based components share the same `regex` crate, so the size impact is per-component, not cumulative at runtime.

---

## Testing Strategy

### Unit Tests (In Component)
Each component: 5-9 unit tests covering:
- ✅ Typical usage
- ✅ Edge cases (empty inputs, boundary conditions)
- ✅ Error handling (invalid patterns, type mismatches)
- ✅ Unicode/special characters
- ✅ Performance (large lists if applicable)

### Integration Test (In UI)
Create `tests/component_tests/kernel_message_engine.json`:
- Load JSONL test data
- Connect all 5 nodes in pipeline
- Execute and verify output
- Test with various pattern combinations

---

## Documentation Updates

After implementation, update:

1. **`components/LIBRARY.md`** - Add new components to reference
2. **`components/README.md`** - Update component count
3. **`CLAUDE.md`** - Add regex/JSONL patterns to "Recent Changes"
4. **Create `specs/FOUNDATIONAL_COMPONENTS.md`** - Detailed implementation notes

---

## Completion Checklist

### Phase 1: Core (Essential)
- [ ] `regex-match` - Tests pass ✓, Built ✓, Installed ✓
- [ ] `list-filter-empty` - Tests pass ✓, Built ✓, Installed ✓
- [ ] `json-extract-each` - Tests pass ✓, Built ✓, Installed ✓

### Phase 2: Filtering
- [ ] `list-filter-regex` - Tests pass ✓, Built ✓, Installed ✓
- [ ] `list-filter-regex-any` - Tests pass ✓, Built ✓, Installed ✓

### Phase 3: Analysis
- [ ] `list-count-regex` - Tests pass ✓, Built ✓, Installed ✓
- [ ] `list-count-regex-any` - Tests pass ✓, Built ✓, Installed ✓

### Phase 4: Advanced
- [ ] `regex-match-any` - Tests pass ✓, Built ✓, Installed ✓
- [ ] `list-reject-regex` - Tests pass ✓, Built ✓, Installed ✓

### Testing & Documentation
- [ ] Kernel engine integration test in UI
- [ ] All components appear in node palette
- [ ] Documentation updated (LIBRARY.md, README.md)
- [ ] Build commands in Justfiles updated

### Final Validation
- [ ] All 9 components load without errors
- [ ] Kernel message test data processes correctly
- [ ] Count output matches expected values
- [ ] Performance acceptable (<10ms per component for typical data)

---

**Started**: [Date]
**Completed**: [Date]
**Time Taken**: [Hours]
**Components Built**: 0/9
