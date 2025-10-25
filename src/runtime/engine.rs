//! Execution engine for running graphs

use crate::graph::graph::NodeGraph;
use crate::graph::node::{ExecutionState, NodeValue};
use crate::runtime::capabilities::CapabilitySet;
use crate::runtime::wasm_host::ComponentManager;
use crate::{ComponentError, GraphError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Default timeout for component execution (30 seconds)
const DEFAULT_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Execution engine for orchestrating graph execution
pub struct ExecutionEngine {
    /// Builtin node executors
    executors: HashMap<String, Box<dyn NodeExecutor>>,
    /// WASM component manager for user-defined components
    component_manager: Arc<Mutex<ComponentManager>>,
    /// Execution timeout for components
    execution_timeout: Duration,
}

/// Trait for executing a node
pub trait NodeExecutor: Send + Sync {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError>;
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        let component_manager = ComponentManager::new()
            .unwrap_or_else(|e| {
                log::error!("Failed to create component manager: {}", e);
                ComponentManager::default()
            });

        Self {
            executors: HashMap::new(),
            component_manager: Arc::new(Mutex::new(component_manager)),
            execution_timeout: DEFAULT_EXECUTION_TIMEOUT,
        }
    }

    /// Register a builtin node executor
    pub fn register_executor(&mut self, component_id: String, executor: Box<dyn NodeExecutor>) {
        self.executors.insert(component_id, executor);
    }

    /// Set the execution timeout for components
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.execution_timeout = timeout;
    }

    /// Get the component manager for loading custom components
    pub fn component_manager(&self) -> Arc<Mutex<ComponentManager>> {
        Arc::clone(&self.component_manager)
    }

    /// Set the component manager (for sharing across threads)
    pub fn set_component_manager(&mut self, component_manager: Arc<Mutex<ComponentManager>>) {
        self.component_manager = component_manager;
    }

    /// Execute the entire graph
    pub fn execute_graph(&mut self, graph: &mut NodeGraph) -> Result<ExecutionReport, GraphError> {
        let mut report = ExecutionReport::default();

        // Get execution order (topological sort)
        let execution_order = graph.execution_order()?;

        // Reset all nodes to idle state
        for node in graph.nodes.values_mut() {
            node.execution_state = ExecutionState::Idle;
        }

        // Execute nodes in dependency order
        for node_id in &execution_order {
            // Update input port values from connections (for UI display)
            Self::update_input_values_from_connections(graph, *node_id);

            // Mark as running and record start time
            if let Some(node) = graph.nodes.get_mut(node_id) {
                node.execution_state = ExecutionState::Running;
                node.execution_started_at = Some(std::time::Instant::now());
            }

            // Check if this is a constant node with pre-set values
            let is_constant_with_value = {
                let node = graph.nodes.get(node_id).unwrap();
                node.component_id.starts_with("builtin:constant:") &&
                    node.outputs.iter().all(|p| p.current_value.is_some())
            };

            if is_constant_with_value {
                // Skip execution for constants that already have values
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.execution_state = ExecutionState::Completed;
                    node.execution_started_at = None;
                }
                report.executed_nodes.push(*node_id);
                continue;
            }

            // Execute the node (read-only access to graph)
            let outputs_result = self.execute_node(graph, *node_id);

            match outputs_result {
                Ok(outputs) => {
                    // Apply outputs to the node
                    Self::apply_outputs(graph, *node_id, outputs)?;

                    // Update footer view for WASM components with custom UI
                    self.update_footer_view(graph, *node_id);

                    // Mark as completed
                    if let Some(node) = graph.nodes.get_mut(node_id) {
                        node.execution_state = ExecutionState::Completed;
                        node.execution_started_at = None;
                    }
                    report.executed_nodes.push(*node_id);
                }
                Err(e) => {
                    // Mark as failed
                    if let Some(node) = graph.nodes.get_mut(node_id) {
                        node.execution_state = ExecutionState::Failed;
                        node.execution_started_at = None;
                    }
                    report.failed_nodes.push((*node_id, e.to_string()));
                    return Err(GraphError::InvalidConnection(format!(
                        "Node execution failed: {}",
                        e
                    )));
                }
            }
        }

        Ok(report)
    }

    /// T084: Execute only dirty nodes in the graph (incremental execution)
    ///
    /// This method only executes nodes marked as dirty and automatically marks
    /// executed nodes as clean. This provides significant performance improvements
    /// for large graphs where only a few nodes have changed.
    pub fn execute_graph_incremental(&mut self, graph: &mut NodeGraph) -> Result<ExecutionReport, GraphError> {
        use crate::graph::execution::{get_dirty_execution_order, count_dirty_nodes};

        let mut report = ExecutionReport::default();

        // Get execution order for dirty nodes only
        let execution_order = get_dirty_execution_order(graph)?;

        if execution_order.is_empty() {
            log::debug!("No dirty nodes to execute");
            return Ok(report);
        }

        log::info!(
            "Incremental execution: processing {} dirty nodes out of {} total",
            execution_order.len(),
            graph.nodes.len()
        );

        // Execute dirty nodes in dependency order
        for node_id in &execution_order {
            // Update input port values from connections (for UI display)
            Self::update_input_values_from_connections(graph, *node_id);

            // Mark as running and record start time
            if let Some(node) = graph.nodes.get_mut(node_id) {
                node.execution_state = ExecutionState::Running;
                node.execution_started_at = Some(std::time::Instant::now());
            }

            // Check if this is a constant node with pre-set values
            let is_constant_with_value = {
                let node = graph.nodes.get(node_id).unwrap();
                node.component_id.starts_with("builtin:constant:") &&
                    node.outputs.iter().all(|p| p.current_value.is_some())
            };

            if is_constant_with_value {
                // Skip execution for constants that already have values
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.execution_state = ExecutionState::Completed;
                    node.dirty = false; // Mark as clean
                    node.execution_started_at = None;
                }
                report.executed_nodes.push(*node_id);
                continue;
            }

            // Execute the node (read-only access to graph)
            let outputs_result = self.execute_node(graph, *node_id);

            match outputs_result {
                Ok(outputs) => {
                    // Apply outputs to the node
                    Self::apply_outputs(graph, *node_id, outputs)?;

                    // Update footer view for WASM components with custom UI
                    self.update_footer_view(graph, *node_id);

                    // Mark as completed and clean
                    if let Some(node) = graph.nodes.get_mut(node_id) {
                        node.execution_state = ExecutionState::Completed;
                        node.dirty = false; // T084: Mark as clean after successful execution
                        node.execution_started_at = None;
                    }
                    report.executed_nodes.push(*node_id);
                }
                Err(e) => {
                    // Mark as failed (keep dirty so it will be re-executed on next run)
                    if let Some(node) = graph.nodes.get_mut(node_id) {
                        node.execution_state = ExecutionState::Failed;
                        node.execution_started_at = None;
                        // Leave dirty = true so it will be retried
                    }
                    report.failed_nodes.push((*node_id, e.to_string()));
                    return Err(GraphError::InvalidConnection(format!(
                        "Node execution failed: {}",
                        e
                    )));
                }
            }
        }

        log::info!(
            "Incremental execution complete: {} nodes executed, {} remaining dirty",
            report.executed_nodes.len(),
            count_dirty_nodes(graph)
        );

        Ok(report)
    }

    /// Execute a single node - internal helper that doesn't update the graph
    fn execute_node(&self, graph: &NodeGraph, node_id: Uuid) -> Result<HashMap<String, NodeValue>, ComponentError> {
        self.execute_node_with_outputs(graph, node_id)
    }

    /// Execute a single node and return outputs (for external update)
    pub fn execute_node_with_outputs(
        &self,
        graph: &NodeGraph,
        node_id: Uuid,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let node = graph.nodes.get(&node_id).ok_or_else(|| {
            ComponentError::ExecutionError(format!("Node {} not found", node_id))
        })?;

        // Gather input values from connected output ports
        let mut inputs = HashMap::new();

        for connection in graph.incoming_connections(node_id) {
            let source_node = graph.nodes.get(&connection.from_node).ok_or_else(|| {
                ComponentError::ExecutionError("Source node not found".to_string())
            })?;

            let source_port = source_node
                .outputs
                .iter()
                .find(|p| p.id == connection.from_port)
                .ok_or_else(|| {
                    ComponentError::ExecutionError("Source port not found".to_string())
                })?;

            let value = source_port.current_value.clone().ok_or_else(|| {
                ComponentError::ExecutionError(format!(
                    "Source port '{}' has no value",
                    source_port.name
                ))
            })?;

            let target_port = node
                .inputs
                .iter()
                .find(|p| p.id == connection.to_port)
                .ok_or_else(|| {
                    ComponentError::ExecutionError("Target port not found".to_string())
                })?;

            inputs.insert(target_port.name.clone(), value);
        }

        // Also check for input ports with pre-set values (for composite node internal execution)
        // These are ports that have values set directly, not from connections
        for input_port in &node.inputs {
            if !inputs.contains_key(&input_port.name) {
                if let Some(value) = &input_port.current_value {
                    log::debug!("Using pre-set value for input port '{}'", input_port.name);
                    inputs.insert(input_port.name.clone(), value.clone());
                }
            }
        }

        // Check if this is a composite node
        if let Some(composition_data) = &node.composition_data {
            log::debug!("Executing composite node with {} internal nodes",
                composition_data.internal_nodes.len());

            // Map external inputs to internal node inputs using port mappings
            // For now, we'll execute the composed WASM binary directly
            // In the future, we could execute the internal graph step-by-step

            // Use Full capabilities for composite nodes (they already went through permission approval)
            let capability_set = CapabilitySet::Full;

            // Execute the composed WASM binary
            return self.execute_composite_node(node_id, &node.component_id, &inputs, &capability_set, composition_data);
        }

        // Check if this is a user-defined component
        if node.component_id.starts_with("user:") {
            // T075: Get capability grant from graph for permission enforcement
            let capability_set = graph
                .get_capability_grant(node_id)
                .map(|grant| grant.capability_set.clone())
                .unwrap_or_else(|| {
                    // No grant found - use None (no permissions)
                    log::warn!("No capability grant found for node {}, using None permissions", node_id);
                    CapabilitySet::none()
                });

            // Execute WASM component with timeout and error handling
            return self.execute_wasm_component(node_id, &node.component_id, &inputs, &capability_set);
        }

        // Get the builtin executor
        let executor = self.executors.get(&node.component_id).ok_or_else(|| {
            ComponentError::ExecutionError(format!(
                "No executor registered for component '{}'",
                node.component_id
            ))
        })?;

        // Execute builtin component and return outputs
        executor.execute(&inputs)
    }

    /// Execute a WASM component with timeout and enhanced error handling
    fn execute_wasm_component(
        &self,
        node_id: Uuid,
        component_id: &str,
        inputs: &HashMap<String, NodeValue>,
        capabilities: &CapabilitySet,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        // Use synchronous execution with tokio runtime
        // This creates a new runtime per execution, which is not ideal for performance
        // but works for now until we have a proper async integration

        log::debug!("Executing WASM component '{}' synchronously", component_id);

        // T083: component_manager needs to be mutable for lazy compilation
        let mut component_manager = self.component_manager.lock().unwrap();
        let timeout = self.execution_timeout;
        let capabilities = capabilities.clone();
        let inputs = inputs.clone();
        let component_id_str = component_id.to_string();

        // Create a new tokio runtime for this execution
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            ComponentError::ExecutionError(format!("Failed to create async runtime: {}", e))
        })?;

        // Run async execution synchronously using block_on
        let result = runtime.block_on(async {
            tokio::time::timeout(
                timeout,
                component_manager.execute_component(&component_id_str, &inputs, capabilities)
            ).await
        });

        match result {
            Ok(Ok(outputs)) => {
                log::debug!("Component '{}' executed successfully", component_id);
                Ok(outputs)
            }
            Ok(Err(e)) => {
                // Component execution failed - provide context
                log::error!("Component '{}' execution failed: {}", component_id, e);
                Err(self.enhance_component_error(node_id, component_id, e))
            }
            Err(_) => {
                // Timeout
                log::error!("Component '{}' execution timed out", component_id);
                Err(ComponentError::ExecutionError(format!(
                    "Component '{}' execution timed out after {:?}",
                    component_id, timeout
                )))
            }
        }
    }

    /// Execute a composite node by running its internal graph
    fn execute_composite_node(
        &self,
        _node_id: Uuid,
        component_id: &str,
        external_inputs: &HashMap<String, NodeValue>,
        _capabilities: &CapabilitySet,
        composition_data: &crate::graph::node::CompositionData,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        log::debug!("Executing composite node '{}' with {} internal nodes",
            component_id, composition_data.internal_nodes.len());

        // Create a temporary graph from the internal structure
        let mut internal_graph = crate::graph::graph::NodeGraph::new(
            composition_data.name.clone(),
            "Composite Execution".to_string(),
        );

        // Add internal nodes to the graph
        for (internal_node_id, node) in &composition_data.internal_nodes {
            let mut node_clone = node.clone();
            node_clone.id = *internal_node_id; // Preserve original node ID
            internal_graph.add_node(node_clone);
        }

        // Add internal connections
        for connection in &composition_data.internal_edges {
            let _ = internal_graph.add_connection(
                connection.from_node,
                connection.from_port,
                connection.to_node,
                connection.to_port,
            );
        }

        // Map external inputs to internal node inputs
        for (external_name, value) in external_inputs {
            if let Some(mapping) = composition_data.exposed_inputs.get(external_name) {
                log::debug!("Mapping external input '{}' to internal node '{}'",
                    external_name, mapping.internal_node_id);

                // Set the input value on the internal node
                if let Some(internal_node) = internal_graph.nodes.get_mut(&mapping.internal_node_id) {
                    if let Some(input_port) = internal_node.inputs.iter_mut()
                        .find(|p| p.name == mapping.internal_port_name) {
                        input_port.current_value = Some(value.clone());
                    }
                }
            }
        }

        // Execute the internal graph in topological order
        log::debug!("Executing internal graph with {} nodes", internal_graph.nodes.len());
        let execution_order = match internal_graph.execution_order() {
            Ok(order) => order,
            Err(e) => {
                log::error!("Failed to get execution order for internal graph: {}", e);
                return Err(ComponentError::ExecutionError(
                    format!("Composite node execution order failed: {}", e)
                ));
            }
        };

        // Execute each node in topological order
        for internal_node_id in &execution_order {
            log::debug!("Executing internal node {}", internal_node_id);

            match self.execute_node(&internal_graph, *internal_node_id) {
                Ok(outputs) => {
                    // Update the internal graph with outputs
                    if let Some(node) = internal_graph.nodes.get_mut(internal_node_id) {
                        for (port_name, value) in outputs {
                            if let Some(output_port) = node.outputs.iter_mut().find(|p| p.name == port_name) {
                                output_port.current_value = Some(value);
                            }
                        }
                        node.execution_state = crate::graph::node::ExecutionState::Completed;
                    }
                }
                Err(e) => {
                    log::error!("Internal node {} execution failed: {}", internal_node_id, e);
                    if let Some(node) = internal_graph.nodes.get_mut(internal_node_id) {
                        node.execution_state = crate::graph::node::ExecutionState::Failed;
                    }
                    return Err(ComponentError::ExecutionError(
                        format!("Composite node internal execution failed at node {}: {}", internal_node_id, e)
                    ));
                }
            }
        }

        log::debug!("Internal graph execution completed: {} nodes executed", execution_order.len());

        // Map internal outputs to external outputs
        let mut external_outputs = HashMap::new();
        for (external_name, mapping) in &composition_data.exposed_outputs {
            if let Some(internal_node) = internal_graph.nodes.get(&mapping.internal_node_id) {
                if let Some(output_port) = internal_node.outputs.iter()
                    .find(|p| p.name == mapping.internal_port_name) {
                    if let Some(value) = &output_port.current_value {
                        log::debug!("Mapping internal output from node '{}' to external '{}'",
                            mapping.internal_node_id, external_name);
                        external_outputs.insert(external_name.clone(), value.clone());
                    } else {
                        log::warn!("Internal output port '{}' on node '{}' has no value",
                            mapping.internal_port_name, mapping.internal_node_id);
                    }
                }
            }
        }

        log::debug!("Composite node execution complete with {} outputs", external_outputs.len());
        Ok(external_outputs)
    }

    #[allow(dead_code)]
    /// Enhance component error with additional context
    fn enhance_component_error(
        &self,
        node_id: Uuid,
        component_id: &str,
        error: ComponentError,
    ) -> ComponentError {
        match error {
            ComponentError::ExecutionError(msg) => {
                ComponentError::ExecutionError(format!(
                    "Node {} ({}): {}",
                    node_id, component_id, msg
                ))
            }
            ComponentError::PermissionDenied { node_id: _, capability } => {
                ComponentError::PermissionDenied {
                    node_id,
                    capability: format!("{} (component: {})", capability, component_id),
                }
            }
            other => other,
        }
    }

    /// Update input port values from connected output ports (for UI display)
    ///
    /// This updates the `current_value` field of input ports with values from
    /// their connected output ports, so that the UI can display them in footer views.
    fn update_input_values_from_connections(
        graph: &mut NodeGraph,
        node_id: Uuid,
    ) {
        // Collect updates first (to avoid borrow checker issues)
        let mut updates: Vec<(Uuid, NodeValue)> = Vec::new();

        for connection in graph.incoming_connections(node_id) {
            if let Some(source_node) = graph.nodes.get(&connection.from_node) {
                if let Some(source_port) = source_node.outputs.iter().find(|p| p.id == connection.from_port) {
                    if let Some(value) = &source_port.current_value {
                        updates.push((connection.to_port, value.clone()));
                    }
                }
            }
        }

        // Apply updates to input ports
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            for (port_id, value) in updates {
                if let Some(input_port) = node.inputs.iter_mut().find(|p| p.id == port_id) {
                    input_port.current_value = Some(value);
                }
            }
        }
    }

    /// Apply outputs to a node's output ports
    pub fn apply_outputs(
        graph: &mut NodeGraph,
        node_id: Uuid,
        outputs: HashMap<String, NodeValue>,
    ) -> Result<(), ComponentError> {
        let node = graph.nodes.get_mut(&node_id).ok_or_else(|| {
            ComponentError::ExecutionError(format!("Node {} not found", node_id))
        })?;

        for (output_name, output_value) in outputs {
            if let Some(output_port) = node.outputs.iter_mut().find(|p| p.name == output_name) {
                output_port.current_value = Some(output_value);
            } else {
                return Err(ComponentError::ExecutionError(format!(
                    "Output port '{}' not found on node",
                    output_name
                )));
            }
        }

        Ok(())
    }

    /// Update footer view for a node after execution
    ///
    /// For WASM components with custom UI, this calls get-footer-view() with
    /// the node's actual output values and caches the result for rendering.
    fn update_footer_view(&self, graph: &mut NodeGraph, node_id: Uuid) {
        let node = match graph.nodes.get(&node_id) {
            Some(n) => n,
            None => return, // Node not found, skip silently
        };

        // Only update footer views for user-defined WASM components
        if !node.component_id.starts_with("user:") {
            return;
        }

        // Get the component manager
        let component_manager = self.component_manager.lock().unwrap();

        // Try to get footer view from component with current outputs
        match component_manager.get_footer_view_for_node(node) {
            Ok(Some(footer_view)) => {
                // Store the footer view on the node
                if let Some(node) = graph.nodes.get_mut(&node_id) {
                    node.cached_footer_view = Some(footer_view);
                    log::debug!("Updated footer view for node {}", node_id);
                }
            }
            Ok(None) => {
                // Component doesn't provide a footer view
                log::trace!("Component {} doesn't provide footer view", node.component_id);
            }
            Err(e) => {
                // Error getting footer view - log but don't fail execution
                log::warn!("Failed to get footer view for node {}: {}", node_id, e);
            }
        }
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Report of graph execution
#[derive(Debug, Default)]
pub struct ExecutionReport {
    pub executed_nodes: Vec<Uuid>,
    pub failed_nodes: Vec<(Uuid, String)>,
}

impl ExecutionReport {
    pub fn success(&self) -> bool {
        self.failed_nodes.is_empty()
    }
}

// Implement NodeExecutor for builtin nodes
use crate::builtin::constants::ConstantNode;

// For constant nodes, we need a wrapper since they don't use the MathOperation trait
pub struct ConstantExecutor {
    value: NodeValue,
}

impl ConstantExecutor {
    pub fn new(value: NodeValue) -> Self {
        Self { value }
    }
}

impl NodeExecutor for ConstantExecutor {
    fn execute(&self, _inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let constant = ConstantNode::new(self.value.clone());
        constant.execute(&HashMap::new())
    }
}

/// Register all builtin node executors
pub fn register_builtin_executors(engine: &mut ExecutionEngine) {
    // Register constant executors for different types
    engine.register_executor(
        "builtin:constant:f32".to_string(),
        Box::new(ConstantExecutor::new(NodeValue::F32(0.0))),
    );
    engine.register_executor(
        "builtin:constant:i32".to_string(),
        Box::new(ConstantExecutor::new(NodeValue::I32(0))),
    );
    engine.register_executor(
        "builtin:constant:u32".to_string(),
        Box::new(ConstantExecutor::new(NodeValue::U32(0))),
    );
    engine.register_executor(
        "builtin:constant:string".to_string(),
        Box::new(ConstantExecutor::new(NodeValue::String(String::new()))),
    );

    // Register continuous timer executor
    engine.register_executor(
        "builtin:continuous:timer".to_string(),
        Box::new(crate::builtin::ContinuousTimerExecutor),
    );

    // T050: Register continuous combiner executor
    engine.register_executor(
        "builtin:continuous:combiner".to_string(),
        Box::new(crate::builtin::ContinuousCombinerExecutor),
    );

    // Register HTTP server listener executor
    engine.register_executor(
        "builtin:http:server-listener".to_string(),
        Box::new(crate::builtin::HttpServerListenerExecutor::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::{ComponentSpec, DataType};

    #[test]
    fn test_execution_engine_basic() {
        let mut engine = ExecutionEngine::new();
        register_builtin_executors(&mut engine);

        let mut graph = NodeGraph::new("Test".to_string(), "Test".to_string());

        // Create nodes: Constant(5) -> Add <- Constant(3)
        let const_spec = ComponentSpec::new_builtin(
            "builtin:constant:f32".to_string(),
            "Constant".to_string(),
            "Constant".to_string(),
            Some("Constants".to_string()),
        )
        .with_output("value".to_string(), DataType::F32, "Value".to_string());

        let add_spec = ComponentSpec::new_builtin(
            "builtin:math:add".to_string(),
            "Add".to_string(),
            "Add".to_string(),
            Some("Math".to_string()),
        )
        .with_input("a".to_string(), DataType::F32, "First".to_string())
        .with_input("b".to_string(), DataType::F32, "Second".to_string())
        .with_output("sum".to_string(), DataType::F32, "Sum".to_string());

        let mut const1 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
        const1.outputs[0].current_value = Some(NodeValue::F32(5.0));

        let mut const2 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
        const2.outputs[0].current_value = Some(NodeValue::F32(3.0));

        let add = add_spec.create_node(egui::Pos2::new(200.0, 50.0));

        let const1_id = const1.id;
        let const2_id = const2.id;
        let add_id = add.id;
        let const1_out = const1.outputs[0].id;
        let const2_out = const2.outputs[0].id;
        let add_in_a = add.inputs[0].id;
        let add_in_b = add.inputs[1].id;

        graph.add_node(const1);
        graph.add_node(const2);
        graph.add_node(add);

        graph.add_connection(const1_id, const1_out, add_id, add_in_a).unwrap();
        graph.add_connection(const2_id, const2_out, add_id, add_in_b).unwrap();

        // Execute graph
        let report = engine.execute_graph(&mut graph).unwrap();
        assert!(report.success());
        assert_eq!(report.executed_nodes.len(), 3);

        // Check result
        let add_node = graph.nodes.get(&add_id).unwrap();
        assert_eq!(add_node.outputs[0].current_value, Some(NodeValue::F32(8.0)));
    }
}
