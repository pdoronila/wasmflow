# Implementation Plan: Optimize WASM Component Loading

**Feature**: 011-optimize-wasm-loading
**Created**: 2025-10-25
**Status**: Planning

## Overview

Optimize WASM component loading with splash screen, progress tracking, and metadata caching to improve startup time from 15-30 seconds to 2-5 seconds for subsequent loads.

## Goals

1. **Improve UX**: Show splash screen with loading progress instead of blank screen
2. **Optimize Performance**: Cache component metadata to reduce subsequent load times by 90%
3. **Maintain Reliability**: Ensure cache invalidation works correctly when components change
4. **Enable Scalability**: Support growing component library without proportional startup time increase

## Technical Approach

### Architecture Changes

**Current Flow** (Synchronous, Blocking):
```
main() → WasmFlowApp::new() → reload_components() [BLOCKS FOR 15-30s]
  → For each .wasm file:
    → Read file → Parse WASM → Instantiate → Extract metadata
```

**New Flow** (Async, Cached):
```
main() → WasmFlowApp::new_with_loading() → start_async_component_loading()
  → Show splash screen immediately
  → Background thread:
    → For each .wasm file:
      → Compute MD5 checksum
      → Check cache (components/bin/.cache/<name>.json)
      → If cached & valid: Load from JSON (fast)
      → If not cached: Extract metadata + cache it
      → Update progress
  → Transition to main UI when complete
```

### Key Components

1. **ComponentCache** (`src/runtime/component_cache.rs`)
   - Manages cache directory (`components/bin/.cache/`)
   - Computes MD5 checksums for .wasm files
   - Serializes/deserializes ComponentSpec to JSON
   - Validates cache freshness

2. **LoadingState** (`src/ui/loading_state.rs`)
   - Tracks loading progress (total, loaded, current component)
   - Thread-safe progress updates (Arc<Mutex<>>)
   - Collects errors during loading

3. **SplashScreen** (`src/ui/splash_screen.rs`)
   - Renders loading UI with progress bar
   - Shows component count, current component, errors
   - Animated spinner for visual feedback

4. **Async Loading** (`src/ui/app/components.rs`)
   - Background thread for component loading
   - Progress callbacks to UI thread
   - Cache integration

## Implementation Phases

### Phase 1: Component Metadata Cache System ⏱️ 2-3 hours

**Objective**: Build cache infrastructure with MD5 validation

**Tasks**:
- [ ] Create `src/runtime/component_cache.rs` module
- [ ] Implement `ComponentCache` struct with cache directory management
- [ ] Implement MD5 checksum computation for .wasm files
- [ ] Implement cache serialization (ComponentSpec ↔ JSON)
- [ ] Implement cache validation (checksum comparison)
- [ ] Add cache version tracking (`cache_version.txt`)
- [ ] Add error handling for corrupt cache entries
- [ ] Export module in `src/runtime/mod.rs`
- [ ] Add `md5` crate to `Cargo.toml`

**Deliverables**:
- `ComponentCache::new()`, `get_cached_spec()`, `save_spec()`, `invalidate_all()`
- Cache directory: `components/bin/.cache/`
- Cache files: `<name>.json` (spec), `<name>.md5` (checksum)

**Success Criteria**:
- ✅ Cache saves/loads ComponentSpec correctly
- ✅ MD5 checksums detect file changes
- ✅ Corrupt cache entries don't crash app

---

### Phase 2: Async Component Loading Infrastructure ⏱️ 3-4 hours

**Objective**: Enable background loading with progress tracking

**Tasks**:
- [ ] Create `src/ui/loading_state.rs` module
- [ ] Define `LoadingState` enum (NotStarted, Loading, Completed, Failed)
- [ ] Define `ComponentLoadProgress` struct (total, loaded, current, errors)
- [ ] Implement `reload_components_async()` in `src/ui/app/components.rs`
- [ ] Create background loading thread with progress updates
- [ ] Integrate cache into component loading flow
- [ ] Add thread-safe progress tracking (Arc<Mutex<>>)
- [ ] Add error aggregation
- [ ] Modify `ComponentManager::load_component_sync()` to use cache

**Deliverables**:
- `LoadingState` and `ComponentLoadProgress` types
- `async_component_loader()` background thread function
- Cache-aware component loading

**Success Criteria**:
- ✅ Components load in background without blocking
- ✅ Progress updates visible in real-time
- ✅ Cache reduces load time by 80%+
- ✅ Errors don't stop loading process

---

### Phase 3: Splash Screen UI Implementation ⏱️ 3-4 hours

**Objective**: Create visual loading screen

**Tasks**:
- [ ] Create `src/ui/splash_screen.rs` module
- [ ] Design splash screen layout (title + progress bar + status)
- [ ] Implement `SplashScreen` struct with egui rendering
- [ ] Add progress bar with percentage (0-100%)
- [ ] Add component count display ("45/76 components")
- [ ] Add current component name display
- [ ] Add error summary display
- [ ] Add loading spinner animation
- [ ] Export module in `src/ui/mod.rs`

**Deliverables**:
- `SplashScreen::new()`, `render()` methods
- Visual design: centered layout with WasmFlow branding

**Success Criteria**:
- ✅ Splash screen displays immediately
- ✅ Progress bar updates smoothly
- ✅ Component info clearly visible
- ✅ Professional appearance

---

### Phase 4: Application Startup Flow Integration ⏱️ 2-3 hours

**Objective**: Wire splash screen into app startup

**Tasks**:
- [ ] Refactor `WasmFlowApp::new()` to `new_with_loading()`
- [ ] Move component loading out of app creation
- [ ] Add `loading_state` and `splash_screen` fields to `WasmFlowApp`
- [ ] Add `start_async_component_loading()` method
- [ ] Modify `update()` to handle loading states
- [ ] Add state transition logic (Loading → Completed → Main UI)
- [ ] Defer graph loading until components ready
- [ ] Handle keyboard shortcuts during loading
- [ ] Add loading timeout warning (>60s)

**Deliverables**:
- Modified `src/main.rs` app creation
- Modified `src/ui/app.rs` with loading state machine
- Smooth transition from splash to main UI

**Success Criteria**:
- ✅ Splash screen shows immediately on startup
- ✅ Main UI only renders after loading completes
- ✅ Graph files wait for components
- ✅ Error states allow retry

---

### Phase 5: Cache Management UI & Tools ⏱️ 2 hours

**Objective**: Give users control over cache

**Tasks**:
- [ ] Add "Clear Component Cache" menu item in File menu
- [ ] Add cache clearing functionality to `src/ui/app/components.rs`
- [ ] Add "Cache Statistics" dialog (optional)
  - Cache location, component count, hit rate, disk usage
- [ ] Add manual cache refresh option
- [ ] Add `components/bin/.cache/` to `.gitignore`

**Deliverables**:
- Cache management menu items
- Cache statistics dialog (optional)
- Updated `.gitignore`

**Success Criteria**:
- ✅ Users can clear cache via menu
- ✅ Cache statistics accurate
- ✅ Cache directory not committed to git

---

### Phase 6: Testing & Performance Validation ⏱️ 2-3 hours

**Objective**: Ensure correctness and measure improvements

**Tasks**:
- [ ] Create `tests/component_cache_test.rs`
  - Test cache save/load
  - Test MD5 validation
  - Test cache invalidation on file change
  - Test corrupted cache handling
- [ ] Create `benches/component_loading.rs` (optional)
  - Benchmark uncached loading
  - Benchmark cached loading
- [ ] Manual testing:
  - Fresh install (no cache) → verify splash screen
  - Second launch (cached) → verify fast startup
  - Modify component → verify re-extraction
  - Delete cache → verify rebuild
- [ ] Measure performance metrics:
  - First startup time (uncached)
  - Subsequent startup time (cached)
  - Cache size on disk

**Deliverables**:
- Unit tests for cache functionality
- Performance benchmarks
- Test report with metrics

**Success Criteria**:
- ✅ All tests pass
- ✅ Cached loading ≥10x faster than uncached
- ✅ No memory leaks
- ✅ No race conditions

---

## Dependencies

### New Crate Dependencies
```toml
md5 = "0.7"              # MD5 checksum computation
chrono = "0.4"           # Timestamp tracking (optional)
```

### Existing Dependencies (No Changes)
- `egui`, `eframe` - UI framework
- `wasmtime` - WASM runtime
- `serde`, `serde_json` - Serialization

## File Changes Summary

### New Files (8)
```
src/runtime/component_cache.rs          # Phase 1
src/ui/loading_state.rs                 # Phase 2
src/ui/splash_screen.rs                 # Phase 3
tests/component_cache_test.rs           # Phase 6
benches/component_loading.rs            # Phase 6 (optional)

components/bin/.cache/                  # Runtime cache directory
├─ cache_version.txt
├─ <component>.json (×76)
└─ <component>.md5 (×76)
```

### Modified Files (7)
```
src/runtime/mod.rs                      # Phase 1: Export component_cache
src/runtime/wasm_host.rs                # Phase 1 & 2: Cache integration
src/ui/mod.rs                           # Phase 2 & 3: Export modules
src/ui/app.rs                           # Phase 2, 3, 4: Loading flow
src/ui/app/components.rs                # Phase 2: Async loading
src/main.rs                             # Phase 4: Startup flow
.gitignore                              # Phase 5: Ignore .cache/
```

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

## Performance Targets

| Metric | Before | After (Uncached) | After (Cached) |
|--------|--------|------------------|----------------|
| Startup Time | 15-30s | 15-30s | **2-5s** |
| UI Feedback | None | Immediate splash | Immediate splash |
| Cache Overhead | 0 MB | ~5 MB | ~5 MB |
| Component Count | 76 | 76 | 76 |

**Target**: 90% reduction in subsequent startup times via caching

## Success Metrics

- ✅ Cached loading ≥10x faster than uncached
- ✅ Splash screen visible within 1 second of launch
- ✅ Progress updates at ≥10 Hz (smooth animation)
- ✅ Zero crashes due to loading errors
- ✅ Cache size <10 MB for 76 components
- ✅ All unit tests pass

## Timeline

| Phase | Estimated Time | Cumulative |
|-------|----------------|------------|
| Phase 1: Cache System | 2-3 hours | 2-3 hours |
| Phase 2: Async Loading | 3-4 hours | 5-7 hours |
| Phase 3: Splash Screen | 3-4 hours | 8-11 hours |
| Phase 4: Integration | 2-3 hours | 10-14 hours |
| Phase 5: Cache Management | 2 hours | 12-16 hours |
| Phase 6: Testing | 2-3 hours | 14-19 hours |
| **Total** | **14-19 hours** | |

## Validation Checklist

After each phase:
- [ ] Code compiles without warnings
- [ ] Existing tests still pass
- [ ] New functionality manually tested
- [ ] No performance regressions
- [ ] Documentation updated

Final validation:
- [ ] All 6 phases complete
- [ ] Performance targets met
- [ ] No memory leaks
- [ ] Cache invalidation works
- [ ] User can clear cache
- [ ] Smooth UX during loading

## Documentation Updates

- [ ] Update `README.md` with faster startup times
- [ ] Add cache clearing instructions
- [ ] Update `ARCHITECTURE.md` with loading flow
- [ ] Add section to `CLAUDE.md` about component loading patterns

## Next Steps

1. **✅ Plan Review** - Review this plan, identify concerns
2. **→ Phase 1 Start** - Begin implementing component cache system
3. Continue through phases sequentially
4. Test after each phase
5. Final validation and documentation

---

**Status**: Ready for Phase 1 implementation
**Estimated Completion**: 14-19 hours total development time
