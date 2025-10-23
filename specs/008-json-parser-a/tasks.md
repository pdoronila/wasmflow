# Tasks: JSON Parser Node

**Input**: Design documents from `/specs/008-json-parser-a/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are included per research.md recommendation (three-tier testing strategy)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions
- Single project structure at repository root
- New files: `src/builtin/json_parser.rs`, `src/wit/json-parser.wit`
- Test files: `tests/unit/`, `tests/contract/`, `tests/integration/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependency setup

- [X] **T001** [P] Add `serde_json = "1.0"` dependency to Cargo.toml
- [X] **T002** [P] Add `thiserror = "1.0"` dependency to Cargo.toml
- [X] **T003** [P] Add `wit-bindgen = "0.30"` to dev dependencies in Cargo.toml
- [X] **T004** Ensure `wasm32-wasip2` target is installed: `rustup target add wasm32-wasip2`
- [X] **T005** Create WIT interface file at `src/wit/json-parser.wit` (copy from contracts/)
- [X] **T006** Create stub file `src/builtin/json_parser.rs` with module structure

**Checkpoint**: ✅ Dependencies installed, stub files created

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and infrastructure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] **T007** [Foundation] Define error types in `src/builtin/json_parser.rs`:
  - `enum JsonParserError { InvalidJson, PathNotFound, MalformedPath, IndexOutOfBounds, TypeMismatch }`
  - Implement `Display` and `Error` traits using `thiserror`
  - Add context fields for error messages

- [X] **T008** [P] [Foundation] Define `Token` enum in `src/builtin/json_parser.rs`:
  - `enum Token { Ident(String), Index(usize) }`
  - This represents parsed key path components

- [X] **T009** [Foundation] Implement key path tokenizer in `src/builtin/json_parser.rs`:
  - Function: `fn tokenize(key_path: &str) -> Result<Vec<Token>, JsonParserError>`
  - Parse dot notation (`.`) and bracket notation (`[n]`)
  - Validate identifier syntax and numeric indices
  - Return `MalformedPath` error for invalid syntax

- [X] **T010** [P] [Foundation] Define output types in `src/builtin/json_parser.rs`:
  - `enum JsonValue { String(String), Number(f64), Boolean(bool), Object(String), Array(String), Null }`
  - Maps to WIT `json-value` variant from contracts/json-parser.wit

- [X] **T011** [P] [Foundation] Implement type conversion utilities:
  - `fn serde_value_to_json_value(value: &serde_json::Value) -> JsonValue`
  - Map all serde_json::Value variants to JsonValue variants
  - Serialize objects/arrays to JSON strings

- [X] **T012** [Foundation] Implement core extraction logic in `src/builtin/json_parser.rs`:
  - Function: `fn extract_value(json: &serde_json::Value, tokens: &[Token]) -> Result<JsonValue, JsonParserError>`
  - Traverse JSON using token sequence
  - Handle all error cases (path not found, type mismatch, index out of bounds)
  - Preserve JSON type in output

**Checkpoint**: ✅ Foundation ready - all core types and extraction logic complete, user story implementation can begin

---

## Phase 3: User Story 1 - Extract Simple Top-Level Values (Priority: P1) 🎯 MVP

**Goal**: Enable extraction of simple top-level properties from JSON using basic key paths

**Independent Test**: Provide JSON `{"version": 1}` with path `"version"` and verify output is `1`

### Tests for User Story 1

**NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] **T013** [P] [US1] Unit test: Simple property extraction in `tests/unit/json_parser_unit.rs`
  - Test: `{"version": 1}` with path `"version"` → `JsonValue::Number(1.0)`
  - Test: `{"author": "me"}` with path `"author"` → `JsonValue::String("me")`
  - Test: `{"enabled": true}` with path `"enabled"` → `JsonValue::Boolean(true)`
  - Test: `{"value": null}` with path `"value"` → `JsonValue::Null`

- [ ] **T014** [P] [US1] Unit test: Error handling in `tests/unit/json_parser_unit.rs`
  - Test: Invalid JSON `"{invalid"` → `InvalidJson` error
  - Test: Empty key path `""` → `MalformedPath` error
  - Test: Nonexistent key `"missing"` in `{}` → `PathNotFound` error

### Implementation for User Story 1

- [ ] **T015** [US1] Implement main parse function in `src/builtin/json_parser.rs`:
  - Function: `pub fn parse(json_string: &str, key_path: &str) -> Result<JsonValue, JsonParserError>`
  - Parse JSON string using `serde_json::from_str`
  - Tokenize key path using `tokenize()`
  - Extract value using `extract_value()`
  - Handle all error paths

- [ ] **T016** [US1] Implement node registration in `src/builtin/mod.rs`:
  - Add `mod json_parser;`
  - Register json-parser node in `register_builtin_nodes()` function
  - Wire up parse function to node input/output ports

- [ ] **T017** [US1] Add integration test in `tests/integration/json_parser_graph.rs`:
  - Create graph with JSON parser node
  - Set inputs: json_string = `{"version": 1}`, key_path = `"version"`
  - Execute graph
  - Verify output: value = `1`, error = None

**Checkpoint**: User Story 1 complete - simple top-level property extraction fully functional

---

## Phase 4: User Story 2 - Navigate Nested Object Properties (Priority: P2)

**Goal**: Enable extraction from nested objects using dot notation

**Independent Test**: Provide JSON `{"metadata": {"author": "me"}}` with path `"metadata.author"` and verify output is `"me"`

### Tests for User Story 2

- [ ] **T018** [P] [US2] Unit test: Nested property extraction in `tests/unit/json_parser_unit.rs`
  - Test: `{"metadata": {"author": "me"}}` with path `"metadata.author"` → `JsonValue::String("me")`
  - Test: `{"config": {"server": {"port": 8080}}}` with path `"config.server.port"` → `JsonValue::Number(8080.0)`
  - Test: `{"a": {"b": {"c": {"d": "deep"}}}}` with path `"a.b.c.d"` → `JsonValue::String("deep")`

- [ ] **T019** [P] [US2] Unit test: Nested error cases in `tests/unit/json_parser_unit.rs`
  - Test: Path `"metadata.missing"` in `{"metadata": {}}` → `PathNotFound` error
  - Test: Path `"version.property"` when `version` is number → `TypeMismatch` error

### Implementation for User Story 2

- [ ] **T020** [US2] Verify tokenizer handles dot notation in `src/builtin/json_parser.rs`:
  - Tokenize `"metadata.author"` → `[Token::Ident("metadata"), Token::Ident("author")]`
  - Already implemented in T009, verify with tests

- [ ] **T021** [US2] Verify extract_value handles nested traversal in `src/builtin/json_parser.rs`:
  - For `Token::Ident`, access object property and continue traversal
  - Return `PathNotFound` if key doesn't exist
  - Return `TypeMismatch` if trying to access property on non-object
  - Already implemented in T012, verify with tests

- [ ] **T022** [US2] Add integration test in `tests/integration/json_parser_graph.rs`:
  - Create graph with JSON parser node
  - Set inputs: json_string = nested object, key_path = `"metadata.author"`
  - Execute graph
  - Verify correct nested value extracted

**Checkpoint**: User Story 2 complete - nested object navigation fully functional

---

## Phase 5: User Story 3 - Access Array Elements by Index (Priority: P2)

**Goal**: Enable extraction from arrays using bracket notation

**Independent Test**: Provide JSON `{"runs": [{"id": 1}, {"id": 2}]}` with path `"runs[1]"` and verify output is `{"id": 2}`

### Tests for User Story 3

- [ ] **T023** [P] [US3] Unit test: Array index extraction in `tests/unit/json_parser_unit.rs`
  - Test: `{"runs": [{"id": 1}, {"id": 2}]}` with path `"runs[1]"` → object `{"id": 2}` as JSON string
  - Test: `{"values": [10, 20, 30]}` with path `"values[0]"` → `JsonValue::Number(10.0)`
  - Test: `{"items": ["first", "second", "third"]}` with path `"items[2]"` → `JsonValue::String("third")`

- [ ] **T024** [P] [US3] Unit test: Array error cases in `tests/unit/json_parser_unit.rs`
  - Test: Path `"runs[999]"` with 2-element array → `IndexOutOfBounds` error
  - Test: Path `"metadata[0]"` when metadata is object → `TypeMismatch` error
  - Test: Path `"runs[abc]"` → `MalformedPath` error

### Implementation for User Story 3

- [ ] **T025** [US3] Verify tokenizer handles bracket notation in `src/builtin/json_parser.rs`:
  - Tokenize `"runs[1]"` → `[Token::Ident("runs"), Token::Index(1)]`
  - Parse numeric indices from brackets
  - Reject non-numeric indices with `MalformedPath`
  - Already implemented in T009, verify with tests

- [ ] **T026** [US3] Verify extract_value handles array indexing in `src/builtin/json_parser.rs`:
  - For `Token::Index`, access array element and continue traversal
  - Return `IndexOutOfBounds` if index >= array length
  - Return `TypeMismatch` if trying to index non-array
  - Already implemented in T012, verify with tests

- [ ] **T027** [US3] Add integration test in `tests/integration/json_parser_graph.rs`:
  - Create graph with JSON parser node
  - Set inputs: json_string = array data, key_path = `"runs[1]"`
  - Execute graph
  - Verify correct array element extracted

**Checkpoint**: User Story 3 complete - array indexing fully functional

---

## Phase 6: User Story 4 - Combine Array Access with Property Navigation (Priority: P3)

**Goal**: Enable complex paths combining dot and bracket notation

**Independent Test**: Provide JSON `{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}` with path `"runs[1].time"` and verify output is `1000`

### Tests for User Story 4

- [ ] **T028** [P] [US4] Unit test: Combined notation extraction in `tests/unit/json_parser_unit.rs`
  - Test: `{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}` with path `"runs[1].time"` → `JsonValue::Number(1000.0)`
  - Test: `{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}` with path `"users[0].name"` → `JsonValue::String("Alice")`
  - Test: `{"data": {"items": [{"value": {"score": 95}}]}}` with path `"data.items[0].value.score"` → `JsonValue::Number(95.0)`

- [ ] **T029** [P] [US4] Unit test: Deep nesting and large arrays in `tests/unit/json_parser_unit.rs`
  - Test: 10+ level deep nesting (per SC-002)
  - Test: 1000+ element array access (per SC-003)
  - Verify performance targets met

### Implementation for User Story 4

- [ ] **T030** [US4] Verify tokenizer handles mixed notation in `src/builtin/json_parser.rs`:
  - Tokenize `"runs[1].time"` → `[Token::Ident("runs"), Token::Index(1), Token::Ident("time")]`
  - Handle arbitrary combinations of dots and brackets
  - Already implemented in T009, verify with tests

- [ ] **T031** [US4] Verify extract_value handles complex traversal in `src/builtin/json_parser.rs`:
  - Process mixed token sequences correctly
  - Maintain state through object → array → object transitions
  - Already implemented in T012, verify with tests

- [ ] **T032** [US4] Add integration test in `tests/integration/json_parser_graph.rs`:
  - Create graph with JSON parser node
  - Set inputs: json_string = complex nested data, key_path = `"runs[1].time"`
  - Execute graph
  - Verify correct value extracted from nested structure

- [ ] **T033** [US4] Add integration test for chained parsers in `tests/integration/json_parser_graph.rs`:
  - Create graph with single JSON source node
  - Connect 3 JSON parser nodes to same source (different paths)
  - Verify all extract correct values independently (per SC-007)

**Checkpoint**: All user stories complete - full JSON parser functionality implemented

---

## Phase 7: Contract Validation & Performance

**Purpose**: Validate WIT interface compliance and performance requirements

- [ ] **T034** [P] Contract test: WIT interface validation in `tests/contract/json_parser_test.rs`
  - Verify all JsonValue variants map correctly to WIT types
  - Verify ParseError serialization matches WIT record structure
  - Test error kind enum matches WIT definition

- [ ] **T035** [P] Contract test: Type conversion correctness in `tests/contract/json_parser_test.rs`
  - Test all JSON type → JsonValue conversions
  - Verify object/array serialization produces valid JSON
  - Test null handling (FR-012)

- [ ] **T036** [P] Performance benchmark: Create benchmark suite using criterion
  - Benchmark: 1KB JSON parsing (<10ms target)
  - Benchmark: 100KB JSON parsing (<50ms target)
  - Benchmark: 1MB JSON parsing (<100ms target per SC-006)
  - Benchmark: 100-level deep nesting
  - Benchmark: 1000-element array access

- [ ] **T037** [P] Performance test: Verify SC-003 in `tests/integration/json_parser_graph.rs`
  - Test 1000-element array access without performance degradation
  - Measure execution time, ensure meets target

**Checkpoint**: Contract compliance verified, performance validated

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] **T038** [P] Documentation: Add inline documentation to all public functions in `src/builtin/json_parser.rs`
  - Document parse function with examples
  - Document error types with when they occur
  - Add usage examples in rustdoc

- [ ] **T039** [P] Error messages: Enhance all error messages with actionable context
  - InvalidJson: Include parse location (line/column)
  - PathNotFound: Include full path attempted and where it failed
  - IndexOutOfBounds: Include index and array length
  - TypeMismatch: Include expected vs actual type
  - MalformedPath: Include invalid segment and position

- [ ] **T040** Code review: Verify all edge cases from spec.md are handled
  - Empty key path → MalformedPath
  - Null values → JsonValue::Null (distinct from missing)
  - Empty objects/arrays → handled correctly
  - Large payloads → performance acceptable
  - Special characters in keys → documented as out of scope

- [ ] **T041** [P] Add comprehensive unit tests for edge cases in `tests/unit/json_parser_unit.rs`
  - All 10 edge cases from data-model.md
  - Verify behavior matches specification

- [ ] **T042** Integration: Test node registration and graph integration
  - Verify node appears in builtin node registry
  - Test connection type validation (string inputs, variant output)
  - Verify error port vs value port exclusivity

- [ ] **T043** Validation: Run all examples from quickstart.md
  - Verify all 9 examples produce expected output
  - Test error handling examples
  - Validate chaining example

- [ ] **T044** [P] Security review: Verify sandboxing constraints
  - Confirm no system access (pure computation)
  - Verify no unsafe code used
  - Validate memory bounds (1MB limit enforced by runtime)

- [ ] **T045** Final testing: Run full test suite
  - cargo test --all
  - cargo test --target wasm32-wasip2
  - Verify all acceptance scenarios from spec.md pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-6)**: All depend on Foundational phase completion
  - User stories CAN proceed in parallel (if staffed) after Phase 2
  - Or sequentially in priority order: US1 (P1) → US2 (P2) → US3 (P2) → US4 (P3)
- **Contract/Performance (Phase 7)**: Depends on all user stories being complete
- **Polish (Phase 8)**: Depends on Phase 7 completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independently testable (just verifies existing code)
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Independently testable (just verifies existing code)
- **User Story 4 (P3)**: Can start after Foundational (Phase 2) - Independently testable (just verifies existing code)

**Note**: User Stories 2-4 primarily add test coverage for functionality already implemented in Foundational phase (T009, T012). They can all run in parallel after Phase 2.

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Tests for a story can run in parallel (marked [P])
- Implementation tasks verify/extend foundational code
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1**: T001, T002, T003 can run in parallel (different dependencies)
- **Phase 2**: T008, T010, T011 can run in parallel (different types, no dependencies)
- **User Story Tests**: All tests within a story marked [P] can run in parallel
- **After Phase 2**: All user stories (Phases 3-6) can start in parallel if team capacity allows
- **Phase 7**: T034, T035, T036, T037 can run in parallel (different test files)
- **Phase 8**: T038, T039, T041, T044 can run in parallel (different concerns)

---

## Parallel Example: Foundational Phase

```bash
# After T007 completes, launch these in parallel:
Task: "Define Token enum in src/builtin/json_parser.rs" (T008)
Task: "Define output types in src/builtin/json_parser.rs" (T010)
Task: "Implement type conversion utilities" (T011)

# After tokenizer (T009) and extraction logic (T012) complete,
# all user story phases can begin in parallel:
Task: "User Story 1 tests" (T013, T014)
Task: "User Story 2 tests" (T018, T019)
Task: "User Story 3 tests" (T023, T024)
Task: "User Story 4 tests" (T028, T029)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T012) - **CRITICAL BLOCKER**
3. Complete Phase 3: User Story 1 (T013-T017)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Ship MVP: Simple top-level property extraction

**MVP Deliverable**: JSON parser node that extracts simple properties like `{"version": 1}` with path `"version"` → `1`

### Incremental Delivery

1. **Foundation** (Phases 1-2) → Core types and extraction logic ready
2. **+US1** (Phase 3) → Test independently → **Ship MVP** (simple properties)
3. **+US2** (Phase 4) → Test independently → Ship (+ nested objects)
4. **+US3** (Phase 5) → Test independently → Ship (+ array indexing)
5. **+US4** (Phase 6) → Test independently → Ship (+ combined paths)
6. **Validation** (Phases 7-8) → Full validation and polish

Each increment adds value without breaking previous functionality.

### Parallel Team Strategy

With multiple developers:

1. **Together**: Complete Setup + Foundational (Phases 1-2)
2. **Once Phase 2 done**:
   - Developer A: User Story 1 (Phase 3)
   - Developer B: User Story 2 (Phase 4)
   - Developer C: User Story 3 (Phase 5)
   - Developer D: User Story 4 (Phase 6)
3. **Converge**: Contract validation and polish (Phases 7-8)

Stories complete and integrate independently since they share the same foundational code.

---

## Task Summary

**Total Tasks**: 45 tasks

### Tasks per Phase:
- Phase 1 (Setup): 6 tasks
- Phase 2 (Foundational): 6 tasks
- Phase 3 (User Story 1 - P1): 5 tasks
- Phase 4 (User Story 2 - P2): 5 tasks
- Phase 5 (User Story 3 - P2): 5 tasks
- Phase 6 (User Story 4 - P3): 6 tasks
- Phase 7 (Contract/Performance): 4 tasks
- Phase 8 (Polish): 8 tasks

### Parallel Opportunities:
- 18 tasks marked [P] for parallel execution
- All 4 user stories can proceed in parallel after Foundational phase

### MVP Scope (Recommended):
- Phases 1-3 only (T001-T017) = **17 tasks**
- Delivers: Simple top-level property extraction (User Story 1)
- Estimated: 1-2 days for experienced Rust developer

### Full Feature Scope:
- All 45 tasks
- Delivers: Complete JSON parser with all 4 user stories
- Estimated: 3-5 days for experienced Rust developer

---

## Notes

- [P] tasks = different files or independent concerns, no dependencies
- [Story] label (US1, US2, US3, US4) maps task to specific user story for traceability
- [Foundation] label indicates blocking prerequisites for all user stories
- Tests written FIRST (TDD approach) and must FAIL before implementation
- Each user story should be independently testable via its acceptance scenarios
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Most implementation happens in Phase 2 (Foundational) - later phases add test coverage and verification
