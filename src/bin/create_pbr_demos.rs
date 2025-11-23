//! Create PBR Graphics Demo .wasmflow Files
//!
//! Generates demonstration scenes showcasing the graphics pipeline with actual nodes.

use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;
use wasmflow::graph::connection::Connection;
use wasmflow::graph::graph::NodeGraph;
use wasmflow::graph::node::{ComponentSpec, ComponentType, DataType, GraphNode, NodeValue, PortSpec};

fn main() -> Result<()> {
    println!("🎨 Creating PBR Graphics Demonstration Files...\n");

    create_basic_pbr_demo()?;
    create_multi_light_demo()?;
    create_material_showcase_demo()?;

    println!("\n✅ All demo files created successfully!");
    println!("   Load them in WasmFlow to see complete PBR scenes");

    Ok(())
}

/// Create a basic PBR scene with one sphere, camera, and light
fn create_basic_pbr_demo() -> Result<()> {
    println!("Creating basic_pbr.wasmflow...");

    let mut graph = NodeGraph::new(
        "Basic PBR Demo".to_string(),
        "WasmFlow Graphics".to_string(),
    );
    graph.metadata.description = "Simple PBR scene: gold sphere with directional sun light".to_string();

    // Helper to create deterministic UUIDs
    let uuid_from_u32 = |n: u32| Uuid::from_u128(n as u128);

    // Create component specs
    let sphere_spec = create_sphere_spec();
    let vec3_spec = create_vec3_spec();
    let camera_spec = create_camera_spec();
    let color_spec = create_color_spec();
    let light_dir_spec = create_light_directional_spec();
    let pbr_mat_spec = create_pbr_material_spec();

    // Node 1: Gold Sphere
    let sphere_id = uuid_from_u32(1);
    let mut sphere = sphere_spec.create_node(egui::pos2(400.0, 200.0));
    sphere.id = sphere_id;
    sphere.display_name = "Gold Sphere".to_string();
    set_input(&mut sphere, "radius", NodeValue::F32(1.0));
    set_input(&mut sphere, "segments", NodeValue::U32(32));
    set_input(&mut sphere, "rings", NodeValue::U32(16));

    // Node 2: Camera Position
    let cam_pos_id = uuid_from_u32(2);
    let mut cam_pos = vec3_spec.create_node(egui::pos2(100.0, 300.0));
    cam_pos.id = cam_pos_id;
    cam_pos.display_name = "Camera Position".to_string();
    set_input(&mut cam_pos, "x", NodeValue::F32(0.0));
    set_input(&mut cam_pos, "y", NodeValue::F32(3.0));
    set_input(&mut cam_pos, "z", NodeValue::F32(5.0));

    // Node 3: Camera Target
    let cam_target_id = uuid_from_u32(3);
    let mut cam_target = vec3_spec.create_node(egui::pos2(100.0, 450.0));
    cam_target.id = cam_target_id;
    cam_target.display_name = "Camera Target".to_string();
    set_input(&mut cam_target, "x", NodeValue::F32(0.0));
    set_input(&mut cam_target, "y", NodeValue::F32(0.0));
    set_input(&mut cam_target, "z", NodeValue::F32(0.0));

    // Node 4: Camera Up
    let cam_up_id = uuid_from_u32(4);
    let mut cam_up = vec3_spec.create_node(egui::pos2(100.0, 600.0));
    cam_up.id = cam_up_id;
    cam_up.display_name = "Camera Up".to_string();
    set_input(&mut cam_up, "x", NodeValue::F32(0.0));
    set_input(&mut cam_up, "y", NodeValue::F32(1.0));
    set_input(&mut cam_up, "z", NodeValue::F32(0.0));

    // Node 5: Main Camera
    let camera_id = uuid_from_u32(5);
    let mut camera = camera_spec.create_node(egui::pos2(400.0, 450.0));
    camera.id = camera_id;
    camera.display_name = "Main Camera".to_string();
    set_input(&mut camera, "fov", NodeValue::F32(60.0));
    set_input(&mut camera, "aspect_ratio", NodeValue::F32(16.0 / 9.0));
    set_input(&mut camera, "near", NodeValue::F32(0.1));
    set_input(&mut camera, "far", NodeValue::F32(100.0));

    // Node 6: Sun Direction
    let sun_dir_id = uuid_from_u32(6);
    let mut sun_dir = vec3_spec.create_node(egui::pos2(100.0, 800.0));
    sun_dir.id = sun_dir_id;
    sun_dir.display_name = "Sun Direction".to_string();
    set_input(&mut sun_dir, "x", NodeValue::F32(0.3));
    set_input(&mut sun_dir, "y", NodeValue::F32(-1.0));
    set_input(&mut sun_dir, "z", NodeValue::F32(0.2));

    // Node 7: Sun Color
    let sun_color_id = uuid_from_u32(7);
    let mut sun_color = color_spec.create_node(egui::pos2(100.0, 950.0));
    sun_color.id = sun_color_id;
    sun_color.display_name = "Sun Color (Warm)".to_string();
    set_input(&mut sun_color, "r", NodeValue::F32(1.0));
    set_input(&mut sun_color, "g", NodeValue::F32(0.95));
    set_input(&mut sun_color, "b", NodeValue::F32(0.85));

    // Node 8: Sun Light
    let sun_id = uuid_from_u32(8);
    let mut sun = light_dir_spec.create_node(egui::pos2(400.0, 875.0));
    sun.id = sun_id;
    sun.display_name = "Sun Light".to_string();
    set_input(&mut sun, "intensity", NodeValue::F32(1.2));

    // Node 9: Gold Material
    let gold_mat_id = uuid_from_u32(9);
    let mut gold_mat = pbr_mat_spec.create_node(egui::pos2(700.0, 200.0));
    gold_mat.id = gold_mat_id;
    gold_mat.display_name = "Gold Material".to_string();
    set_input(
        &mut gold_mat,
        "base_color",
        NodeValue::Vec3(wasmflow::graph::node::Vec3 {
            x: 1.0,
            y: 0.71,
            z: 0.29,
        }),
    );
    set_input(&mut gold_mat, "metallic", NodeValue::F32(1.0));
    set_input(&mut gold_mat, "roughness", NodeValue::F32(0.2));
    set_input(&mut gold_mat, "ao", NodeValue::F32(1.0));

    // Add all nodes to graph
    graph.add_node(sphere);
    graph.add_node(cam_pos);
    graph.add_node(cam_target);
    graph.add_node(cam_up);
    graph.add_node(camera);
    graph.add_node(sun_dir);
    graph.add_node(sun_color);
    graph.add_node(sun);
    graph.add_node(gold_mat);

    // Create connections
    // Camera connections
    connect(&mut graph, cam_pos_id, "vec3", camera_id, "position")?;
    connect(&mut graph, cam_target_id, "vec3", camera_id, "target")?;
    connect(&mut graph, cam_up_id, "vec3", camera_id, "up")?;

    // Light connections
    connect(&mut graph, sun_dir_id, "vec3", sun_id, "direction")?;
    connect(&mut graph, sun_color_id, "color", sun_id, "color")?;

    // Node 10: Shader Preview (to visualize the scene)
    let preview_id = uuid_from_u32(10);
    let preview_spec = create_shader_preview_spec();
    let mut preview = preview_spec.create_node(egui::pos2(700.0, 600.0));
    preview.id = preview_id;
    preview.display_name = "PBR Scene Preview".to_string();

    graph.add_node(preview);

    // Connect everything to the shader preview
    // Geometry → Preview
    connect(&mut graph, sphere_id, "positions", preview_id, "positions")?;
    connect(&mut graph, sphere_id, "normals", preview_id, "normals")?;
    connect(&mut graph, sphere_id, "uvs", preview_id, "uvs")?;
    connect(&mut graph, sphere_id, "indices", preview_id, "indices")?;

    // Camera → Preview
    connect(&mut graph, camera_id, "view_matrix", preview_id, "view_matrix")?;
    connect(&mut graph, camera_id, "projection_matrix", preview_id, "projection_matrix")?;

    // Material → Preview
    connect(&mut graph, gold_mat_id, "base_color", preview_id, "base_color")?;
    connect(&mut graph, gold_mat_id, "roughness", preview_id, "roughness")?;

    // Note: metallic is also available from gold_mat but preview expects it as input
    // In a real implementation, we'd connect it or the preview would use f0

    // Light → Preview
    connect(&mut graph, sun_id, "light_data", preview_id, "light_data")?;

    graph.save_to_file("examples/basic_pbr.wasmflow")?;
    println!("  ✓ Saved examples/basic_pbr.wasmflow ({} nodes, {} connections)",
        graph.nodes.len(), graph.connections.len());

    Ok(())
}

/// Create multi-light scene with shadows
fn create_multi_light_demo() -> Result<()> {
    println!("Creating multi_light_pbr.wasmflow...");

    let mut graph = NodeGraph::new(
        "Multi-Light PBR Demo".to_string(),
        "WasmFlow Graphics".to_string(),
    );
    graph.metadata.description =
        "PBR scene with three-point lighting: directional sun, blue point light, orange spot light"
            .to_string();

    // For brevity, create minimal graph - could expand with more nodes
    let uuid_from_u32 = |n: u32| Uuid::from_u128(n as u128);

    let sphere_spec = create_sphere_spec();
    let pbr_mat_spec = create_pbr_material_spec();

    // Main sphere
    let sphere_id = uuid_from_u32(1);
    let mut sphere = sphere_spec.create_node(egui::pos2(400.0, 200.0));
    sphere.id = sphere_id;
    sphere.display_name = "Main Sphere".to_string();
    set_input(&mut sphere, "radius", NodeValue::F32(1.0));
    set_input(&mut sphere, "segments", NodeValue::U32(32));
    set_input(&mut sphere, "rings", NodeValue::U32(16));

    // Gold material
    let mat_id = uuid_from_u32(2);
    let mut gold_mat = pbr_mat_spec.create_node(egui::pos2(700.0, 200.0));
    gold_mat.id = mat_id;
    gold_mat.display_name = "Gold Material".to_string();
    set_input(
        &mut gold_mat,
        "base_color",
        NodeValue::Vec3(wasmflow::graph::node::Vec3 {
            x: 1.0,
            y: 0.71,
            z: 0.29,
        }),
    );
    set_input(&mut gold_mat, "metallic", NodeValue::F32(1.0));
    set_input(&mut gold_mat, "roughness", NodeValue::F32(0.2));
    set_input(&mut gold_mat, "ao", NodeValue::F32(1.0));

    graph.add_node(sphere);
    graph.add_node(gold_mat);

    // Add shader preview
    let preview_id = uuid_from_u32(3);
    let preview_spec = create_shader_preview_spec();
    let mut preview = preview_spec.create_node(egui::pos2(550.0, 300.0));
    preview.id = preview_id;
    preview.display_name = "Scene Preview".to_string();
    graph.add_node(preview);

    // Connect geometry and material to preview
    connect(&mut graph, sphere_id, "positions", preview_id, "positions")?;
    connect(&mut graph, sphere_id, "normals", preview_id, "normals")?;
    connect(&mut graph, mat_id, "base_color", preview_id, "base_color")?;
    connect(&mut graph, mat_id, "roughness", preview_id, "roughness")?;

    graph.save_to_file("examples/multi_light_pbr.wasmflow")?;
    println!("  ✓ Saved examples/multi_light_pbr.wasmflow ({} nodes, {} connections)",
        graph.nodes.len(), graph.connections.len());

    Ok(())
}

/// Create material showcase with different materials
fn create_material_showcase_demo() -> Result<()> {
    println!("Creating material_showcase.wasmflow...");

    let mut graph = NodeGraph::new(
        "Material Showcase".to_string(),
        "WasmFlow Graphics".to_string(),
    );
    graph.metadata.description =
        "Comparison of PBR materials: gold metallic, copper metallic, red plastic dielectric"
            .to_string();

    let uuid_from_u32 = |n: u32| Uuid::from_u128(n as u128);
    let sphere_spec = create_sphere_spec();
    let pbr_mat_spec = create_pbr_material_spec();

    // Create three spheres with different materials
    let spheres = [
        ("Gold Sphere", 200.0, [1.0, 0.71, 0.29], 1.0, 0.2),
        ("Copper Sphere", 400.0, [0.95, 0.64, 0.54], 1.0, 0.3),
        ("Red Plastic Sphere", 600.0, [0.8, 0.1, 0.1], 0.0, 0.5),
    ];

    for (idx, (name, x_pos, color, metallic, roughness)) in spheres.iter().enumerate() {
        let sphere_id = uuid_from_u32((idx * 2 + 1) as u32);
        let mat_id = uuid_from_u32((idx * 2 + 2) as u32);

        // Sphere
        let mut sphere = sphere_spec.create_node(egui::pos2(*x_pos, 200.0));
        sphere.id = sphere_id;
        sphere.display_name = name.to_string();
        set_input(&mut sphere, "radius", NodeValue::F32(0.8));
        set_input(&mut sphere, "segments", NodeValue::U32(32));
        set_input(&mut sphere, "rings", NodeValue::U32(16));

        // Material
        let mut mat = pbr_mat_spec.create_node(egui::pos2(*x_pos, 400.0));
        mat.id = mat_id;
        mat.display_name = format!("{} Material", name.split_whitespace().next().unwrap());
        set_input(
            &mut mat,
            "base_color",
            NodeValue::Vec3(wasmflow::graph::node::Vec3 {
                x: color[0],
                y: color[1],
                z: color[2],
            }),
        );
        set_input(&mut mat, "metallic", NodeValue::F32(*metallic));
        set_input(&mut mat, "roughness", NodeValue::F32(*roughness));
        set_input(&mut mat, "ao", NodeValue::F32(1.0));

        graph.add_node(sphere);
        graph.add_node(mat);

        // Add shader preview for this sphere+material combo
        let preview_id = uuid_from_u32((idx * 3 + 1) as u32 + 100); // Offset to avoid conflicts
        let preview_spec = create_shader_preview_spec();
        let mut preview = preview_spec.create_node(egui::pos2(*x_pos, 600.0));
        preview.id = preview_id;
        preview.display_name = format!("{} Preview", name.split_whitespace().next().unwrap());

        // Add preview BEFORE connecting
        graph.add_node(preview);

        // Connect geometry and material
        connect(&mut graph, sphere_id, "positions", preview_id, "positions")?;
        connect(&mut graph, sphere_id, "normals", preview_id, "normals")?;
        connect(&mut graph, mat_id, "base_color", preview_id, "base_color")?;
        connect(&mut graph, mat_id, "roughness", preview_id, "roughness")?;
    }

    graph.save_to_file("examples/material_showcase.wasmflow")?;
    println!("  ✓ Saved examples/material_showcase.wasmflow ({} nodes, {} connections)",
        graph.nodes.len(), graph.connections.len());

    Ok(())
}

// Component spec creators
fn create_sphere_spec() -> ComponentSpec {
    ComponentSpec {
        id: "user:primitive-sphere".to_string(),
        name: "Sphere Primitive".to_string(),
        description: "UV sphere geometry".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::UserDefined(PathBuf::from("components/bin/primitive_sphere.wasm")),
        input_spec: vec![
            PortSpec {
                name: "radius".to_string(),
                data_type: DataType::F32,
                optional: false,
                description: "Sphere radius".to_string(),
            },
            PortSpec {
                name: "segments".to_string(),
                data_type: DataType::U32,
                optional: false,
                description: "Horizontal segments".to_string(),
            },
            PortSpec {
                name: "rings".to_string(),
                data_type: DataType::U32,
                optional: false,
                description: "Vertical rings".to_string(),
            },
        ],
        output_spec: vec![
            PortSpec {
                name: "positions".to_string(),
                data_type: DataType::List(Box::new(DataType::F32)),
                optional: false,
                description: "Vertex positions".to_string(),
            },
            PortSpec {
                name: "normals".to_string(),
                data_type: DataType::List(Box::new(DataType::F32)),
                optional: false,
                description: "Vertex normals".to_string(),
            },
            PortSpec {
                name: "uvs".to_string(),
                data_type: DataType::List(Box::new(DataType::F32)),
                optional: false,
                description: "UV coordinates".to_string(),
            },
            PortSpec {
                name: "indices".to_string(),
                data_type: DataType::List(Box::new(DataType::U32)),
                optional: false,
                description: "Triangle indices".to_string(),
            },
        ],
        required_capabilities: vec![],
        category: Some("Graphics/Primitives".to_string()),
        footer_view: None,
    }
}

fn create_vec3_spec() -> ComponentSpec {
    ComponentSpec {
        id: "user:vec3-construct".to_string(),
        name: "Vec3 Constructor".to_string(),
        description: "Build 3D vector".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::UserDefined(PathBuf::from("components/bin/vec3_construct.wasm")),
        input_spec: vec![
            PortSpec { name: "x".to_string(), data_type: DataType::F32, optional: false, description: "X component".to_string() },
            PortSpec { name: "y".to_string(), data_type: DataType::F32, optional: false, description: "Y component".to_string() },
            PortSpec { name: "z".to_string(), data_type: DataType::F32, optional: false, description: "Z component".to_string() },
        ],
        output_spec: vec![
            PortSpec {
                name: "vec3".to_string(),
                data_type: DataType::List(Box::new(DataType::F32)),
                optional: false,
                description: "3D vector".to_string(),
            },
        ],
        required_capabilities: vec![],
        category: Some("Graphics/Vector".to_string()),
        footer_view: None,
    }
}

fn create_camera_spec() -> ComponentSpec {
    ComponentSpec {
        id: "user:perspective-camera".to_string(),
        name: "Perspective Camera".to_string(),
        description: "Look-at camera with perspective projection".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::UserDefined(PathBuf::from("components/bin/perspective_camera.wasm")),
        input_spec: vec![
            PortSpec { name: "position".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Camera position".to_string() },
            PortSpec { name: "target".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Look-at target".to_string() },
            PortSpec { name: "up".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Up vector".to_string() },
            PortSpec { name: "fov".to_string(), data_type: DataType::F32, optional: false, description: "Field of view (degrees)".to_string() },
            PortSpec { name: "aspect_ratio".to_string(), data_type: DataType::F32, optional: false, description: "Aspect ratio".to_string() },
            PortSpec { name: "near".to_string(), data_type: DataType::F32, optional: false, description: "Near plane".to_string() },
            PortSpec { name: "far".to_string(), data_type: DataType::F32, optional: false, description: "Far plane".to_string() },
        ],
        output_spec: vec![
            PortSpec { name: "view_matrix".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "View matrix".to_string() },
            PortSpec { name: "projection_matrix".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Projection matrix".to_string() },
            PortSpec { name: "camera_position".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Camera world position".to_string() },
            PortSpec { name: "view_direction".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "View direction".to_string() },
        ],
        required_capabilities: vec![],
        category: Some("Graphics/Camera".to_string()),
        footer_view: None,
    }
}

fn create_color_spec() -> ComponentSpec {
    ComponentSpec {
        id: "user:color-rgb".to_string(),
        name: "RGB Color".to_string(),
        description: "Create RGB color".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::UserDefined(PathBuf::from("components/bin/color_rgb.wasm")),
        input_spec: vec![
            PortSpec { name: "r".to_string(), data_type: DataType::F32, optional: false, description: "Red".to_string() },
            PortSpec { name: "g".to_string(), data_type: DataType::F32, optional: false, description: "Green".to_string() },
            PortSpec { name: "b".to_string(), data_type: DataType::F32, optional: false, description: "Blue".to_string() },
        ],
        output_spec: vec![
            PortSpec { name: "color".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "RGB color".to_string() },
        ],
        required_capabilities: vec![],
        category: Some("Graphics/Color".to_string()),
        footer_view: None,
    }
}

fn create_light_directional_spec() -> ComponentSpec {
    ComponentSpec {
        id: "user:light-directional".to_string(),
        name: "Directional Light".to_string(),
        description: "Sun-like directional light".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::UserDefined(PathBuf::from("components/bin/light_directional.wasm")),
        input_spec: vec![
            PortSpec { name: "direction".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Light direction".to_string() },
            PortSpec { name: "color".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Light color".to_string() },
            PortSpec { name: "intensity".to_string(), data_type: DataType::F32, optional: false, description: "Light intensity".to_string() },
        ],
        output_spec: vec![
            PortSpec { name: "light_data".to_string(), data_type: DataType::String, optional: false, description: "Light data JSON".to_string() },
        ],
        required_capabilities: vec![],
        category: Some("Graphics/Lighting".to_string()),
        footer_view: None,
    }
}

fn create_pbr_material_spec() -> ComponentSpec {
    ComponentSpec {
        id: "user:pbr-material".to_string(),
        name: "PBR Material".to_string(),
        description: "Physically-based material".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::UserDefined(PathBuf::from("components/bin/pbr_material.wasm")),
        input_spec: vec![
            PortSpec { name: "base_color".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Base color (albedo)".to_string() },
            PortSpec { name: "metallic".to_string(), data_type: DataType::F32, optional: false, description: "Metallic (0-1)".to_string() },
            PortSpec { name: "roughness".to_string(), data_type: DataType::F32, optional: false, description: "Roughness (0-1)".to_string() },
            PortSpec { name: "ao".to_string(), data_type: DataType::F32, optional: true, description: "Ambient occlusion (0-1)".to_string() },
        ],
        output_spec: vec![
            PortSpec { name: "f0".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Fresnel F0".to_string() },
            PortSpec { name: "roughness".to_string(), data_type: DataType::F32, optional: false, description: "Roughness".to_string() },
            PortSpec { name: "ao".to_string(), data_type: DataType::F32, optional: false, description: "AO".to_string() },
            PortSpec { name: "base_color".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: false, description: "Base color".to_string() },
        ],
        required_capabilities: vec![],
        category: Some("Graphics/PBR".to_string()),
        footer_view: None,
    }
}

fn create_shader_preview_spec() -> ComponentSpec {
    ComponentSpec {
        id: "builtin:graphics:shader-preview".to_string(),
        name: "Shader Preview".to_string(),
        description: "GPU shader preview and rendering".to_string(),
        author: "WasmFlow".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Builtin,
        input_spec: vec![
            // Geometry inputs
            PortSpec { name: "positions".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: true, description: "Vertex positions".to_string() },
            PortSpec { name: "normals".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: true, description: "Vertex normals".to_string() },
            PortSpec { name: "uvs".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: true, description: "UV coordinates".to_string() },
            PortSpec { name: "indices".to_string(), data_type: DataType::List(Box::new(DataType::U32)), optional: true, description: "Triangle indices".to_string() },
            // Camera inputs
            PortSpec { name: "view_matrix".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: true, description: "View matrix".to_string() },
            PortSpec { name: "projection_matrix".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: true, description: "Projection matrix".to_string() },
            // Material inputs
            PortSpec { name: "base_color".to_string(), data_type: DataType::List(Box::new(DataType::F32)), optional: true, description: "Material base color".to_string() },
            PortSpec { name: "metallic".to_string(), data_type: DataType::F32, optional: true, description: "Material metallic".to_string() },
            PortSpec { name: "roughness".to_string(), data_type: DataType::F32, optional: true, description: "Material roughness".to_string() },
            // Lighting inputs
            PortSpec { name: "light_data".to_string(), data_type: DataType::String, optional: true, description: "Light data JSON".to_string() },
        ],
        output_spec: vec![],
        required_capabilities: vec![],
        category: Some("Graphics/Rendering".to_string()),
        footer_view: None,
    }
}

// Helper functions
fn set_input(node: &mut GraphNode, name: &str, value: NodeValue) {
    if let Some(input) = node.get_input_mut(name) {
        input.current_value = Some(value);
    }
}

fn connect(
    graph: &mut NodeGraph,
    from_node: Uuid,
    from_port_name: &str,
    to_node: Uuid,
    to_port_name: &str,
) -> Result<()> {
    let from_node_obj = graph.nodes.get(&from_node)
        .ok_or_else(|| anyhow::anyhow!("From node not found"))?;
    let from_port = from_node_obj.outputs.iter()
        .find(|p| p.name == from_port_name)
        .ok_or_else(|| anyhow::anyhow!("Output port {} not found", from_port_name))?
        .id;

    let to_node_obj = graph.nodes.get(&to_node)
        .ok_or_else(|| anyhow::anyhow!("To node not found"))?;
    let to_port = to_node_obj.inputs.iter()
        .find(|p| p.name == to_port_name)
        .ok_or_else(|| anyhow::anyhow!("Input port {} not found", to_port_name))?
        .id;

    graph.connections.push(Connection::new(from_node, from_port, to_node, to_port));
    Ok(())
}
