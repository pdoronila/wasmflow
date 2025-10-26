# Feature Specification: Optimize WASM Component Loading

**Feature ID**: 011-optimize-wasm-loading
**Created**: 2025-10-25
**Author**: User
**Status**: Planning

## Problem Statement

As the WasmFlow component library grows (currently 76 components), application startup time has become noticeably slow:

- **15-30 second startup delay** before UI becomes interactive
- **No visual feedback** during loading (blank screen)
- **Repeated metadata extraction** on every application launch
- **Sequential loading** blocks UI thread

This creates a poor user experience, especially for frequent restarts during development.

## User Stories

### US1: Immediate Visual Feedback
**As a** WasmFlow user
**I want to** see a loading screen immediately when I launch the application
**So that** I know the application is starting and can see progress

**Acceptance Criteria**:
- Splash screen appears within 1 second of launch
- Loading progress bar shows percentage (0-100%)
- Current component name visible during loading
- Component count displayed (e.g., "45/76 components")

### US2: Fast Subsequent Startups
**As a** WasmFlow user
**I want** subsequent application launches to be fast
**So that** I can quickly iterate during development without waiting

**Acceptance Criteria**:
- First startup: 15-30 seconds (uncached, shows progress)
- Subsequent startups: 2-5 seconds (cached, 90% faster)
- Cache automatically invalidates when components change
- No manual intervention required for cache management

### US3: Graceful Error Handling
**As a** WasmFlow user
**I want** to see clear errors if components fail to load
**So that** I can understand and fix loading issues

**Acceptance Criteria**:
- Failed components don't block entire loading process
- Error summary displayed on splash screen (e.g., "3 components failed")
- Detailed errors available in status bar after loading
- Option to retry loading without restarting application

### US4: Cache Management
**As a** WasmFlow developer
**I want to** manually clear the component cache
**So that** I can force a fresh reload when debugging

**Acceptance Criteria**:
- "Clear Component Cache" menu item in File menu
- Cache statistics available (count, size, hit rate)
- Cache directory documented and accessible
- Cache automatically rebuilds after clearing

## Functional Requirements

### FR1: Splash Screen with Progress Tracking
The application must display a loading screen with:
- **Progress bar**: Visual indicator of loading progress (0-100%)
- **Component count**: "Loaded X/Y components"
- **Current component**: Name of component currently being loaded
- **Error summary**: Count of failed components (if any)
- **Loading animation**: Spinner or similar visual feedback
- **Branding**: WasmFlow logo/title for professional appearance

### FR2: Component Metadata Cache
The application must cache component metadata to disk:
- **Cache location**: `components/bin/.cache/`
- **Cache format**: JSON files per component (e.g., `echo.json`)
- **Checksum validation**: MD5 hash to detect file changes
- **Automatic invalidation**: Re-extract metadata when WASM file changes
- **Version tracking**: Cache format version for future migrations
- **Error resilience**: Corrupted cache entries handled gracefully

### FR3: Asynchronous Component Loading
The application must load components in background:
- **Non-blocking**: Loading doesn't block UI thread
- **Progress updates**: Real-time progress sent to UI thread
- **Thread-safe**: Concurrent access to registry handled safely
- **Error collection**: All errors aggregated and reported
- **Deferred graph loading**: Graph files wait until components ready

### FR4: Cache Management UI
The application must provide cache management:
- **Menu item**: "Clear Component Cache" in File menu
- **Cache statistics**: Display cache info (count, size, hit rate)
- **Manual refresh**: Force reload all components
- **Documentation**: Cache location and purpose explained

## Non-Functional Requirements

### NFR1: Performance
- **First startup (uncached)**: ≤30 seconds for 76 components
- **Subsequent startups (cached)**: ≤5 seconds for 76 components
- **Cache overhead**: ≤10 MB disk space
- **Memory usage**: No leaks, steady state after loading
- **Progress updates**: ≥10 Hz refresh rate for smooth animation

### NFR2: Reliability
- **Error tolerance**: Failed components don't crash app
- **Cache corruption**: Invalid cache entries automatically rebuilt
- **Thread safety**: No race conditions during concurrent loading
- **Timeout protection**: Warn if loading takes >60 seconds

### NFR3: Maintainability
- **Clear separation**: Cache, loading, UI in separate modules
- **Testing**: Unit tests for cache, integration tests for loading
- **Documentation**: Code comments, architecture docs, user guides
- **Rollback**: Feature flag to disable caching if needed

### NFR4: Scalability
- **Component growth**: Supports 100+ components without major changes
- **Cache efficiency**: O(1) lookup, O(n) initial build
- **Concurrent loading**: Can parallelize in future (Phase 7+)

## Technical Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     WasmFlowApp                             │
│  ┌───────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ LoadingState  │  │ SplashScreen │  │ ComponentCache  │  │
│  │ (state mgmt)  │  │ (UI render)  │  │ (persistence)   │  │
│  └───────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
│          │                 │                   │            │
│          │                 │                   │            │
│  ┌───────▼─────────────────▼───────────────────▼────────┐  │
│  │          Background Loading Thread                    │  │
│  │  1. Scan components/bin/                              │  │
│  │  2. For each .wasm:                                   │  │
│  │     - Compute MD5 checksum                            │  │
│  │     - Check cache (ComponentCache)                    │  │
│  │     - Load from cache OR extract metadata             │  │
│  │     - Register in ComponentRegistry                   │  │
│  │     - Update LoadingState progress                    │  │
│  │  3. Signal completion                                 │  │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
         │                                  │
         │ Updates progress                 │ Persists cache
         ▼                                  ▼
┌────────────────────┐          ┌──────────────────────────┐
│ UI Thread (egui)   │          │ File System              │
│ - Renders splash   │          │ components/bin/.cache/   │
│ - Shows progress   │          │ ├─ <name>.json (spec)    │
│ - Transitions to   │          │ └─ <name>.md5 (checksum) │
│   main UI          │          └──────────────────────────┘
└────────────────────┘
```

### Data Flow

**Startup Sequence**:
```
1. main() → eframe::run_native()
2. App creation closure:
   a. WasmFlowApp::new_with_loading(cc)
      - Create ComponentRegistry (empty)
      - Create ExecutionEngine + ComponentManager
      - Register builtin nodes
      - Initialize LoadingState::NotStarted
   b. start_async_component_loading()
      - Create ComponentLoadProgress (Arc<Mutex<>>)
      - Create SplashScreen
      - Spawn background loading thread
      - Set LoadingState::Loading
3. update() loop:
   a. If LoadingState::Loading:
      - Render splash screen
      - Check progress updates
      - Request repaint
   b. If LoadingState::Completed:
      - Transition to main UI
      - Load pending graph file (if any)
   c. If LoadingState::Failed:
      - Show error screen
      - Offer retry button
```

**Component Loading Flow**:
```
Background thread:
  For each .wasm file in components/bin/:
    1. Compute MD5: md5::compute(file_bytes)
    2. Check cache:
       - cache_path = .cache/<name>.json
       - cached_md5 = .cache/<name>.md5
       - If cached_md5 == current_md5:
         → Load ComponentSpec from cache (fast, ~1ms)
       - Else:
         → Extract metadata from WASM (slow, ~100-500ms)
         → Save to cache
    3. Register: registry.register_component(spec)
    4. Update progress:
       - progress.loaded_count += 1
       - progress.current_component = Some(name)
```

### Module Breakdown

**New Modules**:

1. **`src/runtime/component_cache.rs`**
   - `ComponentCache` struct
   - Methods: `new()`, `get_cached_spec()`, `save_spec()`, `invalidate_all()`
   - Private: `compute_checksum()`, `is_cache_valid()`

2. **`src/ui/loading_state.rs`**
   - `LoadingState` enum: NotStarted, Loading, Completed, Failed
   - `ComponentLoadProgress` struct: total, loaded, current, errors
   - Thread-safe via Arc<Mutex<>>

3. **`src/ui/splash_screen.rs`**
   - `SplashScreen` struct
   - Methods: `new()`, `render()` → bool (returns true when complete)
   - Private: `render_spinner()`

**Modified Modules**:

4. **`src/ui/app.rs`**
   - Add fields: `loading_state`, `splash_screen`, `pending_graph_load`
   - Modify: `new()` → `new_with_loading()`
   - Add: `start_async_component_loading()`, `poll_loading_progress()`
   - Modify: `update()` to handle loading states

5. **`src/ui/app/components.rs`**
   - Modify: `reload_components()` → keep for manual reload
   - Add: `reload_components_async()` with progress callback
   - Add: `async_component_loader()` thread function

6. **`src/runtime/wasm_host.rs`**
   - Modify: `load_component_sync()` to accept optional cache
   - Add cache lookup before metadata extraction

### Cache File Format

**`components/bin/.cache/cache_version.txt`**:
```
1.0
```

**`components/bin/.cache/<component-name>.json`**:
```json
{
  "version": "1.0",
  "checksum": "a1b2c3d4e5f6789...",
  "cached_at": "2025-10-25T10:30:00Z",
  "component_spec": {
    "id": "user:echo",
    "name": "Echo",
    "version": "1.0.0",
    "description": "Echoes input to output",
    "author": "WasmFlow Core Library",
    "category": "General",
    "inputs": [
      {
        "name": "input",
        "value_type": "StringVal",
        "default_value": null
      }
    ],
    "outputs": [
      {
        "name": "output",
        "value_type": "StringVal"
      }
    ],
    "capabilities": []
  }
}
```

**`components/bin/.cache/<component-name>.md5`**:
```
a1b2c3d4e5f6789abcdef1234567890
```

## User Interface Mockups

### Splash Screen (Loading)
```
╔════════════════════════════════════════════════╗
║                                                ║
║              🌊 WasmFlow v0.1.0                ║
║          Visual Programming with WASM          ║
║                                                ║
║  ┌─────────────────────────────────────────┐  ║
║  │████████████████████░░░░░░░░░░░░░░░░░░░░│  ║ ← 65% complete
║  └─────────────────────────────────────────┘  ║
║                                                ║
║     Loading components: 49 / 76 (65%)         ║
║     Current: http-fetch.wasm                   ║
║                                                ║
║     [●] Loading...                             ║ ← Animated spinner
║                                                ║
╚════════════════════════════════════════════════╝
```

### Splash Screen (Errors)
```
╔════════════════════════════════════════════════╗
║                                                ║
║              🌊 WasmFlow v0.1.0                ║
║          Visual Programming with WASM          ║
║                                                ║
║  ┌─────────────────────────────────────────┐  ║
║  │███████████████████████████████████████░│  ║ ← 99% complete
║  └─────────────────────────────────────────┘  ║
║                                                ║
║     Loading components: 73 / 76 (96%)         ║
║     ⚠ 3 components failed to load              ║ ← Error summary
║                                                ║
║     [●] Completing...                          ║
║                                                ║
╚════════════════════════════════════════════════╝
```

### Cache Statistics Dialog (Optional)
```
╔═══════════════════════════════════════╗
║  Component Cache Statistics           ║
╟───────────────────────────────────────╢
║  Cache Location:                      ║
║    components/bin/.cache/             ║
║                                       ║
║  Cached Components:  73 / 76          ║
║  Cache Hit Rate:     96.1%            ║
║  Disk Space Used:    4.8 MB           ║
║  Last Updated:       2025-10-25 10:30 ║
║                                       ║
║  [ Clear Cache ]  [ Rebuild Cache ]   ║
║                            [ Close ]  ║
╚═══════════════════════════════════════╝
```

## Edge Cases & Error Handling

### EC1: Corrupted Cache File
**Scenario**: `.cache/echo.json` is malformed JSON
**Handling**:
- Log warning: "Invalid cache for echo, rebuilding"
- Delete corrupted file
- Extract fresh metadata
- Save new cache entry
- Continue loading other components

### EC2: Missing .cache Directory
**Scenario**: First launch or user deleted cache
**Handling**:
- Create `.cache/` directory
- Create `cache_version.txt`
- Extract all component metadata
- Save all cache entries
- Normal startup (slightly slower)

### EC3: Component WASM File Changed
**Scenario**: User rebuilt a component
**Handling**:
- Compute new MD5: `new_md5`
- Compare with cached MD5: `cached_md5`
- If `new_md5 != cached_md5`:
  - Extract fresh metadata
  - Update cache entry
  - Update MD5 file
- Continue normally

### EC4: Component Load Timeout
**Scenario**: Metadata extraction takes >10 seconds
**Handling**:
- Log warning: "Component X taking unusually long"
- Continue waiting (no hard timeout)
- If >60 seconds total for all components:
  - Show warning in splash screen
  - Allow user to cancel or continue

### EC5: Thread Panic During Loading
**Scenario**: Background thread panics
**Handling**:
- Catch panic with `std::panic::catch_unwind()`
- Set `LoadingState::Failed { error }`
- Show error screen with retry button
- Don't crash main application

### EC6: Partial Component Registry
**Scenario**: 70/76 components loaded, 6 failed
**Handling**:
- Continue with loaded components
- Show warning: "70/76 components loaded (6 failed)"
- Main UI functions normally
- Missing components show as unavailable in palette

## Testing Strategy

### Unit Tests

**ComponentCache Tests** (`tests/component_cache_test.rs`):
```rust
#[test]
fn test_cache_saves_and_loads_spec()
fn test_md5_detects_file_change()
fn test_corrupted_cache_rebuilt()
fn test_cache_versioning()
fn test_cache_directory_creation()
```

**LoadingState Tests** (`tests/loading_state_test.rs`):
```rust
#[test]
fn test_progress_tracking()
fn test_thread_safe_updates()
fn test_error_collection()
fn test_state_transitions()
```

### Integration Tests

**Full Loading Flow** (`tests/component_loading_integration.rs`):
```rust
#[test]
fn test_first_startup_no_cache()
fn test_second_startup_with_cache()
fn test_modified_component_reloads()
fn test_cache_invalidation()
fn test_concurrent_loading()
```

### Manual Testing Checklist

- [ ] Fresh install (no cache) → splash screen shows → components load
- [ ] Second launch → fast startup with cached metadata
- [ ] Modify a component → verify re-extraction on next launch
- [ ] Delete `.cache/` → verify rebuild
- [ ] Corrupt a cache file → verify graceful handling
- [ ] Launch with 0 components → verify no crash
- [ ] Launch with 100+ components → verify performance
- [ ] Interrupt loading (Ctrl+C) → verify graceful shutdown

### Performance Benchmarks

**Benchmark Suite** (`benches/component_loading.rs`):
```rust
#[bench]
fn bench_load_76_components_uncached(b: &mut Bencher)
fn bench_load_76_components_cached(b: &mut Bencher)
fn bench_md5_computation(b: &mut Bencher)
fn bench_cache_serialization(b: &mut Bencher)
```

**Target Metrics**:
- Uncached: ~15-30 seconds (baseline)
- Cached: ~2-5 seconds (90% improvement)
- MD5 computation: <10ms per component
- Cache serialization: <5ms per component

## Security Considerations

### S1: Cache Poisoning
**Threat**: Malicious actor replaces cache files
**Mitigation**: MD5 checksum validation, cache entries ignored if checksum mismatch

### S2: Arbitrary File Access
**Threat**: Cache reads files outside `components/bin/.cache/`
**Mitigation**: Validate all paths, restrict to cache directory only

### S3: Disk Space Exhaustion
**Threat**: Cache grows unbounded
**Mitigation**: Cache size limited by component count (~5MB for 76 components)

## Future Enhancements (Out of Scope)

### FE1: Parallel Component Loading
Load multiple components concurrently using thread pool (Phase 7+)

### FE2: Incremental Loading
Show partial UI with loaded components, add more as they load (Phase 8+)

### FE3: Remote Component Cache
Share cache across machines via network storage (Phase 9+)

### FE4: Lazy Component Loading
Load components on-demand when added to graph (Phase 10+)

### FE5: Component Preloading Hints
`.preload` file lists frequently used components to prioritize (Phase 11+)

## Success Criteria Summary

### Must Have (Required)
- ✅ Splash screen displays within 1 second of launch
- ✅ Progress bar shows real-time loading progress
- ✅ Cached loading is ≥10x faster than uncached
- ✅ Component changes detected and cache invalidated
- ✅ Errors don't crash application

### Should Have (Important)
- ✅ Cache management UI (clear cache, statistics)
- ✅ Error summary visible on splash screen
- ✅ Smooth animation during loading
- ✅ Professional splash screen design

### Could Have (Nice to Have)
- Cache statistics dialog with detailed info
- Loading timeout warnings
- Retry button on error screen

## Open Questions

1. **Q: Should we cache builtin components?**
   A: No, builtins are fast to register, skip caching

2. **Q: What if cache format changes in future versions?**
   A: Use `cache_version.txt`, invalidate old versions automatically

3. **Q: Should we compress cache files?**
   A: Not initially, JSON is human-readable for debugging

4. **Q: Should we parallelize component loading?**
   A: Phase 7+, current focus is sequential with caching

## References

- **Current Implementation**: `src/ui/app/components.rs:42-102`
- **ComponentManager**: `src/runtime/wasm_host.rs:264-510`
- **ExecutionEngine**: `src/runtime/engine.rs:16-60`
- **WasmFlowApp**: `src/ui/app.rs:34-196`

---

**Status**: Specification Complete, Ready for Implementation
**Next Step**: Begin Phase 1 - Component Metadata Cache System
