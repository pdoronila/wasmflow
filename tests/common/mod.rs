//! Common test utilities for WASI HTTP component testing
//!
//! This module provides shared helpers for integration and contract tests,
//! including wasmtime engine setup, linker configuration, and component state management.

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

/// Component state with WASI and WASI HTTP contexts
///
/// This structure implements both WasiView and WasiHttpView to provide
/// components with access to WASI core functions and HTTP capabilities.
pub struct ComponentState {
    pub wasi: WasiCtx,
    pub http: WasiHttpCtx,
    pub table: ResourceTable,
}

impl WasiView for ComponentState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for ComponentState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

/// Create a wasmtime engine configured for WASI HTTP component testing
///
/// Returns an Engine with:
/// - Component model support enabled
/// - Async execution support enabled
pub fn create_test_engine() -> anyhow::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    Engine::new(&config)
}

/// Create a linker with WASI and WASI HTTP support
///
/// The linker includes:
/// - WASI core functions (filesystem, stdio, clocks, etc.)
/// - WASI HTTP functions (outgoing requests)
pub fn create_test_linker(engine: &Engine) -> anyhow::Result<Linker<ComponentState>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
    Ok(linker)
}

/// Create component state with WASI and HTTP contexts
///
/// The state includes:
/// - WASI context with inherited stdio and network access
/// - WASI HTTP context
/// - Resource table for managing component resources
pub fn create_test_state() -> ComponentState {
    let wasi = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .inherit_network() // Required for HTTP access
        .build();

    let http = WasiHttpCtx::new();
    let table = ResourceTable::new();

    ComponentState { wasi, http, table }
}

/// Load a compiled WASM component from the filesystem
///
/// Returns None if the component file doesn't exist (allowing tests to skip gracefully)
#[allow(dead_code)] // Used in contract tests but not integration tests
pub fn load_component(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

/// Compile a WASM component bytecode into a wasmtime Component
///
/// This validates that the component conforms to the component model specification
#[allow(dead_code)] // Used in contract tests but not integration tests
pub fn compile_component(engine: &Engine, bytecode: &[u8]) -> anyhow::Result<Component> {
    Component::from_binary(engine, bytecode)
}

/// Create a complete test environment (engine, linker, store)
///
/// Returns a tuple of (Engine, Linker, Store) ready for component testing
pub fn create_test_environment(
) -> anyhow::Result<(Engine, Linker<ComponentState>, Store<ComponentState>)> {
    let engine = create_test_engine()?;
    let linker = create_test_linker(&engine)?;
    let state = create_test_state();
    let store = Store::new(&engine, state);
    Ok((engine, linker, store))
}

/// Path to the compiled HTTP Fetch example component
#[allow(dead_code)] // Used in contract tests but not integration tests
pub const HTTP_FETCH_COMPONENT_PATH: &str =
    "/Users/doronila/git/wasmflow_cc/examples/example-http-fetch/target/wasm32-wasip2/release/example_http_fetch.wasm";

/// Check if the HTTP Fetch component has been built
#[allow(dead_code)] // Used in contract tests but not integration tests
pub fn is_component_built() -> bool {
    std::path::Path::new(HTTP_FETCH_COMPONENT_PATH).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_engine() {
        let engine = create_test_engine();
        assert!(engine.is_ok(), "Should create engine successfully");
    }

    #[test]
    fn test_create_test_linker() {
        let engine = create_test_engine().unwrap();
        let linker = create_test_linker(&engine);
        assert!(linker.is_ok(), "Should create linker successfully");
    }

    #[test]
    fn test_create_test_state() {
        let _state = create_test_state();
        // If we get here without panicking, state creation works
    }

    #[tokio::test]
    async fn test_create_test_environment() {
        let result = create_test_environment();
        assert!(result.is_ok(), "Should create complete test environment");
    }
}
