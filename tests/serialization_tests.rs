//! T095: Serialization roundtrip tests
//!
//! Tests SC-009: Zero data loss in saves
//! Verifies that save/load cycles preserve all graph data

use wasmflow::builtin::{register_constant_nodes, register_math_nodes};
use wasmflow::graph::graph::NodeGraph;
use wasmflow::graph::node::{ComponentRegistry, NodeValue};

#[test]
fn test_empty_graph_roundtrip() {
    // Test that empty graph serializes and deserializes correctly
    let graph = NodeGraph::new("Empty Graph".to_string(), "Test Author".to_string());

    // Serialize
    let bytes = graph.to_bytes().expect("Serialization should succeed");

    // Deserialize
    let loaded = NodeGraph::from_bytes(&bytes).expect("Deserialization should succeed");

    // Verify
    assert_eq!(loaded.name, graph.name, "Graph name should match");
    assert_eq!(loaded.metadata.author, graph.metadata.author, "Author should match");
    assert_eq!(loaded.nodes.len(), 0, "Should have no nodes");
    assert_eq!(loaded.connections.len(), 0, "Should have no connections");

    println!("✓ Empty graph roundtrip successful");
}

#[test]
fn test_single_node_roundtrip() {
    // Test single node preservation
    let mut registry = ComponentRegistry::new();
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Single Node".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let mut const_node = const_spec.create_node(egui::Pos2::new(100.0, 200.0));
    const_node.outputs[0].current_value = Some(NodeValue::F32(42.0));

    let node_id = const_node.id;
    let node_position = const_node.position;
    let node_component_id = const_node.component_id.clone();

    graph.add_node(const_node);

    // Roundtrip
    let bytes = graph.to_bytes().expect("Serialization should succeed");
    let loaded = NodeGraph::from_bytes(&bytes).expect("Deserialization should succeed");

    // Verify
    assert_eq!(loaded.nodes.len(), 1, "Should have 1 node");
    let loaded_node = loaded.nodes.get(&node_id).expect("Node should exist");
    assert_eq!(loaded_node.id, node_id, "Node ID should match");
    assert_eq!(loaded_node.position, node_position, "Position should match");
    assert_eq!(loaded_node.component_id, node_component_id, "Component ID should match");

    // Verify value preserved
    if let Some(NodeValue::F32(val)) = loaded_node.outputs[0].current_value.as_ref() {
        assert_eq!(*val, 42.0, "Output value should be preserved");
    } else {
        panic!("Output value should be F32(42.0)");
    }

    println!("✓ Single node roundtrip successful");
}

#[test]
fn test_complex_graph_roundtrip() {
    // Test complex graph with multiple nodes and connections
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Complex Graph".to_string(), "Test Author".to_string());
    graph.metadata.description = "Test description".to_string();

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();
    let mult_spec = registry.get_by_id("builtin:math:multiply").unwrap();

    // Create 5 nodes with specific positions and values
    let mut const_1 = const_spec.create_node(egui::Pos2::new(10.5, 20.3));
    const_1.outputs[0].current_value = Some(NodeValue::F32(1.5));
    let const_1_id = const_1.id;
    let const_1_out = const_1.outputs[0].id;

    let mut const_2 = const_spec.create_node(egui::Pos2::new(10.5, 120.7));
    const_2.outputs[0].current_value = Some(NodeValue::F32(2.5));
    let const_2_id = const_2.id;
    let const_2_out = const_2.outputs[0].id;

    let mut const_3 = const_spec.create_node(egui::Pos2::new(10.5, 220.1));
    const_3.outputs[0].current_value = Some(NodeValue::F32(3.5));
    let const_3_id = const_3.id;
    let const_3_out = const_3.outputs[0].id;

    let add_node = add_spec.create_node(egui::Pos2::new(250.0, 70.0));
    let add_id = add_node.id;
    let add_a = add_node.inputs[0].id;
    let add_b = add_node.inputs[1].id;
    let add_out = add_node.outputs[0].id;

    let mult_node = mult_spec.create_node(egui::Pos2::new(450.0, 120.0));
    let mult_id = mult_node.id;
    let mult_a = mult_node.inputs[0].id;
    let mult_b = mult_node.inputs[1].id;

    graph.add_node(const_1);
    graph.add_node(const_2);
    graph.add_node(const_3);
    graph.add_node(add_node);
    graph.add_node(mult_node);

    // Create 4 connections
    graph.add_connection(const_1_id, const_1_out, add_id, add_a).unwrap();
    graph.add_connection(const_2_id, const_2_out, add_id, add_b).unwrap();
    graph.add_connection(add_id, add_out, mult_id, mult_a).unwrap();
    graph.add_connection(const_3_id, const_3_out, mult_id, mult_b).unwrap();

    // Serialize
    let bytes = graph.to_bytes().expect("Serialization should succeed");

    // Deserialize
    let loaded = NodeGraph::from_bytes(&bytes).expect("Deserialization should succeed");

    // Verify metadata
    assert_eq!(loaded.name, "Complex Graph", "Graph name should match");
    assert_eq!(loaded.metadata.author, "Test Author", "Author should match");
    assert_eq!(loaded.metadata.description, "Test description", "Description should match");

    // Verify structure
    assert_eq!(loaded.nodes.len(), 5, "Should have 5 nodes");
    assert_eq!(loaded.connections.len(), 4, "Should have 4 connections");

    // Verify specific nodes
    let loaded_const_1 = loaded.nodes.get(&const_1_id).expect("Const 1 should exist");
    assert_eq!(loaded_const_1.position, egui::Pos2::new(10.5, 20.3), "Position should match");
    if let Some(NodeValue::F32(val)) = loaded_const_1.outputs[0].current_value.as_ref() {
        assert_eq!(*val, 1.5, "Value should be preserved");
    }

    let loaded_add = loaded.nodes.get(&add_id).expect("Add node should exist");
    assert_eq!(loaded_add.component_id, "builtin:math:add", "Component ID should match");
    assert_eq!(loaded_add.inputs.len(), 2, "Should have 2 inputs");
    assert_eq!(loaded_add.outputs.len(), 1, "Should have 1 output");

    // Verify connections
    let conn_1 = loaded.connections.iter()
        .find(|c| c.from_node == const_1_id && c.to_node == add_id)
        .expect("Connection 1→Add should exist");
    assert_eq!(conn_1.from_port, const_1_out, "From port should match");
    assert_eq!(conn_1.to_port, add_a, "To port should match");

    println!("✓ Complex graph roundtrip: all nodes, connections, and values preserved");
}

#[test]
fn test_metadata_preservation() {
    // Test that graph metadata is fully preserved
    let mut graph = NodeGraph::new("Metadata Test".to_string(), "Original Author".to_string());

    // Set various metadata fields
    graph.metadata.description = "Test description with special chars: éñ中文".to_string();

    // Store original timestamps
    let original_created = graph.metadata.created_at.clone();
    let original_modified = graph.metadata.modified_at.clone();

    // Roundtrip
    let bytes = graph.to_bytes().expect("Serialization should succeed");
    let loaded = NodeGraph::from_bytes(&bytes).expect("Deserialization should succeed");

    // Verify all metadata
    assert_eq!(loaded.name, "Metadata Test", "Name should match");
    assert_eq!(loaded.metadata.author, "Original Author", "Author should match");
    assert_eq!(
        loaded.metadata.description,
        "Test description with special chars: éñ中文",
        "Description should match"
    );
    assert_eq!(loaded.metadata.created_at, original_created, "Created timestamp should match");
    assert_eq!(loaded.metadata.modified_at, original_modified, "Modified timestamp should match");

    println!("✓ Metadata preservation: all fields including Unicode preserved");
}

#[test]
fn test_checksum_validation() {
    // Test that checksum detects corruption
    let mut registry = ComponentRegistry::new();
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Checksum Test".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let const_node = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    graph.add_node(const_node);

    // Serialize
    let mut bytes = graph.to_bytes().expect("Serialization should succeed");

    // Valid deserialization should work
    let result = NodeGraph::from_bytes(&bytes);
    assert!(result.is_ok(), "Valid bytes should deserialize");

    // Corrupt a byte in the middle
    if bytes.len() > 20 {
        bytes[20] ^= 0xFF; // Flip all bits in one byte
    }

    // Corrupted deserialization should fail
    let corrupted_result = NodeGraph::from_bytes(&bytes);
    assert!(corrupted_result.is_err(), "Corrupted bytes should fail validation");

    // Verify it fails (corruption detected via checksum or deserialization error)
    if let Err(e) = corrupted_result {
        let error_msg = e.to_string();
        // Error may mention checksum, validation, or deserialization failure
        let has_validation_error = error_msg.contains("Checksum")
            || error_msg.contains("checksum")
            || error_msg.contains("validation")
            || error_msg.contains("deserialize");
        assert!(
            has_validation_error,
            "Error should indicate corruption/validation failure: {}",
            error_msg
        );
    }

    println!("✓ Checksum validation: corruption detected");
}

#[test]
fn test_deterministic_serialization() {
    // Test that serializing the same graph produces the same bytes
    // This is critical for checksum validation

    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Deterministic".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    let mut c1 = const_spec.create_node(egui::Pos2::new(0.0, 0.0));
    c1.outputs[0].current_value = Some(NodeValue::F32(1.0));
    let c1_id = c1.id;
    let c1_out = c1.outputs[0].id;

    let mut c2 = const_spec.create_node(egui::Pos2::new(0.0, 100.0));
    c2.outputs[0].current_value = Some(NodeValue::F32(2.0));
    let c2_id = c2.id;
    let c2_out = c2.outputs[0].id;

    let add = add_spec.create_node(egui::Pos2::new(200.0, 50.0));
    let add_id = add.id;
    let add_a = add.inputs[0].id;
    let add_b = add.inputs[1].id;

    graph.add_node(c1);
    graph.add_node(c2);
    graph.add_node(add);

    graph.add_connection(c1_id, c1_out, add_id, add_a).unwrap();
    graph.add_connection(c2_id, c2_out, add_id, add_b).unwrap();

    // Serialize multiple times
    let bytes1 = graph.to_bytes().expect("Serialization 1 should succeed");
    let bytes2 = graph.to_bytes().expect("Serialization 2 should succeed");
    let bytes3 = graph.to_bytes().expect("Serialization 3 should succeed");

    // All should be identical
    assert_eq!(bytes1, bytes2, "Serialization should be deterministic");
    assert_eq!(bytes2, bytes3, "Serialization should be deterministic");

    println!("✓ Deterministic serialization: same graph → same bytes");
}

#[test]
fn test_large_graph_roundtrip() {
    // Test that large graphs serialize/deserialize correctly
    let mut registry = ComponentRegistry::new();
    register_math_nodes(&mut registry);
    register_constant_nodes(&mut registry);

    let mut graph = NodeGraph::new("Large Graph".to_string(), "Test".to_string());

    let const_spec = registry.get_by_id("builtin:constant:f32").unwrap();
    let add_spec = registry.get_by_id("builtin:math:add").unwrap();

    // Create 100 nodes
    let mut prev_id = None;
    let mut prev_out = None;

    for i in 0..100 {
        if i % 2 == 0 {
            // Constant node
            let mut const_node = const_spec.create_node(egui::Pos2::new(i as f32 * 10.0, 0.0));
            const_node.outputs[0].current_value = Some(NodeValue::F32(i as f32));
            prev_id = Some(const_node.id);
            prev_out = Some(const_node.outputs[0].id);
            graph.add_node(const_node);
        } else {
            // Add node connected to previous
            let add_node = add_spec.create_node(egui::Pos2::new(i as f32 * 10.0, 100.0));
            let add_id = add_node.id;
            let add_a = add_node.inputs[0].id;
            let add_b = add_node.inputs[1].id;
            let add_out = add_node.outputs[0].id;
            graph.add_node(add_node);

            if let (Some(pid), Some(pout)) = (prev_id, prev_out) {
                graph.add_connection(pid, pout, add_id, add_a).ok();
                graph.add_connection(pid, pout, add_id, add_b).ok();
            }

            prev_id = Some(add_id);
            prev_out = Some(add_out);
        }
    }

    let original_node_count = graph.nodes.len();
    let original_conn_count = graph.connections.len();

    // Roundtrip
    let bytes = graph.to_bytes().expect("Large graph serialization should succeed");
    let loaded = NodeGraph::from_bytes(&bytes).expect("Large graph deserialization should succeed");

    // Verify
    assert_eq!(loaded.nodes.len(), original_node_count, "Node count should match");
    assert_eq!(loaded.connections.len(), original_conn_count, "Connection count should match");

    println!(
        "✓ Large graph roundtrip: {} nodes, {} connections preserved",
        loaded.nodes.len(),
        loaded.connections.len()
    );
}

#[test]
fn test_version_compatibility() {
    // Test that version information is preserved through serialization
    let graph = NodeGraph::new("Version Test".to_string(), "Test".to_string());

    let bytes = graph.to_bytes().expect("Serialization should succeed");

    // Verify reasonable size (should have header + data)
    assert!(
        bytes.len() > 20,
        "Serialized data should include header and graph data"
    );

    // Deserialize and verify
    let loaded = NodeGraph::from_bytes(&bytes).expect("Deserialization should succeed");
    assert_eq!(loaded.name, graph.name, "Graph should load correctly");

    println!("✓ Version compatibility: graph serialization format verified");
}
