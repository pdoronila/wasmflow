// T055: Unit tests for ContinuousExecutionManager

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use wasmflow::graph::{NodeGraph, NodeValue};
use wasmflow::runtime::continuous::{ContinuousExecutionManager, ExecutionResult};
use wasmflow::runtime::wasm_host::ComponentManager;

#[test]
fn test_manager_creation() {
    let manager = ContinuousExecutionManager::new();
    // Manager should be created successfully
    assert!(std::mem::size_of_val(&manager) > 0);
}

#[test]
fn test_start_node_not_running() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));
    let (result_tx, _result_rx) = channel();

    // Create a simple timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Test Timer".to_string(),
        "Test timer node".to_string(),
        None,
    );

    let mut node = spec.create_node("Timer".to_string(), egui::Pos2::ZERO);
    let node_id = node.id;

    // Configure as continuous
    node.continuous_config = Some(wasmflow::graph::node::ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: wasmflow::graph::node::ContinuousRuntimeState::default(),
    });

    // Add default interval
    if let Some(port) = node.get_input_mut("interval") {
        port.current_value = Some(NodeValue::U32(100));
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // Start the node
    let result = manager.start_node(node_id, graph.clone(), component_manager, result_tx);

    // Should succeed
    assert!(result.is_ok(), "Failed to start node: {:?}", result.err());
}

#[test]
fn test_start_node_already_running() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));
    let (result_tx, _result_rx) = channel();

    // Create and configure timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Test Timer".to_string(),
        "Test timer node".to_string(),
        None,
    );

    let mut node = spec.create_node("Timer".to_string(), egui::Pos2::ZERO);
    let node_id = node.id;

    node.continuous_config = Some(wasmflow::graph::node::ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: wasmflow::graph::node::ContinuousRuntimeState::default(),
    });

    if let Some(port) = node.get_input_mut("interval") {
        port.current_value = Some(NodeValue::U32(100));
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // Start the node first time
    let (tx1, _rx1) = channel();
    manager.start_node(node_id, graph.clone(), component_manager.clone(), tx1).unwrap();

    // Try to start again - should fail
    let result = manager.start_node(node_id, graph, component_manager, result_tx);

    assert!(result.is_err(), "Should fail when starting already running node");

    // Cleanup
    let _ = manager.stop_node(node_id);
}

#[test]
fn test_stop_node_gracefully() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));
    let (result_tx, _result_rx) = channel();

    // Create and configure timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Test Timer".to_string(),
        "Test timer node".to_string(),
        None,
    );

    let mut node = spec.create_node("Timer".to_string(), egui::Pos2::ZERO);
    let node_id = node.id;

    node.continuous_config = Some(wasmflow::graph::node::ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: wasmflow::graph::node::ContinuousRuntimeState::default(),
    });

    if let Some(port) = node.get_input_mut("interval") {
        port.current_value = Some(NodeValue::U32(100));
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // Start the node
    manager.start_node(node_id, graph, component_manager, result_tx).unwrap();

    // Stop the node
    let result = manager.stop_node(node_id);

    assert!(result.is_ok(), "Failed to stop node: {:?}", result.err());
}

#[test]
fn test_stop_node_not_running() {
    let mut manager = ContinuousExecutionManager::new();
    let node_id = uuid::Uuid::new_v4();

    // Try to stop a node that's not running
    let result = manager.stop_node(node_id);

    // Should fail gracefully
    assert!(result.is_err(), "Should fail when stopping non-running node");
}

#[test]
fn test_shutdown_all() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));

    // Start multiple nodes
    for i in 0..3 {
        let (result_tx, _result_rx) = channel();

        let mut graph_lock = graph.lock().unwrap();
        let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
            "builtin:continuous:timer".to_string(),
            format!("Test Timer {}", i),
            "Test timer node".to_string(),
            None,
        );

        let mut node = spec.create_node(format!("Timer {}", i), egui::Pos2::ZERO);
        let node_id = node.id;

        node.continuous_config = Some(wasmflow::graph::node::ContinuousNodeConfig {
            supports_continuous: true,
            enabled: true,
            runtime_state: wasmflow::graph::node::ContinuousRuntimeState::default(),
        });

        if let Some(port) = node.get_input_mut("interval") {
            port.current_value = Some(NodeValue::U32(100));
        }

        graph_lock.nodes.insert(node_id, node);
        drop(graph_lock);

        manager.start_node(node_id, graph.clone(), component_manager.clone(), result_tx).unwrap();
    }

    // Shutdown all
    manager.shutdown_all();

    // All nodes should be stopped (test passes if no panic)
}
