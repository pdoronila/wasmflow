//! T052: Unit tests for component registry replacement logic
//!
//! Tests the ability to replace existing components when recompiling
//! with the same component name.

use wasmflow::graph::node::{ComponentRegistry, ComponentSpec, ComponentType, DataType};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_new_component() {
        let mut registry = ComponentRegistry::new();

        let spec = ComponentSpec::new_user_defined(
            "user:TestComponent".to_string(),
            "TestComponent".to_string(),
            "A test component".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/test.wasm"),
        );

        // Should succeed
        let result = registry.register_component(spec);
        assert!(result.is_ok());

        // Should be retrievable
        assert!(registry.has_component("user:TestComponent"));
        let retrieved = registry.get_by_id("user:TestComponent");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_unregister_existing_component() {
        let mut registry = ComponentRegistry::new();

        let spec = ComponentSpec::new_user_defined(
            "user:TestComponent".to_string(),
            "TestComponent".to_string(),
            "A test component".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/test.wasm"),
        );

        registry.register_component(spec).unwrap();
        assert!(registry.has_component("user:TestComponent"));

        // Unregister
        let removed = registry.unregister_component("user:TestComponent");
        assert!(removed, "Should return true when component exists");

        // Should no longer exist
        assert!(!registry.has_component("user:TestComponent"));
        assert!(registry.get_by_id("user:TestComponent").is_none());
    }

    #[test]
    fn test_unregister_nonexistent_component() {
        let mut registry = ComponentRegistry::new();

        // Should return false when component doesn't exist
        let removed = registry.unregister_component("user:NonExistent");
        assert!(!removed, "Should return false when component doesn't exist");
    }

    #[test]
    fn test_replace_component_workflow() {
        // Test the full workflow: register, unregister, register again
        let mut registry = ComponentRegistry::new();

        // Register version 1
        let spec_v1 = ComponentSpec::new_user_defined(
            "user:TestComponent".to_string(),
            "TestComponent".to_string(),
            "Version 1".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/test_v1.wasm"),
        );
        registry.register_component(spec_v1).unwrap();

        // Verify version 1 exists
        let v1 = registry.get_by_id("user:TestComponent").unwrap();
        assert_eq!(v1.description, "Version 1");

        // Unregister version 1
        assert!(registry.unregister_component("user:TestComponent"));

        // Register version 2 with same name
        let spec_v2 = ComponentSpec::new_user_defined(
            "user:TestComponent".to_string(),
            "TestComponent".to_string(),
            "Version 2".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/test_v2.wasm"),
        );
        registry.register_component(spec_v2).unwrap();

        // Verify version 2 replaced version 1
        let v2 = registry.get_by_id("user:TestComponent").unwrap();
        assert_eq!(v2.description, "Version 2");

        // Should only be one component with this name
        let all_components = registry.list_all();
        let test_components: Vec<_> = all_components
            .iter()
            .filter(|c| c.id == "user:TestComponent")
            .collect();
        assert_eq!(test_components.len(), 1, "Should only have one component with this ID");
    }

    #[test]
    fn test_multiple_user_components() {
        let mut registry = ComponentRegistry::new();

        // Register multiple user-defined components
        for i in 1..=5 {
            let spec = ComponentSpec::new_user_defined(
                format!("user:Component{}", i),
                format!("Component{}", i),
                format!("Component number {}", i),
                Some("User-Defined".to_string()),
                PathBuf::from(format!("/tmp/component{}.wasm", i)),
            );
            registry.register_component(spec).unwrap();
        }

        // All should exist
        for i in 1..=5 {
            assert!(registry.has_component(&format!("user:Component{}", i)));
        }

        // Replace component 3
        registry.unregister_component("user:Component3");
        let spec = ComponentSpec::new_user_defined(
            "user:Component3".to_string(),
            "Component3".to_string(),
            "Updated component 3".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/component3_v2.wasm"),
        );
        registry.register_component(spec).unwrap();

        // Verify still have 5 components total
        let user_components: Vec<_> = registry.list_all()
            .iter()
            .filter(|c| matches!(c.component_type, ComponentType::UserDefined(_)))
            .collect();
        assert_eq!(user_components.len(), 5);

        // Verify component 3 was updated
        let comp3 = registry.get_by_id("user:Component3").unwrap();
        assert_eq!(comp3.description, "Updated component 3");
    }

    #[test]
    fn test_builtin_components_not_affected() {
        let mut registry = ComponentRegistry::new();

        // Register a builtin component
        let builtin = ComponentSpec::new_builtin(
            "builtin:math:add".to_string(),
            "Add".to_string(),
            "Adds numbers".to_string(),
            Some("Math".to_string()),
        );
        registry.register_builtin(builtin);

        // Register a user component
        let user = ComponentSpec::new_user_defined(
            "user:MyAdd".to_string(),
            "MyAdd".to_string(),
            "My add".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/myadd.wasm"),
        );
        registry.register_component(user).unwrap();

        // Remove user component
        assert!(registry.unregister_component("user:MyAdd"));

        // Builtin should still exist
        assert!(registry.has_component("builtin:math:add"));

        // User component should be gone
        assert!(!registry.has_component("user:MyAdd"));
    }

    #[test]
    fn test_component_id_format() {
        // Component IDs for user-defined components should follow "user:{name}" format
        let mut registry = ComponentRegistry::new();

        let spec = ComponentSpec::new_user_defined(
            "user:TestComponent".to_string(),
            "TestComponent".to_string(),
            "Test".to_string(),
            Some("User-Defined".to_string()),
            PathBuf::from("/tmp/test.wasm"),
        );

        registry.register_component(spec).unwrap();

        // Should be accessible by full ID
        assert!(registry.get_by_id("user:TestComponent").is_some());

        // Should NOT be accessible by short name
        assert!(registry.get_by_id("TestComponent").is_none());
    }

    #[test]
    fn test_has_component_performance() {
        // has_component should be fast (O(1) lookup)
        let mut registry = ComponentRegistry::new();

        // Register many components
        for i in 0..1000 {
            let spec = ComponentSpec::new_user_defined(
                format!("user:Component{}", i),
                format!("Component{}", i),
                format!("Component {}", i),
                Some("User-Defined".to_string()),
                PathBuf::from(format!("/tmp/comp{}.wasm", i)),
            );
            registry.register_component(spec).unwrap();
        }

        // Lookup should be fast
        use std::time::Instant;
        let start = Instant::now();
        assert!(registry.has_component("user:Component500"));
        let elapsed = start.elapsed();

        // Should be < 1ms (usually microseconds)
        assert!(elapsed.as_millis() < 1, "has_component should be O(1)");
    }

    #[test]
    fn test_list_user_defined_components() {
        let mut registry = ComponentRegistry::new();

        // Add builtin
        registry.register_builtin(ComponentSpec::new_builtin(
            "builtin:test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        ));

        // Add user-defined
        for i in 1..=3 {
            let spec = ComponentSpec::new_user_defined(
                format!("user:Comp{}", i),
                format!("Comp{}", i),
                format!("Component {}", i),
                Some("User-Defined".to_string()),
                PathBuf::from(format!("/tmp/comp{}.wasm", i)),
            );
            registry.register_component(spec).unwrap();
        }

        // Filter user-defined components
        let user_components: Vec<_> = registry.list_all()
            .iter()
            .filter(|c| matches!(c.component_type, ComponentType::UserDefined(_)))
            .collect();

        assert_eq!(user_components.len(), 3);

        // All should be in User-Defined category
        for comp in user_components {
            assert_eq!(comp.category, Some("User-Defined".to_string()));
        }
    }
}
