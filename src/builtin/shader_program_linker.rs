//! Shader Program Linker Node
//!
//! A built-in node that links vertex and fragment shaders into an executable program.
//! Handles compilation, validation, and interface matching.

use crate::gpu::shader::{CompiledShader, ShaderStage};
use crate::graph::node::{ComponentSpec, DataType, PortSpec};
use crate::ui::component_view::ComponentFooterView;
use egui::{Color32, RichText};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Linked shader program (vertex + fragment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedProgram {
    pub id: Uuid,
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
    pub compilation_status: ProgramStatus,
    pub error_message: Option<String>,
}

/// Program linking status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProgramStatus {
    Idle,
    Compiling,
    Success,
    Failed,
}

impl LinkedProgram {
    /// Create a new empty linked program
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            vertex_shader_source: String::new(),
            fragment_shader_source: String::new(),
            compilation_status: ProgramStatus::Idle,
            error_message: None,
        }
    }

    /// Attempt to link vertex and fragment shaders
    ///
    /// This validates and compiles both shaders, checking for interface compatibility.
    pub fn link(
        &mut self,
        vertex_source: String,
        fragment_source: String,
        gpu_context: Option<&crate::gpu::context::GpuContext>,
    ) -> Result<(), String> {
        self.vertex_shader_source = vertex_source;
        self.fragment_shader_source = fragment_source;
        self.compilation_status = ProgramStatus::Compiling;
        self.error_message = None;

        // If no GPU context available, just store the sources
        let Some(ctx) = gpu_context else {
            self.compilation_status = ProgramStatus::Idle;
            return Err("GPU context not available".to_string());
        };

        // Compile vertex shader
        let vertex_result = CompiledShader::from_glsl(
            &ctx.device,
            &self.vertex_shader_source,
            ShaderStage::Vertex,
            None,
        );

        if let Err(e) = vertex_result {
            self.compilation_status = ProgramStatus::Failed;
            self.error_message = Some(format!("Vertex shader compilation failed: {}", e));
            return Err(self.error_message.clone().unwrap());
        }

        // Compile fragment shader
        let fragment_result = CompiledShader::from_glsl(
            &ctx.device,
            &self.fragment_shader_source,
            ShaderStage::Fragment,
            None,
        );

        if let Err(e) = fragment_result {
            self.compilation_status = ProgramStatus::Failed;
            self.error_message = Some(format!("Fragment shader compilation failed: {}", e));
            return Err(self.error_message.clone().unwrap());
        }

        // TODO: Validate interface matching (vertex outputs → fragment inputs)
        // For now, we just check that both compile successfully

        self.compilation_status = ProgramStatus::Success;
        self.id = Uuid::new_v4(); // Generate new ID on successful compilation
        log::info!("Shader program linked successfully: {}", self.id);

        Ok(())
    }

    /// Get status color for UI display
    pub fn status_color(&self) -> Color32 {
        match self.compilation_status {
            ProgramStatus::Idle => Color32::GRAY,
            ProgramStatus::Compiling => Color32::YELLOW,
            ProgramStatus::Success => Color32::from_rgb(0, 200, 0),
            ProgramStatus::Failed => Color32::from_rgb(200, 0, 0),
        }
    }

    /// Get status text for UI display
    pub fn status_text(&self) -> &str {
        match self.compilation_status {
            ProgramStatus::Idle => "Not compiled",
            ProgramStatus::Compiling => "Compiling...",
            ProgramStatus::Success => "✓ Linked successfully",
            ProgramStatus::Failed => "✗ Linking failed",
        }
    }
}

impl Default for LinkedProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// Shader Program Linker Footer View
pub struct ShaderProgramLinkerFooterView {}

impl ShaderProgramLinkerFooterView {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentFooterView for ShaderProgramLinkerFooterView {
    fn render_footer(
        &self,
        ui: &mut egui::Ui,
        node: &mut crate::graph::node::GraphNode,
    ) -> Result<(), String> {
        ui.scope(|ui| {
            ui.set_max_width(400.0);
            ui.set_max_height(300.0);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

            // Get linked program data
            let program = node
                .linked_program
                .as_ref()
                .ok_or("No linked program data")?;

            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());

                // Status indicator
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label(
                        RichText::new(program.status_text())
                            .color(program.status_color())
                            .strong(),
                    );
                });

                ui.add_space(8.0);

                // Program ID (if linked)
                if program.compilation_status == ProgramStatus::Success {
                    ui.horizontal(|ui| {
                        ui.label("Program ID:");
                        ui.monospace(program.id.to_string());
                    });
                    ui.add_space(4.0);
                }

                // Error message (if failed)
                if let Some(error) = &program.error_message {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.label(RichText::new("Error Details:").color(Color32::RED).strong());

                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.colored_label(Color32::LIGHT_RED, error);
                        });
                }

                // Shader sources info
                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("Shader Sources:").strong());

                ui.horizontal(|ui| {
                    ui.label("Vertex:");
                    let vertex_lines = program.vertex_shader_source.lines().count();
                    ui.monospace(format!("{} lines", vertex_lines));
                });

                ui.horizontal(|ui| {
                    ui.label("Fragment:");
                    let fragment_lines = program.fragment_shader_source.lines().count();
                    ui.monospace(format!("{} lines", fragment_lines));
                });

                // Compile button (if idle or failed)
                if program.compilation_status != ProgramStatus::Success {
                    ui.add_space(8.0);
                    if ui.button("🔗 Link Shaders").clicked() {
                        // TODO: Trigger compilation via app state
                        log::info!("Link button clicked - TODO: implement compilation trigger");
                    }
                }

                Ok::<(), String>(())
            })
            .inner
        })
        .inner
    }
}

/// Component specification for shader program linker
pub fn spec() -> ComponentSpec {
    let mut spec = ComponentSpec::new_builtin(
        "builtin:graphics:shader-program-linker".to_string(),
        "Shader Program Linker".to_string(),
        "Link vertex and fragment shaders into an executable program".to_string(),
        Some("Graphics".to_string()),
    );

    spec.input_spec = vec![
        PortSpec {
            name: "vertex_shader".to_string(),
            data_type: DataType::String,
            optional: false,
            description: "Vertex shader GLSL source code".to_string(),
        },
        PortSpec {
            name: "fragment_shader".to_string(),
            data_type: DataType::String,
            optional: false,
            description: "Fragment shader GLSL source code".to_string(),
        },
    ];

    spec.output_spec = vec![PortSpec {
        name: "program".to_string(),
        data_type: DataType::Binary,
        optional: false,
        description: "Linked shader program (binary program ID)".to_string(),
    }];

    spec
}

/// Register shader program linker node
pub fn register_shader_program_linker_node(
    registry: &mut crate::graph::node::ComponentRegistry,
) {
    let spec = spec().with_footer_view(Arc::new(ShaderProgramLinkerFooterView::new()));
    registry.register_builtin(spec);
    log::info!("Registered Shader Program Linker Node");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linked_program_creation() {
        let program = LinkedProgram::new();
        assert_eq!(program.compilation_status, ProgramStatus::Idle);
        assert!(program.vertex_shader_source.is_empty());
        assert!(program.fragment_shader_source.is_empty());
    }

    #[test]
    fn test_status_colors() {
        let mut program = LinkedProgram::new();

        program.compilation_status = ProgramStatus::Idle;
        assert_eq!(program.status_color(), Color32::GRAY);

        program.compilation_status = ProgramStatus::Success;
        assert_eq!(program.status_color(), Color32::from_rgb(0, 200, 0));

        program.compilation_status = ProgramStatus::Failed;
        assert_eq!(program.status_color(), Color32::from_rgb(200, 0, 0));
    }

    #[test]
    fn test_status_text() {
        let mut program = LinkedProgram::new();

        program.compilation_status = ProgramStatus::Idle;
        assert_eq!(program.status_text(), "Not compiled");

        program.compilation_status = ProgramStatus::Compiling;
        assert_eq!(program.status_text(), "Compiling...");

        program.compilation_status = ProgramStatus::Success;
        assert_eq!(program.status_text(), "✓ Linked successfully");

        program.compilation_status = ProgramStatus::Failed;
        assert_eq!(program.status_text(), "✗ Linking failed");
    }

    #[test]
    fn test_link_without_gpu_context() {
        let mut program = LinkedProgram::new();
        let result = program.link(
            "vertex source".to_string(),
            "fragment source".to_string(),
            None,
        );

        assert!(result.is_err());
        assert_eq!(program.compilation_status, ProgramStatus::Idle);
    }
}
