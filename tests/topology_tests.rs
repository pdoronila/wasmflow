//! T097: Cycle detection tests
//!
//! Tests that graph topology validation correctly detects and rejects cycles

use wasmflow::builtin::{register_constant_nodes, register_math_nodes};
use wasmflow::graph::graph::NodeGraph;
use wasmflow::graph::node::ComponentRegistry;
use wasmflow::runtime::engine::{register_builtin_executors, ExecutionEngine};

#[test]
fn test_simple_cycle_detection() {
    // Create a simple 2-node cycle: A → B → A
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);

    let mut graph = NodeGraph::new("Cycle Test".to_string(), "Test".to_string());

    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let node_a = add_spec.create_node(egui::Pos2::new(0.0, 0.0));
    let a_id = node_a.id;
    let a_out = node_a.outputs[0].id;
    let a_in_a = node_a.inputs[0].id;

    let node_b = add_spec.create_node(egui::Pos2::new(200.0, 0.0));
    let b_id = node_b.id;
    let b_out = node_b.outputs[0].id;
    let b_in_a = node_b.inputs[0].id;

    graph.add_node(node_a);
    graph.add_node(node_b);

    // Connect A → B
    graph.add_connection(a_id, a_out, b_id, b_in_a).expect("First connection should succeed");

    // Try to connect B → A (would create cycle - should be prevented)
    let result = graph.add_connection(b_id, b_out, a_id, a_in_a);

    // Should fail at connection time (cycle prevention)
    assert!(result.is_err(), "Connection creating cycle should be rejected");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("cycle") || error_msg.contains("Cycle") || error_msg.contains("circular"),
        "Error should mention cycle: {}",
        error_msg
    );

    println!("✓ Simple 2-node cycle prevented at connection time");
}

#[test]
fn test_three_node_cycle() {
    // Create 3-node cycle: A → B → C → A
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);

    let mut graph = NodeGraph::new("3-Node Cycle".to_string(), "Test".to_string());

    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let node_a = add_spec.create_node(egui::Pos2::new(0.0, 0.0));
    let a_id = node_a.id;
    let a_out = node_a.outputs[0].id;
    let a_in = node_a.inputs[0].id;

    let node_b = add_spec.create_node(egui::Pos2::new(200.0, 0.0));
    let b_id = node_b.id;
    let b_out = node_b.outputs[0].id;
    let b_in = node_b.inputs[0].id;

    let node_c = add_spec.create_node(egui::Pos2::new(100.0, 200.0));
    let c_id = node_c.id;
    let c_out = node_c.outputs[0].id;
    let c_in = node_c.inputs[0].id;

    graph.add_node(node_a);
    graph.add_node(node_b);
    graph.add_node(node_c);

    // Create partial cycle: A → B → C
    graph.add_connection(a_id, a_out, b_id, b_in).expect("A→B");
    graph.add_connection(b_id, b_out, c_id, c_in).expect("B→C");

    // Try to complete cycle: C → A (should be prevented)
    let result = graph.add_connection(c_id, c_out, a_id, a_in);
    assert!(result.is_err(), "Connection creating 3-node cycle should be rejected");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("cycle") || error.to_string().contains("Cycle"));

    println!("✓ 3-node cycle prevented at connection time");
}

#[test]
fn test_self_loop_detection() {
    // Create self-loop: A → A
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);

    let mut graph = NodeGraph::new("Self Loop".to_string(), "Test".to_string());

    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let node_a = add_spec.create_node(egui::Pos2::new(0.0, 0.0));
    let a_id = node_a.id;
    let a_out = node_a.outputs[0].id;
    let a_in = node_a.inputs[0].id;

    graph.add_node(node_a);

    // Try to connect node to itself
    let result = graph.add_connection(a_id, a_out, a_id, a_in);

    // Should fail immediately (self-connection validation)
    assert!(result.is_err(), "Self-loop should be rejected");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("self") || error_msg.contains("Self") || error_msg.contains("same node"),
        "Error should mention self-connection: {}",
        error_msg
    );

    println!("✓ Self-loop prevented by validation");
}

#[test]
fn test_acyclic_graph_execution() {
    // Verify that valid DAGs execute without cycle errors
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Valid DAG".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    // Create linear chain: C1 → Add → C2 (no cycle)
    let mut c1 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    c1.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(5.0));
    let c1_id = c1.id;
    let c1_out = c1.outputs[0].id;

    let mut c2 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    c2.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(3.0));
    let c2_id = c2.id;
    let c2_out = c2.outputs[0].id;

    let add_node = add_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let add_id = add_node.id;
    let add_a = add_node.inputs[0].id;
    let add_b = add_node.inputs[1].id;

    graph.add_node(c1);
    graph.add_node(c2);
    graph.add_node(add_node);

    graph.add_connection(c1_id, c1_out, add_id, add_a).expect("C1→Add");
    graph.add_connection(c2_id, c2_out, add_id, add_b).expect("C2→Add");

    // Should execute successfully (no cycle)
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Valid DAG should execute: {:?}", result);

    println!("✓ Valid DAG executes without cycle error");
}

#[test]
fn test_diamond_dag_no_cycle() {
    // Verify diamond pattern (common ancestor) is not a cycle
    //        A
    //       / \
    //      B   C
    //       \ /
    //        D
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Diamond DAG".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let mut node_a = const_spec.create_node(egui::Pos2::new(100.0, 0.0));
    node_a.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(10.0));
    let a_id = node_a.id;
    let a_out = node_a.outputs[0].id;

    let mut c1 = const_spec.create_node(egui::Pos2::new(0.0, 150.0));
    c1.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(1.0));
    let c1_id = c1.id;
    let c1_out = c1.outputs[0].id;

    let mut c2 = const_spec.create_node(egui::Pos2::new(200.0, 150.0));
    c2.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(2.0));
    let c2_id = c2.id;
    let c2_out = c2.outputs[0].id;

    let node_b = add_spec.create_node(egui::Pos2::new(50.0, 100.0));
    let b_id = node_b.id;
    let b_in_a = node_b.inputs[0].id;
    let b_in_b = node_b.inputs[1].id;
    let b_out = node_b.outputs[0].id;

    let node_c = add_spec.create_node(egui::Pos2::new(150.0, 100.0));
    let c_id = node_c.id;
    let c_in_a = node_c.inputs[0].id;
    let c_in_b = node_c.inputs[1].id;
    let c_out = node_c.outputs[0].id;

    let node_d = add_spec.create_node(egui::Pos2::new(100.0, 200.0));
    let d_id = node_d.id;
    let d_in_a = node_d.inputs[0].id;
    let d_in_b = node_d.inputs[1].id;

    graph.add_node(node_a);
    graph.add_node(c1);
    graph.add_node(c2);
    graph.add_node(node_b);
    graph.add_node(node_c);
    graph.add_node(node_d);

    // Create diamond
    graph.add_connection(a_id, a_out, b_id, b_in_a).expect("A→B");
    graph.add_connection(c1_id, c1_out, b_id, b_in_b).expect("C1→B");

    graph.add_connection(a_id, a_out, c_id, c_in_a).expect("A→C");
    graph.add_connection(c2_id, c2_out, c_id, c_in_b).expect("C2→C");

    graph.add_connection(b_id, b_out, d_id, d_in_a).expect("B→D");
    graph.add_connection(c_id, c_out, d_id, d_in_b).expect("C→D");

    // Should execute successfully (diamond is a DAG, not a cycle)
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Diamond DAG should execute successfully");

    println!("✓ Diamond pattern (DAG) executes correctly - not flagged as cycle");
}

#[test]
fn test_complex_cycle_in_larger_graph() {
    // Create larger graph with cycle buried in the middle
    //  C1 → A → B → D → E
    //            ↑       ↓
    //            +← C ←--+
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Complex Cycle".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let mut c1 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    c1.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(1.0));
    let c1_id = c1.id;
    let c1_out = c1.outputs[0].id;

    let node_a = add_spec.create_node(egui::Pos2::new(100.0, 0.0));
    let a_id = node_a.id;
    let a_in = node_a.inputs[0].id;
    let a_out = node_a.outputs[0].id;

    let node_b = add_spec.create_node(egui::Pos2::new(200.0, 0.0));
    let b_id = node_b.id;
    let b_in = node_b.inputs[0].id;
    let b_out = node_b.outputs[0].id;

    let node_c = add_spec.create_node(egui::Pos2::new(300.0, 100.0));
    let c_id = node_c.id;
    let c_in = node_c.inputs[0].id;
    let c_out = node_c.outputs[0].id;

    let node_d = add_spec.create_node(egui::Pos2::new(300.0, 0.0));
    let d_id = node_d.id;
    let d_in = node_d.inputs[0].id;
    let d_out = node_d.outputs[0].id;

    let node_e = add_spec.create_node(egui::Pos2::new(400.0, 0.0));
    let e_id = node_e.id;
    let e_in = node_e.inputs[0].id;
    let e_out = node_e.outputs[0].id;

    graph.add_node(c1);
    graph.add_node(node_a);
    graph.add_node(node_b);
    graph.add_node(node_c);
    graph.add_node(node_d);
    graph.add_node(node_e);

    // Create chain
    graph.add_connection(c1_id, c1_out, a_id, a_in).expect("C1→A");
    graph.add_connection(a_id, a_out, b_id, b_in).expect("A→B");
    graph.add_connection(b_id, b_out, d_id, d_in).expect("B→D");
    graph.add_connection(d_id, d_out, e_id, e_in).expect("D→E");
    graph.add_connection(e_id, e_out, c_id, c_in).expect("E→C");

    // Try to create cycle: C→B (already have C1→A→B→D→E→C, so C→B would create cycle)
    let result = graph.add_connection(c_id, c_out, b_id, b_in);
    assert!(result.is_err(), "Connection creating complex cycle should be prevented");

    println!("✓ Cycle prevented in larger graph structure");
}

#[test]
fn test_disconnected_components_no_cycle() {
    // Verify disconnected subgraphs don't create false cycle detection
    //  A → B    C → D  (two separate chains)
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Disconnected".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    // First chain
    let mut c1 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    c1.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(1.0));
    let c1_id = c1.id;
    let c1_out = c1.outputs[0].id;

    let node_a = add_spec.create_node(egui::Pos2::new(100.0, 0.0));
    let a_id = node_a.id;
    let a_in_a = node_a.inputs[0].id;
    let a_in_b = node_a.inputs[1].id;

    // Second chain
    let mut c2 = const_spec.create_node(egui::Pos2::new(0.0, 200.0));
    c2.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(2.0));
    let c2_id = c2.id;
    let c2_out = c2.outputs[0].id;

    let mut c3 = const_spec.create_node(egui::Pos2::new(0.0, 300.0));
    c3.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(3.0));
    let c3_id = c3.id;
    let c3_out = c3.outputs[0].id;

    let node_b = add_spec.create_node(egui::Pos2::new(100.0, 250.0));
    let b_id = node_b.id;
    let b_in_a = node_b.inputs[0].id;
    let b_in_b = node_b.inputs[1].id;

    graph.add_node(c1);
    graph.add_node(node_a);
    graph.add_node(c2);
    graph.add_node(c3);
    graph.add_node(node_b);

    // First disconnected component: C1 → Add_A (both inputs from same constant for simplicity)
    graph.add_connection(c1_id, c1_out, a_id, a_in_a).expect("C1→A.a");
    graph.add_connection(c1_id, c1_out, a_id, a_in_b).expect("C1→A.b");

    // Second disconnected component: C2 → Add_B.a, C3 → Add_B.b
    graph.add_connection(c2_id, c2_out, b_id, b_in_a).expect("C2→B.a");
    graph.add_connection(c3_id, c3_out, b_id, b_in_b).expect("C3→B.b");

    // Should execute fine (no cycles, both components have all inputs)
    let mut engine = ExecutionEngine::new();
    register_builtin_executors(&mut engine);

    let result = engine.execute_graph(&mut graph);
    assert!(result.is_ok(), "Disconnected components should execute without errors: {:?}", result);

    println!("✓ Disconnected components execute correctly");
}
