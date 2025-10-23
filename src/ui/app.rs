//! Main application state and UI logic

use super::canvas::NodeCanvas;
use super::dialogs::{
    AboutDialog, GraphMetadataDialog, PermissionAction, PermissionDialog, PermissionsViewDialog,
    UnsavedChangesAction, UnsavedChangesDialog,
};
use super::palette::{Palette, PaletteAction};
use super::theme::Theme;
use crate::builtin::{
    register_constant_nodes, register_continuous_example, register_math_nodes,
    register_wasm_creator_node,
};
use crate::graph::command::CommandHistory;
use crate::graph::graph::NodeGraph;
use crate::graph::node::ComponentRegistry;
use crate::runtime::capabilities::{CapabilityGrant, CapabilitySet};
use crate::runtime::continuous::{ContinuousExecutionManager, ExecutionResult};
use crate::runtime::engine::{register_builtin_executors, ExecutionEngine};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use uuid::Uuid;

/// Main WasmFlow application
pub struct WasmFlowApp {
    /// Current graph being edited
    graph: NodeGraph,
    /// Component registry for available nodes
    registry: ComponentRegistry,
    /// Execution engine
    engine: ExecutionEngine,
    /// Visual node editor canvas
    canvas: NodeCanvas,
    /// Command history for undo/redo
    history: CommandHistory,
    /// Status message
    status_message: String,
    /// Error message (if any)
    error_message: Option<String>,
    /// Current file path (if graph has been saved)
    current_file: Option<std::path::PathBuf>,
    /// Dirty flag tracking unsaved changes
    dirty: bool,
    /// Unsaved changes confirmation dialog
    unsaved_changes_dialog: UnsavedChangesDialog,
    /// Pending action after unsaved changes dialog
    pending_action: Option<PendingAction>,
    /// Recent files list (up to 10)
    recent_files: Vec<PathBuf>,
    /// T073: Permission request dialog
    permission_dialog: PermissionDialog,
    /// T073: Pending component awaiting permission approval
    pending_permission_request: Option<PendingPermissionRequest>,
    /// T078: Permission view dialog
    permissions_view_dialog: PermissionsViewDialog,
    /// T100: About dialog
    about_dialog: AboutDialog,
    /// T091: Component palette with search
    palette: Palette,
    /// T092: Graph metadata editor dialog
    metadata_dialog: GraphMetadataDialog,
    /// Application theme
    theme: Theme,
    /// Incremental execution state
    execution_state: Option<IncrementalExecutionState>,
    /// Continuous execution manager for long-running nodes
    continuous_manager: ContinuousExecutionManager,
    /// Channel for receiving continuous execution results
    continuous_result_rx: Receiver<ExecutionResult>,
    /// Channel for sending continuous execution results
    continuous_result_tx: std::sync::mpsc::Sender<ExecutionResult>,
    /// Channel for receiving downstream node execution results (triggered by continuous nodes)
    downstream_result_rx: Receiver<(Uuid, NodeExecutionResult)>,
    /// Channel for sending downstream node execution results
    downstream_result_tx: std::sync::mpsc::Sender<(Uuid, NodeExecutionResult)>,
    /// T028: Component composer for WAC composition
    composer: crate::runtime::wac_integration::ComponentComposer,
    /// T032: Error dialog for composition failures
    composition_error: Option<String>,
    /// T037: View stack for drill-down navigation
    view_stack: crate::graph::drill_down::ViewStack,
}

/// State for incremental execution on the main thread
struct IncrementalExecutionState {
    /// List of nodes to execute in order
    execution_order: Vec<Uuid>,
    /// Index of the current node being executed
    current_index: usize,
    /// Channel for receiving execution results from background thread
    execution_receiver: Option<
        Receiver<Result<std::collections::HashMap<String, crate::graph::node::NodeValue>, String>>,
    >,
}

/// Result from background node execution
type NodeExecutionResult =
    Result<std::collections::HashMap<String, crate::graph::node::NodeValue>, String>;

/// T073: Component awaiting permission approval
#[derive(Debug, Clone)]
struct PendingPermissionRequest {
    /// Component specification
    component_spec: crate::graph::node::ComponentSpec,
    /// Requested capabilities
    capabilities: CapabilitySet,
    /// Position where to create the node
    position: egui::Pos2,
}

/// Actions that can be pending confirmation from unsaved changes dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    /// User wants to create a new graph
    NewGraph,
    /// User wants to quit the application
    Quit,
}

impl WasmFlowApp {
    /// Create a new WasmFlow application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create component registry and register builtin nodes
        let mut registry = ComponentRegistry::new();
        register_math_nodes(&mut registry);
        register_constant_nodes(&mut registry);
        register_wasm_creator_node(&mut registry);
        register_continuous_example(&mut registry);

        // Create execution engine and register executors
        // The engine creates its own ComponentManager internally
        let mut engine = ExecutionEngine::new();
        register_builtin_executors(&mut engine);

        // Create initial graph
        let graph = NodeGraph::new("Untitled Graph".to_string(), "User".to_string());

        let recent_files = Self::load_recent_files();

        // Create channel for continuous execution results
        let (continuous_result_tx, continuous_result_rx) = channel();

        // Create channel for downstream execution results
        let (downstream_result_tx, downstream_result_rx) = channel();

        let mut app = Self {
            graph,
            registry,
            engine,
            canvas: NodeCanvas::new(),
            history: CommandHistory::new(),
            status_message: "Welcome to WasmFlow! Create nodes from the palette.".to_string(),
            error_message: None,
            current_file: None,
            dirty: false,
            unsaved_changes_dialog: UnsavedChangesDialog::new(),
            pending_action: None,
            recent_files,
            permission_dialog: PermissionDialog::new(),
            pending_permission_request: None,
            permissions_view_dialog: PermissionsViewDialog::new(),
            about_dialog: AboutDialog::new(),
            palette: Palette::new(),
            metadata_dialog: GraphMetadataDialog::new(),
            theme: Theme::dark(),
            execution_state: None,
            continuous_manager: ContinuousExecutionManager::new(),
            continuous_result_rx,
            continuous_result_tx,
            downstream_result_rx,
            downstream_result_tx,
            composer: crate::runtime::wac_integration::ComponentComposer::new(), // T028
            composition_error: None,                                             // T032
            view_stack: crate::graph::drill_down::ViewStack::new(),              // T037
        };

        // Auto-load components from components/ directory on startup
        app.reload_components();

        app
    }

    /// Undo the last command
    fn undo(&mut self) {
        match self.history.undo(&mut self.graph) {
            Ok(()) => {
                self.status_message = "Undone".to_string();
                self.error_message = None;
                self.dirty = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Cannot undo: {}", e));
            }
        }
    }

    /// Redo the last undone command
    fn redo(&mut self) {
        match self.history.redo(&mut self.graph) {
            Ok(()) => {
                self.status_message = "Redone".to_string();
                self.error_message = None;
                self.dirty = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Cannot redo: {}", e));
            }
        }
    }

    /// Save the current graph to a file
    fn save_graph(&mut self) {
        if let Some(path) = &self.current_file {
            // Save to existing file
            match self.graph.save_to_file(path) {
                Ok(()) => {
                    self.status_message = format!("Saved to {}", path.display());
                    self.error_message = None;
                    self.dirty = false;
                    self.add_recent_file(path.clone());
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save: {}", e));
                }
            }
        } else {
            // No current file, show save dialog
            self.save_graph_as();
        }
    }

    /// Save the graph to a new file (Save As)
    fn save_graph_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WasmFlow Graph", &["wasmflow"])
            .set_file_name("graph.wasmflow")
            .save_file()
        {
            match self.graph.save_to_file(&path) {
                Ok(()) => {
                    self.status_message = format!("Saved to {}", path.display());
                    self.error_message = None;
                    self.current_file = Some(path.clone());
                    self.dirty = false;
                    self.add_recent_file(path);
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save: {}", e));
                }
            }
        }
    }

    /// Load a graph from a file
    fn load_graph(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WasmFlow Graph", &["wasmflow"])
            .pick_file()
        {
            match NodeGraph::load_from_file(&path) {
                Ok(graph) => {
                    log::info!("Successfully deserialized graph from {}", path.display());
                    log::info!(
                        "Graph has {} nodes and {} connections",
                        graph.nodes.len(),
                        graph.connections.len()
                    );

                    self.graph = graph;
                    self.current_file = Some(path.clone());
                    self.dirty = false;
                    self.history = CommandHistory::new(); // Clear undo history
                    self.status_message = format!("Loaded {}", path.display());
                    self.error_message = None;
                    self.add_recent_file(path.clone());

                    // Mark canvas dirty to force re-sync with loaded graph
                    self.canvas.mark_dirty();

                    log::info!("Graph loaded successfully from {}", path.display());
                }
                Err(e) => {
                    // Log full error chain for debugging
                    log::error!("Failed to load graph from {}", path.display());
                    log::error!("Error chain: {:#}", e);

                    // Provide user-friendly error messages for common issues
                    let error_msg = if e.to_string().contains("magic bytes") {
                        "Invalid file format: This is not a WasmFlow graph file.".to_string()
                    } else if e.to_string().contains("version") {
                        format!("Incompatible file version: {}. Please upgrade WasmFlow.", e)
                    } else if e.to_string().contains("Checksum mismatch") {
                        "File appears to be corrupted. Please try another file.".to_string()
                    } else {
                        format!("Failed to load graph: {:#}", e)
                    };
                    self.error_message = Some(error_msg);
                }
            }
        }
    }

    /// Create a new graph (with confirmation if current graph has unsaved changes)
    fn new_graph(&mut self) {
        if self.dirty {
            // Show unsaved changes dialog
            self.unsaved_changes_dialog.open();
            self.pending_action = Some(PendingAction::NewGraph);
            return;
        }

        // Create new graph immediately if no unsaved changes
        self.create_new_graph();
    }

    /// Actually create a new graph (called after confirmation or if no unsaved changes)
    fn create_new_graph(&mut self) {
        self.graph = NodeGraph::new("Untitled Graph".to_string(), "User".to_string());
        self.current_file = None;
        self.dirty = false;
        self.history = CommandHistory::new();
        self.status_message = "New graph created".to_string();
        self.error_message = None;
    }

    /// Execute the current graph incrementally on the main thread
    /// T077: Enhanced error messages including permission denied errors
    fn execute_graph(&mut self) {
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
    fn process_execution_step(&mut self) {
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
    fn poll_continuous_results(&mut self) {
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
    fn poll_downstream_results(&mut self) {
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

    /// Handle quit action (with confirmation if dirty)
    fn quit(&mut self, ctx: &egui::Context) {
        if self.dirty {
            // Show unsaved changes dialog
            self.unsaved_changes_dialog.open();
            self.pending_action = Some(PendingAction::Quit);
        } else {
            // Quit immediately if no unsaved changes
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Handle unsaved changes dialog response
    fn handle_unsaved_changes_dialog(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.unsaved_changes_dialog.show(ctx) {
            match action {
                UnsavedChangesAction::Save => {
                    // Save the graph first
                    self.save_graph();

                    // Only proceed with pending action if save succeeded (dirty flag cleared)
                    if !self.dirty {
                        self.execute_pending_action(ctx);
                    }
                }
                UnsavedChangesAction::Discard => {
                    // Discard changes and execute pending action
                    self.dirty = false;
                    self.execute_pending_action(ctx);
                }
                UnsavedChangesAction::Cancel => {
                    // Cancel the pending action
                    self.pending_action = None;
                }
            }
        }
    }

    /// Execute the pending action after unsaved changes dialog
    fn execute_pending_action(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.pending_action.take() {
            match action {
                PendingAction::NewGraph => {
                    self.create_new_graph();
                }
                PendingAction::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    /// T078: Handle permission view dialog
    fn handle_permissions_view_dialog(&mut self, ctx: &egui::Context) {
        // Show the dialog
        self.permissions_view_dialog.show(ctx);

        // Check if an action was requested
        if let Some(action) = self.permissions_view_dialog.take_action() {
            if let Some(node_id) = self.permissions_view_dialog.node_id() {
                match action {
                    super::dialogs::PermissionViewAction::Revoke => {
                        // Revoke the capability grant
                        self.graph.revoke_capability(node_id);
                        self.status_message =
                            "Permissions revoked - node will fail to execute without re-approval"
                                .to_string();
                        self.dirty = true;
                    }
                    super::dialogs::PermissionViewAction::UpgradeToFull => {
                        // Upgrade to Full access
                        let grant = crate::runtime::capabilities::CapabilityGrant {
                            node_id,
                            capability_set: crate::runtime::capabilities::CapabilitySet::Full,
                            granted_at: chrono::Utc::now().to_rfc3339(),
                            scope: "graph".to_string(),
                        };
                        self.graph.grant_capability(grant);
                        self.status_message = "Upgraded to FULL ACCESS (unrestricted)".to_string();
                        self.dirty = true;
                    }
                }
            }
            self.permissions_view_dialog.reset();
        }
    }

    /// T073: Handle permission dialog response
    fn handle_permission_dialog(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.permission_dialog.show(ctx) {
            match action {
                PermissionAction::Approve => {
                    // User approved the permissions - create the node with grant
                    if let Some(pending) = self.pending_permission_request.take() {
                        // Create a capability grant
                        let node = pending.component_spec.create_node(pending.position);
                        let grant = CapabilityGrant {
                            node_id: node.id,
                            capability_set: pending.capabilities,
                            granted_at: chrono::Utc::now().to_rfc3339(),
                            scope: "graph".to_string(),
                        };

                        // Add the grant to the graph
                        self.graph.grant_capability(grant);

                        // Add the node to the graph through command history
                        let cmd = crate::graph::command::Command::AddNode { node };
                        if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                            self.error_message = Some(format!("Failed to add node: {}", e));
                        } else {
                            self.status_message = format!(
                                "Added {} node with approved permissions",
                                pending.component_spec.name
                            );
                            self.error_message = None;
                            self.dirty = true;
                        }
                    }
                }
                PermissionAction::ApproveAsFull => {
                    // User approved with Full access override - create the node with Full capabilities
                    if let Some(pending) = self.pending_permission_request.take() {
                        let node = pending.component_spec.create_node(pending.position);
                        let grant = CapabilityGrant {
                            node_id: node.id,
                            capability_set: CapabilitySet::Full, // Override with Full access
                            granted_at: chrono::Utc::now().to_rfc3339(),
                            scope: "graph".to_string(),
                        };

                        // Add the grant to the graph
                        self.graph.grant_capability(grant);

                        // Add the node to the graph through command history
                        let cmd = crate::graph::command::Command::AddNode { node };
                        if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                            self.error_message = Some(format!("Failed to add node: {}", e));
                        } else {
                            self.status_message = format!(
                                "Added {} node with FULL ACCESS (unrestricted)",
                                pending.component_spec.name
                            );
                            self.error_message = None;
                            self.dirty = true;
                        }
                    }
                }
                PermissionAction::Deny => {
                    // User denied the permissions - don't create the node
                    if let Some(pending) = self.pending_permission_request.take() {
                        self.status_message = format!(
                            "Permission denied for {} - node not created",
                            pending.component_spec.name
                        );
                    }
                }
            }
        }
    }

    /// Parse capability strings into a CapabilitySet
    /// Supports formats:
    /// - "file-read:/path"  -> FileRead { paths }
    /// - "file-write:/path" -> FileWrite { paths }
    /// - "network:host.com" -> Network { allowed_hosts }
    /// - Empty vec          -> None
    fn parse_capabilities(capabilities: &[String]) -> CapabilitySet {
        use std::path::PathBuf;

        if capabilities.is_empty() {
            return CapabilitySet::None;
        }

        let mut file_read_paths = Vec::new();
        let mut file_write_paths = Vec::new();
        let mut network_hosts = Vec::new();

        for cap in capabilities {
            if let Some(path) = cap.strip_prefix("file-read:") {
                file_read_paths.push(PathBuf::from(path));
            } else if let Some(path) = cap.strip_prefix("file-write:") {
                file_write_paths.push(PathBuf::from(path));
            } else if let Some(host) = cap.strip_prefix("network:") {
                network_hosts.push(host.to_string());
            }
        }

        // Determine the most appropriate capability set
        if !file_read_paths.is_empty() && !file_write_paths.is_empty() {
            // Combine read and write paths
            let mut all_paths = file_read_paths;
            all_paths.extend(file_write_paths);
            CapabilitySet::FileReadWrite { paths: all_paths }
        } else if !file_read_paths.is_empty() {
            CapabilitySet::FileRead {
                paths: file_read_paths,
            }
        } else if !file_write_paths.is_empty() {
            CapabilitySet::FileWrite {
                paths: file_write_paths,
            }
        } else if !network_hosts.is_empty() {
            CapabilitySet::Network {
                allowed_hosts: network_hosts,
            }
        } else {
            // Unknown capability format, default to None for safety
            CapabilitySet::None
        }
    }

    /// Get the path to the recent files configuration file
    fn recent_files_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("wasmflow");
            path.push("recent_files.json");
            path
        })
    }

    /// Load recent files from config
    fn load_recent_files() -> Vec<PathBuf> {
        if let Some(path) = Self::recent_files_path() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(files) = serde_json::from_str::<Vec<PathBuf>>(&contents) {
                    // Filter out files that no longer exist
                    return files.into_iter().filter(|f| f.exists()).collect();
                }
            }
        }
        Vec::new()
    }

    /// Save recent files to config
    fn save_recent_files(&self) {
        if let Some(path) = Self::recent_files_path() {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            // Save recent files
            if let Ok(contents) = serde_json::to_string_pretty(&self.recent_files) {
                let _ = std::fs::write(&path, contents);
            }
        }
    }

    /// Add a file to the recent files list
    fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);

        // Add to front
        self.recent_files.insert(0, path);

        // Keep only last 10
        self.recent_files.truncate(10);

        // Save to config
        self.save_recent_files();
    }

    /// Open a file from the recent files list
    fn open_recent_file(&mut self, path: PathBuf) {
        if !path.exists() {
            self.error_message = Some(format!("File not found: {}", path.display()));
            // Remove from recent files
            self.recent_files.retain(|p| p != &path);
            self.save_recent_files();
            return;
        }

        match NodeGraph::load_from_file(&path) {
            Ok(graph) => {
                self.graph = graph;
                self.current_file = Some(path.clone());
                self.dirty = false;
                self.history = CommandHistory::new(); // Clear undo history
                self.status_message = format!("Loaded {}", path.display());
                self.error_message = None;
                self.add_recent_file(path.clone());

                // Mark canvas dirty to force re-sync with loaded graph
                self.canvas.mark_dirty();

                log::info!("Graph loaded successfully from {}", path.display());
            }
            Err(e) => {
                // Provide user-friendly error messages for common issues
                let error_msg = if e.to_string().contains("magic bytes") {
                    "Invalid file format: This is not a WasmFlow graph file.".to_string()
                } else if e.to_string().contains("version") {
                    format!("Incompatible file version: {}. Please upgrade WasmFlow.", e)
                } else if e.to_string().contains("Checksum mismatch") {
                    "File appears to be corrupted. Please try another file.".to_string()
                } else {
                    format!("Failed to load graph: {}", e)
                };
                self.error_message = Some(error_msg);
            }
        }
    }

    /// Load a custom WASM component
    fn load_component(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WebAssembly Component", &["wasm"])
            .pick_file()
        {
            // Get the engine's component manager
            let component_manager = self.engine.component_manager();
            let mut cm = component_manager.lock().unwrap();

            // Load component and register with registry
            match cm.load_component_sync(&path) {
                Ok(component_spec) => {
                    let component_name = component_spec.name.clone();

                    // Register with the component registry
                    match self.registry.register_component(component_spec) {
                        Ok(()) => {
                            self.status_message = format!("Loaded component: {}", component_name);
                            self.error_message = None;
                        }
                        Err(e) => {
                            self.error_message =
                                Some(format!("Failed to register component: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load component: {}", e));
                }
            }
        }
    }

    /// T032: Show composition error dialog
    fn show_composition_error_dialog(&mut self, ctx: &egui::Context) {
        // Clone error message to avoid borrow checker issues
        let error_msg = self.composition_error.clone();

        if let Some(error_text) = error_msg {
            let mut should_close = false;

            egui::Window::new("Composition Error")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(8.0);

                        // Error icon and title
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("❌").size(24.0));
                            ui.label(
                                egui::RichText::new("Failed to compose nodes")
                                    .strong()
                                    .size(16.0),
                            );
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Error message in scrollable area
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 100, 100),
                                    &error_text,
                                );
                            });

                        ui.add_space(12.0);

                        // Help text
                        ui.label(egui::RichText::new("Requirements for composition:").strong());
                        ui.label("• Select at least 2 nodes");
                        ui.label("• All nodes must be connected");
                        ui.label("• Only user-defined WASM components can be composed");

                        ui.add_space(12.0);

                        // Close button
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });

            if should_close {
                self.composition_error = None;
            }
        }
    }

    /// T038: Handle drill-down into a composite node
    ///
    /// Enters drill-down mode to view the internal structure of a composite node
    fn handle_drill_down(&mut self, composite_node_id: Uuid) {
        log::info!("Attempting to drill down into node {}", composite_node_id);

        // Get the composite node
        let node = match self.graph.nodes.get(&composite_node_id) {
            Some(n) => n,
            None => {
                log::warn!("Node {} not found", composite_node_id);
                return;
            }
        };

        // Check if node has composition data
        let composition_data = match &node.composition_data {
            Some(data) => data,
            None => {
                log::warn!("Node {} is not a composite node", composite_node_id);
                return;
            }
        };

        // Drill down using the view stack
        if let Err(e) = self.view_stack.drill_down(
            composite_node_id,
            node.display_name.clone(),
            composition_data,
        ) {
            log::error!("Failed to drill down: {}", e);
            self.error_message = Some(format!("Cannot drill down: {}", e));
            return;
        }

        // Clear any error messages
        self.error_message = None;
        self.status_message = format!("Viewing internal structure of '{}'", node.display_name);

        // Mark canvas as needing sync
        self.canvas.mark_dirty();
    }

    /// T041: Navigate back from drill-down view
    fn handle_back_navigation(&mut self) {
        if self.view_stack.go_back() {
            self.status_message = "Returned to parent view".to_string();
            self.canvas.mark_dirty();
        }
    }

    /// T029: Get the component path for a given node
    ///
    /// For user-defined components, returns the path to the .wasm file.
    /// For composite components, returns the socket component path.
    /// For builtin components, returns None.
    fn get_component_path(
        &self,
        node: &crate::graph::node::GraphNode,
    ) -> Option<std::path::PathBuf> {
        // Look up the component spec
        let spec = self.registry.get_by_id(&node.component_id)?;

        match &spec.component_type {
            crate::graph::node::ComponentType::Builtin => None,
            crate::graph::node::ComponentType::UserDefined(path) => Some(path.clone()),
            crate::graph::node::ComponentType::Composed { socket_path, .. } => {
                Some(socket_path.clone())
            }
        }
    }

    /// Aggregate boundary ports from selected nodes for composite node
    ///
    /// Returns: (input_ports, output_ports, input_mappings, output_mappings)
    fn aggregate_boundary_ports(
        &self,
        selected_nodes: &[Uuid],
    ) -> (
        Vec<crate::graph::node::Port>,
        Vec<crate::graph::node::Port>,
        std::collections::BTreeMap<String, crate::graph::node::PortMapping>,
        std::collections::BTreeMap<String, crate::graph::node::PortMapping>,
    ) {
        use crate::graph::node::PortMapping;
        use std::collections::{BTreeMap, HashSet};

        let selected_set: HashSet<Uuid> = selected_nodes.iter().copied().collect();
        let mut composite_inputs = Vec::new();
        let mut composite_outputs = Vec::new();
        let mut input_mappings = BTreeMap::new();
        let mut output_mappings = BTreeMap::new();

        // For each selected node, check its ports for boundary conditions
        for node_id in selected_nodes {
            let Some(node) = self.graph.nodes.get(node_id) else {
                continue;
            };

            // Check input ports - boundary if they have connections from outside selection
            for input_port in &node.inputs {
                // Find connections TO this input port
                let has_external_input = self.graph.connections.iter().any(|conn| {
                    conn.to_node == *node_id
                        && conn.to_port == input_port.id
                        && !selected_set.contains(&conn.from_node)
                });

                if has_external_input {
                    // This is a boundary input - expose it on composite node
                    let external_name = format!("{}.{}", node.display_name, input_port.name);
                    log::debug!("Boundary input found: {}", external_name);

                    // Create port for composite node
                    let mut composite_port = input_port.clone();
                    composite_port.id = Uuid::new_v4(); // New ID for composite port
                    composite_port.name = external_name.clone();
                    composite_inputs.push(composite_port);

                    // Create port mapping
                    let mapping = PortMapping {
                        external_name: external_name.clone(),
                        internal_node_id: *node_id,
                        internal_port_name: input_port.name.clone(),
                        port_type: input_port.data_type.clone(),
                    };
                    input_mappings.insert(external_name, mapping);
                }
            }

            // Check output ports - boundary if they have NO connections or connections outside selection
            for output_port in &node.outputs {
                // Find connections FROM this output port
                let connections_from_port: Vec<_> = self
                    .graph
                    .connections
                    .iter()
                    .filter(|conn| conn.from_node == *node_id && conn.from_port == output_port.id)
                    .collect();

                // Boundary if: no connections OR any connection goes outside selection
                let is_boundary = connections_from_port.is_empty()
                    || connections_from_port
                        .iter()
                        .any(|conn| !selected_set.contains(&conn.to_node));

                if is_boundary {
                    // This is a boundary output - expose it on composite node
                    let external_name = format!("{}.{}", node.display_name, output_port.name);
                    log::debug!("Boundary output found: {}", external_name);

                    // Create port for composite node
                    let mut composite_port = output_port.clone();
                    composite_port.id = Uuid::new_v4(); // New ID for composite port
                    composite_port.name = external_name.clone();
                    composite_outputs.push(composite_port);

                    // Create port mapping
                    let mapping = PortMapping {
                        external_name: external_name.clone(),
                        internal_node_id: *node_id,
                        internal_port_name: output_port.name.clone(),
                        port_type: output_port.data_type.clone(),
                    };
                    output_mappings.insert(external_name, mapping);
                }
            }
        }

        log::info!(
            "Aggregated {} boundary inputs, {} boundary outputs",
            composite_inputs.len(),
            composite_outputs.len()
        );

        (
            composite_inputs,
            composite_outputs,
            input_mappings,
            output_mappings,
        )
    }

    /// T030: Handle composition action - compose selected nodes into a single composite node
    ///
    /// This is the core composition workflow:
    /// 1. Validate selection (≥2 nodes, all connected)
    /// 2. Extract selected subgraph
    /// 3. Collect component paths
    /// 4. Use ComponentComposer to generate composed binary
    /// 5. Create composite node with internal structure
    /// 6. Replace selected nodes with composite node
    fn handle_compose_action(&mut self) {
        log::info!("Starting composition workflow");

        // T033: Get selected nodes
        let selected_nodes: Vec<Uuid> = self
            .graph
            .nodes
            .iter()
            .filter(|(_, node)| node.selected)
            .map(|(id, _)| *id)
            .collect();

        // T033: Validate selection (need at least 2 nodes)
        if selected_nodes.len() < 2 {
            self.composition_error = Some("Please select at least 2 nodes to compose".to_string());
            log::warn!(
                "Composition failed: {} nodes selected (need ≥2)",
                selected_nodes.len()
            );
            return;
        }

        log::debug!("Selected {} nodes for composition", selected_nodes.len());

        // T033: Validate connectivity using validation module
        match crate::graph::validation::is_connected_subgraph(&self.graph, &selected_nodes) {
            Ok(true) => {
                log::debug!("Selected nodes form a connected subgraph");
            }
            Ok(false) => {
                self.composition_error = Some(
                    "Selected nodes must form a connected graph. Ensure all nodes are connected to each other.".to_string()
                );
                log::warn!("Composition failed: selected nodes are not connected");
                return;
            }
            Err(e) => {
                self.composition_error = Some(format!("Validation error: {}", e));
                log::error!("Composition validation failed: {}", e);
                return;
            }
        }

        // T029: Collect component paths for all selected nodes
        let mut component_paths: Vec<std::path::PathBuf> = Vec::new();
        let mut component_names: Vec<String> = Vec::new();

        for node_id in &selected_nodes {
            if let Some(node) = self.graph.nodes.get(node_id) {
                if let Some(path) = self.get_component_path(node) {
                    component_paths.push(path);
                    component_names.push(node.display_name.clone());
                } else {
                    // T033: Cannot compose builtin nodes (no WASM path)
                    self.composition_error = Some(format!(
                        "Cannot compose builtin node '{}'. Only user-defined WASM components can be composed.",
                        node.display_name
                    ));
                    log::warn!(
                        "Composition failed: builtin node '{}' in selection",
                        node.display_name
                    );
                    return;
                }
            }
        }

        // T033: Need at least 2 component paths
        if component_paths.len() < 2 {
            self.composition_error = Some("Need at least 2 WASM components to compose".to_string());
            return;
        }

        log::debug!("Collected {} component paths", component_paths.len());

        // T022-T025: Perform composition using ComponentComposer
        // Socket is the first component, plugs are the rest
        let socket = &component_paths[0];
        let plugs: Vec<&std::path::Path> =
            component_paths[1..].iter().map(|p| p.as_path()).collect();

        log::info!(
            "Composing: socket={}, plugs={}",
            socket.display(),
            plugs.len()
        );

        let composed_binary = match self.composer.compose(socket, &plugs) {
            Ok(bytes) => {
                log::info!("Composition successful: {} bytes", bytes.len());
                bytes
            }
            Err(e) => {
                // T032: Show composition error dialog
                self.composition_error = Some(format!("Composition failed: {}", e));
                log::error!("Composition failed: {}", e);
                return;
            }
        };

        // T027: Create CompositionData with internal structure
        let internal_nodes: std::collections::BTreeMap<Uuid, crate::graph::node::GraphNode> =
            selected_nodes
                .iter()
                .filter_map(|id| self.graph.nodes.get(id).map(|n| (*id, n.clone())))
                .collect();

        let internal_edges: Vec<crate::graph::connection::Connection> = self
            .graph
            .connections
            .iter()
            .filter(|conn| {
                selected_nodes.contains(&conn.from_node) && selected_nodes.contains(&conn.to_node)
            })
            .cloned()
            .collect();

        // T035: Aggregate boundary ports for the composite node
        let (composite_inputs, composite_outputs, input_mappings, output_mappings) =
            self.aggregate_boundary_ports(&selected_nodes);

        let mut composition_data = crate::graph::node::CompositionData::new(
            "CompositeNode".to_string(),
            socket.clone(),
            component_paths[1..].to_vec(),
            internal_nodes,
            internal_edges,
            component_names,
            composed_binary.clone(),
        );

        // Update composition data with port mappings
        composition_data.exposed_inputs = input_mappings;
        composition_data.exposed_outputs = output_mappings;

        // Calculate center position of selected nodes
        let center_pos = if !selected_nodes.is_empty() {
            let sum_x: f32 = selected_nodes
                .iter()
                .filter_map(|id| self.graph.nodes.get(id))
                .map(|n| n.position.x)
                .sum();
            let sum_y: f32 = selected_nodes
                .iter()
                .filter_map(|id| self.graph.nodes.get(id))
                .map(|n| n.position.y)
                .sum();
            let count = selected_nodes.len() as f32;
            egui::pos2(sum_x / count, sum_y / count)
        } else {
            egui::pos2(200.0, 200.0)
        };

        // T034: Create new composite node
        let mut composite_node = crate::graph::node::GraphNode::new(
            "composite:generated".to_string(),
            "Composite Node".to_string(),
            center_pos,
        );

        // Assign boundary ports to the composite node
        composite_node.inputs = composite_inputs;
        composite_node.outputs = composite_outputs;
        composite_node.composition_data = Some(composition_data);

        // Add the composite node to the graph
        let composite_id = composite_node.id;
        let _ = self.graph.add_node(composite_node);

        // Remove selected nodes
        for node_id in &selected_nodes {
            let _ = self.graph.remove_node(*node_id);
        }

        // Clear selection
        self.canvas.selection.clear_selection();

        // Mark dirty and update status
        self.dirty = true;
        self.status_message = format!(
            "Composed {} nodes into composite node",
            selected_nodes.len()
        );
        log::info!(
            "Composition complete: created composite node {}",
            composite_id
        );

        // Sync canvas
        self.canvas.mark_dirty();
    }

    /// Reload all components from the components/ directory
    fn reload_components(&mut self) {
        let components_dir = std::path::Path::new("components/bin");

        if !components_dir.exists() {
            self.error_message = Some("Components directory not found: components/bin/. Create this directory and place .wasm component files there.".to_string());
            return;
        }

        let mut loaded_count = 0;
        let mut error_count = 0;

        // Get the engine's component manager
        let component_manager = self.engine.component_manager();
        let mut cm = component_manager.lock().unwrap();

        // Scan for .wasm files
        if let Ok(entries) = std::fs::read_dir(components_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    // Load component and register with registry
                    match cm.load_component_sync(&path) {
                        Ok(component_spec) => {
                            // Register with the component registry
                            match self.registry.register_component(component_spec) {
                                Ok(()) => {
                                    loaded_count += 1;
                                }
                                Err(e) => {
                                    error_count += 1;
                                    log::warn!(
                                        "Failed to register component {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            log::warn!("Failed to load component {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        if error_count > 0 {
            self.status_message = format!(
                "Loaded {} components ({} errors)",
                loaded_count, error_count
            );
        } else if loaded_count > 0 {
            self.status_message = format!("Loaded {} components", loaded_count);
        } else {
            self.status_message = "No components found in components/bin/ directory".to_string();
        }

        self.error_message = None;
    }

    /// T099: Load a graph from a specific path (for CLI support)
    pub fn load_graph_from_path(&mut self, path: PathBuf) {
        match NodeGraph::load_from_file(&path) {
            Ok(graph) => {
                self.graph = graph;
                self.current_file = Some(path.clone());
                self.dirty = false;
                self.history = CommandHistory::new();
                self.status_message = format!("Loaded {}", path.display());
                self.error_message = None;
                self.add_recent_file(path.clone());

                // Mark canvas dirty to force re-sync with loaded graph
                self.canvas.mark_dirty();

                log::info!("Graph loaded successfully from {}", path.display());
            }
            Err(e) => {
                let error_msg = format!("Failed to load graph: {:#}", e);
                self.error_message = Some(error_msg.clone());
                log::error!("Failed to load graph from {}: {:#}", path.display(), e);
            }
        }
    }

    /// T099: Control palette visibility (for CLI support)
    pub fn set_palette_visible(&mut self, _visible: bool) {
        // TODO: Implement palette visibility toggle
        // For now, this is a placeholder - full implementation would require
        // adding a boolean field to WasmFlowApp and checking it in render_palette()
        log::warn!("--no-palette flag is not yet fully implemented");
    }

    /// T092: Open the metadata editor dialog
    fn open_metadata_dialog(&mut self) {
        self.metadata_dialog.open(
            self.graph.name.clone(),
            self.graph.metadata.author.clone(),
            self.graph.metadata.description.clone(),
            self.graph.metadata.created_at.clone(),
            self.graph.metadata.modified_at.clone(),
        );
    }

    /// T092: Handle metadata dialog response
    fn handle_metadata_dialog(&mut self, ctx: &egui::Context) {
        self.metadata_dialog.show(ctx);

        // Check if user saved changes
        if self.metadata_dialog.saved() {
            let (name, author, description) = self.metadata_dialog.get_metadata();

            // Update graph metadata
            self.graph.name = name;
            self.graph.metadata.author = author;
            self.graph.metadata.description = description;
            self.graph.metadata.touch();

            // Mark as dirty
            self.dirty = true;
            self.status_message = "Graph metadata updated".to_string();

            // Reset dialog
            self.metadata_dialog.reset();
        }
    }

    /// Render the top menu bar
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Graph").clicked() {
                        self.new_graph();
                        ui.close();
                    }
                    if ui.button("Open... (Ctrl+O)").clicked() {
                        self.load_graph();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Save (Ctrl+S)").clicked() {
                        self.save_graph();
                        ui.close();
                    }
                    if ui.button("Save As... (Ctrl+Shift+S)").clicked() {
                        self.save_graph_as();
                        ui.close();
                    }
                    ui.separator();

                    // Recent Files submenu
                    ui.menu_button("Recent Files", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label("No recent files");
                        } else {
                            // Clone to avoid borrow checker issues
                            let recent_files = self.recent_files.clone();
                            for path in recent_files {
                                let file_name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");

                                if ui
                                    .button(file_name)
                                    .on_hover_text(path.display().to_string())
                                    .clicked()
                                {
                                    self.open_recent_file(path);
                                    ui.close();
                                }
                            }
                        }
                    });

                    ui.separator();

                    // Component loading menu items
                    if ui.button("Load Component...").clicked() {
                        self.load_component();
                        ui.close();
                    }
                    if ui.button("Reload Components").clicked() {
                        self.reload_components();
                        ui.close();
                    }

                    ui.separator();
                    if ui.button("Quit").clicked() {
                        self.quit(ctx);
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    ui.add_enabled_ui(self.history.can_undo(), |ui| {
                        if ui.button("Undo (Ctrl+Z)").clicked() {
                            self.undo();
                            ui.close();
                        }
                    });
                    ui.add_enabled_ui(self.history.can_redo(), |ui| {
                        if ui.button("Redo (Ctrl+Y)").clicked() {
                            self.redo();
                            ui.close();
                        }
                    });

                    ui.separator();

                    // T092: Graph metadata editor
                    if ui.button("Graph Properties...").clicked() {
                        self.open_metadata_dialog();
                        ui.close();
                    }
                });

                // T100: Help menu with About dialog
                ui.menu_button("Help", |ui| {
                    if ui.button("About WasmFlow").clicked() {
                        self.about_dialog.open();
                        ui.close();
                    }
                });

                ui.separator();

                // T041: Back button - only show in drill-down mode
                if self.view_stack.is_drill_down() {
                    if ui.button("⬅ Back").clicked() {
                        self.handle_back_navigation();
                    }
                    ui.separator();
                }

                if ui.button("▶ Execute").clicked() {
                    self.execute_graph();
                }

                // T031: Compose button - only enabled when 2+ nodes are selected AND viewing main canvas
                let selected_count = self.graph.nodes.values().filter(|n| n.selected).count();
                let can_compose = selected_count >= 2 && self.view_stack.is_main_canvas();
                ui.add_enabled_ui(can_compose, |ui| {
                    let compose_button = ui.button("🔧 Compose");
                    if compose_button.clicked() {
                        self.handle_compose_action();
                    }
                    if !self.view_stack.is_main_canvas() {
                        compose_button.on_hover_text("Return to main canvas to compose nodes");
                    } else if selected_count < 2 {
                        compose_button.on_hover_text("Select at least 2 nodes to compose");
                    } else {
                        compose_button.on_hover_text(format!(
                            "Compose {} selected nodes into a composite",
                            selected_count
                        ));
                    }
                });

                ui.separator();

                // Show dirty indicator
                if self.dirty {
                    ui.label("●"); // Dirty indicator
                }

                // Show current file name
                if let Some(path) = &self.current_file {
                    ui.label(path.file_name().unwrap().to_string_lossy().to_string());
                } else {
                    ui.label("Untitled");
                }

                ui.separator();

                // T039: Breadcrumb navigation
                if self.view_stack.is_drill_down() {
                    ui.separator();
                    ui.label("📍");
                    let breadcrumbs = self.view_stack.breadcrumb_path();
                    for (i, (name, _depth)) in breadcrumbs.iter().enumerate() {
                        if i > 0 {
                            ui.label("›");
                        }
                        ui.label(egui::RichText::new(name).strong());
                    }
                    ui.separator();
                }

                // T040: Show node count from current view context
                let (node_count, connection_count) = match self.view_stack.current() {
                    crate::graph::drill_down::ViewContext::MainCanvas => {
                        (self.graph.nodes.len(), self.graph.connections.len())
                    }
                    crate::graph::drill_down::ViewContext::DrillDown {
                        internal_nodes,
                        internal_edges,
                        ..
                    } => (internal_nodes.len(), internal_edges.len()),
                };
                ui.label(format!("Nodes: {}", node_count));
                ui.label(format!("Connections: {}", connection_count));
            });
        });
    }

    /// Render the status bar at the bottom
    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
                } else {
                    ui.label(&self.status_message);
                }
            });
        });
    }

    /// T091: Render the component palette on the left with search functionality
    fn render_palette(&mut self, ctx: &egui::Context) {
        // Show the palette widget and handle any actions
        if let Some(action) = self.palette.show(ctx, &self.registry, &self.theme) {
            match action {
                PaletteAction::AddComponent { spec, position } => {
                    // T073: Check if this is a user-defined component that needs permission
                    if let crate::graph::node::ComponentType::UserDefined(_) = spec.component_type {
                        // Extract capabilities from component metadata
                        let requested_capabilities =
                            Self::parse_capabilities(&spec.required_capabilities);

                        // T080: Check for capability escalation
                        // Look for existing grants for this component_id
                        let existing_grant = self
                            .graph
                            .nodes
                            .values()
                            .find(|node| node.component_id == spec.id)
                            .and_then(|node| self.graph.get_capability_grant(node.id));

                        let needs_approval = if let Some(grant) = existing_grant {
                            // Compare requested vs granted capabilities
                            // If they differ, we need re-approval
                            grant.capability_set != requested_capabilities
                        } else {
                            // No existing grant - always need approval
                            true
                        };

                        if needs_approval {
                            // Show permission dialog for approval
                            self.permission_dialog.open(
                                spec.name.clone(),
                                spec.description.clone(),
                                requested_capabilities.clone(),
                            );

                            // Store pending request for later approval
                            self.pending_permission_request = Some(PendingPermissionRequest {
                                component_spec: spec,
                                capabilities: requested_capabilities,
                                position,
                            });
                        } else {
                            // Reuse existing grant - add node directly
                            let node = spec.create_node(position);
                            let grant = existing_grant.unwrap(); // Safe because we checked above

                            // Create a new grant for this node instance
                            let new_grant = CapabilityGrant {
                                node_id: node.id,
                                capability_set: grant.capability_set.clone(),
                                granted_at: chrono::Utc::now().to_rfc3339(),
                                scope: grant.scope.clone(),
                            };
                            self.graph.grant_capability(new_grant);

                            // Add the node
                            let cmd = crate::graph::command::Command::AddNode { node };
                            if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                                self.error_message = Some(format!("Failed to add node: {}", e));
                            } else {
                                self.status_message = format!(
                                    "Added {} node (reusing existing permissions)",
                                    spec.name
                                );
                                self.error_message = None;
                                self.dirty = true;
                            }
                        }
                    } else {
                        // Builtin component - add directly without permission dialog
                        let node = spec.create_node(position);
                        let cmd = crate::graph::command::Command::AddNode { node };

                        if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                            self.error_message = Some(format!("Failed to add node: {}", e));
                        } else {
                            self.status_message = format!("Added {} node", spec.name);
                            self.error_message = None;
                            self.dirty = true;
                        }
                    }
                }
            }
        }
    }

    /// Render the main canvas area
    fn render_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // T040: Check if we're in drill-down mode
            match self.view_stack.current() {
                crate::graph::drill_down::ViewContext::MainCanvas => {
                    // Show the main graph
                    self.canvas.show(ui, &mut self.graph, &self.registry);
                }
                crate::graph::drill_down::ViewContext::DrillDown {
                    composite_node_id: _,
                    composite_node_name,
                    internal_nodes,
                    internal_edges,
                } => {
                    // Create a temporary graph from the internal structure
                    let mut temp_graph = crate::graph::NodeGraph::new(
                        format!("Internal: {}", composite_node_name),
                        "Composite".to_string(),
                    );

                    // Clone internal nodes into the temp graph
                    temp_graph.nodes = internal_nodes.clone();
                    temp_graph.connections = internal_edges.clone();

                    // Show the internal structure
                    self.canvas.show(ui, &mut temp_graph, &self.registry);
                }
            }
        });

        // T043: Only process modifications when viewing main canvas (read-only drill-down)
        if self.view_stack.is_main_canvas() {
            // Process pending node deletions through command history
            if !self.canvas.pending_deletions.is_empty() {
                for node_id in self.canvas.pending_deletions.drain(..) {
                    // Create RemoveNode command
                    let cmd = crate::graph::command::Command::RemoveNode {
                        node_id,
                        node: crate::graph::node::GraphNode::new(
                            String::new(),
                            String::new(),
                            egui::Pos2::ZERO,
                        ), // Placeholder, will be filled by execute()
                        connections: Vec::new(), // Placeholder, will be filled by execute()
                    };

                    if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                        self.error_message = Some(format!("Failed to delete node: {}", e));
                    } else {
                        self.status_message = "Node deleted".to_string();
                        self.error_message = None;
                        self.dirty = true;
                    }
                }
            }
        } else {
            // In drill-down mode - discard any modification attempts
            if !self.canvas.pending_deletions.is_empty() {
                self.canvas.pending_deletions.clear();
                self.status_message =
                    "Drill-down view is read-only. Return to main canvas to make changes."
                        .to_string();
            }
        }

        // T078: Process pending permission view request
        if let Some(node_id) = self.canvas.pending_permission_view.take() {
            if let Some(node) = self.graph.nodes.get(&node_id) {
                let grant = self.graph.get_capability_grant(node_id).cloned();
                self.permissions_view_dialog
                    .open(node_id, node.display_name.clone(), grant);
            }
        }

        // T040: Process pending drill-down request
        if let Some(composite_node_id) = self.canvas.pending_drill_down.take() {
            self.handle_drill_down(composite_node_id);
        }

        // Handle pending continuous node start requests
        let pending_starts: Vec<Uuid> = self.canvas.pending_continuous_start.drain(..).collect();
        if !pending_starts.is_empty() {
            // T044: Pass Arc<Mutex<NodeGraph>> so continuous nodes can read updated input values
            let graph_arc = std::sync::Arc::new(std::sync::Mutex::new(self.graph.clone()));
            let component_manager = self.engine.component_manager();
            let result_tx = self.continuous_result_tx.clone();

            for node_id in pending_starts {
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    if let Some(config) = &mut node.continuous_config {
                        // T041: Clear error state if restarting from error
                        if matches!(
                            config.runtime_state.execution_state,
                            crate::graph::node::ContinuousExecutionState::Error
                        ) {
                            config.runtime_state.last_error = None;
                            log::info!("Clearing error state for continuous node {}", node_id);
                        }

                        // Update state to Starting
                        config.runtime_state.execution_state =
                            crate::graph::node::ContinuousExecutionState::Starting;

                        // Start the continuous execution
                        if let Err(e) = self.continuous_manager.start_node(
                            node_id,
                            graph_arc.clone(),
                            component_manager.clone(),
                            result_tx.clone(),
                        ) {
                            self.error_message =
                                Some(format!("Failed to start continuous node: {}", e));
                            config.runtime_state.execution_state =
                                crate::graph::node::ContinuousExecutionState::Error;
                            config.runtime_state.last_error = Some(e.to_string());
                        } else {
                            self.status_message = format!("Started continuous node");
                        }
                    }
                }
            }
            self.canvas.mark_dirty();
        }

        // Handle pending continuous node stop requests
        let pending_stops: Vec<Uuid> = self.canvas.pending_continuous_stop.drain(..).collect();
        if !pending_stops.is_empty() {
            for node_id in pending_stops {
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    if let Some(config) = &mut node.continuous_config {
                        // Update state to Stopping
                        config.runtime_state.execution_state =
                            crate::graph::node::ContinuousExecutionState::Stopping;

                        // Stop the continuous execution
                        if let Err(e) = self.continuous_manager.stop_node(node_id) {
                            self.error_message =
                                Some(format!("Failed to stop continuous node: {}", e));
                            config.runtime_state.execution_state =
                                crate::graph::node::ContinuousExecutionState::Error;
                            config.runtime_state.last_error = Some(e.to_string());
                        } else {
                            self.status_message = format!("Stopping continuous node...");
                        }
                    }
                }
            }
            self.canvas.mark_dirty();
        }
    }
}

impl eframe::App for WasmFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incremental execution step on main thread
        if self.execution_state.is_some() {
            self.process_execution_step();
            ctx.request_repaint(); // Keep processing until done
        }

        // Poll continuous execution results
        self.poll_continuous_results();

        // Poll downstream execution results (triggered by continuous nodes)
        self.poll_downstream_results();

        // Request continuous repaints when there are running nodes (for spinner/elapsed time)
        let has_running_nodes = self.graph.nodes.values().any(|n| {
            matches!(
                n.execution_state,
                crate::graph::node::ExecutionState::Running
            )
        });

        // Also check for continuous nodes in running state
        let has_continuous_running = self.graph.nodes.values().any(|n| {
            if let Some(config) = &n.continuous_config {
                matches!(
                    config.runtime_state.execution_state,
                    crate::graph::node::ContinuousExecutionState::Running
                        | crate::graph::node::ContinuousExecutionState::Starting
                        | crate::graph::node::ContinuousExecutionState::Stopping
                )
            } else {
                false
            }
        });

        if has_running_nodes || has_continuous_running {
            ctx.request_repaint();
        }

        // Intercept window close event to show unsaved changes dialog
        if ctx.input(|i| i.viewport().close_requested())
            && self.dirty
            && !self.unsaved_changes_dialog.is_open()
        {
            // Prevent close and show dialog
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.unsaved_changes_dialog.open();
            self.pending_action = Some(PendingAction::Quit);
        }

        // Handle unsaved changes dialog
        self.handle_unsaved_changes_dialog(ctx);

        // T073: Handle permission dialog
        self.handle_permission_dialog(ctx);

        // T078: Handle permissions view dialog
        self.handle_permissions_view_dialog(ctx);

        // T092: Handle graph metadata dialog
        self.handle_metadata_dialog(ctx);

        // T100: Show about dialog
        self.about_dialog.show(ctx);

        // T032: Show composition error dialog
        self.show_composition_error_dialog(ctx);

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command) {
            if ctx.input(|i| i.modifiers.shift) {
                // Ctrl+Shift+Z -> Redo (alternative to Ctrl+Y)
                self.redo();
            } else {
                // Ctrl+Z -> Undo
                self.undo();
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.command) {
            // Ctrl+Y -> Redo
            self.redo();
        } else if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command) {
            if ctx.input(|i| i.modifiers.shift) {
                // Ctrl+Shift+S -> Save As
                self.save_graph_as();
            } else {
                // Ctrl+S -> Save
                self.save_graph();
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.command) {
            // Ctrl+O -> Open
            self.load_graph();
        }

        self.render_menu_bar(ctx);
        self.render_status_bar(ctx);
        self.render_palette(ctx);
        self.render_canvas(ctx);
    }
}

impl Drop for WasmFlowApp {
    fn drop(&mut self) {
        // Gracefully shutdown all running continuous nodes
        log::info!("Shutting down WasmFlow application");
        self.continuous_manager.shutdown();
    }
}
