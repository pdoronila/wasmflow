# Feature 011: Optimize WASM Component Loading

**Status**: Planning Complete, Ready for Implementation
**Created**: 2025-10-25
**Estimated Time**: 14-19 hours
**Branch**: `claude/optimize-wasm-loading-011CUUY1vZZ4HuRLFoh5kzWx`

## Quick Summary

Improve WasmFlow startup experience by adding:
1. **Splash screen with progress bar** - Show loading progress instead of blank screen
2. **Component metadata cache** - Cache extracted metadata with MD5 validation
3. **90% faster subsequent startups** - From 15-30s to 2-5s via caching

## Problem

Current startup loads 76 WASM components synchronously, causing:
- 15-30 second blank screen before UI appears
- No feedback on loading progress
- Metadata re-extracted on every launch

## Solution

**Three-tier optimization**:

1. **Splash Screen** (`src/ui/splash_screen.rs`)
   - Shows immediately on launch
   - Progress bar, component count, current component
   - Error summary if failures occur

2. **Metadata Cache** (`src/runtime/component_cache.rs`)
   - Saves ComponentSpec to JSON after extraction
   - MD5 checksums detect file changes
   - Cache invalidation when components modified

3. **Async Loading** (`src/ui/loading_state.rs`, `src/ui/app/components.rs`)
   - Background thread loads components
   - Real-time progress updates
   - UI remains responsive

## Architecture

```
Startup:
  main() → WasmFlowApp::new_with_loading()
    → Show splash screen
    → start_async_component_loading()
      → Background thread:
        → For each .wasm:
          → Compute MD5
          → Check cache
          → If cached: load JSON (fast)
          → If not: extract metadata (slow) + cache it
          → Update progress
      → Transition to main UI when complete
```

## Documents

### Core Documents
- **[spec.md](./spec.md)** - Complete feature specification
  - User stories, requirements, architecture
  - UI mockups, error handling, testing strategy
  - 200+ lines of detailed design

- **[plan.md](./plan.md)** - Implementation plan (SpecKit format)
  - 6 phases, 14-19 hours total
  - Phase breakdown with tasks and dependencies
  - Success criteria, risks, timeline

- **[tasks.md](./tasks.md)** - Detailed task breakdown
  - 41 granular tasks across 6 phases + final tasks
  - Dependencies, time estimates, verification steps
  - Progress tracking checklist

- **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** - Extended implementation guide
  - Detailed code samples for each phase
  - API designs, data structures, patterns
  - Performance targets, testing strategy

### Quick Reference

**Key Files Created**:
- `src/runtime/component_cache.rs` - Cache system
- `src/ui/loading_state.rs` - Progress tracking
- `src/ui/splash_screen.rs` - Loading UI

**Key Files Modified**:
- `src/ui/app.rs` - Loading state machine
- `src/ui/app/components.rs` - Async loading
- `src/runtime/wasm_host.rs` - Cache integration
- `src/main.rs` - Startup flow

**New Dependencies**:
- `md5 = "0.7"` - Checksum computation

**Cache Location**:
- `components/bin/.cache/` - Cached metadata
  - `<component>.json` - ComponentSpec
  - `<component>.md5` - Checksum
  - `cache_version.txt` - Format version

## Performance Targets

| Metric | Before | After (Uncached) | After (Cached) |
|--------|--------|------------------|----------------|
| Startup Time | 15-30s | 15-30s + splash | **2-5s** |
| UI Feedback | None (blank) | Immediate splash | Immediate splash |
| Cache Overhead | 0 MB | ~5 MB | ~5 MB |

**Target**: 90% reduction in subsequent startup times

## Implementation Phases

### Phase 1: Component Metadata Cache System ⏱️ 2-3 hours
Build cache infrastructure with MD5 validation
- 8 tasks: Setup, MD5 computation, validation, serialization, loading

### Phase 2: Async Component Loading Infrastructure ⏱️ 3-4 hours
Enable background loading with progress tracking
- 7 tasks: Loading state, async loader, cache integration, progress tracking

### Phase 3: Splash Screen UI Implementation ⏱️ 3-4 hours
Create visual loading screen
- 9 tasks: UI layout, progress bar, component display, spinner animation

### Phase 4: Application Startup Flow Integration ⏱️ 2-3 hours
Wire splash screen into app startup
- 7 tasks: Refactor constructor, state machine, transitions, error handling

### Phase 5: Cache Management UI & Tools ⏱️ 2 hours
Give users control over cache
- 4 tasks: Clear cache menu, statistics dialog, .gitignore

### Phase 6: Testing & Performance Validation ⏱️ 2-3 hours
Ensure correctness and measure improvements
- 6 tasks: Unit tests, integration tests, benchmarks, manual testing

## Getting Started

### 1. Review Documentation
```bash
cd specs/011-optimize-wasm-loading

# Start with spec for big picture
cat spec.md

# Then plan for implementation strategy
cat plan.md

# Then tasks for step-by-step execution
cat tasks.md

# Reference IMPLEMENTATION_PLAN.md for code samples
cat IMPLEMENTATION_PLAN.md
```

### 2. Start Phase 1
```bash
# Create cache module
mkdir -p src/runtime
touch src/runtime/component_cache.rs

# Add dependency
echo 'md5 = "0.7"' >> Cargo.toml

# Follow tasks.md Task 1.1 → 1.8
```

### 3. Test After Each Phase
```bash
cargo test
cargo clippy
cargo build --release
```

### 4. Track Progress
Update `tasks.md` checkboxes as you complete tasks:
- [ ] Task → ☐ Not done
- [x] Task → ☑ Done

## Success Criteria

### Must Have
- ✅ Splash screen displays within 1 second of launch
- ✅ Progress bar shows real-time loading progress
- ✅ Cached loading is ≥10x faster than uncached
- ✅ Component changes detected and cache invalidated
- ✅ Errors don't crash application

### Should Have
- ✅ Cache management UI (clear cache, statistics)
- ✅ Error summary visible on splash screen
- ✅ Smooth animation during loading
- ✅ Professional splash screen design

## Testing Strategy

### Unit Tests
- ComponentCache: save, load, MD5, validation
- LoadingState: progress tracking, thread safety
- 10+ test cases in `tests/component_cache_test.rs`

### Integration Tests
- Full loading flow with real components
- Cache hit/miss scenarios
- Error handling and recovery
- 5+ test cases in `tests/component_loading_integration.rs`

### Performance Benchmarks
- Uncached load: ~15-30 seconds (baseline)
- Cached load: ~2-5 seconds (target)
- `benches/component_loading.rs`

### Manual Testing
- Fresh install, second launch, component changes
- Cache clearing, corruption handling
- UI smoothness, error display

## Risk Mitigation

### High Risk: Breaking startup flow
- **Mitigation**: Extensive testing, maintain backward compatibility
- **Rollback**: Feature flag `--no-cache` to disable caching

### Medium Risk: Cache corruption
- **Mitigation**: Robust error handling, automatic cache invalidation
- **Fallback**: If cache invalid, fall back to direct extraction

### Medium Risk: Thread safety
- **Mitigation**: Use Arc<Mutex<>> for shared state
- **Testing**: Concurrent loading stress tests

## Next Steps

1. **✅ Review this plan** - Understand architecture and scope
2. **→ Begin Phase 1** - Implement component cache system
   - Start with Task 1.1: Setup Cache Module Structure
   - Follow tasks.md sequentially
3. **Test frequently** - Run tests after each task
4. **Update progress** - Check off tasks in tasks.md
5. **Move to Phase 2** - After Phase 1 complete and tested

## Questions?

- **How long will this take?** 14-19 hours across 6 phases
- **Can I implement phases in parallel?** No, phases have dependencies
- **What if I get stuck?** Refer to IMPLEMENTATION_PLAN.md for code samples
- **How do I test?** `cargo test` after each phase, manual testing at end
- **What if performance targets aren't met?** Phase 7+ can add parallelization

## Resources

- **Current Implementation**: `src/ui/app/components.rs:42-102`
- **ComponentManager**: `src/runtime/wasm_host.rs:264-510`
- **WasmFlowApp**: `src/ui/app.rs:34-196`
- **Component Directory**: `components/bin/` (76 .wasm files)

---

**Status**: 📋 Planning Complete
**Next Action**: 🚀 Begin Phase 1 - Component Metadata Cache System
**Start With**: Task 1.1 - Setup Cache Module Structure
