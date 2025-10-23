//! T098: Type checking tests
//!
//! Tests SC-005: 100% type error prevention
//! Verifies type compatibility validation works correctly

use wasmflow::builtin::{register_constant_nodes, register_math_nodes};
use wasmflow::graph::graph::NodeGraph;
use wasmflow::graph::node::{ComponentRegistry, DataType};

#[test]
fn test_exact_type_match() {
    // Test that identical types are compatible
    assert!(NodeGraph::types_compatible(&DataType::F32, &DataType::F32), "F32 should match F32");
    assert!(NodeGraph::types_compatible(&DataType::I32, &DataType::I32), "I32 should match I32");
    assert!(NodeGraph::types_compatible(&DataType::U32, &DataType::U32), "U32 should match U32");
    assert!(NodeGraph::types_compatible(&DataType::String, &DataType::String), "String should match String");
    assert!(NodeGraph::types_compatible(&DataType::Binary, &DataType::Binary), "Binary should match Binary");

    println!("✓ Exact type matching works correctly");
}

#[test]
fn test_type_mismatch() {
    // Test that different types are not compatible
    assert!(!NodeGraph::types_compatible(&DataType::F32, &DataType::I32), "F32 should not match I32");
    assert!(!NodeGraph::types_compatible(&DataType::I32, &DataType::U32), "I32 should not match U32");
    assert!(!NodeGraph::types_compatible(&DataType::U32, &DataType::F32), "U32 should not match F32");
    assert!(!NodeGraph::types_compatible(&DataType::String, &DataType::F32), "String should not match F32");
    assert!(!NodeGraph::types_compatible(&DataType::F32, &DataType::Binary), "F32 should not match Binary");

    println!("✓ Type mismatch detection works correctly");
}

#[test]
fn test_any_type_compatibility() {
    // Test that Any type accepts all other types
    assert!(NodeGraph::types_compatible(&DataType::F32, &DataType::Any), "F32 should connect to Any");
    assert!(NodeGraph::types_compatible(&DataType::I32, &DataType::Any), "I32 should connect to Any");
    assert!(NodeGraph::types_compatible(&DataType::U32, &DataType::Any), "U32 should connect to Any");
    assert!(NodeGraph::types_compatible(&DataType::String, &DataType::Any), "String should connect to Any");
    assert!(NodeGraph::types_compatible(&DataType::Binary, &DataType::Any), "Binary should connect to Any");

    // Test that Any can output to specific types
    assert!(NodeGraph::types_compatible(&DataType::Any, &DataType::F32), "Any should output to F32");
    assert!(NodeGraph::types_compatible(&DataType::Any, &DataType::I32), "Any should output to I32");
    assert!(NodeGraph::types_compatible(&DataType::Any, &DataType::U32), "Any should output to U32");
    assert!(NodeGraph::types_compatible(&DataType::Any, &DataType::String), "Any should output to String");

    println!("✓ Any type compatibility works correctly");
}

#[test]
fn test_list_type_compatibility() {
    // Test list type matching
    let list_f32 = DataType::List(Box::new(DataType::F32));
    let list_i32 = DataType::List(Box::new(DataType::I32));
    let list_f32_2 = DataType::List(Box::new(DataType::F32));

    assert!(
        NodeGraph::types_compatible(&list_f32, &list_f32_2),
        "List<F32> should match List<F32>"
    );
    assert!(
        !NodeGraph::types_compatible(&list_f32, &list_i32),
        "List<F32> should not match List<I32>"
    );
    assert!(
        !NodeGraph::types_compatible(&list_f32, &DataType::F32),
        "List<F32> should not match F32"
    );

    println!("✓ List type compatibility works correctly");
}

#[test]
fn test_record_type_compatibility() {
    // Test record type matching
    let fields1 = vec![
        ("x".to_string(), DataType::F32),
        ("y".to_string(), DataType::F32),
    ];

    let fields2 = vec![
        ("x".to_string(), DataType::F32),
        ("y".to_string(), DataType::F32),
    ];

    let fields3 = vec![
        ("x".to_string(), DataType::I32),
        ("y".to_string(), DataType::I32),
    ];

    let record1 = DataType::Record(fields1);
    let record2 = DataType::Record(fields2);
    let record3 = DataType::Record(fields3);

    assert!(NodeGraph::types_compatible(&record1, &record2), "Same record structure should match");
    assert!(
        !NodeGraph::types_compatible(&record1, &record3),
        "Different field types should not match"
    );

    println!("✓ Record type compatibility works correctly");
}

#[test]
fn test_connection_type_validation() {
    // Test that graph rejects invalid type connections
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Type Test".to_string(), "Test".to_string());

    // Get F32 constant and I32 constant specs
    let f32_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let i32_spec = registry.get_by_id("builtin:constant:i32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    // Create F32 constant
    let mut f32_const = f32_spec.create_node(egui::Pos2::new(0.0, 0.0));
    f32_const.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(5.0));
    let f32_id = f32_const.id;
    let f32_out = f32_const.outputs[0].id;

    // Create I32 constant
    let mut i32_const = i32_spec.create_node(egui::Pos2::new(0.0, 100.0));
    i32_const.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::I32(3));
    let i32_id = i32_const.id;
    let i32_out = i32_const.outputs[0].id;

    // Create Add node (expects F32 inputs)
    let add_node = add_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let add_id = add_node.id;
    let add_input_a = add_node.inputs[0].id;
    let add_input_b = add_node.inputs[1].id;

    graph.add_node(f32_const);
    graph.add_node(i32_const);
    graph.add_node(add_node);

    // Valid connection: F32 → F32
    let result = graph.add_connection(f32_id, f32_out, add_id, add_input_a);
    assert!(result.is_ok(), "F32→F32 connection should succeed");

    // Invalid connection: I32 → F32
    let result = graph.add_connection(i32_id, i32_out, add_id, add_input_b);
    assert!(result.is_err(), "I32→F32 connection should fail");

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("type") || error_msg.contains("Type") || error_msg.contains("compatible"),
        "Error should mention type incompatibility: {}",
        error_msg
    );

    println!("✓ Graph connection validation prevents type mismatches");
}

#[test]
fn test_multiple_type_validation() {
    // Test validation across multiple connections
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Multi-Type Test".to_string(), "Test".to_string());

    let f32_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let u32_spec = registry.get_by_id("builtin:constant:u32").unwrap();
    let string_spec = registry.get_by_id("builtin:constant:string").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    // Create various typed constants
    let mut f32_1 = f32_spec.create_node(egui::Pos2::new(0.0, 0.0));
    f32_1.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(1.0));
    let f32_1_id = f32_1.id;
    let f32_1_out = f32_1.outputs[0].id;

    let mut f32_2 = f32_spec.create_node(egui::Pos2::new(0.0, 50.0));
    f32_2.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(2.0));
    let f32_2_id = f32_2.id;
    let f32_2_out = f32_2.outputs[0].id;

    let mut u32_const = u32_spec.create_node(egui::Pos2::new(0.0, 100.0));
    u32_const.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::U32(3));
    let u32_id = u32_const.id;
    let u32_out = u32_const.outputs[0].id;

    let mut string_const = string_spec.create_node(egui::Pos2::new(0.0, 150.0));
    string_const.outputs[0].current_value =
        Some(wasmflow::graph::node::NodeValue::String("test".to_string()));
    let string_id = string_const.id;
    let string_out = string_const.outputs[0].id;

    let add_node = add_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let add_id = add_node.id;
    let add_a = add_node.inputs[0].id;
    let add_b = add_node.inputs[1].id;

    graph.add_node(f32_1);
    graph.add_node(f32_2);
    graph.add_node(u32_const);
    graph.add_node(string_const);
    graph.add_node(add_node);

    // Valid: F32 → F32
    assert!(graph.add_connection(f32_1_id, f32_1_out, add_id, add_a).is_ok());
    assert!(graph.add_connection(f32_2_id, f32_2_out, add_id, add_b).is_ok());

    // After valid connections, clear and try invalid ones
    graph.connections.clear();

    // Invalid: U32 → F32
    assert!(graph.add_connection(u32_id, u32_out, add_id, add_a).is_err());

    // Invalid: String → F32
    assert!(graph.add_connection(string_id, string_out, add_id, add_a).is_err());

    println!("✓ Multiple type validation works correctly");
}

#[test]
fn test_port_direction_validation() {
    // Test that connections respect port directions (output→input)
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);

    let mut graph = NodeGraph::new("Direction Test".to_string(), "Test".to_string());

    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let node1 = add_spec.create_node(egui::Pos2::new(0.0, 0.0));
    let node1_id = node1.id;
    let node1_input = node1.inputs[0].id;
    let node1_output = node1.outputs[0].id;

    let node2 = add_spec.create_node(egui::Pos2::new(200.0, 0.0));
    let node2_id = node2.id;
    let node2_input = node2.inputs[0].id;
    let node2_output = node2.outputs[0].id;

    graph.add_node(node1);
    graph.add_node(node2);

    // Valid: output → input
    let result = graph.add_connection(node1_id, node1_output, node2_id, node2_input);
    assert!(result.is_ok(), "Output→Input should be valid");

    // Clear connections for next test
    graph.connections.clear();

    // Invalid: input → output (backwards)
    let result = graph.add_connection(node1_id, node1_input, node2_id, node2_output);
    assert!(result.is_err(), "Input→Output should be invalid");

    // Invalid: output → output
    let result = graph.add_connection(node1_id, node1_output, node2_id, node2_output);
    assert!(result.is_err(), "Output→Output should be invalid");

    // Invalid: input → input
    let result = graph.add_connection(node1_id, node1_input, node2_id, node2_input);
    assert!(result.is_err(), "Input→Input should be invalid");

    println!("✓ Port direction validation works correctly");
}

#[test]
fn test_duplicate_input_prevention() {
    // Test that an input port can only have one connection
    let mut registry = ComponentRegistry::new();
    register_constant_nodes(&mut registry);
    register_math_nodes(&mut registry);

    let mut graph = NodeGraph::new("Duplicate Input Test".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let mut const1 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    const1.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(1.0));
    let const1_id = const1.id;
    let const1_out = const1.outputs[0].id;

    let mut const2 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    const2.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(2.0));
    let const2_id = const2.id;
    let const2_out = const2.outputs[0].id;

    let add_node = add_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let add_id = add_node.id;
    let add_input_a = add_node.inputs[0].id;

    graph.add_node(const1);
    graph.add_node(const2);
    graph.add_node(add_node);

    // First connection to input A
    let result1 = graph.add_connection(const1_id, const1_out, add_id, add_input_a);
    assert!(result1.is_ok(), "First connection to input should succeed");

    // Second connection to same input A should fail
    let result2 = graph.add_connection(const2_id, const2_out, add_id, add_input_a);
    assert!(result2.is_err(), "Duplicate connection to same input should fail");

    let error = result2.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("already") || error_msg.contains("duplicate") || error_msg.contains("connected"),
        "Error should mention duplicate/existing connection: {}",
        error_msg
    );

    println!("✓ Duplicate input prevention works correctly");
}

#[test]
fn test_multiple_outputs_allowed() {
    // Test that an output port CAN connect to multiple inputs
    let mut registry = ComponentRegistry::new();
    register_constant_nodes(&mut registry);
    register_math_nodes(&mut registry);

    let mut graph = NodeGraph::new("Multiple Outputs Test".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let mut const_node = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    const_node.outputs[0].current_value = Some(wasmflow::graph::node::NodeValue::F32(5.0));
    let const_id = const_node.id;
    let const_out = const_node.outputs[0].id;

    let add1 = add_spec.create_node(egui::Pos2::new(200.0, 0.0));
    let add1_id = add1.id;
    let add1_input_a = add1.inputs[0].id;

    let add2 = add_spec.create_node(egui::Pos2::new(200.0, 100.0));
    let add2_id = add2.id;
    let add2_input_a = add2.inputs[0].id;

    let add3 = add_spec.create_node(egui::Pos2::new(200.0, 200.0));
    let add3_id = add3.id;
    let add3_input_a = add3.inputs[0].id;

    graph.add_node(const_node);
    graph.add_node(add1);
    graph.add_node(add2);
    graph.add_node(add3);

    // Connect same output to multiple inputs (should all succeed)
    assert!(graph.add_connection(const_id, const_out, add1_id, add1_input_a).is_ok());
    assert!(graph.add_connection(const_id, const_out, add2_id, add2_input_a).is_ok());
    assert!(graph.add_connection(const_id, const_out, add3_id, add3_input_a).is_ok());

    assert_eq!(graph.connections.len(), 3, "All three connections should exist");

    println!("✓ Multiple outputs from single port allowed correctly");
}

#[test]
fn test_type_name_formatting() {
    // Test that type names are human-readable
    assert_eq!(DataType::F32.name(), "f32");
    assert_eq!(DataType::I32.name(), "i32");
    assert_eq!(DataType::U32.name(), "u32");
    assert_eq!(DataType::String.name(), "string");
    assert_eq!(DataType::Binary.name(), "binary");
    assert_eq!(DataType::Any.name(), "any");

    let list_type = DataType::List(Box::new(DataType::F32));
    assert!(list_type.name().contains("list") || list_type.name().contains("f32"));

    println!("✓ Type name formatting is human-readable");
}
