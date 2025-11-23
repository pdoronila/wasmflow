//! Create PBR Graphics Demo .wasmflow Files
//!
//! Generates demonstration scenes showcasing the graphics pipeline.
//! Since creating nodes programmatically is complex, we create minimal
//! working graphs that can be expanded in the UI.

use anyhow::Result;
use wasmflow::graph::graph::NodeGraph;

fn main() -> Result<()> {
    println!("🎨 Creating PBR Graphics Demonstration Files...\n");

    create_basic_pbr_demo()?;
    create_multi_light_demo()?;
    create_material_showcase_demo()?;

    println!("\n✅ All demo files created successfully!");
    println!("   Load them in WasmFlow and add graphics components from the palette");
    println!("\n📚 Recommended workflow:");
    println!("   1. Load a demo file");
    println!("   2. Add nodes from Graphics palette:");
    println!("      - Primitives: sphere, cube, plane");
    println!("      - Camera: perspective-camera");
    println!("      - Lighting: light-directional, light-point, light-spot");
    println!("      - PBR: pbr-material, pbr-brdf components");
    println!("      - Shadows: shadow-directional, shadow-point, shadow-spot");
    println!("   3. Connect nodes and execute");

    Ok(())
}

fn create_basic_pbr_demo() -> Result<()> {
    println!("Creating basic_pbr.wasmflow...");

    let mut graph = NodeGraph::new(
        "Basic PBR Demo".to_string(),
        "WasmFlow Graphics".to_string(),
    );

    graph.metadata.description = r#"Basic PBR scene starter template.

Recommended setup:
1. Add primitive-sphere (radius=1.0, segments=32, rings=16)
2. Add vec3-construct nodes for camera position (0, 3, 5), target (0, 0, 0), up (0, 1, 0)
3. Add perspective-camera (fov=60, aspect=1.777, near=0.1, far=100)
4. Add light-directional with vec3-construct for direction (0.3, -1, 0.2)
5. Add color-rgb for sun color (1.0, 0.95, 0.85)
6. Add pbr-material with gold settings:
   - base_color: [1.0, 0.71, 0.29]
   - metallic: 1.0
   - roughness: 0.2
   - ao: 1.0

Connect the nodes:
- Camera: position/target/up → perspective-camera → view/projection matrices
- Light: direction + color → light-directional → light_data
- Material: base_color/metallic/roughness/ao → pbr-material → f0/roughness/ao"#.to_string();

    graph.save_to_file("examples/basic_pbr.wasmflow")?;
    println!("  ✓ Saved examples/basic_pbr.wasmflow");

    Ok(())
}

fn create_multi_light_demo() -> Result<()> {
    println!("Creating multi_light_pbr.wasmflow...");

    let mut graph = NodeGraph::new(
        "Multi-Light PBR Demo".to_string(),
        "WasmFlow Graphics".to_string(),
    );

    graph.metadata.description = r#"Multi-light PBR scene with shadows.

Demonstrates multiple light types working together:

Lights to add:
1. Directional Light (Sun):
   - direction: (0.3, -1.0, 0.2)
   - color: (1.0, 0.95, 0.85) warm white
   - intensity: 1.2
   - Add shadow-directional with cascade_count=4

2. Point Light (Accent):
   - position: (2.0, 3.0, 1.0)
   - color: (0.4, 0.6, 1.0) cool blue
   - intensity: 0.8
   - radius: 10.0

3. Spot Light (Rim):
   - position: (-3.0, 4.0, 2.0)
   - direction: (0.5, -1.0, -0.3)
   - color: (1.0, 0.5, 0.2) orange
   - intensity: 1.5
   - inner_angle: 20.0, outer_angle: 30.0
   - Add shadow-spot with cone_angle=30.0

Scene objects:
- Main sphere: gold material (metallic=1.0, roughness=0.2)
- Ground plane: 20×20 units with 10×10 segments

This creates dramatic three-point lighting with soft shadows."#.to_string();

    graph.save_to_file("examples/multi_light_pbr.wasmflow")?;
    println!("  ✓ Saved examples/multi_light_pbr.wasmflow");

    Ok(())
}

fn create_material_showcase_demo() -> Result<()> {
    println!("Creating material_showcase.wasmflow...");

    let mut graph = NodeGraph::new(
        "Material Showcase".to_string(),
        "WasmFlow Graphics".to_string(),
    );

    graph.metadata.description = r#"PBR material comparison demo.

Create three spheres with different materials side by side:

1. Gold (Metallic):
   - Position: (-2, 0, 0)
   - base_color: [1.0, 0.71, 0.29]
   - metallic: 1.0
   - roughness: 0.2
   - ao: 1.0

2. Copper (Metallic):
   - Position: (0, 0, 0)
   - base_color: [0.95, 0.64, 0.54]
   - metallic: 1.0
   - roughness: 0.3
   - ao: 1.0

3. Red Plastic (Dielectric):
   - Position: (2, 0, 0)
   - base_color: [0.8, 0.1, 0.1]
   - metallic: 0.0
   - roughness: 0.5
   - ao: 1.0

Lighting:
- Directional sun from above-right
- Point light for fill from left

Camera:
- Position: (0, 2, 6)
- Looking at: (0, 0, 0)

This demonstrates:
- Metallic vs dielectric materials
- How roughness affects specular highlights
- F0 calculation based on metallic/base_color
- Energy conservation in PBR"#.to_string();

    graph.save_to_file("examples/material_showcase.wasmflow")?;
    println!("  ✓ Saved examples/material_showcase.wasmflow");

    Ok(())
}
