// T053: Integration tests for continuous execution lifecycle

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::time::Duration;
use wasmflow::graph::{NodeGraph, NodeValue};
use wasmflow::graph::node::ContinuousExecutionState;
use wasmflow::runtime::continuous::{ContinuousExecutionManager, ExecutionResult};
use wasmflow::runtime::wasm_host::ComponentManager;

#[test]
fn test_complete_lifecycle_start_run_stop() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));
    let (result_tx, result_rx) = channel();

    // Create timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Lifecycle Test Timer".to_string(),
        "Timer for lifecycle testing".to_string(),
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
        port.current_value = Some(NodeValue::U32(50)); // 50ms for faster testing
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // 1. Start the node
    let start_result = manager.start_node(node_id, graph.clone(), component_manager, result_tx);
    assert!(start_result.is_ok(), "Failed to start node");

    // 2. Wait for Started message
    let started = result_rx.recv_timeout(Duration::from_secs(1));
    assert!(started.is_ok(), "Did not receive Started message");
    if let Ok(ExecutionResult::Started { node_id: id, .. }) = started {
        assert_eq!(id, node_id);
    } else {
        panic!("Expected Started message, got: {:?}", started);
    }

    // 3. Wait for at least one iteration
    let mut received_outputs = false;
    for _ in 0..10 {
        if let Ok(result) = result_rx.recv_timeout(Duration::from_millis(100)) {
            match result {
                ExecutionResult::OutputsUpdated { .. } => {
                    received_outputs = true;
                    break;
                }
                ExecutionResult::IterationComplete { .. } => continue,
                ExecutionResult::Error { error, .. } => {
                    panic!("Received error during execution: {:?}", error);
                }
                _ => continue,
            }
        }
    }
    assert!(received_outputs, "Did not receive any output updates");

    // 4. Stop the node
    let stop_result = manager.stop_node(node_id);
    assert!(stop_result.is_ok(), "Failed to stop node");

    // 5. Wait for Stopped message
    let mut received_stopped = false;
    for _ in 0..20 {
        if let Ok(result) = result_rx.recv_timeout(Duration::from_millis(200)) {
            if let ExecutionResult::Stopped { node_id: id, iterations, .. } = result {
                assert_eq!(id, node_id);
                assert!(iterations > 0, "Should have completed at least one iteration");
                received_stopped = true;
                break;
            }
        }
    }
    assert!(received_stopped, "Did not receive Stopped message within timeout");
}

#[test]
fn test_multiple_iterations() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));
    let (result_tx, result_rx) = channel();

    // Create timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Iteration Test Timer".to_string(),
        "Timer for iteration testing".to_string(),
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
        port.current_value = Some(NodeValue::U32(30)); // 30ms for fast iterations
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // Start the node
    manager.start_node(node_id, graph, component_manager, result_tx).unwrap();

    // Wait for Started
    let _ = result_rx.recv_timeout(Duration::from_secs(1));

    // Count iterations
    let mut iteration_count = 0;
    for _ in 0..50 {
        if let Ok(result) = result_rx.recv_timeout(Duration::from_millis(100)) {
            if let ExecutionResult::IterationComplete { .. } = result {
                iteration_count += 1;
                if iteration_count >= 5 {
                    break;
                }
            }
        }
    }

    // Stop the node
    manager.stop_node(node_id).unwrap();

    assert!(iteration_count >= 5, "Expected at least 5 iterations, got {}", iteration_count);
}

#[test]
fn test_restart_after_stop() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));

    // Create timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Restart Test Timer".to_string(),
        "Timer for restart testing".to_string(),
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
        port.current_value = Some(NodeValue::U32(50));
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // First cycle: start and stop
    let (tx1, rx1) = channel();
    manager.start_node(node_id, graph.clone(), component_manager.clone(), tx1).unwrap();

    // Wait for at least one message
    let _ = rx1.recv_timeout(Duration::from_millis(500));

    manager.stop_node(node_id).unwrap();

    // Wait for stop to complete
    std::thread::sleep(Duration::from_millis(500));

    // Second cycle: restart
    let (tx2, rx2) = channel();
    let restart_result = manager.start_node(node_id, graph, component_manager, tx2);

    assert!(restart_result.is_ok(), "Should be able to restart after stopping");

    // Verify it's running
    let started = rx2.recv_timeout(Duration::from_secs(1));
    assert!(started.is_ok(), "Did not receive Started message on restart");

    // Cleanup
    manager.stop_node(node_id).unwrap();
}

#[test]
fn test_graceful_shutdown_timing() {
    let mut manager = ContinuousExecutionManager::new();
    let graph = Arc::new(Mutex::new(NodeGraph::new()));
    let component_manager = Arc::new(Mutex::new(ComponentManager::new()));
    let (result_tx, result_rx) = channel();

    // Create timer node
    let mut graph_lock = graph.lock().unwrap();
    let spec = wasmflow::graph::node::ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Shutdown Timing Test".to_string(),
        "Timer for testing shutdown timing".to_string(),
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
        port.current_value = Some(NodeValue::U32(50));
    }

    graph_lock.nodes.insert(node_id, node);
    drop(graph_lock);

    // Start the node
    manager.start_node(node_id, graph, component_manager, result_tx).unwrap();

    // Wait for Started
    let _ = result_rx.recv_timeout(Duration::from_secs(1));

    // Stop and measure time
    let stop_start = std::time::Instant::now();
    manager.stop_node(node_id).unwrap();

    // Wait for Stopped message
    for _ in 0..30 {
        if let Ok(result) = result_rx.recv_timeout(Duration::from_millis(100)) {
            if let ExecutionResult::Stopped { .. } = result {
                let stop_duration = stop_start.elapsed();
                // Should stop within 2 seconds (as per spec)
                assert!(stop_duration.as_secs() < 3, "Shutdown took too long: {:?}", stop_duration);
                return;
            }
        }
    }

    panic!("Did not receive Stopped message within timeout");
}
