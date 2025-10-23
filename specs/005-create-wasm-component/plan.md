# Implementation Plan: WASM Component Creator Node

**Branch**: `005-create-wasm-component` | **Date**: 2025-10-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-create-wasm-component/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create a built-in "WASM Component Creator" node that enables users to write Rust code directly in a visual code editor, specify component metadata via structured comments, compile the code into WASM components, and dynamically load them into the component palette. This feature eliminates the need for external build tools and enables rapid prototyping of custom components within the visual programming environment. The implementation will use egui's code editor widget for syntax highlighting, invoke cargo-component for compilation, and integrate with the existing component registry for dynamic loading.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)

**Primary Dependencies**:
- egui 0.29 + eframe 0.29 (UI framework)
- egui-snarl 0.3 (node editor)
- wasmtime 27.0 with component-model (WASM runtime)
- cargo-component 0.21+ (WASM component build tool, invoked via CLI)
- egui_code_editor 0.2.20 (Rust syntax highlighting, lightweight ~10KB)

**Storage**: File system (temporary build artifacts in `/tmp/wasmflow-build-{uuid}/`, optional code persistence in graph JSON via BTreeMap)

**Testing**: cargo test (unit tests for parser, integration tests for compilation workflow, contract tests for generated components)

**Target Platform**: Desktop (macOS, Linux, Windows) via eframe native backend

**Project Type**: Single desktop application with visual node editor

**Performance Goals**:
- UI responsiveness: 60 FPS during code editing (up to 1000 lines)
- Compilation: 3-20 seconds for typical components (cargo-component external process)
- Component loading: < 100ms after compilation completes (wasmtime instantiation)

**Constraints**:
- Code editor must support at least 500 lines without lag (SC-005)
- Maximum code size: 10,000 lines OR 500KB (whichever smaller) - prevents abuse
- Compilation timeout: 120 seconds (2 minutes) - prevents infinite builds
- Must work offline (no network dependencies for compilation)

**Scale/Scope**:
- Support 20+ user-defined components per session (SC-006)
- Code editor handling up to 1000 lines with syntax highlighting
- Compilation queue supporting 1 concurrent build per creator node

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Component-First Architecture ✅ PASS

- **Status**: COMPLIANT
- **Verification**: The WASM Creator node itself is a built-in component that generates user-defined WASM components. Generated components will define WIT interfaces via template generation.
- **Notes**: The creator node acts as a meta-component, enabling user extensibility while maintaining component-first principles.

### Principle II: Capability-Based Security ⚠️ REQUIRES ATTENTION

- **Status**: PARTIAL COMPLIANCE - Requires design clarification
- **Issue**: Generated components must declare capabilities, but the creator node itself needs file system and process execution capabilities (for cargo-component invocation)
- **Required Actions**:
  1. Creator node must request Developer-level capabilities (file write + process spawn)
  2. Generated component code must include capability declarations parsed from user annotations
  3. User must approve creator node capabilities before first use
- **Security Verification**: Template generation must enforce capability declaration structure

### Principle III: Typed Data Flow ✅ PASS

- **Status**: COMPLIANT
- **Verification**: Port specification via structured comments (`// @input name:type`) ensures WIT type definitions are generated. Parser validates type names against supported types (F32, I32, U32, String, Boolean per FR-020).
- **Notes**: Type checking happens at template generation time, not runtime.

### Principle IV: Performance-First Execution ✅ PASS

- **Status**: COMPLIANT
- **Verification**:
  - Code editor performance target: 60 FPS (SC-005: 500 lines without degradation)
  - Compilation offloaded to background process (doesn't block UI)
  - Component loading uses existing wasmtime pooling/caching mechanisms
  - Compilation per-node prevents global blocking
- **Notes**: Compilation time is external dependency (cargo-component), not under our control.

### Principle V: Developer Ergonomics ✅ PASS

- **Status**: COMPLIANT
- **Verification**:
  - Structured comment syntax reduces boilerplate
  - Template auto-generation eliminates WIT binding complexity
  - Clear error messages from compilation (FR-011: line numbers + descriptions)
  - No external tool setup required (in-app workflow)
- **Notes**: This feature directly enhances developer ergonomics by lowering component creation barriers.

### Principle VI: Composability & Modularity ✅ PASS

- **Status**: COMPLIANT
- **Verification**:
  - Generated components are first-class citizens in palette
  - Component metadata included in template (name, version, description, category)
  - Components can be reused across graphs
  - Purple color coding distinguishes user-defined components
- **Notes**: Future enhancement could support exporting user components as standalone files.

### Summary

**Gate Status**: ✅ PASS with 1 clarification required

**Action Items Before Phase 0**:
- Resolve Principle II security model: Document exact capability requirements for creator node
- Confirm capability declaration template structure for generated components

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── builtin/
│   ├── mod.rs                    # Register creator node
│   ├── wasm_creator.rs           # NEW: Creator node implementation
│   └── views.rs                  # Existing footer views
├── ui/
│   ├── mod.rs
│   ├── palette.rs                # MODIFY: Add user-defined category
│   ├── code_editor.rs            # NEW: Code editor widget wrapper
│   └── theme.rs                  # MODIFY: Add purple color for user components
├── graph/
│   ├── node.rs                   # MODIFY: Add creator node type, optional code field
│   └── serialization.rs          # MODIFY: Handle optional code persistence
├── runtime/
│   ├── wasm_host.rs              # MODIFY: Support dynamic component registration
│   ├── compiler.rs               # NEW: Cargo-component invocation wrapper
│   └── template_generator.rs    # NEW: Generate component Rust code from template
└── lib.rs

tests/
├── contract/
│   └── generated_component_test.rs  # NEW: Validate generated component WIT contracts
├── integration/
│   └── compilation_workflow_test.rs # NEW: End-to-end compilation test
└── unit/
    ├── comment_parser_test.rs       # NEW: Test structured comment parsing
    └── template_generator_test.rs   # NEW: Test template code generation

templates/
└── component_template.rs.tmpl       # NEW: Base template for generated components
```

**Structure Decision**: Single desktop application structure. New code primarily in `src/builtin/wasm_creator.rs` for the creator node UI and logic, `src/runtime/` for compilation infrastructure, and `src/ui/code_editor.rs` for the editor widget. Modifications to existing palette, theme, and node serialization to support user-defined components.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations requiring justification.** Constitution check passed with one clarification item that was addressed in Phase 0 research.

---

## Planning Phase Summary

### Phase 0: Research ✅ COMPLETE

**Output**: [research.md](./research.md) (730 lines, 22KB)

**Key Decisions**:
1. **cargo-component 0.21+**: CLI invocation with JSON output parsing
2. **egui_code_editor 0.2.20**: Purpose-built for egui, lightweight
3. **Code Limits**: 10K lines / 500KB, 120s timeout
4. **Capability Syntax**: Runtime-enforced via `get_capabilities()` in Rust
5. **Templates**: Two base templates (Simple, HTTP)

**All NEEDS CLARIFICATION items resolved.**

### Phase 1: Design & Contracts ✅ COMPLETE

**Outputs**:
- [data-model.md](./data-model.md): 6 core entities, state machines, validation rules
- [contracts/](./contracts/): Rust module interfaces, WIT definitions, comment protocol
- [quickstart.md](./quickstart.md): 4-week implementation roadmap with code examples
- CLAUDE.md updated with new technologies

**Design Highlights**:
- **WasmCreatorNode**: Code editor + name input + compile button
- **CompilationState**: State machine (Idle → Compiling → Success/Failed)
- **GeneratedComponent**: Purple-colored user components in palette
- **Template System**: Variable substitution with {{placeholders}}
- **Comment Annotations**: `// @input name:Type description` DSL

### Constitution Re-Check (Post-Design) ✅ PASS

All principles remain compliant:
- ✅ Component-First Architecture: Creator node enables component creation
- ✅ Capability-Based Security: Template enforces capability declarations
- ✅ Typed Data Flow: Port types validated at template generation
- ✅ Performance-First: Background compilation, 60 FPS editor
- ✅ Developer Ergonomics: Structured comments reduce boilerplate
- ✅ Composability: Generated components are first-class palette items

**Security Note**: Creator node requires Developer-level capabilities (file write + process spawn) per Principle II. User must approve on first use.

### Next Steps

**Ready for Phase 2**: Task Generation
- Run `/speckit.tasks` to generate dependency-ordered task list
- Estimated implementation: 4 weeks (8 tasks across 4 phases)
- All design artifacts available for implementation

**Implementation Priority**:
1. Week 1: Template generator + compiler infrastructure
2. Week 2: UI integration (code editor + creator node)
3. Week 3: Dynamic loading + palette integration
4. Week 4: Error handling + validation + polish

---

## Artifact Summary

| Artifact | Lines | Status | Purpose |
|----------|-------|--------|---------|
| spec.md | 147 | ✅ Complete | Feature requirements |
| plan.md | 166 | ✅ Complete | This file (implementation plan) |
| research.md | 730 | ✅ Complete | Technology decisions |
| data-model.md | 454 | ✅ Complete | Entity definitions, state machines |
| quickstart.md | 387 | ✅ Complete | Developer onboarding guide |
| contracts/ | 4 files | ✅ Complete | Interface contracts |
| checklists/requirements.md | 48 | ✅ Complete | Spec validation checklist |

**Total**: 1,932 lines of planning documentation

---

**Planning completed**: 2025-10-18
**Branch**: `005-create-wasm-component`
**Ready for**: `/speckit.tasks` (Phase 2 - Task Generation)
