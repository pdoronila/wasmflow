//! T094: Integration tests for graph execution
//!
//! Tests end-to-end graph creation, execution, and output validation

use wasmflow::builtin::{register_constant_nodes, register_math_nodes};
use wasmflow::graph::graph::NodeGraph;
use wasmflow::graph::node::{ComponentRegistry, NodeValue};
use wasmflow::runtime::engine::{register_builtin_executors, ExecutionEngine};

#[test]
fn test_simple_add_execution() {
    // Create graph: Constant(5) → Add.a, Constant(3) → Add.b
    // Expected output: Add.sum = 8.0

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Test Add".to_string(), "Test".to_string());

    // Create constant nodes
    let const_spec = registry
        .get_by_id("builtin:constant:f32")
        .expect("Constant spec");
    let add_spec = registry.get_by_id("builtin:math:add").expect("Add spec");

    let mut const_5 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    const_5.outputs[0].current_value = Some(NodeValue::F32(5.0));
    let const_5_id = const_5.id;
    let const_5_output = const_5.outputs[0].id;

    let mut const_3 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    const_3.outputs[0].current_value = Some(NodeValue::F32(3.0));
    let const_3_id = const_3.id;
    let const_3_output = const_3.outputs[0].id;

    let add_node = add_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let add_id = add_node.id;
    let add_input_a = add_node.inputs[0].id;
    let add_input_b = add_node.inputs[1].id;

    graph.add_node(const_5);
    graph.add_node(const_3);
    graph.add_node(add_node);

    // Connect nodes
    graph
        .add_connection(const_5_id, const_5_output, add_id, add_input_a)
        .expect("Connect 5 → Add.a");
    graph
        .add_connection(const_3_id, const_3_output, add_id, add_input_b)
        .expect("Connect 3 → Add.b");

    // Execute
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Execution should succeed");

    let report = result.unwrap();
    assert!(report.success(), "Execution should be successful");

    // Verify output
    let add_node = graph.nodes.get(&add_id).expect("Add node should exist");
    let output_value = add_node.outputs[0]
        .current_value
        .as_ref()
        .expect("Output should have value");

    match output_value {
        NodeValue::F32(val) => {
            assert_eq!(*val, 8.0, "5 + 3 should equal 8");
        }
        _ => panic!("Output should be F32"),
    }

    println!("✓ Simple add execution: 5 + 3 = 8");
}

#[test]
fn test_chain_execution() {
    // Create chain: Constant(2) → Multiply(x2) → Add(+3) → Subtract(-1)
    // Expected: ((2 * 2) + 3) - 1 = 6

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Test Chain".to_string(), "Test".to_string());

    // Get specs
    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let multiply_spec = registry.get_by_id("builtin:math:multiply").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();
    let subtract_spec = registry.get_by_id("builtin:math:subtract").unwrap();

    // Create nodes
    let mut const_2 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    const_2.outputs[0].current_value = Some(NodeValue::F32(2.0));
    let const_2_id = const_2.id;
    let const_2_out = const_2.outputs[0].id;

    let mut const_mult = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    const_mult.outputs[0].current_value = Some(NodeValue::F32(2.0));
    let const_mult_id = const_mult.id;
    let const_mult_out = const_mult.outputs[0].id;

    let mut const_add = const_spec.create_node(egui::Pos2::new(0.0, 200.0));
    const_add.outputs[0].current_value = Some(NodeValue::F32(3.0));
    let const_add_id = const_add.id;
    let const_add_out = const_add.outputs[0].id;

    let mut const_sub = const_spec.create_node(egui::Pos2::new(0.0, 300.0));
    const_sub.outputs[0].current_value = Some(NodeValue::F32(1.0));
    let const_sub_id = const_sub.id;
    let const_sub_out = const_sub.outputs[0].id;

    let mult_node = multiply_spec.create_node(egui::Pos2::new(200.0, 0.0));
    let mult_id = mult_node.id;
    let mult_a = mult_node.inputs[0].id;
    let mult_b = mult_node.inputs[1].id;
    let mult_out = mult_node.outputs[0].id;

    let add_node = add_spec.create_node(egui::Pos2::new(400.0, 0.0));
    let add_id = add_node.id;
    let add_a = add_node.inputs[0].id;
    let add_b = add_node.inputs[1].id;
    let add_out = add_node.outputs[0].id;

    let sub_node = subtract_spec.create_node(egui::Pos2::new(600.0, 0.0));
    let sub_id = sub_node.id;
    let sub_a = sub_node.inputs[0].id;
    let sub_b = sub_node.inputs[1].id;

    // Add all nodes
    graph.add_node(const_2);
    graph.add_node(const_mult);
    graph.add_node(const_add);
    graph.add_node(const_sub);
    graph.add_node(mult_node);
    graph.add_node(add_node);
    graph.add_node(sub_node);

    // Connect chain
    graph.add_connection(const_2_id, const_2_out, mult_id, mult_a).unwrap();
    graph.add_connection(const_mult_id, const_mult_out, mult_id, mult_b).unwrap();
    graph.add_connection(mult_id, mult_out, add_id, add_a).unwrap();
    graph.add_connection(const_add_id, const_add_out, add_id, add_b).unwrap();
    graph.add_connection(add_id, add_out, sub_id, sub_a).unwrap();
    graph.add_connection(const_sub_id, const_sub_out, sub_id, sub_b).unwrap();

    // Execute
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Chain execution should succeed");

    // Verify final output
    let sub_node = graph.nodes.get(&sub_id).unwrap();
    let output = sub_node.outputs[0].current_value.as_ref().unwrap();

    match output {
        NodeValue::F32(val) => {
            assert_eq!(*val, 6.0, "((2 * 2) + 3) - 1 should equal 6");
        }
        _ => panic!("Output should be F32"),
    }

    println!("✓ Chain execution: ((2 * 2) + 3) - 1 = 6");
}

#[test]
fn test_diamond_graph_execution() {
    // Create diamond pattern:
    //        Constant(10)
    //        /          \
    //    Add(+5)      Multiply(x2)
    //        \          /
    //         Subtract
    // Expected: (10 + 5) - (10 * 2) = 15 - 20 = -5

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Test Diamond".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();
    let multiply_spec = registry.get_by_id("builtin:math:multiply").unwrap();
    let subtract_spec = registry.get_by_id("builtin:math:subtract").unwrap();

    // Create center constant
    let mut const_10 = const_spec.create_node(egui::Pos2::new(200.0, 0.0));
    const_10.outputs[0].current_value = Some(NodeValue::F32(10.0));
    let const_10_id = const_10.id;
    let const_10_out = const_10.outputs[0].id;

    // Create constants for operations
    let mut const_5 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    const_5.outputs[0].current_value = Some(NodeValue::F32(5.0));
    let const_5_id = const_5.id;
    let const_5_out = const_5.outputs[0].id;

    let mut const_2 = const_spec.create_node(egui::Pos2::new(400.0, 100.0));
    const_2.outputs[0].current_value = Some(NodeValue::F32(2.0));
    let const_2_id = const_2.id;
    let const_2_out = const_2.outputs[0].id;

    // Create operation nodes
    let add_node = add_spec.create_node(egui::Pos2::new(100.0, 200.0));
    let add_id = add_node.id;
    let add_a = add_node.inputs[0].id;
    let add_b = add_node.inputs[1].id;
    let add_out = add_node.outputs[0].id;

    let mult_node = multiply_spec.create_node(egui::Pos2::new(300.0, 200.0));
    let mult_id = mult_node.id;
    let mult_a = mult_node.inputs[0].id;
    let mult_b = mult_node.inputs[1].id;
    let mult_out = mult_node.outputs[0].id;

    let sub_node = subtract_spec.create_node(egui::Pos2::new(200.0, 300.0));
    let sub_id = sub_node.id;
    let sub_a = sub_node.inputs[0].id;
    let sub_b = sub_node.inputs[1].id;

    // Add nodes
    graph.add_node(const_10);
    graph.add_node(const_5);
    graph.add_node(const_2);
    graph.add_node(add_node);
    graph.add_node(mult_node);
    graph.add_node(sub_node);

    // Connect diamond
    graph.add_connection(const_10_id, const_10_out, add_id, add_a).unwrap();
    graph.add_connection(const_5_id, const_5_out, add_id, add_b).unwrap();
    graph.add_connection(const_10_id, const_10_out, mult_id, mult_a).unwrap();
    graph.add_connection(const_2_id, const_2_out, mult_id, mult_b).unwrap();
    graph.add_connection(add_id, add_out, sub_id, sub_a).unwrap();
    graph.add_connection(mult_id, mult_out, sub_id, sub_b).unwrap();

    // Execute
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Diamond execution should succeed");

    // Verify
    let sub_node = graph.nodes.get(&sub_id).unwrap();
    let output = sub_node.outputs[0].current_value.as_ref().unwrap();

    match output {
        NodeValue::F32(val) => {
            assert_eq!(*val, -5.0, "(10 + 5) - (10 * 2) should equal -5");
        }
        _ => panic!("Output should be F32"),
    }

    println!("✓ Diamond graph execution: (10 + 5) - (10 * 2) = -5");
}

#[test]
fn test_division_by_zero_error() {
    // Test error handling: Constant(5) / Constant(0) should fail gracefully

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Test Divide by Zero".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let divide_spec = registry.get_by_id("builtin:math:divide").unwrap();

    let mut const_5 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    const_5.outputs[0].current_value = Some(NodeValue::F32(5.0));
    let const_5_id = const_5.id;
    let const_5_out = const_5.outputs[0].id;

    let mut const_0 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    const_0.outputs[0].current_value = Some(NodeValue::F32(0.0));
    let const_0_id = const_0.id;
    let const_0_out = const_0.outputs[0].id;

    let div_node = divide_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let div_id = div_node.id;
    let div_a = div_node.inputs[0].id;
    let div_b = div_node.inputs[1].id;

    graph.add_node(const_5);
    graph.add_node(const_0);
    graph.add_node(div_node);

    graph.add_connection(const_5_id, const_5_out, div_id, div_a).unwrap();
    graph.add_connection(const_0_id, const_0_out, div_id, div_b).unwrap();

    // Execute
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);

    // Current implementation: division by zero returns an error
    // Future improvement: could track as failed node and continue execution
    match result {
        Err(_) => {
            println!("✓ Division by zero detected and reported as error");
        }
        Ok(report) => {
            assert!(!report.success(), "Execution should report failure");
            assert!(!report.failed_nodes.is_empty(), "Should have failed nodes");
            println!("✓ Division by zero handled gracefully with failed node tracking");
        }
    }
}

#[test]
fn test_multiple_outputs() {
    // Test that a node's output can feed multiple downstream nodes
    //     Constant(10)
    //      /    |    \
    //   Add   Mult  Subtract
    // All three operations should receive the same value

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Test Multiple Outputs".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();
    let mult_spec = registry.get_by_id("builtin:math:multiply").unwrap();
    let sub_spec = registry.get_by_id("builtin:math:subtract").unwrap();

    // Source constant
    let mut const_10 = const_spec.create_node(egui::Pos2::new(200.0, 0.0));
    const_10.outputs[0].current_value = Some(NodeValue::F32(10.0));
    let const_10_id = const_10.id;
    let const_10_out = const_10.outputs[0].id;

    // Other constants
    let mut const_5 = const_spec.create_node(egui::Pos2::new(0.0, 200.0));
    const_5.outputs[0].current_value = Some(NodeValue::F32(5.0));
    let const_5_id = const_5.id;
    let const_5_out = const_5.outputs[0].id;

    // Create operation nodes
    let add_node = add_spec.create_node(egui::Pos2::new(100.0, 150.0));
    let add_id = add_node.id;
    let add_a = add_node.inputs[0].id;
    let add_b = add_node.inputs[1].id;

    let mult_node = mult_spec.create_node(egui::Pos2::new(200.0, 150.0));
    let mult_id = mult_node.id;
    let mult_a = mult_node.inputs[0].id;
    let mult_b = mult_node.inputs[1].id;

    let sub_node = sub_spec.create_node(egui::Pos2::new(300.0, 150.0));
    let sub_id = sub_node.id;
    let sub_a = sub_node.inputs[0].id;
    let sub_b = sub_node.inputs[1].id;

    graph.add_node(const_10);
    graph.add_node(const_5);
    graph.add_node(add_node);
    graph.add_node(mult_node);
    graph.add_node(sub_node);

    // Connect const_10 to all three operations
    graph.add_connection(const_10_id, const_10_out, add_id, add_a).unwrap();
    graph.add_connection(const_5_id, const_5_out, add_id, add_b).unwrap();

    graph.add_connection(const_10_id, const_10_out, mult_id, mult_a).unwrap();
    graph.add_connection(const_5_id, const_5_out, mult_id, mult_b).unwrap();

    graph.add_connection(const_10_id, const_10_out, sub_id, sub_a).unwrap();
    graph.add_connection(const_5_id, const_5_out, sub_id, sub_b).unwrap();

    // Execute
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Execution should succeed");

    // Verify all got correct values
    let add_node = graph.nodes.get(&add_id).unwrap();
    let mult_node = graph.nodes.get(&mult_id).unwrap();
    let sub_node = graph.nodes.get(&sub_id).unwrap();

    if let Some(NodeValue::F32(val)) = add_node.outputs[0].current_value.as_ref() {
        assert_eq!(*val, 15.0, "10 + 5 = 15");
    }

    if let Some(NodeValue::F32(val)) = mult_node.outputs[0].current_value.as_ref() {
        assert_eq!(*val, 50.0, "10 * 5 = 50");
    }

    if let Some(NodeValue::F32(val)) = sub_node.outputs[0].current_value.as_ref() {
        assert_eq!(*val, 5.0, "10 - 5 = 5");
    }

    println!("✓ Multiple outputs: single source feeds multiple consumers");
}
