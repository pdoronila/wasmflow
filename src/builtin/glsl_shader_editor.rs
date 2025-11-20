//! GLSL Shader Editor Node
//!
//! A built-in node that allows users to write and validate GLSL shaders
//! directly in the visual editor. Supports vertex, fragment, and compute shaders.

use crate::graph::node::{
    ComponentSpec, DataType, GraphNode, NodeValue, PortSpec, ShaderType,
    ShaderValidationState,
};
use crate::ui::code_editor::{CodeEditorWidget, CodeTheme};
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{Color32, RichText};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// GLSL Shader Editor Node
///
/// This node provides a code editor interface for writing GLSL shaders.
/// Users can select shader type (vertex/fragment/compute), write GLSL code,
/// and validate it in real-time.
pub struct GlslShaderEditorNode {
    /// Unique node identifier
    pub id: Uuid,

    /// User-specified shader name (e.g., "BasicPBR", "Skybox")
    pub shader_name: String,

    /// Shader type (Vertex, Fragment, Compute)
    pub shader_type: ShaderType,

    /// GLSL source code
    pub source_code: String,

    /// Whether to save code in graph file (default: true)
    pub save_code: bool,

    /// Current validation state
    pub validation_state: ShaderValidationState,

    /// Last validation error message (if validation failed)
    pub last_error: Option<String>,

    /// Code editor widget (not serialized, recreated on load)
    code_editor: CodeEditorWidget,

    /// Selected color theme for code editor
    pub editor_theme: CodeTheme,
}

impl GlslShaderEditorNode {
    /// Create a new GLSL Shader Editor Node with default values
    pub fn new() -> Self {
        let default_code = Self::default_template(ShaderType::Fragment);

        Self {
            id: Uuid::new_v4(),
            shader_name: String::new(),
            shader_type: ShaderType::Fragment,
            source_code: default_code,
            save_code: true,
            validation_state: ShaderValidationState::Idle,
            last_error: None,
            code_editor: CodeEditorWidget::new().with_rows(25),
            editor_theme: CodeTheme::default(),
        }
    }

    /// Get default template for a shader type
    pub fn default_template(shader_type: ShaderType) -> String {
        match shader_type {
            ShaderType::Vertex => Self::default_vertex_template(),
            ShaderType::Fragment => Self::default_fragment_template(),
            ShaderType::Compute => Self::default_compute_template(),
        }
    }

    /// Default vertex shader template
    fn default_vertex_template() -> String {
        r#"#version 450

// Vertex inputs
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

// Outputs to fragment shader
layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_uv;

// Uniforms
layout(set = 0, binding = 0) uniform Transforms {
    mat4 model;
    mat4 view;
    mat4 projection;
};

void main() {
    // Transform position to world space
    vec4 world_pos = model * vec4(position, 1.0);
    frag_position = world_pos.xyz;

    // Transform normal to world space
    frag_normal = mat3(model) * normal;

    // Pass through UV coordinates
    frag_uv = uv;

    // Transform to clip space
    gl_Position = projection * view * world_pos;
}
"#
        .to_string()
    }

    /// Default fragment shader template (Basic PBR)
    fn default_fragment_template() -> String {
        r#"#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;

// Output color
layout(location = 0) out vec4 out_color;

// Material uniforms
layout(set = 0, binding = 1) uniform Material {
    vec4 base_color;
    float metallic;
    float roughness;
    float ao;
};

// Light uniforms
layout(set = 0, binding = 2) uniform Light {
    vec3 light_position;
    vec3 light_color;
    float light_intensity;
};

// Camera uniforms
layout(set = 0, binding = 3) uniform Camera {
    vec3 camera_position;
};

// Constants
const float PI = 3.14159265359;

// Normal Distribution Function (GGX/Trowbridge-Reitz)
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

// Geometry function (Schlick-GGX)
float geometry_schlick_ggx(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float nom = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_ggx(NdotV, roughness);
    float ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

// Fresnel-Schlick approximation
vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

void main() {
    // Normalize vectors
    vec3 N = normalize(frag_normal);
    vec3 V = normalize(camera_position - frag_position);

    // Calculate F0 (reflectance at normal incidence)
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, base_color.rgb, metallic);

    // Light direction
    vec3 L = normalize(light_position - frag_position);
    vec3 H = normalize(V + L);

    // Calculate distance attenuation
    float distance = length(light_position - frag_position);
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance = light_color * light_intensity * attenuation;

    // Cook-Torrance BRDF
    float NDF = distribution_ggx(N, H, roughness);
    float G = geometry_smith(N, V, L, roughness);
    vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    vec3 specular = numerator / denominator;

    // Energy conservation
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    // Lambert diffuse
    float NdotL = max(dot(N, L), 0.0);
    vec3 Lo = (kD * base_color.rgb / PI + specular) * radiance * NdotL;

    // Ambient lighting (simplified)
    vec3 ambient = vec3(0.03) * base_color.rgb * ao;
    vec3 color = ambient + Lo;

    // HDR tonemapping (Reinhard)
    color = color / (color + vec3(1.0));

    // Gamma correction
    color = pow(color, vec3(1.0/2.2));

    out_color = vec4(color, base_color.a);
}
"#
        .to_string()
    }

    /// Default compute shader template
    fn default_compute_template() -> String {
        r#"#version 450

// Workgroup size
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

// Input texture
layout(set = 0, binding = 0, rgba8) uniform readonly image2D input_image;

// Output texture
layout(set = 0, binding = 1, rgba8) uniform writeonly image2D output_image;

// Example: Simple blur filter
void main() {
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
    ivec2 image_size = imageSize(input_image);

    // Bounds check
    if (pixel_coords.x >= image_size.x || pixel_coords.y >= image_size.y) {
        return;
    }

    // Simple box blur (3x3 kernel)
    vec4 color = vec4(0.0);
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            ivec2 sample_coords = pixel_coords + ivec2(x, y);
            sample_coords = clamp(sample_coords, ivec2(0), image_size - 1);
            color += imageLoad(input_image, sample_coords);
        }
    }
    color /= 9.0;

    // Write result
    imageStore(output_image, pixel_coords, color);
}
"#
        .to_string()
    }

    /// Get the component spec for this shader editor node
    pub fn spec() -> ComponentSpec {
        let mut spec = ComponentSpec::new_builtin(
            "builtin:graphics:glsl-shader-editor".to_string(),
            "GLSL Shader Editor".to_string(),
            "Write and validate GLSL shaders (vertex/fragment/compute)".to_string(),
            Some("Graphics".to_string()),
        );

        // No inputs - code is written in footer
        // Define outputs
        spec.output_spec = vec![
            PortSpec {
                name: "shader_source".to_string(),
                data_type: DataType::String,
                optional: false,
                description: "GLSL source code".to_string(),
            },
            PortSpec {
                name: "shader_type".to_string(),
                data_type: DataType::String,
                optional: false,
                description: "Shader type (vertex/fragment/compute)".to_string(),
            },
            PortSpec {
                name: "entry_point".to_string(),
                data_type: DataType::String,
                optional: false,
                description: "Entry point function name (usually 'main')".to_string(),
            },
        ];

        spec
    }

    /// Execute the shader editor node (returns shader source and metadata)
    pub fn execute(
        &self,
        _inputs: &HashMap<String, NodeValue>,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let mut outputs = HashMap::new();

        // Output the shader source code
        outputs.insert("shader_source".to_string(), NodeValue::String(self.source_code.clone()));

        // Output the shader type
        outputs.insert(
            "shader_type".to_string(),
            NodeValue::String(self.shader_type.as_str().to_string()),
        );

        // Output the entry point (always "main" for GLSL)
        outputs.insert("entry_point".to_string(), NodeValue::String("main".to_string()));

        Ok(outputs)
    }

    /// Validate the shader code (Phase 1: placeholder, Phase 2: use naga)
    pub fn validate_shader(&mut self) -> Result<(), String> {
        // Phase 1: Basic validation (check for non-empty code)
        if self.source_code.trim().is_empty() {
            self.validation_state = ShaderValidationState::Invalid;
            self.last_error = Some("Shader code is empty".to_string());
            return Err("Shader code is empty".to_string());
        }

        // Phase 1: Basic GLSL version check
        if !self.source_code.contains("#version") {
            self.validation_state = ShaderValidationState::Invalid;
            self.last_error = Some(
                "Missing #version directive (e.g., #version 450)".to_string(),
            );
            return Err("Missing #version directive".to_string());
        }

        // TODO Phase 2: Use naga for full GLSL → SPIR-V validation
        // For now, mark as valid if basic checks pass
        self.validation_state = ShaderValidationState::Valid;
        self.last_error = None;

        Ok(())
    }
}

impl Default for GlslShaderEditorNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Footer view for GLSL Shader Editor
pub struct GlslShaderEditorFooterView;

impl GlslShaderEditorFooterView {
    pub fn new() -> Arc<dyn ComponentFooterView> {
        Arc::new(Self)
    }
}

impl ComponentFooterView for GlslShaderEditorFooterView {
    fn render_footer(
        &self,
        ui: &mut egui::Ui,
        node: &mut GraphNode,
    ) -> Result<(), String> {
        // Get the shader editor data from the node
        if let Some(shader_data) = &mut node.shader_editor_data {
            // Shader name input
            ui.horizontal(|ui| {
                ui.label(RichText::new("Shader Name:").color(Color32::from_gray(180)));
                if ui
                    .text_edit_singleline(&mut shader_data.shader_name)
                    .changed()
                {
                    node.dirty = true;
                }
            });

            ui.add_space(8.0);

            // Shader type selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("Shader Type:").color(Color32::from_gray(180)));

                for shader_type in ShaderType::all() {
                    let is_selected = shader_data.shader_type == shader_type;
                    if ui
                        .selectable_label(is_selected, shader_type.as_str())
                        .clicked()
                    {
                        shader_data.shader_type = shader_type;
                        shader_data.source_code = GlslShaderEditorNode::default_template(shader_type);
                        node.dirty = true;
                    }
                }
            });

            ui.add_space(8.0);

            // Validation controls
            ui.horizontal(|ui| {
                if ui.button("🔍 Validate Shader").clicked() {
                    // Phase 1: Basic validation
                    if shader_data.source_code.trim().is_empty() {
                        shader_data.validation_state = ShaderValidationState::Invalid;
                        shader_data.last_error = Some("Shader code is empty".to_string());
                    } else if !shader_data.source_code.contains("#version") {
                        shader_data.validation_state = ShaderValidationState::Invalid;
                        shader_data.last_error = Some(
                            "Missing #version directive (e.g., #version 450)".to_string(),
                        );
                    } else {
                        // TODO Phase 2: Use naga for full GLSL → SPIR-V validation
                        shader_data.validation_state = ShaderValidationState::Valid;
                        shader_data.last_error = None;
                    }
                    node.dirty = true;
                }

                // Show validation state
                match shader_data.validation_state {
                    ShaderValidationState::Idle => {
                        ui.label(RichText::new("⏸ Not validated").color(Color32::from_gray(150)));
                    }
                    ShaderValidationState::Validating => {
                        ui.label(RichText::new("⏳ Validating...").color(Color32::from_rgb(255, 200, 0)));
                    }
                    ShaderValidationState::Valid => {
                        ui.label(RichText::new("✅ Valid").color(Color32::from_rgb(100, 255, 100)));
                    }
                    ShaderValidationState::Invalid => {
                        ui.label(RichText::new("❌ Invalid").color(Color32::from_rgb(255, 100, 100)));
                    }
                }
            });

            // Show error message if validation failed
            if let Some(error) = &shader_data.last_error {
                ui.add_space(4.0);
                ui.colored_label(Color32::from_rgb(255, 100, 100), format!("Error: {}", error));
            }

            ui.add_space(8.0);

            // Code editor
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.set_max_width(800.0);
                ui.set_max_height(400.0);

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        let editor = egui::TextEdit::multiline(&mut shader_data.source_code)
                            .code_editor()
                            .desired_rows(20)
                            .desired_width(f32::INFINITY);

                        if ui.add(editor).changed() {
                            node.dirty = true;
                            // Reset validation state when code changes
                            shader_data.validation_state = ShaderValidationState::Idle;
                            shader_data.last_error = None;
                        }
                    });
            });

            ui.add_space(8.0);

            // Save code checkbox
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut shader_data.save_code, "Save shader code in graph file")
                    .changed()
                {
                    node.dirty = true;
                }
                ui.label(RichText::new("(Disable for large shaders)").color(Color32::from_gray(120)));
            });

            Ok(())
        } else {
            Err("No shader editor data found for this node".to_string())
        }
    }
}

/// Register the GLSL Shader Editor node in the component registry
pub fn register_glsl_shader_editor_node(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = GlslShaderEditorNode::spec().with_footer_view(GlslShaderEditorFooterView::new());
    registry.register_builtin(spec);
    log::info!("Registered GLSL Shader Editor Node with footer view");
}
