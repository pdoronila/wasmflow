wit_bindgen::generate!({
    path: "wit",
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

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Texture Sampler".to_string(),
            version: "1.0.0".to_string(),
            description: "Sample texture at UV coordinates with bilinear filtering".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "uv".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "UV coordinates [u, v] in range [0, 1]".to_string(),
            },
            PortSpec {
                name: "texture_data".to_string(),
                data_type: DataType::BinaryType,
                optional: false,
                description: "Texture pixel data (RGBA8)".to_string(),
            },
            PortSpec {
                name: "width".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Texture width in pixels".to_string(),
            },
            PortSpec {
                name: "height".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Texture height in pixels".to_string(),
            },
            PortSpec {
                name: "wrap_mode".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "UV wrapping mode: repeat, clamp, mirror".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "color".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Sampled RGB color [r, g, b] (0.0-1.0)".to_string(),
            },
            PortSpec {
                name: "alpha".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Sampled alpha value (0.0-1.0)".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

/// Wrapping mode for UV coordinates
enum WrapMode {
    Repeat,
    Clamp,
    Mirror,
}

impl WrapMode {
    fn from_string(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "repeat" => Ok(WrapMode::Repeat),
            "clamp" => Ok(WrapMode::Clamp),
            "mirror" => Ok(WrapMode::Mirror),
            _ => Err(format!(
                "Invalid wrap mode '{}'. Expected: repeat, clamp, or mirror",
                s
            )),
        }
    }

    /// Apply wrapping to a coordinate
    fn wrap(&self, coord: f32) -> f32 {
        match self {
            WrapMode::Repeat => coord - coord.floor(),
            WrapMode::Clamp => coord.max(0.0).min(1.0),
            WrapMode::Mirror => {
                let frac = coord - coord.floor();
                let int_part = coord.floor() as i32;
                if int_part % 2 == 0 {
                    frac
                } else {
                    1.0 - frac
                }
            }
        }
    }
}

/// Sample a single texel from RGBA8 data
fn sample_texel(data: &[u8], x: u32, y: u32, width: u32) -> [f32; 4] {
    let index = ((y * width + x) * 4) as usize;
    [
        data[index] as f32 / 255.0,     // R
        data[index + 1] as f32 / 255.0, // G
        data[index + 2] as f32 / 255.0, // B
        data[index + 3] as f32 / 255.0, // A
    ]
}

/// Bilinear interpolation between 4 texels
fn bilinear_sample(
    data: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
    wrap_mode: &WrapMode,
) -> [f32; 4] {
    // Apply wrapping to UV coordinates
    let u_wrapped = wrap_mode.wrap(u);
    let v_wrapped = wrap_mode.wrap(v);

    // Convert to pixel coordinates
    let x = u_wrapped * (width - 1) as f32;
    let y = v_wrapped * (height - 1) as f32;

    // Get integer and fractional parts
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    // Sample 4 neighboring texels
    let c00 = sample_texel(data, x0, y0, width);
    let c10 = sample_texel(data, x1, y0, width);
    let c01 = sample_texel(data, x0, y1, width);
    let c11 = sample_texel(data, x1, y1, width);

    // Bilinear interpolation
    let mut result = [0.0; 4];
    for i in 0..4 {
        let c0 = c00[i] * (1.0 - fx) + c10[i] * fx;
        let c1 = c01[i] * (1.0 - fx) + c11[i] * fx;
        result[i] = c0 * (1.0 - fy) + c1 * fy;
    }

    result
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract UV coordinates
        let uv = inputs
            .iter()
            .find(|(name, _)| name == "uv")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: uv".to_string(),
                input_name: Some("uv".to_string()),
                recovery_hint: Some("Provide UV coordinates as [u, v]".to_string()),
            })?;

        let (u, v) = match &uv.1 {
            Value::F32ListVal(coords) if coords.len() == 2 => (coords[0], coords[1]),
            Value::F32ListVal(coords) => {
                return Err(ExecutionError {
                    message: format!("UV coordinates must have exactly 2 values, got {}", coords.len()),
                    input_name: Some("uv".to_string()),
                    recovery_hint: Some("Provide UV as [u, v] where both are in range [0, 1]".to_string()),
                });
            }
            _ => {
                return Err(ExecutionError {
                    message: "Expected F32List for UV coordinates".to_string(),
                    input_name: Some("uv".to_string()),
                    recovery_hint: Some("Provide UV as [u, v] list".to_string()),
                });
            }
        };

        // Extract texture data
        let texture_data = inputs
            .iter()
            .find(|(name, _)| name == "texture_data")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: texture_data".to_string(),
                input_name: Some("texture_data".to_string()),
                recovery_hint: Some("Connect texture data from texture-loader".to_string()),
            })?;

        let data = match &texture_data.1 {
            Value::BinaryVal(bytes) => bytes,
            _ => {
                return Err(ExecutionError {
                    message: "Expected Binary for texture_data".to_string(),
                    input_name: Some("texture_data".to_string()),
                    recovery_hint: Some("Connect texture data from texture-loader".to_string()),
                });
            }
        };

        // Extract width
        let width = inputs
            .iter()
            .find(|(name, _)| name == "width")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: width".to_string(),
                input_name: Some("width".to_string()),
                recovery_hint: Some("Provide texture width in pixels".to_string()),
            })?;

        let width = match &width.1 {
            Value::U32Val(w) => *w,
            _ => {
                return Err(ExecutionError {
                    message: "Expected U32 for width".to_string(),
                    input_name: Some("width".to_string()),
                    recovery_hint: Some("Provide width as unsigned integer".to_string()),
                });
            }
        };

        // Extract height
        let height = inputs
            .iter()
            .find(|(name, _)| name == "height")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: height".to_string(),
                input_name: Some("height".to_string()),
                recovery_hint: Some("Provide texture height in pixels".to_string()),
            })?;

        let height = match &height.1 {
            Value::U32Val(h) => *h,
            _ => {
                return Err(ExecutionError {
                    message: "Expected U32 for height".to_string(),
                    input_name: Some("height".to_string()),
                    recovery_hint: Some("Provide height as unsigned integer".to_string()),
                });
            }
        };

        // Extract wrap mode
        let wrap_mode_input = inputs
            .iter()
            .find(|(name, _)| name == "wrap_mode")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: wrap_mode".to_string(),
                input_name: Some("wrap_mode".to_string()),
                recovery_hint: Some("Provide wrap mode: repeat, clamp, or mirror".to_string()),
            })?;

        let wrap_mode_str = match &wrap_mode_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: "Expected String for wrap_mode".to_string(),
                    input_name: Some("wrap_mode".to_string()),
                    recovery_hint: Some("Provide wrap mode as string: repeat, clamp, or mirror".to_string()),
                });
            }
        };

        let wrap_mode = WrapMode::from_string(wrap_mode_str).map_err(|e| ExecutionError {
            message: e,
            input_name: Some("wrap_mode".to_string()),
            recovery_hint: Some("Use: repeat, clamp, or mirror".to_string()),
        })?;

        // Validate data size
        let expected_size = (width * height * 4) as usize;
        if data.len() != expected_size {
            return Err(ExecutionError {
                message: format!(
                    "Texture data size mismatch: expected {} bytes ({}x{}x4), got {}",
                    expected_size,
                    width,
                    height,
                    data.len()
                ),
                input_name: Some("texture_data".to_string()),
                recovery_hint: Some("Ensure texture data matches width and height".to_string()),
            });
        }

        // Perform bilinear sampling
        let sampled = bilinear_sample(data, u, v, width, height, &wrap_mode);

        // Output results
        let mut outputs = Vec::new();
        outputs.push((
            "color".to_string(),
            Value::F32ListVal(vec![sampled[0], sampled[1], sampled[2]]),
        ));
        outputs.push(("alpha".to_string(), Value::F32Val(sampled[3])));

        Ok(outputs)
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a simple 2x2 test texture
    fn create_test_texture() -> Vec<u8> {
        vec![
            255, 0, 0, 255, // Red (top-left)
            0, 255, 0, 255, // Green (top-right)
            0, 0, 255, 255, // Blue (bottom-left)
            255, 255, 0, 255, // Yellow (bottom-right)
        ]
    }

    #[test]
    fn test_wrap_mode_repeat() {
        let mode = WrapMode::Repeat;
        assert_eq!(mode.wrap(0.5), 0.5);
        assert_eq!(mode.wrap(1.5), 0.5);
        assert_eq!(mode.wrap(-0.5), 0.5);
    }

    #[test]
    fn test_wrap_mode_clamp() {
        let mode = WrapMode::Clamp;
        assert_eq!(mode.wrap(0.5), 0.5);
        assert_eq!(mode.wrap(1.5), 1.0);
        assert_eq!(mode.wrap(-0.5), 0.0);
    }

    #[test]
    fn test_wrap_mode_mirror() {
        let mode = WrapMode::Mirror;
        assert_eq!(mode.wrap(0.5), 0.5);
        assert_eq!(mode.wrap(1.5), 0.5);
        // Mirror flips on odd integer crossings
    }

    #[test]
    fn test_sample_texel() {
        let data = create_test_texture();
        let red = sample_texel(&data, 0, 0, 2);
        assert_eq!(red, [1.0, 0.0, 0.0, 1.0]);

        let green = sample_texel(&data, 1, 0, 2);
        assert_eq!(green, [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_bilinear_sample_exact() {
        let data = create_test_texture();
        let wrap_mode = WrapMode::Clamp;

        // Sample exact corner (should be pure red)
        let color = bilinear_sample(&data, 0.0, 0.0, 2, 2, &wrap_mode);
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_bilinear_sample_center() {
        let data = create_test_texture();
        let wrap_mode = WrapMode::Clamp;

        // Sample center (should be blend of all 4 colors)
        let color = bilinear_sample(&data, 0.5, 0.5, 2, 2, &wrap_mode);
        // Center should be average of R, G, B, Y = ~(0.5, 0.5, 0.25)
        assert!(color[0] > 0.4 && color[0] < 0.6); // R component
        assert!(color[1] > 0.4 && color[1] < 0.6); // G component
        assert!(color[2] > 0.2 && color[2] < 0.3); // B component
    }

    #[test]
    fn test_execute_basic() {
        let data = create_test_texture();
        let inputs = vec![
            ("uv".to_string(), Value::F32ListVal(vec![0.0, 0.0])),
            ("texture_data".to_string(), Value::BinaryVal(data)),
            ("width".to_string(), Value::U32Val(2)),
            ("height".to_string(), Value::U32Val(2)),
            ("wrap_mode".to_string(), Value::StringVal("clamp".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "color");
        assert_eq!(result[1].0, "alpha");
    }

    #[test]
    fn test_execute_missing_input() {
        let inputs = vec![
            ("texture_data".to_string(), Value::BinaryVal(vec![])),
            ("width".to_string(), Value::U32Val(2)),
            ("height".to_string(), Value::U32Val(2)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_invalid_wrap_mode() {
        let data = create_test_texture();
        let inputs = vec![
            ("uv".to_string(), Value::F32ListVal(vec![0.5, 0.5])),
            ("texture_data".to_string(), Value::BinaryVal(data)),
            ("width".to_string(), Value::U32Val(2)),
            ("height".to_string(), Value::U32Val(2)),
            ("wrap_mode".to_string(), Value::StringVal("invalid".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_data_size_mismatch() {
        let inputs = vec![
            ("uv".to_string(), Value::F32ListVal(vec![0.5, 0.5])),
            ("texture_data".to_string(), Value::BinaryVal(vec![1, 2, 3])), // Wrong size
            ("width".to_string(), Value::U32Val(2)),
            ("height".to_string(), Value::U32Val(2)),
            ("wrap_mode".to_string(), Value::StringVal("clamp".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
