//! Continuous execution manager for long-running nodes
//!
//! Manages lifecycle of continuous nodes that run indefinitely until stopped.
//! Uses background threads with tokio runtimes for async execution.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::graph::{NodeGraph, NodeValue};
use crate::runtime::wasm_host::ComponentManager;
use crate::ContinuousNodeError;

// Static cache for HTTP server listener executors
// This allows the TcpListener to persist across execution iterations
use once_cell::sync::Lazy;
use std::collections::HashMap as StdHashMap;
use crate::builtin::{HttpServerListenerExecutor, TimePartitionedSchedulerExecutor};

static HTTP_EXECUTORS: Lazy<Arc<Mutex<StdHashMap<Uuid, HttpServerListenerExecutor>>>> =
    Lazy::new(|| Arc::new(Mutex::new(StdHashMap::new())));

// Static cache for scheduler executors
// This allows the scheduler state to persist across execution iterations
static SCHEDULER_EXECUTORS: Lazy<Arc<Mutex<StdHashMap<Uuid, TimePartitionedSchedulerExecutor>>>> =
    Lazy::new(|| Arc::new(Mutex::new(StdHashMap::new())));

// Helper function to safely read interval from graph
fn read_interval_from_graph(graph: &Arc<Mutex<NodeGraph>>, node_id: Uuid) -> u64 {
    if let Ok(graph_lock) = graph.lock() {
        if let Some(node) = graph_lock.nodes.get(&node_id) {
            if let Some(interval_port) = node.inputs.iter().find(|p| p.name == "interval") {
                if let Some(NodeValue::U32(interval)) = &interval_port.current_value {
                    return *interval as u64;
                }
            }
        }
    }
    100 // Default 100ms
}

/// Commands sent from UI thread to execution manager
#[derive(Debug)]
pub enum ControlMessage {
    /// Begin continuous execution for a node
    Start { node_id: Uuid },
    /// Request graceful shutdown for a node
    Stop { node_id: Uuid },
    /// Stop all continuous nodes (app shutdown)
    Shutdown,
}

/// Results sent from execution tasks back to UI thread
#[derive(Debug)]
pub enum ExecutionResult {
    /// Execution began successfully
    Started {
        node_id: Uuid,
        timestamp: std::time::Instant,
    },
    /// Execution stopped cleanly
    Stopped {
        node_id: Uuid,
        iterations: u64,
        duration: Duration,
    },
    /// New output values available
    OutputsUpdated {
        node_id: Uuid,
        outputs: HashMap<String, NodeValue>,
    },
    /// Execution failed
    Error {
        node_id: Uuid,
        error: ContinuousNodeError,
    },
    /// One cycle finished (for monitoring/debugging)
    IterationComplete {
        node_id: Uuid,
        iteration: u64,
        duration: Duration,
    },
}

/// Information about a running continuous node task
struct ContinuousNodeTask {
    /// Cancellation token for graceful shutdown
    cancellation_token: CancellationToken,
    /// Thread handle for the execution task
    join_handle: Option<JoinHandle<()>>,
    /// Started timestamp (for future use in monitoring)
    #[allow(dead_code)]
    started_at: Instant,
}

impl Drop for ContinuousNodeTask {
    fn drop(&mut self) {
        // Ensure cancellation on drop
        self.cancellation_token.cancel();
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Continuous execution manager
pub struct ContinuousExecutionManager {
    /// Active tasks (node_id -> task info)
    active_tasks: HashMap<Uuid, ContinuousNodeTask>,
}

impl ContinuousExecutionManager {
    /// Create a new execution manager
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
        }
    }

    /// Start continuous execution for a node
    /// T044: Updated to accept Arc<Mutex<NodeGraph>> for reactive input changes
    pub fn start_node(
        &mut self,
        node_id: Uuid,
        graph: Arc<Mutex<NodeGraph>>,
        component_manager: Arc<Mutex<ComponentManager>>,
        result_tx: Sender<ExecutionResult>,
    ) -> Result<(), ContinuousNodeError> {
        // Check if already running
        if self.active_tasks.contains_key(&node_id) {
            return Err(ContinuousNodeError::ExecutionFailed {
                node_id,
                node_name: "".to_string(),
                message: "Node is already running".to_string(),
                source_location: None,
                timestamp: chrono::Utc::now(),
            });
        }

        // IMPORTANT: Clean up stale HTTP executors from cache before starting
        // This handles cases where:
        // 1. The app crashed or the previous stop didn't complete properly
        // 2. Nodes were deleted but their executors remained in the cache
        // 3. Any orphaned executors from nodes that no longer exist
        if let Ok(mut executors) = HTTP_EXECUTORS.lock() {
            log::debug!("Checking for stale HTTP executors before starting node {}. Current cache size: {}", node_id, executors.len());

            // First, clean up the specific executor for this node if it exists
            if let Some(executor) = executors.get(&node_id) {
                log::warn!(
                    "Found stale HTTP server listener executor for node {} - cleaning up before start",
                    node_id
                );
                executor.shutdown();
                thread::sleep(Duration::from_millis(50)); // Give OS time to release port
            }
            executors.remove(&node_id);

            // Second, clean up any orphaned executors (executors whose nodes are not running)
            // This handles the case where nodes were deleted but executors remained in cache
            let active_node_ids: std::collections::HashSet<Uuid> =
                self.active_tasks.keys().copied().collect();

            log::debug!("Active node count: {}. Checking {} cached executors for orphans", active_node_ids.len(), executors.len());

            let mut orphaned_ids = Vec::new();
            for executor_node_id in executors.keys() {
                if *executor_node_id != node_id && !active_node_ids.contains(executor_node_id) {
                    orphaned_ids.push(*executor_node_id);
                }
            }

            if !orphaned_ids.is_empty() {
                log::warn!("Found {} orphaned HTTP server listener executors", orphaned_ids.len());
            }

            for orphaned_id in orphaned_ids {
                log::warn!(
                    "Found orphaned HTTP server listener executor for non-active node {} - cleaning up",
                    orphaned_id
                );
                if let Some(executor) = executors.get(&orphaned_id) {
                    executor.shutdown();
                    thread::sleep(Duration::from_millis(50)); // Give OS time to release port
                }
                executors.remove(&orphaned_id);
            }

            log::debug!("Executor cache size after cleanup: {}", executors.len());
        }

        let cancellation_token = CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let started_at = Instant::now();

        // Spawn execution thread
        let join_handle = thread::spawn(move || {
            Self::execution_loop(node_id, graph, component_manager, token_clone, result_tx);
        });

        // Track the task
        self.active_tasks.insert(
            node_id,
            ContinuousNodeTask {
                cancellation_token,
                join_handle: Some(join_handle),
                started_at,
            },
        );

        // T062: Log successful start
        log::info!("Started continuous execution for node {}", node_id);

        Ok(())
    }

    /// Stop continuous execution for a node
    pub fn stop_node(&mut self, node_id: Uuid) -> Result<(), ContinuousNodeError> {
        if let Some(mut task) = self.active_tasks.remove(&node_id) {
            // Phase 1: Request graceful cancellation
            task.cancellation_token.cancel();

            // Phase 2: Wait with timeout (1.5s)
            if let Some(handle) = task.join_handle.take() {
                let timeout = Duration::from_millis(1500);
                let start = Instant::now();

                // Try to join with timeout
                while start.elapsed() < timeout {
                    if handle.is_finished() {
                        let _ = handle.join();

                        // Cleanup: Explicitly shutdown and remove HTTP executor from cache
                        // This ensures the TcpListener is properly closed and the port is released
                        if let Ok(mut executors) = HTTP_EXECUTORS.lock() {
                            if let Some(executor) = executors.get(&node_id) {
                                log::debug!("Shutting down HTTP server listener executor for node {}", node_id);
                                executor.shutdown();
                                // Give the OS a moment to fully release the port (especially on macOS)
                                drop(executors); // Release lock before sleeping
                                thread::sleep(Duration::from_millis(50));
                                // Re-acquire lock to remove from cache
                                if let Ok(mut executors) = HTTP_EXECUTORS.lock() {
                                    if let Some(_) = executors.remove(&node_id) {
                                        log::info!("Removed HTTP server listener executor for node {} from cache", node_id);
                                    }
                                }
                            } else if let Some(_) = executors.remove(&node_id) {
                                log::info!("Removed HTTP server listener executor for node {} from cache (no shutdown needed)", node_id);
                            }
                        }

                        // T062: Log successful graceful shutdown
                        log::info!(
                            "Continuous node {} stopped gracefully in {:?}",
                            node_id,
                            start.elapsed()
                        );
                        return Ok(());
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                // Phase 3: Force abort if still running
                log::warn!(
                    "Node {} did not stop gracefully, forcing termination",
                    node_id
                );
                // Thread will be dropped, which terminates it

                // Cleanup even if forced termination
                if let Ok(mut executors) = HTTP_EXECUTORS.lock() {
                    if let Some(executor) = executors.get(&node_id) {
                        log::debug!("Shutting down HTTP server listener executor for node {} (forced)", node_id);
                        executor.shutdown();
                        // Give the OS a moment to fully release the port (especially on macOS)
                        drop(executors); // Release lock before sleeping
                        thread::sleep(Duration::from_millis(50));
                        // Re-acquire lock to remove from cache
                        if let Ok(mut executors) = HTTP_EXECUTORS.lock() {
                            if let Some(_) = executors.remove(&node_id) {
                                log::info!("Removed HTTP server listener executor for node {} from cache (forced)", node_id);
                            }
                        }
                    } else if let Some(_) = executors.remove(&node_id) {
                        log::info!("Removed HTTP server listener executor for node {} from cache (forced, no shutdown needed)", node_id);
                    }
                }
            }

            Ok(())
        } else {
            Err(ContinuousNodeError::ExecutionFailed {
                node_id,
                node_name: "".to_string(),
                message: "Node is not running".to_string(),
                source_location: None,
                timestamp: chrono::Utc::now(),
            })
        }
    }

    /// Shutdown all continuous nodes
    pub fn shutdown(&mut self) {
        log::info!("Shutting down {} continuous nodes", self.active_tasks.len());

        // Cancel all tasks
        for (node_id, task) in self.active_tasks.drain() {
            task.cancellation_token.cancel();
            log::info!("Cancelled continuous node {}", node_id);
        }

        // Cleanup all HTTP server listener executors
        // This ensures all TcpListeners are properly closed and ports are released
        if let Ok(mut executors) = HTTP_EXECUTORS.lock() {
            let count = executors.len();
            // Explicitly shutdown each executor before clearing
            for (node_id, executor) in executors.iter() {
                log::debug!("Shutting down HTTP server listener executor for node {} (app shutdown)", node_id);
                executor.shutdown();
            }
            executors.clear();
            if count > 0 {
                log::info!("Shutdown and removed {} HTTP server listener executors", count);
            }
        }
    }

    /// Execution loop for a single continuous node
    /// T039-T040: Enhanced with panic catching and error handling
    /// T044: Updated to use Arc<Mutex<NodeGraph>> for reactive input reading
    fn execution_loop(
        node_id: Uuid,
        graph: Arc<Mutex<NodeGraph>>,
        _component_manager: Arc<Mutex<ComponentManager>>,
        cancellation_token: CancellationToken,
        result_tx: Sender<ExecutionResult>,
    ) {
        // T040: Wrap entire execution in panic catcher
        let execution_result = catch_unwind(AssertUnwindSafe(|| {
            // Create tokio runtime for async operations
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    // T039: Send error if runtime creation fails
                    let _ = result_tx.send(ExecutionResult::Error {
                        node_id,
                        error: ContinuousNodeError::ExecutionFailed {
                            node_id,
                            node_name: "".to_string(),
                            message: format!("Failed to create tokio runtime: {}", e),
                            source_location: None,
                            timestamp: chrono::Utc::now(),
                        },
                    });
                    return;
                }
            };

            let started_at = Instant::now();
            let mut iterations = 0u64;

            // Send started notification
            let _ = result_tx.send(ExecutionResult::Started {
                node_id,
                timestamp: started_at,
            });

            // Track previous interval for change detection
            let mut previous_interval_ms = 100u64;

            rt.block_on(async {
                while !cancellation_token.is_cancelled() {
                    let iteration_start = Instant::now();

                    iterations += 1;

                    // T044-T045: Fetch latest interval from graph each iteration (reactive to input changes)
                    let interval_ms = read_interval_from_graph(&graph, node_id);

                    // T051: Log interval changes
                    if interval_ms != previous_interval_ms {
                        log::debug!(
                            "Continuous node {} interval changed: {}ms -> {}ms",
                            node_id,
                            previous_interval_ms,
                            interval_ms
                        );
                        previous_interval_ms = interval_ms;
                    }

                    // T040: Wrap iteration in panic catcher
                    let iteration_result = catch_unwind(AssertUnwindSafe(|| {
                        // Determine which continuous node type this is and execute accordingly
                        let component_id = if let Ok(graph_lock) = graph.lock() {
                            graph_lock
                                .nodes
                                .get(&node_id)
                                .map(|n| n.component_id.clone())
                        } else {
                            None
                        };

                        match component_id.as_deref() {
                            Some("builtin:continuous:timer") => {
                                // Timer node: generate counter and elapsed time
                                let elapsed_seconds = started_at.elapsed().as_secs_f32();
                                let mut outputs = HashMap::new();
                                outputs.insert(
                                    "counter".to_string(),
                                    NodeValue::U32(iterations as u32),
                                );
                                outputs.insert(
                                    "elapsed_seconds".to_string(),
                                    NodeValue::F32(elapsed_seconds),
                                );
                                Ok(outputs)
                            }
                            Some("builtin:continuous:http-server-listener") => {
                                // HTTP Server Listener: execute listener to accept connections
                                // IMPORTANT: Use the module-level static executor so the TcpListener persists across iterations
                                use crate::runtime::engine::NodeExecutor;

                                // Helper function to resolve inputs from graph (we'll call this twice)
                                let resolve_inputs = || -> HashMap<String, NodeValue> {
                                    if let Ok(graph_lock) = graph.lock() {
                                        let mut inputs = HashMap::new();

                                        log::debug!("🔍 Resolving inputs for HTTP Listener (iteration {})", iterations);

                                        // First, resolve connections to get values from other nodes (for response, connection_id)
                                        for connection in graph_lock.connections.iter() {
                                            if connection.to_node == node_id {
                                                // Get the source node's output value
                                                if let Some(source_node) = graph_lock.nodes.get(&connection.from_node) {
                                                    log::debug!("  Found connection FROM: {}", source_node.display_name);
                                                    if let Some(source_port) = source_node.outputs.iter().find(|p| p.id == connection.from_port) {
                                                        log::debug!("    Source port '{}' has value: {:?}", source_port.name, source_port.current_value.is_some());
                                                        if let Some(value) = &source_port.current_value {
                                                            // Map to target input port name
                                                            if let Some(target_node) = graph_lock.nodes.get(&node_id) {
                                                                if let Some(target_port) = target_node.inputs.iter().find(|p| p.id == connection.to_port) {
                                                                    log::debug!("    Mapping to target port '{}': {:?}", target_port.name, value);
                                                                    inputs.insert(target_port.name.clone(), value.clone());
                                                                }
                                                            }
                                                        } else {
                                                            log::debug!("    Source port '{}' has NO value!", source_port.name);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Second, get node's direct input port values for unconnected inputs (for constants like host, port)
                                        if let Some(node) = graph_lock.nodes.get(&node_id) {
                                            for input in &node.inputs {
                                                if !inputs.contains_key(&input.name) {
                                                    if let Some(value) = &input.current_value {
                                                        inputs.insert(input.name.clone(), value.clone());
                                                    }
                                                }
                                            }
                                        }

                                        inputs
                                    } else {
                                        HashMap::new()
                                    }
                                };

                                // Resolve inputs with FRESH values right before execution
                                // This ensures we get the latest response value from the processing chain
                                let inputs = resolve_inputs();

                                // Debug logging for response input
                                if let Some(response) = inputs.get("response") {
                                    log::debug!("HTTP Listener iteration {}: response input = {:?}", iterations, response);
                                } else {
                                    log::debug!("HTTP Listener iteration {}: NO response input", iterations);
                                }

                                // Get or create persistent executor for this node
                                let mut executors = HTTP_EXECUTORS.lock().unwrap();
                                let is_new = !executors.contains_key(&node_id);
                                let executor = executors
                                    .entry(node_id)
                                    .or_insert_with(|| {
                                        log::info!("Creating new HTTP server listener executor for node {}", node_id);
                                        HttpServerListenerExecutor::new()
                                    });

                                if is_new {
                                    log::debug!("First execution iteration for HTTP server listener node {}", node_id);
                                } else {
                                    log::trace!("Reusing existing HTTP server listener executor for node {}", node_id);
                                }

                                executor.execute(&inputs).map_err(|e| e.to_string())
                            }
                            Some("builtin:continuous:combiner") => {
                                // T050: Combiner node: fetch inputs from connected nodes and execute
                                if let Ok(graph_lock) = graph.lock() {
                                    // Collect input values by resolving connections
                                    let mut inputs = HashMap::new();

                                    // Find all incoming connections to this node
                                    for connection in graph_lock.connections.iter() {
                                        if connection.to_node == node_id {
                                            // Get the source node's output value
                                            if let Some(source_node) =
                                                graph_lock.nodes.get(&connection.from_node)
                                            {
                                                if let Some(source_port) = source_node
                                                    .outputs
                                                    .iter()
                                                    .find(|p| p.id == connection.from_port)
                                                {
                                                    if let Some(value) = &source_port.current_value
                                                    {
                                                        // Map to target input port name
                                                        if let Some(target_node) =
                                                            graph_lock.nodes.get(&node_id)
                                                        {
                                                            if let Some(target_port) =
                                                                target_node.inputs.iter().find(
                                                                    |p| p.id == connection.to_port,
                                                                )
                                                            {
                                                                inputs.insert(
                                                                    target_port.name.clone(),
                                                                    value.clone(),
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // For unconnected inputs, use default values from the port
                                    if let Some(node) = graph_lock.nodes.get(&node_id) {
                                        for port in &node.inputs {
                                            if !inputs.contains_key(&port.name) {
                                                if let Some(value) = &port.current_value {
                                                    inputs.insert(port.name.clone(), value.clone());
                                                }
                                            }
                                        }
                                    }

                                    // Execute the combiner logic
                                    use crate::builtin::ContinuousCombinerExecutor;
                                    use crate::runtime::engine::NodeExecutor;

                                    let executor = ContinuousCombinerExecutor;
                                    executor.execute(&inputs).map_err(|e| e.to_string())
                                } else {
                                    Err("Failed to lock graph".to_string())
                                }
                            }
                            Some("builtin:continuous:scheduler") => {
                                // Time-Partitioned Scheduler: execute tasks with deterministic budgets
                                use crate::runtime::engine::NodeExecutor;

                                // Resolve inputs from graph connections
                                let inputs = if let Ok(graph_lock) = graph.lock() {
                                    let mut resolved_inputs = HashMap::new();

                                    // Get all input values for the scheduler node
                                    if let Some(node) = graph_lock.nodes.get(&node_id) {
                                        for input_port in &node.inputs {
                                            // Try to get value from connections first
                                            let mut found_connection = false;
                                            for connection in graph_lock.connections.iter() {
                                                if connection.to_node == node_id
                                                    && connection.to_port == input_port.id
                                                {
                                                    // Get source node's output value
                                                    if let Some(source_node) =
                                                        graph_lock.nodes.get(&connection.from_node)
                                                    {
                                                        if let Some(source_port) = source_node
                                                            .outputs
                                                            .iter()
                                                            .find(|p| p.id == connection.from_port)
                                                        {
                                                            if let Some(value) =
                                                                &source_port.current_value
                                                            {
                                                                resolved_inputs.insert(
                                                                    input_port.name.clone(),
                                                                    value.clone(),
                                                                );
                                                                found_connection = true;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // If no connection, use port's current value
                                            if !found_connection {
                                                if let Some(value) = &input_port.current_value {
                                                    resolved_inputs
                                                        .insert(input_port.name.clone(), value.clone());
                                                }
                                            }
                                        }
                                    }

                                    resolved_inputs
                                } else {
                                    HashMap::new()
                                };

                                // Get or create persistent executor for this node
                                let mut executors = SCHEDULER_EXECUTORS.lock().unwrap();
                                let executor = executors.entry(node_id).or_insert_with(|| {
                                    log::info!(
                                        "Creating new scheduler executor for node {}",
                                        node_id
                                    );
                                    // Create scheduler with ComponentManager for real execution
                                    TimePartitionedSchedulerExecutor::with_component_manager(
                                        _component_manager.clone()
                                    )
                                });

                                executor.execute(&inputs).map_err(|e| e.to_string())
                            }
                            _ => {
                                // Unknown continuous node type
                                Err(format!("Unknown continuous node type: {:?}", component_id))
                            }
                        }
                    }));

                    // T039: Handle iteration errors
                    match iteration_result {
                        Ok(Ok(outputs)) => {
                            // Execution succeeded - send outputs
                            let _ = result_tx
                                .send(ExecutionResult::OutputsUpdated { node_id, outputs });
                        }
                        Ok(Err(exec_error)) => {
                            // Execution returned an error
                            log::error!(
                                "Continuous node {} execution failed: {}",
                                node_id,
                                exec_error
                            );

                            let _ = result_tx.send(ExecutionResult::Error {
                                node_id,
                                error: ContinuousNodeError::ExecutionFailed {
                                    node_id,
                                    node_name: "".to_string(),
                                    message: exec_error,
                                    source_location: None,
                                    timestamp: chrono::Utc::now(),
                                },
                            });
                            return; // Exit the loop on execution error
                        }
                        Err(panic_info) => {
                            // T040: Panic caught - send error and stop
                            let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                                s.to_string()
                            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                                s.clone()
                            } else {
                                "Unknown panic".to_string()
                            };

                            log::error!("Continuous node {} panicked: {}", node_id, panic_msg);

                            let _ = result_tx.send(ExecutionResult::Error {
                                node_id,
                                error: ContinuousNodeError::ExecutionFailed {
                                    node_id,
                                    node_name: "".to_string(),
                                    message: format!("Component panicked: {}", panic_msg),
                                    source_location: None,
                                    timestamp: chrono::Utc::now(),
                                },
                            });
                            return; // Exit the loop on panic
                        }
                    }

                    // Wait for the specified interval
                    tokio::time::sleep(Duration::from_millis(interval_ms)).await;

                    let iteration_duration = iteration_start.elapsed();

                    // Send iteration complete notification
                    let _ = result_tx.send(ExecutionResult::IterationComplete {
                        node_id,
                        iteration: iterations,
                        duration: iteration_duration,
                    });

                    // Check for cancellation
                    if cancellation_token.is_cancelled() {
                        break;
                    }

                    // Log every 10 iterations
                    if iterations % 10 == 0 {
                        log::debug!(
                            "Continuous node {} completed {} iterations",
                            node_id,
                            iterations
                        );
                    }
                }
            });

            let total_duration = started_at.elapsed();

            // Send stopped notification
            let _ = result_tx.send(ExecutionResult::Stopped {
                node_id,
                iterations,
                duration: total_duration,
            });

            log::info!(
                "Continuous node {} stopped after {} iterations in {:?}",
                node_id,
                iterations,
                total_duration
            );
        }));

        // T040: Handle outer panic (execution_loop itself panicking)
        if let Err(panic_info) = execution_result {
            let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic in execution loop".to_string()
            };

            log::error!(
                "Continuous execution loop panicked for node {}: {}",
                node_id,
                panic_msg
            );

            let _ = result_tx.send(ExecutionResult::Error {
                node_id,
                error: ContinuousNodeError::ExecutionFailed {
                    node_id,
                    node_name: "".to_string(),
                    message: format!("Execution loop panicked: {}", panic_msg),
                    source_location: None,
                    timestamp: chrono::Utc::now(),
                },
            });
        }
    }
}

impl Default for ContinuousExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}
