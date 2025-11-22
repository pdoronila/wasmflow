wit_bindgen::generate!({
    path: "./wit",
    world: "component",
    with: {
        "wasmflow:node/types@1.1.0": generate,
        "wasmflow:node/host@1.1.0": generate,
        "wasmflow:node/metadata@1.1.0": generate,
        "wasmflow:node/execution@1.1.0": generate,
    },
});

use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use wasmflow::node::types::*;

use glam::Vec3;
use std::f32::consts::PI;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "PBR Cook-Torrance BRDF".to_string(),
            version: "1.0.0".to_string(),
            description: "Complete Cook-Torrance BRDF combining GGX distribution, Fresnel-Schlick, and Smith geometry".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics/PBR".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "normal".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Surface normal (normalized)".to_string(),
            },
            PortSpec {
                name: "view_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "View direction (normalized)".to_string(),
            },
            PortSpec {
                name: "light_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light direction (normalized)".to_string(),
            },
            PortSpec {
                name: "f0".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Base reflectivity at normal incidence".to_string(),
            },
            PortSpec {
                name: "roughness".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Surface roughness (0=smooth, 1=rough)".to_string(),
            },
            PortSpec {
                name: "base_color".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Base color (albedo) for diffuse term".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "diffuse".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Diffuse BRDF term (Lambertian with energy conservation)".to_string(),
            },
            PortSpec {
                name: "specular".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Specular BRDF term (Cook-Torrance)".to_string(),
            },
            PortSpec {
                name: "total_brdf".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Complete BRDF (diffuse + specular)".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract and normalize normal
        let normal = extract_vec3(&inputs, "normal")?;
        let normal_vec = Vec3::new(normal.x, normal.y, normal.z);
        let n = normal_vec.normalize_or_zero();
        if n == Vec3::ZERO {
            return Err(ExecutionError {
                message: "normal cannot be zero vector".to_string(),
                input_name: Some("normal".to_string()),
                recovery_hint: Some("Provide a non-zero normal vector".to_string()),
            });
        }

        // Extract and normalize view direction
        let view_dir = extract_vec3(&inputs, "view_dir")?;
        let view_vec = Vec3::new(view_dir.x, view_dir.y, view_dir.z);
        let v = view_vec.normalize_or_zero();
        if v == Vec3::ZERO {
            return Err(ExecutionError {
                message: "view_dir cannot be zero vector".to_string(),
                input_name: Some("view_dir".to_string()),
                recovery_hint: Some("Provide a non-zero view direction".to_string()),
            });
        }

        // Extract and normalize light direction
        let light_dir = extract_vec3(&inputs, "light_dir")?;
        let light_vec = Vec3::new(light_dir.x, light_dir.y, light_dir.z);
        let l = light_vec.normalize_or_zero();
        if l == Vec3::ZERO {
            return Err(ExecutionError {
                message: "light_dir cannot be zero vector".to_string(),
                input_name: Some("light_dir".to_string()),
                recovery_hint: Some("Provide a non-zero light direction".to_string()),
            });
        }

        // Extract F0
        let f0 = extract_vec3(&inputs, "f0")?;
        let f0_vec = Vec3::new(f0.x, f0.y, f0.z);

        // Extract roughness
        let roughness = extract_f32(&inputs, "roughness")?;
        if roughness < 0.0 || roughness > 1.0 {
            return Err(ExecutionError {
                message: format!("Roughness must be in range [0, 1], got {}", roughness),
                input_name: Some("roughness".to_string()),
                recovery_hint: Some("Use 0 for smooth surfaces, 1 for very rough surfaces".to_string()),
            });
        }

        // Extract base color
        let base_color = extract_vec3(&inputs, "base_color")?;
        let base_color_vec = Vec3::new(base_color.x, base_color.y, base_color.z);

        // Calculate BRDF
        let (diffuse, specular) = cook_torrance_brdf(n, v, l, f0_vec, roughness, base_color_vec);
        let total = diffuse + specular;

        Ok(vec![
            (
                "diffuse".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: diffuse.x,
                    y: diffuse.y,
                    z: diffuse.z,
                }),
            ),
            (
                "specular".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: specular.x,
                    y: specular.y,
                    z: specular.z,
                }),
            ),
            (
                "total_brdf".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: total.x,
                    y: total.y,
                    z: total.z,
                }),
            ),
        ])
    }
}

/// Cook-Torrance BRDF with Lambertian diffuse
///
/// Returns: (diffuse, specular)
///
/// Specular formula: f_s = (D * F * G) / (4 * (n · v) * (n · l))
/// Diffuse formula: f_d = (base_color / π) * (1 - F)
///
/// Where:
/// - D = GGX normal distribution function
/// - F = Fresnel-Schlick approximation
/// - G = Smith geometry term (GGX variant)
/// - n, v, l = normal, view, light (all normalized)
fn cook_torrance_brdf(
    normal: Vec3,
    view: Vec3,
    light: Vec3,
    f0: Vec3,
    roughness: f32,
    base_color: Vec3,
) -> (Vec3, Vec3) {
    // Calculate half vector
    let half = (view + light).normalize_or_zero();
    if half == Vec3::ZERO {
        // Edge case: view and light are opposite directions
        return (Vec3::ZERO, Vec3::ZERO);
    }

    // Calculate dot products
    let n_dot_v = normal.dot(view).max(0.0);
    let n_dot_l = normal.dot(light).max(0.0);

    // Early exit if view or light is below the surface
    if n_dot_v <= 0.0 || n_dot_l <= 0.0 {
        return (Vec3::ZERO, Vec3::ZERO);
    }

    // Calculate GGX distribution (D)
    let distribution = ggx_distribution(normal, half, roughness);

    // Calculate Fresnel term (F)
    let fresnel = fresnel_schlick(f0, view, half);

    // Calculate Smith geometry term (G)
    let geometry = smith_geometry_ggx(normal, view, light, roughness);

    // Calculate Cook-Torrance specular BRDF
    let denom = 4.0 * n_dot_v * n_dot_l;
    let specular = if denom > 0.0 {
        fresnel * distribution * geometry / denom
    } else {
        Vec3::ZERO
    };

    // Calculate Lambertian diffuse with energy conservation
    // Energy not reflected specularly is available for diffuse
    let k_diffuse = Vec3::ONE - fresnel;
    let diffuse = k_diffuse * base_color / PI;

    (diffuse, specular)
}

/// GGX/Trowbridge-Reitz normal distribution function
fn ggx_distribution(normal: Vec3, half: Vec3, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;

    let n_dot_h = normal.dot(half).max(0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;

    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    let denom2 = denom * denom;

    a2 / (PI * denom2)
}

/// Fresnel-Schlick approximation
fn fresnel_schlick(f0: Vec3, view: Vec3, half: Vec3) -> Vec3 {
    let cos_theta = view.dot(half).max(0.0);
    let one_minus_cos = 1.0 - cos_theta;
    let power5 = one_minus_cos.powi(5);

    f0 + (Vec3::ONE - f0) * power5
}

/// Smith geometry/visibility term using GGX distribution
fn smith_geometry_ggx(normal: Vec3, view: Vec3, light: Vec3, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;

    let g1_view = smith_g1_ggx(normal, view, a2);
    let g1_light = smith_g1_ggx(normal, light, a2);

    g1_view * g1_light
}

/// Single direction term for Smith-GGX geometry
fn smith_g1_ggx(normal: Vec3, direction: Vec3, a2: f32) -> f32 {
    let n_dot_v = normal.dot(direction).max(0.0);

    if n_dot_v <= 0.0 {
        return 0.0;
    }

    let n_dot_v2 = n_dot_v * n_dot_v;
    let denom = n_dot_v + (a2 + (1.0 - a2) * n_dot_v2).sqrt();

    (2.0 * n_dot_v) / denom
}

// Helper functions
fn extract_vec3(
    inputs: &[(String, Value)],
    name: &str,
) -> Result<wasmflow::node::types::Vec3, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a vec3 value to this input".to_string()),
        })?;

    match &input.1 {
        Value::Vec3Val(v) => Ok(v.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected vec3 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a vec3 value".to_string()),
        }),
    }
}

fn extract_f32(inputs: &[(String, Value)], name: &str) -> Result<f32, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.1 {
        Value::F32Val(v) => Ok(*v),
        _ => Err(ExecutionError {
            message: format!("Expected f32 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide an f32 value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    fn create_brdf_inputs(
        normal: (f32, f32, f32),
        view: (f32, f32, f32),
        light: (f32, f32, f32),
        f0: (f32, f32, f32),
        roughness: f32,
        base_color: (f32, f32, f32),
    ) -> Vec<(String, Value)> {
        vec![
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: normal.0,
                    y: normal.1,
                    z: normal.2,
                }),
            ),
            (
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: view.0,
                    y: view.1,
                    z: view.2,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: light.0,
                    y: light.1,
                    z: light.2,
                }),
            ),
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: f0.0,
                    y: f0.1,
                    z: f0.2,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(roughness)),
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: base_color.0,
                    y: base_color.1,
                    z: base_color.2,
                }),
            ),
        ]
    }

    #[test]
    fn test_brdf_smooth_dielectric() {
        // Smooth dielectric with aligned directions
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0), // normal
            (0.0, 1.0, 0.0), // view
            (0.0, 1.0, 0.0), // light
            (0.04, 0.04, 0.04), // F0 for dielectric
            0.1,             // smooth
            (0.8, 0.8, 0.8), // base color
        );

        let result = Component::execute(inputs).unwrap();

        // Verify we have all three outputs
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "diffuse");
        assert_eq!(result[1].0, "specular");
        assert_eq!(result[2].0, "total_brdf");

        // For smooth dielectric with perfect alignment, expect some specular reflection
        let specular = match &result[1].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };
        assert!(specular.length() > 0.0);

        // Total BRDF should be non-zero
        let total = match &result[2].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };
        assert!(total.length() > 0.0);
    }

    #[test]
    fn test_brdf_rough_metal() {
        // Rough metal
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0),    // normal
            (0.0, 1.0, 0.0),    // view
            (0.0, 1.0, 0.0),    // light
            (1.0, 0.71, 0.29),  // F0 for gold
            0.8,                // rough
            (1.0, 0.71, 0.29),  // base color (not used for pure metal)
        );

        let result = Component::execute(inputs).unwrap();

        let specular = match &result[1].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        // Metals should have specular reflection even when rough
        assert!(specular.length() > 0.0);

        // For pure metal, diffuse should be very low (high F means low k_diffuse)
        let diffuse = match &result[0].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };
        assert!(diffuse.length() < specular.length());
    }

    #[test]
    fn test_brdf_energy_conservation() {
        // Test that energy is conserved (total <= 1.0)
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.04, 0.04, 0.04),
            0.5,
            (0.8, 0.8, 0.8),
        );

        let result = Component::execute(inputs).unwrap();

        let total = match &result[2].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        // Each channel should not exceed 1.0 (energy conservation)
        assert!(total.x <= 1.0);
        assert!(total.y <= 1.0);
        assert!(total.z <= 1.0);
    }

    #[test]
    fn test_brdf_grazing_angle() {
        // Grazing angle should show Fresnel effect
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0),  // normal
            (0.866, 0.5, 0.0), // view at 60° from normal
            (0.0, 1.0, 0.0),  // light aligned with normal
            (0.04, 0.04, 0.04),
            0.5,
            (0.8, 0.8, 0.8),
        );

        let result = Component::execute(inputs).unwrap();

        let specular = match &result[1].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        // At grazing angle, Fresnel effect increases specular
        assert!(specular.length() > 0.0);
    }

    #[test]
    fn test_brdf_perpendicular_light() {
        // Light perpendicular to surface should give zero BRDF
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0), // normal
            (0.0, 1.0, 0.0), // view
            (1.0, 0.0, 0.0), // light perpendicular
            (0.04, 0.04, 0.04),
            0.5,
            (0.8, 0.8, 0.8),
        );

        let result = Component::execute(inputs).unwrap();

        let total = match &result[2].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        // Perpendicular light should give zero contribution
        assert_eq!(total, Vec3::ZERO);
    }

    #[test]
    fn test_brdf_invalid_roughness() {
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.04, 0.04, 0.04),
            1.5, // Invalid
            (0.8, 0.8, 0.8),
        );

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_brdf_moderate_angle() {
        // Test with moderate viewing and lighting angles
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0),     // normal
            (0.0, 0.866, 0.5),   // view at ~30° from normal
            (-0.5, 0.866, 0.0),  // light at ~30° from normal, different azimuth
            (0.04, 0.04, 0.04),
            0.5,
            (0.6, 0.6, 0.6),
        );

        let result = Component::execute(inputs).unwrap();

        let diffuse = match &result[0].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        let specular = match &result[1].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        // Both diffuse and specular should contribute
        assert!(diffuse.length() > 0.0);
        assert!(specular.length() > 0.0);
    }

    #[test]
    fn test_brdf_colored_material() {
        // Test with colored base color
        let inputs = create_brdf_inputs(
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.04, 0.04, 0.04),
            0.5,
            (1.0, 0.2, 0.1), // Reddish
        );

        let result = Component::execute(inputs).unwrap();

        let diffuse = match &result[0].1 {
            Value::Vec3Val(v) => Vec3::new(v.x, v.y, v.z),
            _ => panic!("Expected Vec3Val"),
        };

        // Diffuse should preserve color ratios (red > green > blue)
        assert!(diffuse.x > diffuse.y);
        assert!(diffuse.y > diffuse.z);
    }
}
