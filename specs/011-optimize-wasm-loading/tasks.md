# Implementation Tasks: Optimize WASM Component Loading

**Feature**: 011-optimize-wasm-loading
**Status**: Planning → Ready for Implementation
**Created**: 2025-10-25

## Task Overview

Total Tasks: 41
Estimated Time: 14-19 hours

---

## Phase 1: Component Metadata Cache System ⏱️ 2-3 hours

### Task 1.1: Setup Cache Module Structure
**Estimated**: 15 min
**Dependencies**: None

- [ ] Create `src/runtime/component_cache.rs` file
- [ ] Add module export to `src/runtime/mod.rs`
- [ ] Add `md5 = "0.7"` to `Cargo.toml` dependencies
- [ ] Add basic module structure and imports

**Verification**: `cargo build` succeeds

---

### Task 1.2: Implement Cache Directory Management
**Estimated**: 30 min
**Dependencies**: 1.1

- [ ] Define `ComponentCache` struct with `cache_dir: PathBuf`, `cache_version: String`
- [ ] Implement `ComponentCache::new(cache_dir: PathBuf) -> Result<Self>`
- [ ] Add directory creation logic (create if not exists)
- [ ] Add cache version file creation (`cache_version.txt` = "1.0")
- [ ] Add cache version validation (reject incompatible versions)

**Verification**: Cache directory created on first instantiation

---

### Task 1.3: Implement MD5 Checksum Computation
**Estimated**: 20 min
**Dependencies**: 1.2

- [ ] Implement `compute_checksum(wasm_path: &Path) -> Result<String>`
- [ ] Read WASM file bytes
- [ ] Compute MD5 hash using `md5` crate
- [ ] Return hex-encoded checksum string
- [ ] Add error handling for file read failures

**Verification**: Same file produces same checksum consistently

---

### Task 1.4: Implement Cache Validation Logic
**Estimated**: 30 min
**Dependencies**: 1.3

- [ ] Implement `is_cache_valid(wasm_path: &Path, cache_path: &Path) -> bool`
- [ ] Read cached MD5 from `.cache/<name>.md5`
- [ ] Compute current MD5 from WASM file
- [ ] Compare checksums
- [ ] Return true if match, false if mismatch or error
- [ ] Handle missing cache files gracefully

**Verification**: Returns false when WASM file changes, true otherwise

---

### Task 1.5: Implement Cache Serialization
**Estimated**: 30 min
**Dependencies**: 1.2

- [ ] Define `CachedComponentSpec` struct:
  ```rust
  struct CachedComponentSpec {
      version: String,
      checksum: String,
      cached_at: String,
      component_spec: ComponentSpec,
  }
  ```
- [ ] Implement `save_spec(wasm_path: &Path, spec: &ComponentSpec) -> Result<()>`
- [ ] Compute checksum
- [ ] Create `CachedComponentSpec` wrapper
- [ ] Serialize to JSON using `serde_json`
- [ ] Write to `.cache/<name>.json`
- [ ] Write checksum to `.cache/<name>.md5`

**Verification**: Cache files created with valid JSON

---

### Task 1.6: Implement Cache Loading
**Estimated**: 30 min
**Dependencies**: 1.4, 1.5

- [ ] Implement `get_cached_spec(wasm_path: &Path) -> Result<Option<ComponentSpec>>`
- [ ] Derive cache file path from WASM path
- [ ] Check if cache file exists
- [ ] Validate cache using `is_cache_valid()`
- [ ] If valid: deserialize JSON and return Some(ComponentSpec)
- [ ] If invalid: return None
- [ ] Handle corrupted JSON gracefully (log warning, return None)

**Verification**: Returns cached spec when valid, None when invalid

---

### Task 1.7: Implement Cache Invalidation
**Estimated**: 15 min
**Dependencies**: 1.2

- [ ] Implement `invalidate_all() -> Result<()>`
- [ ] Delete all files in cache directory
- [ ] Recreate cache version file
- [ ] Return error if deletion fails

**Verification**: Cache directory emptied (except cache_version.txt)

---

### Task 1.8: Add Error Handling and Logging
**Estimated**: 20 min
**Dependencies**: 1.1-1.7

- [ ] Add comprehensive error handling for all methods
- [ ] Add logging for cache hits/misses
- [ ] Add logging for cache invalidation events
- [ ] Add logging for errors (corrupted files, I/O failures)
- [ ] Add debug logging for checksum computation

**Verification**: Meaningful logs appear during cache operations

---

## Phase 2: Async Component Loading Infrastructure ⏱️ 3-4 hours

### Task 2.1: Setup Loading State Module
**Estimated**: 20 min
**Dependencies**: None

- [ ] Create `src/ui/loading_state.rs` file
- [ ] Add module export to `src/ui/mod.rs`
- [ ] Add basic imports (Arc, Mutex, etc.)

**Verification**: `cargo build` succeeds

---

### Task 2.2: Define Loading State Types
**Estimated**: 30 min
**Dependencies**: 2.1

- [ ] Define `LoadingState` enum:
  ```rust
  pub enum LoadingState {
      NotStarted,
      Loading { progress: Arc<Mutex<ComponentLoadProgress>> },
      Completed { total: usize, errors: Vec<String> },
      Failed { error: String },
  }
  ```
- [ ] Define `ComponentLoadProgress` struct:
  ```rust
  pub struct ComponentLoadProgress {
      pub total_components: usize,
      pub loaded_count: usize,
      pub current_component: Option<String>,
      pub errors: Vec<String>,
  }
  ```
- [ ] Implement `Default` for `ComponentLoadProgress`
- [ ] Add helper methods (percentage, is_complete, etc.)

**Verification**: Types compile and can be instantiated

---

### Task 2.3: Implement Async Component Loader Function
**Estimated**: 1.5 hours
**Dependencies**: 1.8, 2.2

- [ ] Create `async_component_loader()` function in `src/ui/app/components.rs`
- [ ] Accept parameters:
  - `components_dir: PathBuf`
  - `registry: Arc<Mutex<ComponentRegistry>>`
  - `component_manager: Arc<Mutex<ComponentManager>>`
  - `cache: Arc<ComponentCache>`
  - `progress: Arc<Mutex<ComponentLoadProgress>>`
- [ ] Scan `components_dir` for `.wasm` files
- [ ] Update `progress.total_components`
- [ ] For each WASM file:
  - [ ] Update `progress.current_component`
  - [ ] Try cache: `cache.get_cached_spec()`
  - [ ] If cached: use cached spec (log "cache hit")
  - [ ] If not cached: extract metadata and save to cache (log "cache miss")
  - [ ] Register spec in registry
  - [ ] Increment `progress.loaded_count`
  - [ ] Catch errors, add to `progress.errors`, continue
- [ ] Clear `progress.current_component` when done
- [ ] Log summary (total loaded, errors)

**Verification**: Function loads all components in background

---

### Task 2.4: Integrate Cache into ComponentManager
**Estimated**: 30 min
**Dependencies**: 1.8

- [ ] Modify `ComponentManager::load_component_sync()` in `src/runtime/wasm_host.rs`
- [ ] Add optional `cache: Option<&ComponentCache>` parameter
- [ ] If cache provided:
  - [ ] Try `cache.get_cached_spec(path)`
  - [ ] If Some(spec): return immediately (skip extraction)
- [ ] If no cache or cache miss:
  - [ ] Perform normal metadata extraction
  - [ ] If cache provided: `cache.save_spec(path, &spec)`
- [ ] Return ComponentSpec

**Verification**: Component loading uses cache when available

---

### Task 2.5: Add Progress Tracking to WasmFlowApp
**Estimated**: 30 min
**Dependencies**: 2.2

- [ ] Add fields to `WasmFlowApp` struct in `src/ui/app.rs`:
  ```rust
  loading_state: LoadingState,
  splash_screen: Option<SplashScreen>,
  pending_graph_load: Option<PathBuf>,
  ```
- [ ] Initialize in constructor:
  ```rust
  loading_state: LoadingState::NotStarted,
  splash_screen: None,
  pending_graph_load: None,
  ```
- [ ] Add `set_pending_graph_load(path: PathBuf)` method

**Verification**: App compiles with new fields

---

### Task 2.6: Implement start_async_component_loading Method
**Estimated**: 45 min
**Dependencies**: 2.3, 2.5

- [ ] Implement `WasmFlowApp::start_async_component_loading()` in `src/ui/app.rs`
- [ ] Create `ComponentLoadProgress` wrapped in `Arc<Mutex<>>`
- [ ] Create `SplashScreen` with progress reference (implemented in Phase 3)
- [ ] Get Arc clones of registry and component_manager
- [ ] Create `ComponentCache` instance
- [ ] Spawn thread calling `async_component_loader()`
- [ ] Set `self.loading_state = LoadingState::Loading { progress }`

**Verification**: Loading starts in background thread

---

### Task 2.7: Add Loading State Polling
**Estimated**: 20 min
**Dependencies**: 2.6

- [ ] Implement `WasmFlowApp::poll_loading_progress()` method
- [ ] Check if loading thread completed
- [ ] If complete:
  - [ ] Lock progress
  - [ ] Transition to `LoadingState::Completed { total, errors }`
  - [ ] Clear `self.splash_screen`
  - [ ] Return true
- [ ] If still loading: return false

**Verification**: State transitions from Loading to Completed

---

## Phase 3: Splash Screen UI Implementation ⏱️ 3-4 hours

### Task 3.1: Setup Splash Screen Module
**Estimated**: 15 min
**Dependencies**: None

- [ ] Create `src/ui/splash_screen.rs` file
- [ ] Add module export to `src/ui/mod.rs`
- [ ] Add imports (egui, Arc, Mutex, Instant, etc.)

**Verification**: `cargo build` succeeds

---

### Task 3.2: Define SplashScreen Struct
**Estimated**: 20 min
**Dependencies**: 3.1, 2.2

- [ ] Define `SplashScreen` struct:
  ```rust
  pub struct SplashScreen {
      progress: Arc<Mutex<ComponentLoadProgress>>,
      animation_start: Instant,
  }
  ```
- [ ] Implement `new(progress: Arc<Mutex<ComponentLoadProgress>>) -> Self`
- [ ] Initialize `animation_start` to current time

**Verification**: SplashScreen can be instantiated

---

### Task 3.3: Implement Title and Branding
**Estimated**: 30 min
**Dependencies**: 3.2

- [ ] Implement `render()` method skeleton
- [ ] Use `egui::CentralPanel::default().show()`
- [ ] Add vertical centered layout
- [ ] Add WasmFlow title:
  ```rust
  ui.heading(RichText::new("🌊 WasmFlow")
      .size(48.0)
      .color(Color32::from_rgb(100, 150, 255)));
  ```
- [ ] Add subtitle: "Visual Programming with WASM"
- [ ] Add spacing

**Verification**: Title displays centered

---

### Task 3.4: Implement Progress Bar
**Estimated**: 45 min
**Dependencies**: 3.3

- [ ] Lock progress
- [ ] Calculate fraction: `loaded_count / total_components`
- [ ] Add progress bar:
  ```rust
  ui.add(egui::ProgressBar::new(fraction)
      .desired_width(400.0)
      .show_percentage());
  ```
- [ ] Handle division by zero (total_components = 0)

**Verification**: Progress bar animates from 0% to 100%

---

### Task 3.5: Implement Component Count Display
**Estimated**: 20 min
**Dependencies**: 3.4

- [ ] Add label: "Loading components: X / Y (Z%)"
- [ ] Format with:
  - `progress.loaded_count`
  - `progress.total_components`
  - `fraction * 100.0`
- [ ] Style with appropriate font size and color

**Verification**: Count updates in real-time

---

### Task 3.6: Implement Current Component Display
**Estimated**: 15 min
**Dependencies**: 3.5

- [ ] Check `progress.current_component`
- [ ] If Some(name): display "Current: {name}"
- [ ] If None: display nothing or "Initializing..."
- [ ] Use secondary text color

**Verification**: Current component name visible

---

### Task 3.7: Implement Error Summary Display
**Estimated**: 20 min
**Dependencies**: 3.6

- [ ] Check `progress.errors.len()`
- [ ] If > 0: display "⚠ X components failed to load"
- [ ] Use warning color (yellow/orange)
- [ ] Add tooltip with first few error messages

**Verification**: Error count visible when errors occur

---

### Task 3.8: Implement Loading Spinner
**Estimated**: 30 min
**Dependencies**: 3.7

- [ ] Calculate elapsed time since `animation_start`
- [ ] Use `ui.spinner()` for default egui spinner, OR
- [ ] Implement custom rotating circle:
  ```rust
  let angle = elapsed.as_secs_f32() * 2.0 * PI;
  // Draw rotating arc or dots
  ```
- [ ] Position below error summary

**Verification**: Spinner animates continuously

---

### Task 3.9: Implement Completion Detection
**Estimated**: 15 min
**Dependencies**: 3.4

- [ ] At end of `render()`:
- [ ] Check if `progress.loaded_count >= progress.total_components`
- [ ] Return true if complete, false otherwise
- [ ] Handle edge case: total_components = 0

**Verification**: render() returns true when loading finishes

---

## Phase 4: Application Startup Flow Integration ⏱️ 2-3 hours

### Task 4.1: Refactor WasmFlowApp Constructor
**Estimated**: 45 min
**Dependencies**: 2.5

- [ ] Rename `WasmFlowApp::new()` to `new_with_loading()`
- [ ] Remove `reload_components()` call from constructor
- [ ] Keep builtin node registration
- [ ] Keep ExecutionEngine and ComponentRegistry creation
- [ ] Initialize new loading-related fields
- [ ] Update doc comments

**Verification**: App constructs without loading components

---

### Task 4.2: Modify Main Function Startup Flow
**Estimated**: 30 min
**Dependencies**: 4.1, 2.6

- [ ] Modify `main.rs` app creation closure
- [ ] Call `WasmFlowApp::new_with_loading(cc)`
- [ ] Call `app.start_async_component_loading()`
- [ ] If graph file provided: call `app.set_pending_graph_load(path)` instead of immediate load
- [ ] Return app

**Verification**: App starts with splash screen

---

### Task 4.3: Implement Loading State Handling in update()
**Estimated**: 1 hour
**Dependencies**: 3.9, 4.2

- [ ] Modify `WasmFlowApp::update()` in `src/ui/app.rs`
- [ ] Add state machine at start:
  ```rust
  match &self.loading_state {
      LoadingState::Loading { .. } => {
          // Render splash screen
      }
      LoadingState::Completed { .. } => {
          // Render main UI
      }
      LoadingState::Failed { error } => {
          // Render error screen
      }
      LoadingState::NotStarted => {
          // Should never happen, but handle gracefully
      }
  }
  ```
- [ ] For Loading: render splash, poll progress, request repaint
- [ ] For Completed: check once, transition to normal UI
- [ ] For Failed: show error with retry button

**Verification**: State transitions work correctly

---

### Task 4.4: Implement Splash Screen Transition
**Estimated**: 30 min
**Dependencies**: 4.3

- [ ] In Loading state:
- [ ] Call `self.splash_screen.as_mut().unwrap().render(ctx)`
- [ ] If render returns true:
  - [ ] Extract errors from progress
  - [ ] Set `self.loading_state = LoadingState::Completed { ... }`
  - [ ] Set `self.splash_screen = None`
  - [ ] Load pending graph if any: `self.pending_graph_load.take()`

**Verification**: Smooth transition to main UI after loading

---

### Task 4.5: Implement Error State UI
**Estimated**: 30 min
**Dependencies**: 4.3

- [ ] In Failed state:
- [ ] Show error message in center
- [ ] Add "Retry" button
- [ ] On retry click:
  - [ ] Reset state to NotStarted
  - [ ] Call `self.start_async_component_loading()`
- [ ] Add "Quit" button
- [ ] Style with appropriate error colors

**Verification**: Error state allows retry without restart

---

### Task 4.6: Handle Keyboard Shortcuts During Loading
**Estimated**: 20 min
**Dependencies**: 4.3

- [ ] In Loading state:
- [ ] Disable most keyboard shortcuts
- [ ] Allow Ctrl+Q / Cmd+Q to quit
- [ ] Allow Esc to cancel loading (optional)

**Verification**: Shortcuts work correctly in each state

---

### Task 4.7: Add Loading Timeout Warning
**Estimated**: 20 min
**Dependencies**: 4.3

- [ ] Track loading start time
- [ ] If loading > 60 seconds:
  - [ ] Display warning in splash screen
  - [ ] "Loading is taking unusually long. This may indicate a problem."
  - [ ] Offer cancel button (optional)

**Verification**: Warning appears after 60 seconds

---

## Phase 5: Cache Management UI & Tools ⏱️ 2 hours

### Task 5.1: Add Clear Cache Menu Item
**Estimated**: 30 min
**Dependencies**: 1.7

- [ ] Modify file menu rendering in `src/ui/app.rs`
- [ ] Add separator before "Quit"
- [ ] Add "Clear Component Cache" menu item
- [ ] On click:
  - [ ] Create ComponentCache instance
  - [ ] Call `cache.invalidate_all()`
  - [ ] Show success/error message
  - [ ] Optionally trigger reload

**Verification**: Menu item clears cache

---

### Task 5.2: Add Cache Statistics Display
**Estimated**: 1 hour
**Dependencies**: 5.1

- [ ] Add `show_cache_stats: bool` field to WasmFlowApp
- [ ] Add "Cache Statistics" menu item
- [ ] Implement statistics dialog:
  - [ ] Cache location path
  - [ ] Cached component count
  - [ ] Total disk space used
  - [ ] Last updated timestamp
- [ ] Add "Close" button
- [ ] Implement cache size calculation helper

**Verification**: Statistics dialog shows accurate info

---

### Task 5.3: Update .gitignore
**Estimated**: 5 min
**Dependencies**: None

- [ ] Add `components/bin/.cache/` to `.gitignore`
- [ ] Ensure cache files not committed

**Verification**: Git ignores cache directory

---

### Task 5.4: Add Manual Refresh Option
**Estimated**: 15 min
**Dependencies**: 2.6

- [ ] Ensure existing "Reload Components" menu item still works
- [ ] Modify to use async loading with progress
- [ ] Optionally: add "Rebuild Cache" that clears then reloads

**Verification**: Manual refresh works correctly

---

## Phase 6: Testing & Performance Validation ⏱️ 2-3 hours

### Task 6.1: Write ComponentCache Unit Tests
**Estimated**: 1 hour
**Dependencies**: 1.8

- [ ] Create `tests/component_cache_test.rs`
- [ ] Test: `test_cache_saves_and_loads_spec()`
- [ ] Test: `test_md5_detects_file_change()`
- [ ] Test: `test_corrupted_cache_rebuilt()`
- [ ] Test: `test_cache_versioning()`
- [ ] Test: `test_cache_directory_creation()`
- [ ] Add test fixtures (sample WASM files)

**Verification**: All cache tests pass

---

### Task 6.2: Write Loading State Unit Tests
**Estimated**: 30 min
**Dependencies**: 2.2

- [ ] Create `tests/loading_state_test.rs`
- [ ] Test: `test_progress_tracking()`
- [ ] Test: `test_thread_safe_updates()`
- [ ] Test: `test_error_collection()`
- [ ] Test: `test_state_transitions()`

**Verification**: All loading state tests pass

---

### Task 6.3: Write Integration Tests
**Estimated**: 1 hour
**Dependencies**: 4.7

- [ ] Create `tests/component_loading_integration.rs`
- [ ] Test: `test_first_startup_no_cache()`
- [ ] Test: `test_second_startup_with_cache()`
- [ ] Test: `test_modified_component_reloads()`
- [ ] Test: `test_partial_component_failure()`

**Verification**: All integration tests pass

---

### Task 6.4: Performance Benchmarking
**Estimated**: 45 min
**Dependencies**: 4.7

- [ ] Create `benches/component_loading.rs`
- [ ] Benchmark: `bench_load_76_components_uncached()`
- [ ] Benchmark: `bench_load_76_components_cached()`
- [ ] Benchmark: `bench_md5_computation()`
- [ ] Run benchmarks, record results
- [ ] Verify cached is ≥10x faster

**Verification**: Performance targets met

---

### Task 6.5: Manual Testing Checklist
**Estimated**: 1 hour
**Dependencies**: All phases

- [ ] Test: Fresh install (no cache)
- [ ] Test: Second launch (with cache)
- [ ] Test: Modify component, verify reload
- [ ] Test: Delete cache, verify rebuild
- [ ] Test: Corrupt cache file, verify handling
- [ ] Test: Launch with 0 components
- [ ] Test: Interrupt loading (Ctrl+C)
- [ ] Test: Clear cache via menu
- [ ] Test: View cache statistics
- [ ] Record any issues found

**Verification**: All manual tests pass

---

### Task 6.6: Memory Leak Testing
**Estimated**: 30 min
**Dependencies**: 4.7

- [ ] Run app with memory profiler (e.g., valgrind, heaptrack)
- [ ] Load components multiple times
- [ ] Verify no memory leaks
- [ ] Check steady-state memory usage

**Verification**: No leaks detected

---

## Final Tasks

### Task 7.1: Documentation Updates
**Estimated**: 30 min

- [ ] Update `README.md` with cache information
- [ ] Add cache location documentation
- [ ] Add troubleshooting section (cache issues)
- [ ] Update architecture docs with loading flow

**Verification**: Documentation complete and accurate

---

### Task 7.2: Code Cleanup and Review
**Estimated**: 30 min

- [ ] Remove debug logging if excessive
- [ ] Ensure consistent code style
- [ ] Add doc comments to public APIs
- [ ] Run `cargo clippy`, fix warnings
- [ ] Run `cargo fmt`

**Verification**: No clippy warnings, formatted

---

### Task 7.3: Final Integration Testing
**Estimated**: 30 min

- [ ] Build release version
- [ ] Test full workflow end-to-end
- [ ] Verify performance targets
- [ ] Check UI polish (animations, spacing, colors)
- [ ] Test on fresh install

**Verification**: All acceptance criteria met

---

### Task 7.4: Git Commit and Push
**Estimated**: 15 min

- [ ] Review all changes
- [ ] Create comprehensive commit message
- [ ] Commit all changes
- [ ] Push to branch: `claude/optimize-wasm-loading-011CUUY1vZZ4HuRLFoh5kzWx`

**Verification**: Changes pushed successfully

---

## Progress Tracking

**Phase 1**: ☐ Not Started | ☐ In Progress | ☐ Complete
**Phase 2**: ☐ Not Started | ☐ In Progress | ☐ Complete
**Phase 3**: ☐ Not Started | ☐ In Progress | ☐ Complete
**Phase 4**: ☐ Not Started | ☐ In Progress | ☐ Complete
**Phase 5**: ☐ Not Started | ☐ In Progress | ☐ Complete
**Phase 6**: ☐ Not Started | ☐ In Progress | ☐ Complete
**Final**: ☐ Not Started | ☐ In Progress | ☐ Complete

**Overall Progress**: 0 / 41 tasks complete (0%)

---

## Notes

- Tasks are intentionally granular for clear progress tracking
- Estimated times are approximate, adjust as needed
- Dependencies must be completed before dependent tasks
- Run `cargo test` after each phase
- Run `cargo clippy` regularly to catch issues early
- Update this file as tasks are completed

**Status**: Ready to begin Phase 1
**Next Task**: Task 1.1 - Setup Cache Module Structure
