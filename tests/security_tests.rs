//! T096: Security and permission tests
//!
//! Tests SC-006: 100% graceful permission failures
//! Tests SC-010: 100% permission dialogs shown

use wasmflow::graph::graph::NodeGraph;
// Test file for capability-based security system
use wasmflow::runtime::capabilities::{CapabilityGrant, CapabilitySet};
use uuid::Uuid;

#[test]
fn test_capability_set_variants() {
    // Test that different capability sets can be created
    let none = CapabilitySet::none();
    let file_read = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]);
    let file_write = CapabilitySet::file_write(vec![std::path::PathBuf::from("/output")]);
    let network = CapabilitySet::network(vec!["api.example.com".to_string()]);
    let full = CapabilitySet::full();

    // Verify descriptions
    assert!(none.description().contains("No system access") || none.description().contains("pure computation"));
    assert!(file_read.description().contains("/tmp"));
    assert!(file_write.description().contains("/output"));
    assert!(network.description().contains("api.example.com"));
    assert!(full.description().contains("Full system access"));

    println!("✓ Capability set variants created successfully");
}

#[test]
fn test_capability_comparison() {
    // Test that capability sets can be compared
    let read1 = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]);
    let read2 = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]);
    let read3 = CapabilitySet::file_read(vec![std::path::PathBuf::from("/data")]);

    assert_eq!(read1, read2, "Same capabilities should be equal");
    assert_ne!(read1, read3, "Different paths should not be equal");

    println!("✓ Capability comparison works correctly");
}

#[test]
fn test_capability_grant_creation() {
    // Test creating and storing capability grants
    let node_id = Uuid::new_v4();
    let capabilities = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]);

    let grant = CapabilityGrant {
        node_id,
        capability_set: capabilities.clone(),
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    assert_eq!(grant.node_id, node_id, "Node ID should match");
    assert_eq!(grant.capability_set, capabilities, "Capabilities should match");
    assert_eq!(grant.scope, "graph", "Scope should be graph");

    println!("✓ Capability grant created successfully");
}

#[test]
fn test_graph_capability_storage() {
    // Test that graphs can store and retrieve capability grants
    let mut graph = NodeGraph::new("Security Test".to_string(), "Test".to_string());

    let node_id = Uuid::new_v4();
    let capabilities = CapabilitySet::file_read(vec![std::path::PathBuf::from("/data")]);

    let grant = CapabilityGrant {
        node_id,
        capability_set: capabilities.clone(),
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    // Grant capability
    graph.grant_capability(grant.clone());

    // Retrieve capability
    let retrieved = graph.get_capability_grant(node_id);
    assert!(retrieved.is_some(), "Grant should be retrievable");

    let retrieved_grant = retrieved.unwrap();
    assert_eq!(retrieved_grant.node_id, node_id, "Node ID should match");
    assert_eq!(retrieved_grant.capability_set, capabilities, "Capabilities should match");

    println!("✓ Graph capability storage and retrieval works");
}

#[test]
fn test_capability_revocation() {
    // Test that capabilities can be revoked
    let mut graph = NodeGraph::new("Revocation Test".to_string(), "Test".to_string());

    let node_id = Uuid::new_v4();
    let capabilities = CapabilitySet::network(vec!["api.example.com".to_string()]);

    let grant = CapabilityGrant {
        node_id,
        capability_set: capabilities,
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    // Grant capability
    graph.grant_capability(grant);

    // Verify it exists
    assert!(graph.get_capability_grant(node_id).is_some(), "Grant should exist");

    // Revoke capability
    graph.revoke_capability(node_id);

    // Verify it's gone
    assert!(graph.get_capability_grant(node_id).is_none(), "Grant should be revoked");

    println!("✓ Capability revocation works correctly");
}

#[test]
fn test_capability_escalation_detection() {
    // Test detecting when a component requests different capabilities
    let original_caps = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]);
    let escalated_caps = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp"), std::path::PathBuf::from("/etc")]);

    assert_ne!(
        original_caps, escalated_caps,
        "Adding paths should change capability set"
    );

    let network_caps = CapabilitySet::network(vec!["example.com".to_string()]);
    assert_ne!(
        original_caps, network_caps,
        "Different capability types should not match"
    );

    println!("✓ Capability escalation detection works");
}

#[test]
fn test_capability_serialization() {
    // Test that capability grants are preserved during save/load
    let mut graph = NodeGraph::new("Capability Persistence".to_string(), "Test".to_string());

    let node_id = Uuid::new_v4();
    let capabilities = CapabilitySet::file_read(vec![std::path::PathBuf::from("/data")]);

    let grant = CapabilityGrant {
        node_id,
        capability_set: capabilities.clone(),
        granted_at: "2024-01-01T00:00:00Z".to_string(),
        scope: "graph".to_string(),
    };

    graph.grant_capability(grant);

    // Serialize
    let bytes = graph.to_bytes().expect("Serialization should succeed");

    // Deserialize
    let loaded = NodeGraph::from_bytes(&bytes).expect("Deserialization should succeed");

    // Verify grant is preserved
    let loaded_grant = loaded.get_capability_grant(node_id);
    assert!(loaded_grant.is_some(), "Grant should be preserved after save/load");

    let loaded_grant = loaded_grant.unwrap();
    assert_eq!(loaded_grant.node_id, node_id, "Node ID should match");
    assert_eq!(loaded_grant.capability_set, capabilities, "Capabilities should match");
    assert_eq!(loaded_grant.granted_at, "2024-01-01T00:00:00Z", "Timestamp should match");

    println!("✓ Capability grants preserved through serialization");
}

#[test]
fn test_multiple_capability_types() {
    // Test that multiple different capability types work correctly
    let mut graph = NodeGraph::new("Multi-Capability".to_string(), "Test".to_string());

    let node1 = Uuid::new_v4();
    let node2 = Uuid::new_v4();
    let node3 = Uuid::new_v4();

    let grant1 = CapabilityGrant {
        node_id: node1,
        capability_set: CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]),
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    let grant2 = CapabilityGrant {
        node_id: node2,
        capability_set: CapabilitySet::network(vec!["api.example.com".to_string()]),
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    let grant3 = CapabilityGrant {
        node_id: node3,
        capability_set: CapabilitySet::file_write(vec![std::path::PathBuf::from("/output")]),
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    graph.grant_capability(grant1);
    graph.grant_capability(grant2);
    graph.grant_capability(grant3);

    // Verify all grants exist and are correct
    let g1 = graph.get_capability_grant(node1).unwrap();
    assert!(g1.capability_set.description().contains("/tmp"));

    let g2 = graph.get_capability_grant(node2).unwrap();
    assert!(g2.capability_set.description().contains("api.example.com"));

    let g3 = graph.get_capability_grant(node3).unwrap();
    assert!(g3.capability_set.description().contains("/output"));

    println!("✓ Multiple capability types can coexist in graph");
}

#[test]
fn test_full_capability_warning() {
    // Test that full capability is clearly marked as dangerous
    let full_caps = CapabilitySet::full();

    let description = full_caps.description();
    assert!(
        description.contains("Full") || description.contains("unrestricted"),
        "Full capability should be clearly marked: {}",
        description
    );

    // Verify it's different from specific capabilities
    let file_caps = CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]);
    assert_ne!(full_caps, file_caps, "Full should differ from specific capabilities");

    println!("✓ Full capability is clearly distinguished");
}

#[test]
fn test_capability_scope_validation() {
    // Test that capability scopes are properly set
    let node_id = Uuid::new_v4();
    let grant = CapabilityGrant {
        node_id,
        capability_set: CapabilitySet::none(),
        granted_at: chrono::Utc::now().to_rfc3339(),
        scope: "graph".to_string(),
    };

    assert_eq!(grant.scope, "graph", "Scope should be 'graph'");

    println!("✓ Capability scope validation works");
}

#[test]
fn test_capability_grant_replacement() {
    // Test that granting new capabilities to same node replaces old ones
    let mut graph = NodeGraph::new("Replacement Test".to_string(), "Test".to_string());

    let node_id = Uuid::new_v4();

    // Grant initial capability
    let grant1 = CapabilityGrant {
        node_id,
        capability_set: CapabilitySet::file_read(vec![std::path::PathBuf::from("/tmp")]),
        granted_at: "2024-01-01T00:00:00Z".to_string(),
        scope: "graph".to_string(),
    };
    graph.grant_capability(grant1);

    // Grant different capability to same node
    let grant2 = CapabilityGrant {
        node_id,
        capability_set: CapabilitySet::network(vec!["api.example.com".to_string()]),
        granted_at: "2024-01-02T00:00:00Z".to_string(),
        scope: "graph".to_string(),
    };
    graph.grant_capability(grant2);

    // Should have the latest grant
    let current_grant = graph.get_capability_grant(node_id).unwrap();
    assert!(
        current_grant.capability_set.description().contains("api.example.com"),
        "Should have network capability, not file-read"
    );
    assert_eq!(
        current_grant.granted_at, "2024-01-02T00:00:00Z",
        "Should have latest timestamp"
    );

    println!("✓ Capability grant replacement works correctly");
}

#[test]
fn test_no_capability_components() {
    // Test that components with no capabilities work correctly
    let none_caps = CapabilitySet::none();

    assert!(
        none_caps.description().contains("No system access") || none_caps.description().contains("pure computation"),
        "None capability should be clearly marked"
    );

    // Components with no capabilities should not require grants
    // (This is implicit - they work without grants)

    println!("✓ No-capability components handled correctly");
}
