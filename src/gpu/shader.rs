//! Shader Compilation System
//!
//! Handles GLSL → SPIR-V compilation using naga for WebGPU.

use thiserror::Error;
use uuid::Uuid;

/// Shader stage (vertex or fragment)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    /// Convert to naga shader stage
    fn to_naga_stage(&self) -> naga::ShaderStage {
        match self {
            ShaderStage::Vertex => naga::ShaderStage::Vertex,
            ShaderStage::Fragment => naga::ShaderStage::Fragment,
        }
    }

    /// Get default entry point name
    pub fn default_entry_point(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "main",
            ShaderStage::Fragment => "main",
        }
    }
}

/// Compiled shader module ready for GPU execution
pub struct CompiledShader {
    pub id: Uuid,
    pub source: String,
    pub module: wgpu::ShaderModule,
    pub stage: ShaderStage,
    pub entry_point: String,
}

/// Shader compilation errors
#[derive(Error, Debug)]
pub enum ShaderCompilationError {
    #[error("GLSL parsing failed: {0}")]
    ParseError(String),

    #[error("Shader validation failed: {0}")]
    ValidationError(String),

    #[error("SPIR-V generation failed: {0}")]
    SpirVGenerationError(String),

    #[error("Invalid shader stage for source code")]
    InvalidStage,

    #[error("Entry point '{0}' not found in shader")]
    EntryPointNotFound(String),
}

impl CompiledShader {
    /// Compile GLSL source to SPIR-V and create wgpu shader module
    ///
    /// # Arguments
    /// * `device` - WebGPU device to create module on
    /// * `source` - GLSL source code
    /// * `stage` - Shader stage (vertex or fragment)
    /// * `entry_point` - Entry point function name (usually "main")
    ///
    /// # Returns
    /// Compiled shader ready for use in render pipelines
    pub fn from_glsl(
        device: &wgpu::Device,
        source: &str,
        stage: ShaderStage,
        entry_point: Option<&str>,
    ) -> Result<Self, ShaderCompilationError> {
        let entry_point = entry_point.unwrap_or_else(|| stage.default_entry_point());

        log::info!("Compiling {:?} shader (entry: {})", stage, entry_point);
        log::debug!("Source:\n{}", source);

        // Step 1: Parse GLSL using naga
        let module = Self::parse_glsl(source, stage)?;

        // Step 2: Validate shader
        Self::validate_shader(&module)?;

        // Step 3: Generate WGSL from naga module
        let wgsl = Self::generate_wgsl(&module)?;

        // Step 4: Create wgpu shader module from WGSL
        let wgpu_module = Self::create_wgpu_module(device, &wgsl, stage)?;

        log::info!("Shader compiled successfully");

        Ok(CompiledShader {
            id: Uuid::new_v4(),
            source: source.to_string(),
            module: wgpu_module,
            stage,
            entry_point: entry_point.to_string(),
        })
    }

    /// Parse GLSL source code into naga IR
    fn parse_glsl(
        source: &str,
        stage: ShaderStage,
    ) -> Result<naga::Module, ShaderCompilationError> {
        let mut parser = naga::front::glsl::Frontend::default();

        let options = naga::front::glsl::Options {
            stage: stage.to_naga_stage(),
            defines: Default::default(),
        };

        parser
            .parse(&options, source)
            .map_err(|errors| {
                // ParseErrors doesn't implement Iterator in naga 22.x
                let error_message = format!("{:?}", errors);
                ShaderCompilationError::ParseError(error_message)
            })
    }

    /// Validate shader module
    fn validate_shader(module: &naga::Module) -> Result<(), ShaderCompilationError> {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(module)
            .map(|_| ())
            .map_err(|e| ShaderCompilationError::ValidationError(format!("{}", e)))
    }

    /// Generate WGSL source from validated naga module
    fn generate_wgsl(module: &naga::Module) -> Result<String, ShaderCompilationError> {
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(module)
        .map_err(|e| ShaderCompilationError::ValidationError(format!("{}", e)))?;

        let wgsl = naga::back::wgsl::write_string(
            module,
            &info,
            naga::back::wgsl::WriterFlags::empty(),
        )
        .map_err(|e| ShaderCompilationError::SpirVGenerationError(format!("{}", e)))?;

        Ok(wgsl)
    }

    /// Create wgpu shader module from WGSL source
    fn create_wgpu_module(
        device: &wgpu::Device,
        wgsl: &str,
        stage: ShaderStage,
    ) -> Result<wgpu::ShaderModule, ShaderCompilationError> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{:?} Shader", stage)),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(wgsl)),
        });

        Ok(module)
    }

    /// Get shader statistics for display
    pub fn stats(&self) -> String {
        format!(
            "Stage: {:?}\nEntry Point: {}\nSource Lines: {}",
            self.stage,
            self.entry_point,
            self.source.lines().count()
        )
    }
}

/// Shader compilation result with detailed error information
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub errors: Vec<CompilationError>,
    pub warnings: Vec<String>,
    pub shader_id: Option<Uuid>,
}

/// Detailed compilation error with location information
#[derive(Debug, Clone)]
pub struct CompilationError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl CompilationResult {
    /// Create success result
    pub fn success(shader_id: Uuid) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            shader_id: Some(shader_id),
        }
    }

    /// Create failure result from error
    pub fn failure(error: ShaderCompilationError) -> Self {
        Self {
            success: false,
            errors: vec![CompilationError {
                message: error.to_string(),
                line: None,
                column: None,
                severity: ErrorSeverity::Error,
            }],
            warnings: Vec::new(),
            shader_id: None,
        }
    }

    /// Format errors for display in UI
    pub fn format_errors(&self) -> String {
        if self.success {
            return "Compilation successful".to_string();
        }

        let mut output = String::new();
        for error in &self.errors {
            output.push_str(&format!("Error: {}\n", error.message));
            if let Some(line) = error.line {
                output.push_str(&format!("  at line {}\n", line));
            }
        }

        for warning in &self.warnings {
            output.push_str(&format!("Warning: {}\n", warning));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERTEX_SHADER: &str = r#"
        #version 450

        layout(location = 0) in vec3 position;
        layout(location = 1) in vec3 normal;

        layout(set = 0, binding = 0) uniform Uniforms {
            mat4 modelViewProj;
        };

        layout(location = 0) out vec3 fragNormal;

        void main() {
            gl_Position = modelViewProj * vec4(position, 1.0);
            fragNormal = normal;
        }
    "#;

    const FRAGMENT_SHADER: &str = r#"
        #version 450

        layout(location = 0) in vec3 fragNormal;
        layout(location = 0) out vec4 outColor;

        void main() {
            vec3 normal = normalize(fragNormal);
            float lighting = max(dot(normal, vec3(0.0, 0.0, 1.0)), 0.2);
            outColor = vec4(vec3(lighting), 1.0);
        }
    "#;

    #[test]
    fn test_shader_stage_conversion() {
        assert_eq!(
            ShaderStage::Vertex.to_naga_stage(),
            naga::ShaderStage::Vertex
        );
        assert_eq!(
            ShaderStage::Fragment.to_naga_stage(),
            naga::ShaderStage::Fragment
        );
    }

    #[test]
    fn test_parse_vertex_shader() {
        let result = CompiledShader::parse_glsl(VERTEX_SHADER, ShaderStage::Vertex);
        assert!(result.is_ok(), "Failed to parse vertex shader: {:?}", result.err());
    }

    #[test]
    fn test_parse_fragment_shader() {
        let result = CompiledShader::parse_glsl(FRAGMENT_SHADER, ShaderStage::Fragment);
        assert!(result.is_ok(), "Failed to parse fragment shader: {:?}", result.err());
    }

    #[test]
    fn test_validate_shader() {
        let module = CompiledShader::parse_glsl(VERTEX_SHADER, ShaderStage::Vertex).unwrap();
        let result = CompiledShader::validate_shader(&module);
        assert!(result.is_ok(), "Shader validation failed: {:?}", result.err());
    }

    #[test]
    fn test_generate_wgsl() {
        let module = CompiledShader::parse_glsl(VERTEX_SHADER, ShaderStage::Vertex).unwrap();
        let result = CompiledShader::generate_wgsl(&module);
        assert!(result.is_ok(), "WGSL generation failed: {:?}", result.err());
        assert!(!result.unwrap().is_empty(), "WGSL output is empty");
    }

    #[test]
    fn test_invalid_glsl() {
        let invalid_shader = "not valid glsl code";
        let result = CompiledShader::parse_glsl(invalid_shader, ShaderStage::Vertex);
        assert!(result.is_err(), "Should fail to parse invalid GLSL");
    }

    #[test]
    fn test_compilation_result_success() {
        let id = Uuid::new_v4();
        let result = CompilationResult::success(id);
        assert!(result.success);
        assert_eq!(result.shader_id, Some(id));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_compilation_result_failure() {
        let error = ShaderCompilationError::ParseError("test error".to_string());
        let result = CompilationResult::failure(error);
        assert!(!result.success);
        assert!(result.shader_id.is_none());
        assert!(!result.errors.is_empty());
    }
}
