//! Template Generator for User-Defined WASM Components
//!
//! This module handles parsing user annotations and generating complete
//! WASM component source code from templates.

use crate::graph::node::{DataType, Language};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Port specification parsed from annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    pub name: String,
    pub data_type: DataType,
    pub optional: bool,
    pub description: String,
}

/// Component metadata parsed from user code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub inputs: Vec<PortSpec>,
    pub outputs: Vec<PortSpec>,
    pub capabilities: Vec<String>,
}

/// Template type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    Simple,  // Pure computation
    Http,    // Network-enabled
}

/// Annotation types from code comments
#[derive(Debug, Clone)]
enum Annotation {
    Input(PortSpec),
    Output(PortSpec),
    Description(String),
    Category(String),
    Capability(String),
}

/// Regular expression for annotation parsing (Rust/JavaScript: // @annotation)
fn annotation_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^//\s*@(\w+)\s+(.+)$").unwrap())
}

/// Regular expression for Python annotation parsing (# @annotation)
fn annotation_regex_python() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#\s*@(\w+)\s+(.+)$").unwrap())
}

/// Regular expression for port format: "name:Type description"
fn port_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\w+):(\w+)\s*(.*)$").unwrap())
}

/// Get annotation regex for the given language
fn annotation_regex_for_language(language: Language) -> &'static Regex {
    match language {
        Language::Rust | Language::JavaScript => annotation_regex(),
        Language::Python => annotation_regex_python(),
    }
}

/// Template Generator
pub struct TemplateGenerator;

impl TemplateGenerator {
    /// Strip annotation lines and template comments from user code, and add indentation
    fn strip_annotations_and_indent(code: &str, language: Language, indent_spaces: usize) -> String {
        let regex = annotation_regex_for_language(language);
        let indent = " ".repeat(indent_spaces);

        code.lines()
            .filter_map(|line| {
                let trimmed = line.trim();

                // Skip empty lines
                if trimmed.is_empty() {
                    return Some(String::new());
                }

                // Skip annotation lines
                if regex.is_match(trimmed) {
                    return None;
                }

                // Skip template comment lines
                if (language == Language::Python && (trimmed == "# Your code here" || trimmed == "# @description" || trimmed == "# @category"))
                    || ((language == Language::Rust || language == Language::JavaScript) &&
                        (trimmed == "// Your code here" || trimmed == "// @description" || trimmed == "// @category")) {
                    return None;
                }

                // Add indentation to non-empty lines
                Some(format!("{}{}", indent, trimmed))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse structured comments from user code (T006, T007)
    ///
    /// Format varies by language:
    /// - Rust/JavaScript: `// @input name:Type description`
    /// - Python: `# @input name:Type description`
    ///
    /// Supported annotations: @input, @output, @description, @category, @capability
    pub fn parse_annotations(
        component_name: &str,
        source_code: &str,
        language: Language,
    ) -> Result<ComponentMetadata, String> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut description = None;
        let mut category = None;
        let mut capabilities = Vec::new();

        for line in source_code.lines() {
            if let Some(ann) = Self::parse_annotation_line(line, language)? {
                match ann {
                    Annotation::Input(port) => inputs.push(port),
                    Annotation::Output(port) => outputs.push(port),
                    Annotation::Description(desc) => description = Some(desc),
                    Annotation::Category(cat) => category = Some(cat),
                    Annotation::Capability(cap) => capabilities.push(cap),
                }
            }
        }

        // Apply defaults if missing
        if inputs.is_empty() {
            inputs.push(PortSpec {
                name: "input".to_string(),
                data_type: DataType::F32,
                optional: false,
                description: "Input value".to_string(),
            });
        }

        if outputs.is_empty() {
            outputs.push(PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32,
                optional: false,
                description: "Output value".to_string(),
            });
        }

        Ok(ComponentMetadata {
            name: component_name.to_string(),
            description: description.unwrap_or_else(|| component_name.to_string()),
            category: category.unwrap_or_else(|| "User-Defined".to_string()),
            inputs,
            outputs,
            capabilities,
        })
    }

    /// Parse a single annotation line (T006)
    fn parse_annotation_line(line: &str, language: Language) -> Result<Option<Annotation>, String> {
        let regex = annotation_regex_for_language(language);
        let Some(caps) = regex.captures(line.trim()) else {
            return Ok(None);
        };

        let tag = caps.get(1).unwrap().as_str();
        let content = caps.get(2).unwrap().as_str();

        match tag {
            "input" => Ok(Some(Annotation::Input(Self::parse_port(content, false)?))),
            "output" => Ok(Some(Annotation::Output(Self::parse_port(content, false)?))),
            "description" => Ok(Some(Annotation::Description(content.to_string()))),
            "category" => Ok(Some(Annotation::Category(content.to_string()))),
            "capability" => Ok(Some(Annotation::Capability(Self::validate_capability(content)?))),
            _ => Ok(None), // Unknown annotation, ignore
        }
    }

    /// T075: Validate capability format
    ///
    /// Expected pattern: `(network|file-read|file-write):value`
    /// Examples:
    /// - `network:example.com`
    /// - `file-read:/data`
    /// - `file-write:/tmp`
    fn validate_capability(content: &str) -> Result<String, String> {
        let parts: Vec<&str> = content.splitn(2, ':').collect();

        if parts.len() != 2 {
            return Err(format!(
                "Invalid capability format: '{}'\n\
                Expected format: 'type:value'\n\
                Examples:\n\
                - // @capability network:example.com\n\
                - // @capability file-read:/data\n\
                - // @capability file-write:/tmp",
                content
            ));
        }

        let cap_type = parts[0].trim();
        let cap_value = parts[1].trim();

        // T075: Validate capability type
        match cap_type {
            "network" | "file-read" | "file-write" => {
                if cap_value.is_empty() {
                    return Err(format!(
                        "Capability '{}' requires a value\n\
                        Example: // @capability {}:example.com",
                        cap_type, cap_type
                    ));
                }
                Ok(content.to_string())
            }
            // Common mistakes
            "http" | "https" | "net" | "Network" => {
                Err(format!(
                    "Invalid capability type: '{}'\n\
                    Did you mean 'network'? (lowercase)\n\
                    Valid types: network, file-read, file-write\n\
                    Example: // @capability network:example.com",
                    cap_type
                ))
            }
            "file" | "File" | "read" | "write" => {
                Err(format!(
                    "Invalid capability type: '{}'\n\
                    Did you mean 'file-read' or 'file-write'?\n\
                    Valid types: network, file-read, file-write\n\
                    Examples:\n\
                    - // @capability file-read:/data\n\
                    - // @capability file-write:/tmp",
                    cap_type
                ))
            }
            _ => {
                Err(format!(
                    "Invalid capability type: '{}'\n\
                    Valid capability types:\n\
                    - network:domain (e.g., network:api.example.com)\n\
                    - file-read:path (e.g., file-read:/data)\n\
                    - file-write:path (e.g., file-write:/tmp)\n\
                    \n\
                    Note: Capabilities grant access to system resources.\n\
                    Use with caution and only when necessary.",
                    cap_type
                ))
            }
        }
    }

    /// Parse port specification: "name:Type description" (T007, T074, T076)
    fn parse_port(content: &str, optional: bool) -> Result<PortSpec, String> {
        let Some(caps) = port_regex().captures(content.trim()) else {
            return Err(format!(
                "Invalid port format: '{}'\n\
                Expected format: 'name:Type description'\n\
                Example: 'value:F32 The input number' or 'result:String The output text'",
                content
            ));
        };

        let name = caps.get(1).unwrap().as_str().to_string();
        let type_str = caps.get(2).unwrap().as_str();
        let description = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();

        // T076: Validate port names are valid Rust identifiers (snake_case recommended)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!(
                "Invalid port name: '{}'\n\
                Port names must be valid Rust identifiers (letters, numbers, underscores).\n\
                Recommended: use snake_case (e.g., 'input_value', 'result_text')",
                name
            ));
        }
        if name.chars().next().map_or(false, |c| c.is_numeric()) {
            return Err(format!(
                "Invalid port name: '{}'\n\
                Port names cannot start with a number. Try 'value_{}' or 'input_{}'",
                name, name, name
            ));
        }

        // T074: Enhanced type validation with helpful error messages
        let data_type = match type_str {
            "F32" => DataType::F32,
            "I32" => DataType::I32,
            "U32" => DataType::U32,
            "String" => DataType::String,
            "Boolean" | "Bool" => DataType::U32, // Boolean mapped to U32 for WIT
            // Common mistakes - provide helpful suggestions
            "f32" | "float" | "Float" => {
                return Err(format!(
                    "Invalid port type: '{}'\n\
                    Did you mean 'F32'? (uppercase)\n\
                    Valid types: F32, I32, U32, String, Boolean",
                    type_str
                ));
            }
            "i32" | "int" | "Int" | "Integer" => {
                return Err(format!(
                    "Invalid port type: '{}'\n\
                    Did you mean 'I32'? (uppercase)\n\
                    Valid types: F32, I32, U32, String, Boolean",
                    type_str
                ));
            }
            "u32" | "uint" | "UInt" | "Unsigned" => {
                return Err(format!(
                    "Invalid port type: '{}'\n\
                    Did you mean 'U32'? (uppercase)\n\
                    Valid types: F32, I32, U32, String, Boolean",
                    type_str
                ));
            }
            "string" | "str" | "Str" | "TEXT" | "Text" => {
                return Err(format!(
                    "Invalid port type: '{}'\n\
                    Did you mean 'String'? (capitalized)\n\
                    Valid types: F32, I32, U32, String, Boolean",
                    type_str
                ));
            }
            "boolean" | "bool" | "BOOL" | "BOOLEAN" => {
                return Err(format!(
                    "Invalid port type: '{}'\n\
                    Did you mean 'Boolean' or 'Bool'? (capitalized)\n\
                    Valid types: F32, I32, U32, String, Boolean",
                    type_str
                ));
            }
            _ => {
                return Err(format!(
                    "Invalid port type: '{}'\n\
                    Valid types are:\n\
                    - F32 (32-bit floating point)\n\
                    - I32 (32-bit signed integer)\n\
                    - U32 (32-bit unsigned integer)\n\
                    - String (text)\n\
                    - Boolean (true/false)\n\
                    Example: // @input value:F32 The input number",
                    type_str
                ));
            }
        };

        Ok(PortSpec {
            name,
            data_type,
            optional,
            description,
        })
    }

    /// Select template based on capabilities (T008)
    pub fn select_template(metadata: &ComponentMetadata) -> TemplateType {
        // If any network capability, use HTTP template
        if metadata.capabilities.iter().any(|cap| cap.starts_with("network:")) {
            TemplateType::Http
        } else {
            TemplateType::Simple
        }
    }

    /// Generate complete component code (T009)
    pub fn generate_component_code(
        metadata: &ComponentMetadata,
        user_code: &str,
        template_type: TemplateType,
        language: Language,
    ) -> String {
        let template = match (language, template_type) {
            (Language::Rust, TemplateType::Simple) => include_str!("../../templates/component_template.rs.tmpl"),
            (Language::Rust, TemplateType::Http) => include_str!("../../templates/http_component_template.rs.tmpl"),
            (Language::Python, _) => include_str!("../../templates/component_template.py.tmpl"),
            (Language::JavaScript, _) => include_str!("../../templates/component_template.js.tmpl"),
        };

        match language {
            Language::Rust => Self::generate_rust_code(template, metadata, user_code),
            Language::Python => Self::generate_python_code(template, metadata, user_code),
            Language::JavaScript => Self::generate_javascript_code(template, metadata, user_code),
        }
    }

    /// Generate Rust component code
    fn generate_rust_code(template: &str, metadata: &ComponentMetadata, user_code: &str) -> String {
        let cleaned_code = Self::strip_annotations_and_indent(user_code, Language::Rust, 8);
        template
            .replace("{{COMPONENT_NAME}}", &metadata.name)
            .replace("{{DESCRIPTION}}", &metadata.description)
            .replace("{{CATEGORY}}", &metadata.category)
            .replace("{{INPUTS}}", &Self::format_ports(&metadata.inputs))
            .replace("{{OUTPUTS}}", &Self::format_ports(&metadata.outputs))
            .replace("{{CAPABILITIES}}", &Self::format_capabilities(&metadata.capabilities))
            .replace("{{INPUT_EXTRACTION}}", &Self::generate_input_extraction(&metadata.inputs))
            .replace("{{USER_CODE}}", &cleaned_code)
            .replace("{{OUTPUT_CONSTRUCTION}}", &Self::generate_output_construction(&metadata.outputs))
    }

    /// Generate Python component code
    fn generate_python_code(template: &str, metadata: &ComponentMetadata, user_code: &str) -> String {
        let cleaned_code = Self::strip_annotations_and_indent(user_code, Language::Python, 8);
        template
            .replace("{{COMPONENT_NAME}}", &metadata.name)
            .replace("{{DESCRIPTION}}", &metadata.description)
            .replace("{{CATEGORY}}", &metadata.category)
            .replace("{{INPUTS}}", &Self::format_ports_python(&metadata.inputs))
            .replace("{{OUTPUTS}}", &Self::format_ports_python(&metadata.outputs))
            .replace("{{CAPABILITIES_PYTHON}}", &Self::format_capabilities_python(&metadata.capabilities))
            .replace("{{INPUT_EXTRACTION_PYTHON}}", &Self::generate_input_extraction_python(&metadata.inputs))
            .replace("{{USER_CODE}}", &cleaned_code)
            .replace("{{OUTPUT_CONSTRUCTION_PYTHON}}", &Self::generate_output_construction_python(&metadata.outputs))
    }

    /// Generate JavaScript component code
    fn generate_javascript_code(template: &str, metadata: &ComponentMetadata, user_code: &str) -> String {
        let cleaned_code = Self::strip_annotations_and_indent(user_code, Language::JavaScript, 8);
        template
            .replace("{{COMPONENT_NAME}}", &metadata.name)
            .replace("{{DESCRIPTION}}", &metadata.description)
            .replace("{{CATEGORY}}", &metadata.category)
            .replace("{{INPUTS_JS}}", &Self::format_ports_javascript(&metadata.inputs))
            .replace("{{OUTPUTS_JS}}", &Self::format_ports_javascript(&metadata.outputs))
            .replace("{{CAPABILITIES_JS}}", &Self::format_capabilities_javascript(&metadata.capabilities))
            .replace("{{INPUT_EXTRACTION_JS}}", &Self::generate_input_extraction_javascript(&metadata.inputs))
            .replace("{{USER_CODE}}", &cleaned_code)
            .replace("{{OUTPUT_CONSTRUCTION_JS}}", &Self::generate_output_construction_javascript(&metadata.outputs))
    }

    /// Format ports as Rust code
    fn format_ports(ports: &[PortSpec]) -> String {
        ports
            .iter()
            .map(|port| {
                let type_name = match port.data_type {
                    DataType::F32 => "F32Type",
                    DataType::I32 => "I32Type",
                    DataType::U32 => "U32Type",
                    DataType::String => "StringType",
                    _ => "F32Type", // Default fallback
                };

                format!(
                    r#"PortSpec {{
            name: "{}".to_string(),
            data_type: DataType::{},
            optional: {},
            description: "{}".to_string(),
        }}"#,
                    port.name, type_name, port.optional, port.description
                )
            })
            .collect::<Vec<_>>()
            .join(",\n        ")
    }

    /// Format capabilities as Rust code
    fn format_capabilities(capabilities: &[String]) -> String {
        if capabilities.is_empty() {
            "None".to_string()
        } else {
            let caps = capabilities
                .iter()
                .map(|cap| format!(r#""{}".to_string()"#, cap))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Some(vec![{}])", caps)
        }
    }

    /// Generate input extraction code
    fn generate_input_extraction(inputs: &[PortSpec]) -> String {
        inputs
            .iter()
            .map(|port| {
                let value_type = match port.data_type {
                    DataType::F32 => "F32Val(f) => Some(*f)",
                    DataType::I32 => "I32Val(i) => Some(*i)",
                    DataType::U32 => "U32Val(u) => Some(*u)",
                    DataType::String => "StringVal(s) => Some(s.clone())",
                    _ => "F32Val(f) => Some(*f)",
                };

                format!(
                    r#"let {} = inputs
            .iter()
            .find(|(name, _)| name == "{}")
            .and_then(|(_, val)| match val {{
                Value::{},
                _ => None,
            }})
            .ok_or_else(|| ExecutionError {{
                message: "Missing or invalid '{}' value".to_string(),
                input_name: Some("{}".to_string()),
                recovery_hint: Some("Connect a value to the {} port".to_string()),
            }})?;"#,
                    port.name, port.name, value_type, port.name, port.name, port.name
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n        ")
    }

    /// Generate output construction code
    fn generate_output_construction(outputs: &[PortSpec]) -> String {
        let output_items = outputs
            .iter()
            .map(|port| {
                let value_constructor = match port.data_type {
                    DataType::F32 => format!("Value::F32Val({})", port.name),
                    DataType::I32 => format!("Value::I32Val({})", port.name),
                    DataType::U32 => format!("Value::U32Val({})", port.name),
                    DataType::String => format!("Value::StringVal({}.clone())", port.name),
                    _ => format!("Value::F32Val({})", port.name),
                };

                format!(r#"("{}".to_string(), {})"#, port.name, value_constructor)
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!("Ok(vec![{}])", output_items)
    }

    /// Generate WIT interface (T010)
    pub fn generate_wit(_metadata: &ComponentMetadata) -> String {
        // Generate complete WIT file with all necessary interface definitions
        r#"// WasmFlow Component Interface
package wasmflow:node@1.0.0;

/// Data types supported by WasmFlow
interface types {
    variant data-type {
        u32-type,
        i32-type,
        f32-type,
        string-type,
        binary-type,
        list-type,
        any-type,
    }

    variant value {
        u32-val(u32),
        i32-val(s32),
        f32-val(f32),
        string-val(string),
        binary-val(list<u8>),
    }

    record port-spec {
        name: string,
        data-type: data-type,
        optional: bool,
        description: string,
    }

    record component-info {
        name: string,
        version: string,
        description: string,
        author: string,
        category: option<string>,
    }

    record execution-error {
        message: string,
        input-name: option<string>,
        recovery-hint: option<string>,
    }
}

/// Host functions provided by WasmFlow
interface host {
    log: func(level: string, message: string);
    get-temp-dir: func() -> result<string, string>;
}

/// Metadata interface
interface metadata {
    use types.{component-info, port-spec};

    get-info: func() -> component-info;
    get-inputs: func() -> list<port-spec>;
    get-outputs: func() -> list<port-spec>;
    get-capabilities: func() -> option<list<string>>;
}

/// Execution interface
interface execution {
    use types.{value, execution-error};

    execute: func(inputs: list<tuple<string, value>>) -> result<list<tuple<string, value>>, execution-error>;
}

/// Main component world
world component {
    import host;
    export metadata;
    export execution;
}
"#
        .to_string()
    }

    /// Generate Cargo.toml (T011)
    pub fn generate_cargo_toml(component_name: &str) -> String {
        let crate_name = component_name
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_ascii_lowercase().to_string()
                } else if c.is_ascii_uppercase() {
                    format!("_{}", c.to_ascii_lowercase())
                } else {
                    c.to_string()
                }
            })
            .collect::<String>();

        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

# Configure cargo-component to use wasm32-wasip2
[package.metadata.component]
package = "wasmflow:node"

[package.metadata.component.target]
path = "wit"

[dependencies]
# Component model bindings
cargo-component-bindings = "0.6"
wit-bindgen-rt = "0.44"

[profile.release]
opt-level = "s"
lto = true
strip = true
"#,
            crate_name
        )
    }

    // ==================== Python Code Generation ====================

    /// Format ports for Python
    fn format_ports_python(ports: &[PortSpec]) -> String {
        ports
            .iter()
            .map(|port| {
                let data_type_variant = match port.data_type {
                    DataType::F32 => "F32Type()",
                    DataType::I32 => "I32Type()",
                    DataType::U32 => "U32Type()",
                    DataType::String => "StringType()",
                    _ => "F32Type()",
                };

                // Python uses True/False, not true/false
                let optional_str = if port.optional { "True" } else { "False" };

                format!(
                    r#"PortSpec(
                name="{}",
                data_type={},
                optional={},
                description="{}"
            )"#,
                    port.name, data_type_variant, optional_str, port.description
                )
            })
            .collect::<Vec<_>>()
            .join(",\n            ")
    }

    /// Format capabilities for Python
    fn format_capabilities_python(capabilities: &[String]) -> String {
        if capabilities.is_empty() {
            "return None".to_string()
        } else {
            let caps = capabilities
                .iter()
                .map(|cap| format!(r#""{}""#, cap))
                .collect::<Vec<_>>()
                .join(", ");
            format!("return [{}]", caps)
        }
    }

    /// Generate input extraction for Python
    fn generate_input_extraction_python(inputs: &[PortSpec]) -> String {
        inputs
            .iter()
            .map(|port| {
                let value_variant = match port.data_type {
                    DataType::F32 => "F32Val",
                    DataType::I32 => "I32Val",
                    DataType::U32 => "U32Val",
                    DataType::String => "StringVal",
                    _ => "F32Val",
                };

                format!(
                    r#"# Extract {}
        {} = None
        for name, val in inputs:
            if name == "{}":
                if isinstance(val, {}):
                    {} = val.value
                    break
        if {} is None:
            raise Exception("Missing '{}' input")"#,
                    port.name,
                    port.name,
                    port.name,
                    value_variant,
                    port.name,
                    port.name,
                    port.name
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n        ")
    }

    /// Generate output construction for Python
    fn generate_output_construction_python(outputs: &[PortSpec]) -> String {
        let output_items = outputs
            .iter()
            .map(|port| {
                let value_variant = match port.data_type {
                    DataType::F32 => format!("F32Val(value={})", port.name),
                    DataType::I32 => format!("I32Val(value={})", port.name),
                    DataType::U32 => format!("U32Val(value={})", port.name),
                    DataType::String => format!("StringVal(value={})", port.name),
                    _ => format!("F32Val(value={})", port.name),
                };

                format!(r#"("{}", {})"#, port.name, value_variant)
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!("return [{}]", output_items)
    }

    // ==================== JavaScript Code Generation ====================

    /// Format ports for JavaScript
    fn format_ports_javascript(ports: &[PortSpec]) -> String {
        ports
            .iter()
            .map(|port| {
                let type_tag = match port.data_type {
                    DataType::F32 => "f32-type",
                    DataType::I32 => "i32-type",
                    DataType::U32 => "u32-type",
                    DataType::String => "string-type",
                    _ => "f32-type",
                };

                format!(
                    r#"{{
            name: "{}",
            dataType: {{ tag: "{}" }},
            optional: {},
            description: "{}"
        }}"#,
                    port.name, type_tag, port.optional, port.description
                )
            })
            .collect::<Vec<_>>()
            .join(",\n        ")
    }

    /// Format capabilities for JavaScript
    fn format_capabilities_javascript(capabilities: &[String]) -> String {
        if capabilities.is_empty() {
            "return null;".to_string()
        } else {
            let caps = capabilities
                .iter()
                .map(|cap| format!(r#""{}""#, cap))
                .collect::<Vec<_>>()
                .join(", ");
            format!("return [{}];", caps)
        }
    }

    /// Generate input extraction for JavaScript
    fn generate_input_extraction_javascript(inputs: &[PortSpec]) -> String {
        inputs
            .iter()
            .map(|port| {
                let value_tag = match port.data_type {
                    DataType::F32 => "f32-val",
                    DataType::I32 => "i32-val",
                    DataType::U32 => "u32-val",
                    DataType::String => "string-val",
                    _ => "f32-val",
                };

                format!(
                    r#"// Extract {}
const {} = (() => {{
    const input = inputs.find(([name, _]) => name === "{}");
    if (!input || input[1].tag !== "{}") {{
        throw {{
            message: "Missing or invalid '{}' value",
            inputName: "{}",
            recoveryHint: "Connect a value to the {} port"
        }};
    }}
    return input[1].val;
}})();"#,
                    port.name,
                    port.name,
                    port.name,
                    value_tag,
                    port.name,
                    port.name,
                    port.name
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Generate output construction for JavaScript
    fn generate_output_construction_javascript(outputs: &[PortSpec]) -> String {
        let output_items = outputs
            .iter()
            .map(|port| {
                let value_constructor = match port.data_type {
                    DataType::F32 => format!("{{ tag: \"f32-val\", val: {} }}", port.name),
                    DataType::I32 => format!("{{ tag: \"i32-val\", val: {} }}", port.name),
                    DataType::U32 => format!("{{ tag: \"u32-val\", val: {} }}", port.name),
                    DataType::String => format!("{{ tag: \"string-val\", val: {} }}", port.name),
                    _ => format!("{{ tag: \"f32-val\", val: {} }}", port.name),
                };

                format!(r#"["{}", {}]"#, port.name, value_constructor)
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!("return [{}];", output_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_annotation() {
        let line = "// @input value:F32 The input number";
        let result = TemplateGenerator::parse_annotation_line(line, Language::Rust).unwrap();

        assert!(matches!(result, Some(Annotation::Input(_))));
    }

    #[test]
    fn test_parse_annotations_with_defaults() {
        let code = r#"
// No annotations, should use defaults
let result = value * 3.0;
"#;
        let metadata = TemplateGenerator::parse_annotations("TestComponent", code, Language::Rust).unwrap();

        assert_eq!(metadata.inputs.len(), 1);
        assert_eq!(metadata.outputs.len(), 1);
        assert_eq!(metadata.inputs[0].name, "input");
        assert_eq!(metadata.outputs[0].name, "output");
    }

    #[test]
    fn test_template_selection() {
        let mut metadata = ComponentMetadata {
            name: "Test".to_string(),
            description: "Test".to_string(),
            category: "Test".to_string(),
            inputs: vec![],
            outputs: vec![],
            capabilities: vec![],
        };

        assert_eq!(TemplateGenerator::select_template(&metadata), TemplateType::Simple);

        metadata.capabilities.push("network:api.example.com".to_string());
        assert_eq!(TemplateGenerator::select_template(&metadata), TemplateType::Http);
    }
}
