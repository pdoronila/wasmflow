# Feature 011: Optimize WASM Component Loading

**Created**: 2025-10-25
**Status**: Planning
**Priority**: High - Performance & UX improvement

## Problem Statement

As the WASM component library grows (currently 76 components, 50KB-1MB each), application startup time increases significantly. The current implementation:

- **Blocks UI initialization**: All components loaded synchronously in `WasmFlowApp::new()`
- **No progress feedback**: Users see blank screen during 5-30 second load
- **No caching**: Metadata extracted from WASM bytecode on every application start
- **Sequential loading**: Components loaded one-by-one on single thread

**Current Flow** (from codebase analysis):
```
main.rs
  └─> eframe::run_native() with app creation closure
      └─> WasmFlowApp::new()
          └─> reload_components() [BLOCKING, SYNCHRONOUS]
              └─> For each .wasm file in components/bin/:
                  └─> load_component_sync()
                      └─> create_basic_spec_from_bytecode()
                          ├─> Component::from_binary()  [Parse WASM]
                          ├─> Instantiate component
                          └─> Extract metadata (get_info, get_inputs, get_outputs)
```

## Solution Overview

Implement two-tier optimization:

1. **Splash Screen with Progress Bar**
   - Show loading UI before main application window
   - Track and display component loading progress
   - Provide visual feedback during initial load

2. **Component Metadata Cache with Checksum Validation**
   - Cache extracted metadata to disk after first load
   - Use MD5 checksums to detect component changes
   - Skip metadata extraction for unchanged components
   - Dramatically reduce subsequent startup times

## Architecture Changes

### Current Architecture
```
Startup (Synchronous, Blocking)
├─> Create ComponentRegistry
├─> Create ExecutionEngine + ComponentManager
├─> Load all components (5-30 seconds, blocks UI)
└─> Show main UI

Component Loading (Per Component)
├─> Read .wasm file from disk (~50KB-1MB)
├─> Parse WASM binary (Component::from_binary)
├─> Instantiate component
├─> Call get_info(), get_inputs(), get_outputs()
└─> Register ComponentSpec in registry
```

### New Architecture
```
Startup (Async, Non-Blocking)
├─> Show splash screen with progress bar
├─> Create ComponentRegistry + ExecutionEngine
├─> Spawn background loading thread
│   ├─> For each .wasm file:
│   │   ├─> Compute MD5 checksum
│   │   ├─> Check cache (components/bin/.cache/<name>.json)
│   │   ├─> If cache valid: Load from cache (fast)
│   │   └─> If cache invalid: Extract metadata + update cache
│   └─> Send progress updates to UI thread
├─> UI thread receives updates, updates progress bar
└─> When loading complete: Transition to main UI

Cache Structure (components/bin/.cache/)
├─> <component-name>.json          # Cached ComponentSpec
├─> <component-name>.md5            # MD5 checksum of .wasm file
└─> cache_version.txt               # Cache format version for invalidation
```

## Implementation Phases

### Phase 1: Component Metadata Cache System

**Estimated Time**: 2-3 hours

**Files to Create**:
- `src/runtime/component_cache.rs` - Cache management logic

**Files to Modify**:
- `src/runtime/mod.rs` - Export new cache module
- `src/runtime/wasm_host.rs` - Add cache integration to `load_component_sync()`
- `Cargo.toml` - Add `md5` crate dependency

**Tasks**:
1. ✅ Create `ComponentCache` struct with cache directory management
2. ✅ Implement MD5 checksum computation for .wasm files
3. ✅ Implement cache serialization (ComponentSpec → JSON)
4. ✅ Implement cache validation (checksum comparison)
5. ✅ Implement cache loading/saving
6. ✅ Add cache directory initialization (`components/bin/.cache/`)
7. ✅ Add cache version tracking for format migrations
8. ✅ Add error handling for corrupt cache entries

**Cache File Format** (`components/bin/.cache/<name>.json`):
```json
{
  "version": "1.0",
  "checksum": "a1b2c3d4e5f6...",
  "cached_at": "2025-10-25T10:30:00Z",
  "component_spec": {
    "id": "user:echo",
    "name": "Echo",
    "version": "1.0.0",
    "description": "Echoes input to output",
    "author": "WasmFlow Core Library",
    "category": "General",
    "inputs": [...],
    "outputs": [...],
    "capabilities": []
  }
}
```

**API Design**:
```rust
pub struct ComponentCache {
    cache_dir: PathBuf,
    cache_version: String,
}

impl ComponentCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self>;

    pub fn get_cached_spec(
        &self,
        wasm_path: &Path,
    ) -> Result<Option<ComponentSpec>>;

    pub fn save_spec(
        &self,
        wasm_path: &Path,
        spec: &ComponentSpec,
    ) -> Result<()>;

    pub fn invalidate_all(&self) -> Result<()>;

    fn compute_checksum(wasm_path: &Path) -> Result<String>;
    fn is_cache_valid(wasm_path: &Path, cache_path: &Path) -> bool;
}
```

**Success Criteria**:
- ✅ Cache correctly saves/loads ComponentSpec
- ✅ MD5 checksums correctly detect file changes
- ✅ Invalid/corrupt cache entries handled gracefully
- ✅ Cache versioning allows future format updates

---

### Phase 2: Async Component Loading Infrastructure

**Estimated Time**: 3-4 hours

**Files to Create**:
- `src/ui/loading_state.rs` - Loading state tracking and progress

**Files to Modify**:
- `src/ui/mod.rs` - Export loading_state module
- `src/ui/app.rs` - Add loading state to WasmFlowApp
- `src/ui/app/components.rs` - Refactor to async loading with progress
- `src/runtime/wasm_host.rs` - Add async variant of load_component_sync

**Tasks**:
1. ✅ Create `LoadingState` enum (NotStarted, Loading, Completed, Failed)
2. ✅ Create `ComponentLoadProgress` struct with progress tracking
3. ✅ Create channel for progress updates (Arc<Mutex<ComponentLoadProgress>>)
4. ✅ Implement `reload_components_async()` with progress callbacks
5. ✅ Modify ComponentManager to use cache in `load_component_sync()`
6. ✅ Add thread-safe progress tracking (components loaded / total)
7. ✅ Add error aggregation (collect all load errors)
8. ✅ Handle component loading cancellation

**API Design**:
```rust
pub enum LoadingState {
    NotStarted,
    Loading { progress: Arc<Mutex<ComponentLoadProgress>> },
    Completed { total: usize, errors: Vec<String> },
    Failed { error: String },
}

pub struct ComponentLoadProgress {
    pub total_components: usize,
    pub loaded_count: usize,
    pub current_component: Option<String>,
    pub errors: Vec<String>,
}

impl WasmFlowApp {
    pub fn start_async_component_loading(&mut self) {
        // Spawn background thread that:
        // 1. Scans components/bin/
        // 2. For each .wasm: check cache → load/extract → register
        // 3. Updates progress atomically
        // 4. Sends completion signal
    }

    pub fn poll_loading_progress(&mut self) -> Option<LoadingState> {
        // Check if loading complete, update UI state
    }
}
```

**Loading Thread Design**:
```rust
fn async_component_loader(
    components_dir: PathBuf,
    registry: Arc<Mutex<ComponentRegistry>>,
    component_manager: Arc<Mutex<ComponentManager>>,
    cache: Arc<ComponentCache>,
    progress: Arc<Mutex<ComponentLoadProgress>>,
) {
    let entries = std::fs::read_dir(&components_dir)?;
    let wasm_files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension() == Some("wasm"))
        .collect();

    // Update total count
    {
        let mut p = progress.lock().unwrap();
        p.total_components = wasm_files.len();
    }

    for entry in wasm_files {
        let path = entry.path();
        let component_name = path.file_stem()?.to_string_lossy().to_string();

        // Update current component
        {
            let mut p = progress.lock().unwrap();
            p.current_component = Some(component_name.clone());
        }

        // Try cache first, then extract if needed
        let spec = match cache.get_cached_spec(&path)? {
            Some(spec) => {
                log::info!("Loaded {} from cache", component_name);
                spec
            }
            None => {
                log::info!("Extracting metadata for {}", component_name);
                let mut cm = component_manager.lock().unwrap();
                let spec = cm.load_component_sync(&path)?;
                cache.save_spec(&path, &spec)?;
                spec
            }
        };

        // Register component
        {
            let mut reg = registry.lock().unwrap();
            reg.register_component(spec)?;
        }

        // Update progress
        {
            let mut p = progress.lock().unwrap();
            p.loaded_count += 1;
        }
    }

    // Signal completion
    {
        let mut p = progress.lock().unwrap();
        p.current_component = None;
    }
}
```

**Success Criteria**:
- ✅ Components load in background thread without blocking UI
- ✅ Progress updates visible in real-time
- ✅ Cache integration reduces load time by 80%+ for cached components
- ✅ Errors collected and displayed without crashing

---

### Phase 3: Splash Screen UI Implementation

**Estimated Time**: 3-4 hours

**Files to Create**:
- `src/ui/splash_screen.rs` - Splash screen UI component

**Files to Modify**:
- `src/ui/mod.rs` - Export splash_screen module
- `src/ui/app.rs` - Add splash screen rendering logic
- `src/main.rs` - Modify app creation to show splash screen first

**Tasks**:
1. ✅ Design splash screen layout (logo/title + progress bar + status text)
2. ✅ Implement `SplashScreen` struct with egui rendering
3. ✅ Add progress bar with percentage (0-100%)
4. ✅ Add component count display ("Loaded 45/76 components")
5. ✅ Add current component name display ("Loading: http-fetch.wasm")
6. ✅ Add error summary display (if errors occur)
7. ✅ Add smooth transition to main UI when loading completes
8. ✅ Add loading spinner animation

**UI Design**:
```
╔════════════════════════════════════════════════╗
║                                                ║
║              🌊 WasmFlow v0.1.0                ║
║          Visual Programming with WASM          ║
║                                                ║
║  ┌─────────────────────────────────────────┐  ║
║  │████████████████░░░░░░░░░░░░░░░░░░░░░░░░│  ║ ← Progress bar (60%)
║  └─────────────────────────────────────────┘  ║
║                                                ║
║     Loading components: 45 / 76 (60%)         ║ ← Count & percentage
║     Current: http-fetch.wasm                   ║ ← Current component
║     ⚠ 2 components failed to load              ║ ← Error summary (if errors)
║                                                ║
║  [●] Loading...                                ║ ← Spinner animation
║                                                ║
╚════════════════════════════════════════════════╝
```

**API Design**:
```rust
pub struct SplashScreen {
    progress: Arc<Mutex<ComponentLoadProgress>>,
    animation_start: Instant,
}

impl SplashScreen {
    pub fn new(progress: Arc<Mutex<ComponentLoadProgress>>) -> Self;

    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        // Returns true when loading complete and ready to transition
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Title
                ui.heading(egui::RichText::new("🌊 WasmFlow")
                    .size(48.0)
                    .color(egui::Color32::from_rgb(100, 150, 255)));

                ui.add_space(20.0);

                // Progress bar
                let progress = self.progress.lock().unwrap();
                let fraction = if progress.total_components > 0 {
                    progress.loaded_count as f32 / progress.total_components as f32
                } else {
                    0.0
                };

                ui.add(egui::ProgressBar::new(fraction)
                    .desired_width(400.0)
                    .show_percentage());

                // Component count
                ui.label(format!(
                    "Loading components: {} / {} ({:.0}%)",
                    progress.loaded_count,
                    progress.total_components,
                    fraction * 100.0
                ));

                // Current component
                if let Some(name) = &progress.current_component {
                    ui.label(format!("Current: {}", name));
                }

                // Error summary
                if !progress.errors.is_empty() {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("⚠ {} components failed to load", progress.errors.len())
                    );
                }

                // Spinner
                self.render_spinner(ui);
            });
        });

        // Check if loading complete
        let progress = self.progress.lock().unwrap();
        progress.loaded_count >= progress.total_components
    }

    fn render_spinner(&self, ui: &mut egui::Ui) {
        // Animated spinning indicator
        let elapsed = self.animation_start.elapsed().as_secs_f32();
        let angle = elapsed * 2.0 * std::f32::consts::PI;
        ui.spinner();  // or custom rotating circle
    }
}
```

**Success Criteria**:
- ✅ Splash screen displays immediately on startup
- ✅ Progress bar updates smoothly during loading
- ✅ Component count and current component name visible
- ✅ Errors displayed clearly without blocking progress
- ✅ Smooth transition to main UI when complete

---

### Phase 4: Application Startup Flow Integration

**Estimated Time**: 2-3 hours

**Files to Modify**:
- `src/main.rs` - Modify app creation flow
- `src/ui/app.rs` - Refactor `WasmFlowApp::new()` and `update()`

**Tasks**:
1. ✅ Move component loading out of `WasmFlowApp::new()`
2. ✅ Create two-stage initialization:
   - Stage 1: Minimal app creation (registry, engine, UI state)
   - Stage 2: Async component loading with splash screen
3. ✅ Modify `update()` to handle loading state transitions
4. ✅ Add `is_loading()` check in update loop
5. ✅ Handle keyboard shortcuts during loading (disable most, allow quit)
6. ✅ Add loading timeout (warn if loading > 60 seconds)
7. ✅ Ensure graph files don't load until components ready

**New Startup Flow**:
```rust
// main.rs
fn main() -> Result<(), eframe::Error> {
    let args = parse_args();
    env_logger::init();

    eframe::run_native(
        "WasmFlow",
        options,
        Box::new(move |cc| {
            // Stage 1: Create app with splash screen
            let mut app = WasmFlowApp::new_with_loading(cc);

            // Stage 2: Start async component loading
            app.start_async_component_loading();

            // Load graph file after components ready (deferred)
            if let Some(path) = graph_file {
                app.set_pending_graph_load(path);
            }

            Ok(Box::new(app))
        }),
    )
}

// app.rs
impl WasmFlowApp {
    pub fn new_with_loading(cc: &eframe::CreationContext<'_>) -> Self {
        let registry = ComponentRegistry::new();
        register_constant_nodes(&mut registry);
        register_wasm_creator_node(&mut registry);
        // ... register builtins ...

        let engine = ExecutionEngine::new();
        let graph = NodeGraph::new("Untitled".to_string(), "User".to_string());

        // Create with loading state
        Self {
            graph,
            registry: Arc::new(Mutex::new(registry)),
            engine,
            loading_state: LoadingState::NotStarted,
            splash_screen: None,
            pending_graph_load: None,
            // ... other fields ...
        }
    }

    pub fn start_async_component_loading(&mut self) {
        let progress = Arc::new(Mutex::new(ComponentLoadProgress::default()));
        self.splash_screen = Some(SplashScreen::new(progress.clone()));

        // Spawn loading thread
        let registry = self.registry.clone();
        let component_manager = self.engine.component_manager().clone();
        let cache = Arc::new(ComponentCache::new("components/bin/.cache")?);

        std::thread::spawn(move || {
            async_component_loader(
                "components/bin".into(),
                registry,
                component_manager,
                cache,
                progress,
            );
        });

        self.loading_state = LoadingState::Loading { progress };
    }
}

impl eframe::App for WasmFlowApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Check loading state first
        match &self.loading_state {
            LoadingState::Loading { .. } => {
                // Render splash screen
                if let Some(splash) = &mut self.splash_screen {
                    let complete = splash.render(ctx);

                    if complete {
                        // Transition to main UI
                        self.loading_state = LoadingState::Completed { ... };
                        self.splash_screen = None;

                        // Load pending graph if any
                        if let Some(path) = self.pending_graph_load.take() {
                            self.load_graph_from_path(path);
                        }
                    }
                }

                ctx.request_repaint();  // Keep animating
            }
            LoadingState::Completed { .. } | LoadingState::NotStarted => {
                // Normal UI rendering
                self.render_status_bar(ctx);
                self.render_palette(ctx);
                self.render_canvas(ctx);
                // ... rest of update logic ...
            }
            LoadingState::Failed { error } => {
                // Show error screen
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, format!("Failed to load components: {}", error));
                    if ui.button("Retry").clicked() {
                        self.start_async_component_loading();
                    }
                });
            }
        }
    }
}
```

**Success Criteria**:
- ✅ App shows splash screen immediately on startup
- ✅ Main UI only renders after loading completes
- ✅ Graph files don't load until components ready
- ✅ Keyboard shortcuts work correctly based on loading state
- ✅ Error state allows retry without restarting app

---

### Phase 5: Cache Management UI & Tools

**Estimated Time**: 2 hours

**Files to Modify**:
- `src/ui/app.rs` - Add cache management menu
- `src/ui/app/components.rs` - Add cache clearing functionality

**Tasks**:
1. ✅ Add "Clear Component Cache" menu item
2. ✅ Add cache statistics display (cache size, hit rate)
3. ✅ Add manual cache refresh option
4. ✅ Add cache location display
5. ✅ Add .cache/ directory to .gitignore

**Menu Structure**:
```
File
├─ New Graph
├─ Open...
├─ Save
├─ Save As...
├─ Recent Files ▶
├─ ─────────────
├─ Reload Components          ← Existing
├─ Clear Component Cache       ← NEW: Deletes .cache/ directory
├─ Cache Statistics...         ← NEW: Shows cache info dialog
├─ ─────────────
└─ Quit
```

**Cache Statistics Dialog**:
```
╔═══════════════════════════════════════╗
║  Component Cache Statistics           ║
╟───────────────────────────────────────╢
║  Cache Location:                      ║
║    components/bin/.cache/             ║
║                                       ║
║  Cached Components:  72 / 76          ║
║  Cache Hit Rate:     94.7%            ║
║  Disk Space Used:    4.2 MB           ║
║  Last Updated:       2025-10-25 10:30 ║
║                                       ║
║  [ Clear Cache ]  [ Rebuild Cache ]   ║
║                            [ Close ]  ║
╚═══════════════════════════════════════╝
```

**Success Criteria**:
- ✅ Users can clear cache via menu
- ✅ Cache statistics accurate and helpful
- ✅ Cache directory excluded from git
- ✅ Manual rebuild works correctly

---

### Phase 6: Testing & Performance Validation

**Estimated Time**: 2-3 hours

**Files to Create**:
- `tests/component_cache_test.rs` - Cache functionality tests
- `benches/component_loading.rs` - Performance benchmarks

**Tasks**:
1. ✅ Test cache save/load correctness
2. ✅ Test MD5 checksum validation
3. ✅ Test cache invalidation on file change
4. ✅ Test concurrent loading (multiple threads)
5. ✅ Test corrupted cache handling
6. ✅ Benchmark loading times (cached vs uncached)
7. ✅ Test with 0, 10, 50, 76 components
8. ✅ Test memory usage during loading

**Test Cases**:
```rust
#[test]
fn test_cache_saves_and_loads_spec() {
    let cache = ComponentCache::new("test_cache")?;
    let wasm_path = Path::new("components/bin/echo.wasm");

    // First load: cache miss
    let spec1 = load_component_with_cache(&cache, wasm_path)?;

    // Second load: cache hit
    let spec2 = load_component_with_cache(&cache, wasm_path)?;

    assert_eq!(spec1, spec2);
}

#[test]
fn test_cache_invalidates_on_file_change() {
    let cache = ComponentCache::new("test_cache")?;
    let wasm_path = Path::new("components/bin/echo.wasm");

    // Load and cache
    let spec1 = load_component_with_cache(&cache, wasm_path)?;

    // Modify file (change timestamp)
    std::fs::write(wasm_path, modified_bytecode)?;

    // Reload: should detect change and re-extract
    let spec2 = load_component_with_cache(&cache, wasm_path)?;

    assert_ne!(spec1.checksum, spec2.checksum);
}

#[bench]
fn bench_loading_76_components_uncached(b: &mut Bencher) {
    b.iter(|| {
        // Clear cache
        std::fs::remove_dir_all("components/bin/.cache")?;

        // Load all 76 components
        let mut registry = ComponentRegistry::new();
        reload_all_components(&mut registry)?;
    });
}

#[bench]
fn bench_loading_76_components_cached(b: &mut Bencher) {
    // Pre-populate cache
    let cache = ComponentCache::new("components/bin/.cache")?;
    populate_cache(&cache)?;

    b.iter(|| {
        let mut registry = ComponentRegistry::new();
        reload_all_components(&mut registry)?;
    });
}
```

**Performance Targets**:
- ✅ **Uncached load (first startup)**: ≤30 seconds for 76 components
- ✅ **Cached load (subsequent startups)**: ≤3 seconds for 76 components
- ✅ **Cache overhead**: <5MB disk space
- ✅ **Memory usage**: No leaks, steady state after loading

**Success Criteria**:
- ✅ All tests pass
- ✅ Cached loading is 10x faster than uncached
- ✅ No memory leaks detected
- ✅ Concurrent loading works without race conditions

---

## File Structure

### New Files
```
src/runtime/component_cache.rs          # Phase 1: Cache implementation
src/ui/loading_state.rs                 # Phase 2: Loading state tracking
src/ui/splash_screen.rs                 # Phase 3: Splash screen UI
tests/component_cache_test.rs           # Phase 6: Cache tests
benches/component_loading.rs            # Phase 6: Performance benchmarks

components/bin/.cache/                  # Cache directory (gitignored)
├─ cache_version.txt                    # "1.0"
├─ echo.json                            # Cached ComponentSpec
├─ echo.md5                             # MD5 checksum
├─ string_trim.json
├─ string_trim.md5
└─ ... (76 components × 2 files each)
```

### Modified Files
```
src/runtime/mod.rs                      # Phase 1: Export component_cache
src/runtime/wasm_host.rs                # Phase 1 & 2: Cache integration
src/ui/mod.rs                           # Phase 2 & 3: Export new modules
src/ui/app.rs                           # Phase 2, 3, 4: Loading flow
src/ui/app/components.rs                # Phase 2: Async loading
src/main.rs                             # Phase 4: Startup flow
Cargo.toml                              # Phase 1: Add md5 dependency
.gitignore                              # Phase 5: Ignore .cache/
```

## Dependencies

### New Crate Dependencies
```toml
[dependencies]
md5 = "0.7"              # Phase 1: MD5 checksum computation
serde_json = "1.0"       # Phase 1: Cache serialization (already present)
chrono = "0.4"           # Phase 1: Timestamp tracking
```

## Risk Assessment

### High Risk
- **Breaking change to startup flow**: Requires careful refactoring of `WasmFlowApp::new()`
  - *Mitigation*: Extensive testing, feature flag for rollback

### Medium Risk
- **Cache corruption**: Invalid cache files could cause load failures
  - *Mitigation*: Robust error handling, automatic cache invalidation

- **Thread safety**: Concurrent access to registry during loading
  - *Mitigation*: Use Arc<Mutex<>> for shared state, test concurrent scenarios

### Low Risk
- **Disk space**: Cache files add ~5MB overhead
  - *Mitigation*: Negligible, cache is optional and clearable

## Testing Strategy

### Unit Tests (Phase 6)
- ComponentCache: save, load, invalidate, versioning
- MD5 checksum: computation, comparison
- Progress tracking: thread-safe updates
- Loading state transitions

### Integration Tests (Phase 6)
- Full loading flow with real components
- Cache hit/miss scenarios
- Concurrent loading stress test
- Error recovery (corrupted cache, missing files)

### Manual Testing
- ✅ Fresh install (no cache) → verify splash screen shows
- ✅ Second launch (with cache) → verify fast startup
- ✅ Modify component → verify re-extraction
- ✅ Delete cache → verify rebuild
- ✅ Interrupt loading → verify graceful handling

## Performance Metrics

### Before Optimization (Current)
- **First startup**: 15-30 seconds (synchronous, blocking)
- **Subsequent startups**: 15-30 seconds (no caching)
- **UI feedback**: None (blank screen)
- **Component count**: 76 WASM files

### After Optimization (Target)
- **First startup (uncached)**: 15-30 seconds (with progress bar visible within 1 second)
- **Subsequent startups (cached)**: 2-5 seconds (90% improvement)
- **UI feedback**: Immediate splash screen, real-time progress
- **Cache overhead**: ~5MB disk space

### Success Criteria
- ✅ Cached loading is ≥10x faster than uncached
- ✅ Splash screen visible within 1 second of launch
- ✅ Progress updates at ≥10 Hz (smooth animation)
- ✅ Zero crashes due to loading errors

## Rollback Plan

If issues arise during deployment:

1. **Feature flag**: Add `--no-cache` CLI flag to disable caching
2. **Environment variable**: `WASMFLOW_DISABLE_CACHE=1` to force fresh loads
3. **Automatic fallback**: If cache loading fails, fall back to direct extraction
4. **Manual recovery**: Users can delete `.cache/` directory to reset

## Documentation Updates

### User-Facing
- **README.md**: Document faster startup times, cache location
- **TROUBLESHOOTING.md**: Add cache clearing instructions

### Developer-Facing
- **ARCHITECTURE.md**: Document loading flow, cache design
- **CLAUDE.md**: Add component loading patterns section
- **This file**: Serves as implementation guide

## Acceptance Criteria

### Phase 1: Component Metadata Cache
- ✅ Cache correctly saves/loads ComponentSpec to/from JSON
- ✅ MD5 checksums accurately detect file changes
- ✅ Corrupted cache entries handled without crashing
- ✅ Cache version tracking prevents format mismatches

### Phase 2: Async Loading
- ✅ Components load in background thread
- ✅ Progress updates visible in real-time
- ✅ Cache integration reduces load time by ≥80%
- ✅ All errors collected and reported

### Phase 3: Splash Screen
- ✅ Splash screen displays within 1 second of launch
- ✅ Progress bar updates smoothly (≥10 Hz)
- ✅ Component count and current component name displayed
- ✅ Smooth transition to main UI

### Phase 4: Startup Integration
- ✅ App initializes with splash screen first
- ✅ Main UI only renders after loading completes
- ✅ Graph files load correctly after components ready
- ✅ Error states allow retry

### Phase 5: Cache Management
- ✅ Users can clear cache via menu
- ✅ Cache statistics accurate
- ✅ Manual rebuild works

### Phase 6: Testing
- ✅ All unit tests pass
- ✅ Cached loading ≥10x faster than uncached
- ✅ No memory leaks
- ✅ Concurrent loading works

## Timeline Estimate

| Phase | Description | Time | Dependencies |
|-------|-------------|------|--------------|
| 1 | Component Metadata Cache | 2-3 hours | None |
| 2 | Async Loading Infrastructure | 3-4 hours | Phase 1 |
| 3 | Splash Screen UI | 3-4 hours | Phase 2 |
| 4 | Startup Integration | 2-3 hours | Phase 2, 3 |
| 5 | Cache Management UI | 2 hours | Phase 1 |
| 6 | Testing & Validation | 2-3 hours | All phases |
| **Total** | | **14-19 hours** | |

## Next Steps

1. **Review this plan** - Verify approach, identify concerns
2. **Phase 1 implementation** - Build cache system first (most critical)
3. **Phase 2 implementation** - Add async loading with progress
4. **Phase 3 implementation** - Create splash screen UI
5. **Phase 4 integration** - Wire everything together
6. **Phase 5 polish** - Add cache management features
7. **Phase 6 validation** - Test thoroughly, measure performance
8. **Documentation** - Update guides, record lessons learned

---

**Status**: Ready for implementation
**Next Action**: Begin Phase 1 - Component Metadata Cache System
