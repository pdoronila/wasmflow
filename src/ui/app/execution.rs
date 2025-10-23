//! Graph execution and continuous node management
//!
//! This module handles synchronous graph execution, continuous node lifecycle,
//! and reactive dataflow propagation.

use super::{IncrementalExecutionState, NodeExecutionResult, WasmFlowApp};
use crate::runtime::continuous::ExecutionResult;
use crate::runtime::engine::{register_builtin_executors, ExecutionEngine};
use std::sync::mpsc::channel;
use std::thread;
use uuid::Uuid;

impl WasmFlowApp {
    /// Execute the graph in incremental mode
    pub(super) fn execute_graph(&mut self) {
        // Don't start new execution if already executing
        if self.execution_state.is_some() {
            self.status_message = "Execution already in progress...".to_string();
            return;
        }

        self.error_message = None;

        // Get execution order
        let execution_order = match self.graph.execution_order() {
            Ok(order) => order,
            Err(e) => {
                self.error_message = Some(format!("Failed to determine execution order: {}", e));
                return;
            }
        };

        if execution_order.is_empty() {
            self.status_message = "No nodes to execute".to_string();
            return;
        }

        // Reset all nodes to idle
        for node in self.graph.nodes.values_mut() {
            node.execution_state = crate::graph::node::ExecutionState::Idle;
        }

        // Start incremental execution
        self.execution_state = Some(IncrementalExecutionState {
            execution_order,
            current_index: 0,
            execution_receiver: None,
        });

        // Auto-start all continuous nodes that are enabled
        let continuous_nodes: Vec<uuid::Uuid> = self
            .graph
            .nodes
            .iter()
            .filter(|(_, node)| {
                if let Some(config) = &node.continuous_config {
                    config.enabled
                        && matches!(
                            config.runtime_state.execution_state,
                            crate::graph::node::ContinuousExecutionState::Idle
                        )
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect();

        // Add them to pending start queue
        self.canvas
            .pending_continuous_start
            .extend(continuous_nodes);

        self.status_message = "Starting execution...".to_string();
    }

    /// Process one step of incremental execution on the main thread
    pub(super) fn process_execution_step(&mut self) {
        let mut exec_state = match self.execution_state.take() {
            Some(state) => state,
            None => return,
        };

        let node_id = exec_state.execution_order[exec_state.current_index];

        // If we don't have a receiver yet, start execution
        if exec_state.execution_receiver.is_none() {
            // First frame: mark node as running
            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                node.execution_state = crate::graph::node::ExecutionState::Running;
                node.execution_started_at = Some(std::time::Instant::now());
            }
            self.canvas.mark_dirty();

            // Check if this is a constant node with pre-set values
            let is_constant_with_value = {
                let node = self.graph.nodes.get(&node_id).unwrap();
                node.component_id.starts_with("builtin:constant:")
                    && node.outputs.iter().all(|p| p.current_value.is_some())
            };

            if is_constant_with_value {
                // Skip execution for constants that already have values
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    node.execution_state = crate::graph::node::ExecutionState::Completed;
                    node.execution_started_at = None;
                    node.execution_completed_at = Some(std::time::Instant::now());
                    node.dirty = false;
                }

                // Move to next node immediately
                self.move_to_next_node(exec_state);
                return;
            }

            // Update input values from connected outputs before executing
            self.update_input_values_from_connections(node_id);

            // Spawn background thread for execution
            let (tx, rx) = channel();
            let graph_clone = self.graph.clone();

            // Share the component manager with the background thread
            let component_manager = self.engine.component_manager();

            thread::spawn(move || {
                // Create execution engine in background thread
                let mut engine = ExecutionEngine::new();
                register_builtin_executors(&mut engine);

                // Replace the engine's component manager with the shared one
                // This gives access to all loaded WASM components
                engine.set_component_manager(component_manager);

                // Execute the node
                let result = engine.execute_node_with_outputs(&graph_clone, node_id);

                // Send result back
                let _ = tx.send(result.map_err(|e| e.to_string()));
            });

            // Store receiver and wait for result
            exec_state.execution_receiver = Some(rx);
            self.execution_state = Some(exec_state);
        } else {
            // We have a receiver - check if result is ready
            let receiver = exec_state.execution_receiver.as_ref().unwrap();

            match receiver.try_recv() {
                Ok(result) => {
                    // Result is ready! Apply it
                    self.apply_execution_result(node_id, result, exec_state);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still executing - put state back and wait
                    self.execution_state = Some(exec_state);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Thread died - treat as error
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        node.execution_state = crate::graph::node::ExecutionState::Failed;
                        node.execution_started_at = None;
                    }
                    self.error_message =
                        Some("Background execution thread disconnected".to_string());
                    self.canvas.mark_dirty();
                }
            }
        }
    }

    /// Poll continuous execution results and update node states
    pub(super) fn poll_continuous_results(&mut self) {
        use std::sync::mpsc::TryRecvError;

        loop {
            match self.continuous_result_rx.try_recv() {
                Ok(result) => {
                    match result {
                        ExecutionResult::Started {
                            node_id,
                            timestamp: _,
                        } => {
                            // Update node state to Running
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                if let Some(config) = &mut node.continuous_config {
                                    config.runtime_state.execution_state =
                                        crate::graph::node::ContinuousExecutionState::Running;
                                    config.runtime_state.is_running = true;
                                    config.runtime_state.started_at =
                                        Some(std::time::Instant::now());
                                }
                            }
                            self.canvas.mark_dirty();
                        }
                        ExecutionResult::Stopped {
                            node_id,
                            iterations,
                            duration,
                        } => {
                            // Update node state to Idle
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                if let Some(config) = &mut node.continuous_config {
                                    config.runtime_state.execution_state =
                                        crate::graph::node::ContinuousExecutionState::Idle;
                                    config.runtime_state.is_running = false;
                                    config.runtime_state.started_at = None;
                                }
                            }
                            self.status_message = format!(
                                "Continuous node stopped after {} iterations in {:?}",
                                iterations, duration
                            );
                            self.canvas.mark_dirty();
                        }
                        ExecutionResult::OutputsUpdated { node_id, outputs } => {
                            // Update node outputs
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                for (port_name, value) in outputs {
                                    if let Some(port) = node.get_output_mut(&port_name) {
                                        port.current_value = Some(value);
                                    }
                                }
                            }

                            // Propagate values to connected downstream nodes and trigger their execution
                            self.propagate_continuous_outputs(node_id);

                            self.canvas.mark_dirty();
                        }
                        ExecutionResult::Error { node_id, error } => {
                            // Update node state to Error
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                if let Some(config) = &mut node.continuous_config {
                                    config.runtime_state.execution_state =
                                        crate::graph::node::ContinuousExecutionState::Error;
                                    config.runtime_state.is_running = false;
                                    config.runtime_state.last_error = Some(error.to_string());
                                }
                            }
                            self.error_message = Some(format!("Continuous node error: {}", error));
                            self.canvas.mark_dirty();
                        }
                        ExecutionResult::IterationComplete {
                            node_id,
                            iteration,
                            duration: _,
                        } => {
                            // Update iteration counter
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                if let Some(config) = &mut node.continuous_config {
                                    config.runtime_state.iterations = iteration;
                                }
                            }
                            // Don't mark dirty for iteration updates to avoid excessive repaints
                        }
                    }
                }
                Err(TryRecvError::Empty) => break, // No more results
                Err(TryRecvError::Disconnected) => {
                    log::error!("Continuous result channel disconnected");
                    break;
                }
            }
        }
    }

    /// Move to the next node in execution order
    fn move_to_next_node(&mut self, exec_state: IncrementalExecutionState) {
        let next_index = exec_state.current_index + 1;
        if next_index < exec_state.execution_order.len() {
            self.execution_state = Some(IncrementalExecutionState {
                execution_order: exec_state.execution_order,
                current_index: next_index,
                execution_receiver: None,
            });
        } else {
            // Execution complete
            self.status_message = format!(
                "Execution successful! Executed {} nodes.",
                exec_state.execution_order.len()
            );
            self.canvas.mark_dirty();
        }
    }

    /// Apply execution result and move to next node
    fn apply_execution_result(
        &mut self,
        node_id: Uuid,
        result: NodeExecutionResult,
        exec_state: IncrementalExecutionState,
    ) {
        match result {
            Ok(outputs) => {
                // Apply outputs to the node's output ports
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    for (port_name, value) in outputs {
                        if let Some(port) = node.get_output_mut(&port_name) {
                            port.current_value = Some(value);
                        }
                    }
                    node.execution_state = crate::graph::node::ExecutionState::Completed;
                    node.execution_started_at = None;
                    node.execution_completed_at = Some(std::time::Instant::now());
                    node.dirty = false;
                }

                // Update footer view for WASM components
                self.update_footer_view(node_id);

                // Move to next node
                self.move_to_next_node(exec_state);
            }
            Err(e) => {
                // Mark as failed
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    node.execution_state = crate::graph::node::ExecutionState::Failed;
                    node.execution_started_at = None;
                }

                let error_msg = if e.contains("Permission denied") || e.contains("PermissionDenied")
                {
                    format!(
                        "🔒 Permission Denied: A component attempted to access resources without permission. {}",
                        e
                    )
                } else {
                    format!("Node execution failed: {}", e)
                };

                self.error_message = Some(error_msg);
                self.status_message = "Execution failed".to_string();
                self.canvas.mark_dirty();
                // Don't continue execution on error
            }
        }
    }

    /// Update input port values from connected output ports
    fn update_input_values_from_connections(&mut self, node_id: Uuid) {
        // Collect updates first (to avoid borrow checker issues)
        let mut updates: Vec<(Uuid, crate::graph::node::NodeValue)> = Vec::new();

        for connection in self.graph.incoming_connections(node_id) {
            if let Some(source_node) = self.graph.nodes.get(&connection.from_node) {
                if let Some(source_port) = source_node
                    .outputs
                    .iter()
                    .find(|p| p.id == connection.from_port)
                {
                    if let Some(value) = &source_port.current_value {
                        updates.push((connection.to_port, value.clone()));
                    }
                }
            }
        }

        // Apply updates to input ports
        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
            for (port_id, value) in updates {
                if let Some(input_port) = node.inputs.iter_mut().find(|p| p.id == port_id) {
                    input_port.current_value = Some(value);
                }
            }
        }
    }

    /// Propagate continuous node outputs to connected downstream nodes and trigger their execution
    fn propagate_continuous_outputs(&mut self, source_node_id: Uuid) {
        // Find all downstream nodes that are connected to this source node
        let mut downstream_nodes: std::collections::HashSet<Uuid> =
            std::collections::HashSet::new();
        let mut updates: Vec<(Uuid, Uuid, crate::graph::node::NodeValue)> = Vec::new();

        // Get the source node's outputs
        if let Some(source_node) = self.graph.nodes.get(&source_node_id) {
            // Find all connections from this node
            for connection in &self.graph.connections {
                if connection.from_node == source_node_id {
                    downstream_nodes.insert(connection.to_node);

                    // Find the output value
                    if let Some(output_port) = source_node
                        .outputs
                        .iter()
                        .find(|p| p.id == connection.from_port)
                    {
                        if let Some(value) = &output_port.current_value {
                            updates.push((connection.to_node, connection.to_port, value.clone()));
                        }
                    }
                }
            }
        }

        // Apply updates to downstream nodes' input ports
        for (node_id, port_id, value) in updates {
            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                if let Some(input_port) = node.inputs.iter_mut().find(|p| p.id == port_id) {
                    input_port.current_value = Some(value);
                }
            }
        }

        // Trigger execution of downstream nodes (in background threads)
        let result_tx = self.downstream_result_tx.clone();

        for downstream_node_id in downstream_nodes {
            // Skip if it's a continuous node (they manage their own execution)
            if let Some(node) = self.graph.nodes.get(&downstream_node_id) {
                if node.continuous_config.is_some() {
                    continue;
                }
            }

            // Execute the downstream node in a background thread
            let graph_clone = self.graph.clone();
            let component_manager = self.engine.component_manager();
            let node_id_for_thread = downstream_node_id;
            let tx = result_tx.clone();

            thread::spawn(move || {
                // Create execution engine in background thread
                let mut engine = ExecutionEngine::new();
                register_builtin_executors(&mut engine);
                engine.set_component_manager(component_manager);

                // Execute the node
                let result = engine.execute_node_with_outputs(&graph_clone, node_id_for_thread);

                // Send result back to UI
                let _ = tx.send((node_id_for_thread, result.map_err(|e| e.to_string())));
            });
        }
    }

    /// Poll downstream execution results (triggered by continuous nodes)
    pub(super) fn poll_downstream_results(&mut self) {
        use std::sync::mpsc::TryRecvError;

        loop {
            match self.downstream_result_rx.try_recv() {
                Ok((node_id, result)) => {
                    match result {
                        Ok(outputs) => {
                            // Apply outputs to the node's output ports
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                for (port_name, value) in outputs {
                                    if let Some(port) = node.get_output_mut(&port_name) {
                                        port.current_value = Some(value);
                                    }
                                }
                            }

                            // IMPORTANT: Cascade the propagation to this node's downstream nodes
                            // This ensures the entire dataflow graph updates reactively
                            self.propagate_continuous_outputs(node_id);

                            self.canvas.mark_dirty();
                        }
                        Err(e) => {
                            log::error!("Downstream node {} execution failed: {}", node_id, e);
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    log::error!("Downstream result channel disconnected");
                    break;
                }
            }
        }
    }

    /// Update footer view for a node after execution
    fn update_footer_view(&mut self, node_id: Uuid) {
        let node = match self.graph.nodes.get(&node_id) {
            Some(n) => n,
            None => return,
        };

        // Only update footer views for user-defined WASM components
        if !node.component_id.starts_with("user:") {
            return;
        }

        // Get the component manager
        let component_manager = self.engine.component_manager();
        let component_manager = component_manager.lock().unwrap();

        // Try to get footer view from component with current outputs
        match component_manager.get_footer_view_for_node(node) {
            Ok(Some(footer_view)) => {
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    node.cached_footer_view = Some(footer_view);
                }
            }
            Ok(None) => {}
            Err(e) => {
                log::warn!("Failed to get footer view for node {}: {}", node_id, e);
            }
        }
    }
}
