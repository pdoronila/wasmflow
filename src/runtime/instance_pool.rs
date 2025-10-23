//! T082: WASM instance pooling for performance
//!
//! This module provides instance pooling to reuse WASM component instances,
//! significantly improving performance by avoiding repeated instantiation overhead.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::component::{Component, InstancePre, Linker};
use wasmtime::{Engine, Store};
use anyhow::Result;

use super::wasm_host::HostState;

/// Maximum number of instances to pool per component
const MAX_INSTANCES_PER_COMPONENT: usize = 10;

/// Pool of pre-instantiated WASM component instances
///
/// Maintains a pool of InstancePre objects for each component to avoid
/// repeated instantiation overhead. Uses a lazy eviction strategy.
pub struct InstancePool {
    /// Engine for creating instances
    #[allow(dead_code)]
    engine: Arc<Engine>,
    /// Linker for component instantiation
    linker: Arc<Mutex<Linker<HostState>>>,
    /// Pool of pre-instantiated instances per component ID
    pool: Arc<Mutex<HashMap<String, Vec<InstancePre<HostState>>>>>,
}

impl InstancePool {
    /// Create a new instance pool
    pub fn new(engine: Arc<Engine>, linker: Arc<Mutex<Linker<HostState>>>) -> Self {
        Self {
            engine,
            linker,
            pool: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a pre-instantiated instance from the pool, or create a new one
    ///
    /// If an instance is available in the pool, it will be returned immediately.
    /// Otherwise, a new InstancePre will be created from the component.
    pub fn get(
        &self,
        component_id: &str,
        component: &Component,
    ) -> Result<InstancePre<HostState>> {
        let mut pool = self.pool.lock().unwrap();

        // Try to get from pool
        if let Some(instances) = pool.get_mut(component_id) {
            if let Some(instance_pre) = instances.pop() {
                log::debug!("Reusing pooled instance for component: {}", component_id);
                return Ok(instance_pre);
            }
        }

        // No instance available, create a new one
        log::debug!("Creating new instance for component: {}", component_id);
        self.create_instance_pre(component)
    }

    /// Return an instance to the pool for reuse
    ///
    /// If the pool for this component is full, the instance is simply dropped.
    pub fn return_instance(&self, component_id: &str, instance_pre: InstancePre<HostState>) {
        let mut pool = self.pool.lock().unwrap();

        let instances = pool.entry(component_id.to_string()).or_default();

        // Only pool if we haven't reached the limit
        if instances.len() < MAX_INSTANCES_PER_COMPONENT {
            instances.push(instance_pre);
            log::debug!(
                "Returned instance to pool for component: {} (pool size: {})",
                component_id,
                instances.len()
            );
        } else {
            log::debug!(
                "Pool full for component: {}, dropping instance (limit: {})",
                component_id,
                MAX_INSTANCES_PER_COMPONENT
            );
            // Instance is dropped here, will be garbage collected
        }
    }

    /// Create a new pre-instantiated instance
    fn create_instance_pre(&self, component: &Component) -> Result<InstancePre<HostState>> {
        let linker = self.linker.lock().unwrap();
        let instance_pre = linker.instantiate_pre(component)?;
        Ok(instance_pre)
    }

    /// Clear all pooled instances for a specific component
    ///
    /// Useful when a component is reloaded or updated
    pub fn clear_component(&self, component_id: &str) {
        let mut pool = self.pool.lock().unwrap();
        if let Some(removed) = pool.remove(component_id) {
            log::info!(
                "Cleared {} pooled instances for component: {}",
                removed.len(),
                component_id
            );
        }
    }

    /// Clear all pooled instances
    pub fn clear_all(&self) {
        let mut pool = self.pool.lock().unwrap();
        let total_cleared: usize = pool.values().map(|v| v.len()).sum();
        pool.clear();
        log::info!("Cleared all pooled instances (total: {})", total_cleared);
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let pool = self.pool.lock().unwrap();
        let total_pooled: usize = pool.values().map(|v| v.len()).sum();
        let components_with_instances = pool.len();

        PoolStats {
            total_pooled_instances: total_pooled,
            components_with_instances,
            max_instances_per_component: MAX_INSTANCES_PER_COMPONENT,
        }
    }

    /// Instantiate an instance from an InstancePre with a store
    ///
    /// Helper method to convert InstancePre to an actual instance
    pub async fn instantiate_async<T>(
        instance_pre: &InstancePre<T>,
        store: &mut Store<T>,
    ) -> Result<wasmtime::component::Instance>
    where
        T: Send,
    {
        instance_pre.instantiate_async(store).await
    }
}

/// Statistics about the instance pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total number of instances currently pooled
    pub total_pooled_instances: usize,
    /// Number of components that have at least one pooled instance
    pub components_with_instances: usize,
    /// Maximum instances allowed per component
    pub max_instances_per_component: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Config;

    #[test]
    fn test_pool_creation() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Arc::new(Engine::new(&config).unwrap());
        let linker = Arc::new(Mutex::new(Linker::new(&engine)));

        let pool = InstancePool::new(engine, linker);
        let stats = pool.stats();

        assert_eq!(stats.total_pooled_instances, 0);
        assert_eq!(stats.components_with_instances, 0);
        assert_eq!(stats.max_instances_per_component, MAX_INSTANCES_PER_COMPONENT);
    }

    #[test]
    fn test_pool_stats() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Arc::new(Engine::new(&config).unwrap());
        let linker = Arc::new(Mutex::new(Linker::new(&engine)));

        let pool = InstancePool::new(engine, linker);

        // Initially empty
        let stats = pool.stats();
        assert_eq!(stats.total_pooled_instances, 0);

        // Clear all should not panic on empty pool
        pool.clear_all();
        let stats = pool.stats();
        assert_eq!(stats.total_pooled_instances, 0);
    }
}
