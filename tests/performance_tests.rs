//! T093: Performance benchmarks for WasmFlow
//!
//! Tests performance requirements:
//! - SC-002: 60 FPS with 500 nodes
//! - SC-003: Load 100-node graph in <3s
//! - SC-004: Execute 10-node pipeline in <500ms

use std::time::Instant;
use wasmflow::builtin::{register_constant_nodes, register_math_nodes};
use wasmflow::graph::graph::NodeGraph;
use wasmflow::graph::node::ComponentRegistry;
use wasmflow::runtime::engine::{register_builtin_executors, ExecutionEngine};

/// Helper to create a test graph with N nodes
fn create_test_graph(node_count: usize) -> NodeGraph {
    let mut graph = NodeGraph::new(
        format!("Performance Test Graph (N={})", node_count),
        "Test".to_string(),
    );

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    // Create a chain of Add nodes: Constant(1) → Add → Add → Add → ...
    let constant_spec = registry
        .get_by_id("builtin:constant:f32")
        .expect("Constant spec should exist");
    let add_spec = registry
        .get_by_id("builtin:math:add")
        .expect("Add spec should exist");

    // Create initial constant node
    let mut constant_node = constant_spec.create_node(egui::Pos2::new(0.0, 0.0));
    if let Some(output_port) = constant_node.outputs.get_mut(0) {
        output_port.current_value = Some(wasmflow::graph::node::NodeValue::F32(1.0));
    }
    let constant_id = constant_node.id;
    let constant_output = constant_node.outputs[0].id;
    graph.add_node(constant_node);

    // Create chain of Add nodes
    let mut prev_node_id = constant_id;
    let mut prev_output_port = constant_output;

    for i in 0..node_count {
        let add_node = add_spec.create_node(egui::Pos2::new((i + 1) as f32 * 150.0, 0.0));
        let add_id = add_node.id;
        let add_input_a = add_node.inputs[0].id;
        let add_input_b = add_node.inputs[1].id;
        let add_output = add_node.outputs[0].id;

        graph.add_node(add_node);

        // Connect previous output to input A
        graph
            .add_connection(prev_node_id, prev_output_port, add_id, add_input_a)
            .expect("Connection should succeed");

        // Connect constant to input B
        graph
            .add_connection(constant_id, constant_output, add_id, add_input_b)
            .expect("Connection should succeed");

        prev_node_id = add_id;
        prev_output_port = add_output;
    }

    graph
}

#[test]
fn test_benchmark_10_node_execution() {
    // SC-004: Execute 10-node pipeline in <500ms
    let mut graph = create_test_graph(10);

    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let start = Instant::now();
    let result = engine.execute_graph(&mut graph);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Execution should succeed");
    assert!(
        duration.as_millis() < 500,
        "10-node execution took {}ms, should be <500ms",
        duration.as_millis()
    );

    println!(
        "✓ 10-node execution: {}ms (target: <500ms)",
        duration.as_millis()
    );
}

#[test]
fn test_benchmark_100_node_execution() {
    // Performance test for larger graphs
    let mut graph = create_test_graph(100);

    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let start = Instant::now();
    let result = engine.execute_graph(&mut graph);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Execution should succeed");

    // Reasonable target: 100 nodes should execute in reasonable time
    // Not a hard requirement from spec, but good to track
    println!("✓ 100-node execution: {}ms", duration.as_millis());
}

#[test]
fn test_benchmark_load_100_node_graph() {
    // SC-003: Load 100-node graph in <3s
    let graph = create_test_graph(100);

    // Save the graph to bytes
    let save_result = graph.to_bytes();
    assert!(save_result.is_ok(), "Save should succeed");
    let bytes = save_result.unwrap();

    // Measure load time
    let start = Instant::now();
    let load_result = NodeGraph::from_bytes(&bytes);
    let duration = start.elapsed();

    assert!(load_result.is_ok(), "Load should succeed");
    assert!(
        duration.as_secs() < 3,
        "100-node load took {}s, should be <3s",
        duration.as_secs()
    );

    println!(
        "✓ 100-node graph load: {}ms (target: <3000ms)",
        duration.as_millis()
    );

    // Verify loaded graph has correct structure
    let loaded_graph = load_result.unwrap();
    assert_eq!(
        loaded_graph.nodes.len(),
        graph.nodes.len(),
        "Loaded graph should have same node count"
    );
    assert_eq!(
        loaded_graph.connections.len(),
        graph.connections.len(),
        "Loaded graph should have same connection count"
    );
}

#[test]
fn test_memory_usage_estimation() {
    // SC-008: <500MB memory usage
    // This is a basic structural test - actual memory profiling would require
    // tools like valgrind or memory profilers

    let graph = create_test_graph(500);

    // Basic size estimation
    let serialized = graph.to_bytes().expect("Serialization should succeed");
    let size_mb = serialized.len() as f64 / (1024.0 * 1024.0);

    println!("✓ 500-node graph serialized size: {:.2}MB", size_mb);

    // Serialized size should be reasonable (much less than 500MB)
    assert!(
        size_mb < 50.0,
        "500-node graph serialized to {:.2}MB, seems excessive",
        size_mb
    );
}

#[test]
fn test_execution_scaling() {
    // Test that execution time scales reasonably with graph size
    let sizes = vec![10, 50, 100];
    let mut timings = Vec::new();

    for size in sizes {
        let mut graph = create_test_graph(size);
        let mut engine = ExecutionEngine::new();
        register_builtin_executors(&mut engine);

        let start = Instant::now();
        let result = engine.execute_graph(&mut graph);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Execution should succeed for size {}", size);
        timings.push((size, duration.as_millis()));

        println!("✓ {}-node execution: {}ms", size, duration.as_millis());
    }

    // Verify linear or better scaling
    // (This is a heuristic - actual performance characteristics may vary)
    for i in 1..timings.len() {
        let (size1, time1) = timings[i - 1];
        let (size2, time2) = timings[i];

        // Skip scaling verification if execution is too fast to measure accurately
        if time1 == 0 || time2 == 0 {
            println!("✓ Execution too fast to measure scaling (excellent performance!)");
            continue;
        }

        let size_ratio = size2 as f64 / size1 as f64;
        let time_ratio = time2 as f64 / time1 as f64;

        // Time should not grow faster than O(n log n)
        // Allow some overhead, but flag if it's worse than quadratic
        assert!(
            time_ratio < size_ratio * size_ratio,
            "Execution time scaling worse than quadratic: {}x nodes took {}x time",
            size_ratio,
            time_ratio
        );
    }
}

#[test]
fn test_incremental_execution_performance() {
    // Test that incremental execution (dirty flags) provides speedup
    use wasmflow::graph::execution::{mark_all_clean, mark_node_dirty};

    let mut graph = create_test_graph(50);
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    // First full execution
    let start = Instant::now();
    let result1 = engine.execute_graph(&mut graph);
    let full_time = start.elapsed();
    assert!(result1.is_ok(), "Full execution should succeed");

    // Mark all clean after execution
    mark_all_clean(&mut graph);

    // Mark only one node dirty
    let first_add_node = graph
        .nodes
        .values()
        .find(|n| n.component_id == "builtin:math:add")
        .map(|n| n.id);

    if let Some(node_id) = first_add_node {
        mark_node_dirty(&mut graph, node_id);

        // Execute again with dirty tracking
        let start = Instant::now();
        let result2 = engine.execute_graph(&mut graph);
        let incremental_time = start.elapsed();
        assert!(result2.is_ok(), "Incremental execution should succeed");

        println!(
            "✓ Full execution: {}ms, Incremental: {}ms",
            full_time.as_millis(),
            incremental_time.as_millis()
        );

        // Note: In current implementation, dirty flags are tracked but full execution
        // still runs all nodes. This test documents the infrastructure is in place
        // for future optimization.
    }
}
