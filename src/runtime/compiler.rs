//! Component Compiler - Invokes language-specific compilers to build WASM components
//!
//! This module handles workspace creation, compilation, and cleanup for
//! user-defined WASM components in Rust, Python, and JavaScript.

use crate::graph::node::Language;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Configuration for component compilation
pub struct CompilationConfig {
    /// Component name (PascalCase)
    pub component_name: String,
    /// Generated source code (language-specific)
    pub source_code: String,
    /// WIT interface definition
    pub wit_definition: String,
    /// Build configuration (Cargo.toml for Rust, package.json for JS, etc.)
    pub cargo_toml: String,
    /// Maximum build time before timeout
    pub timeout: Duration,
    /// Programming language
    pub language: Language,
}

/// Result of compilation attempt
#[derive(Debug)]
pub enum CompilationResult {
    Success {
        /// Path to compiled .wasm file
        wasm_path: PathBuf,
        /// Build duration in milliseconds
        build_time_ms: u64,
        /// Compiler output (stdout)
        output: String,
    },
    Failure {
        /// Error message from compiler
        error_message: String,
        /// Line number if available
        line_number: Option<usize>,
        /// Full compiler output (stderr)
        stderr: String,
    },
    Timeout {
        /// How long it ran before timeout
        elapsed: Duration,
    },
}

/// Main compilation service (T012)
pub struct ComponentCompiler {
    /// Base directory for build workspaces
    workspace_root: PathBuf,
}

impl ComponentCompiler {
    /// Create new compiler with specified workspace root (T012)
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Create new compiler with default temp directory
    pub fn with_default_workspace() -> Self {
        let root = std::env::temp_dir().join("wasmflow-builds");
        Self::new(root)
    }

    /// Compile a component from config (T013, T014, T015, T016)
    pub fn compile(&self, config: CompilationConfig) -> Result<CompilationResult, String> {
        let start = Instant::now();

        // Create workspace
        let workspace = self.create_workspace(&config)?;

        // Dispatch to language-specific compiler
        let result = match config.language {
            Language::Rust => {
                self.invoke_cargo(&workspace, &config.component_name, config.timeout)?
            }
            Language::Python => {
                self.invoke_componentize_py(&workspace, &config.component_name, config.timeout)?
            }
            Language::JavaScript => {
                self.invoke_componentize_js(&workspace, &config.component_name, config.timeout)?
            }
        };

        // If compilation succeeded, copy the wasm file to a persistent location before cleanup
        let result = match result {
            CompilationResult::Success { wasm_path, output, .. } => {
                // Copy to persistent temp directory before workspace cleanup
                let persistent_temp = std::env::temp_dir().join("wasmflow-compiled");
                if let Err(e) = fs::create_dir_all(&persistent_temp) {
                    log::warn!("Failed to create persistent temp directory: {}", e);
                }

                let persistent_path = persistent_temp.join(
                    wasm_path.file_name().ok_or("Invalid wasm path")?
                );

                match fs::copy(&wasm_path, &persistent_path) {
                    Ok(_) => {
                        log::debug!("Copied wasm to persistent location: {}", persistent_path.display());
                        CompilationResult::Success {
                            wasm_path: persistent_path,
                            build_time_ms: 0, // Will be set below
                            output,
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to copy wasm to persistent location: {}", e);
                        // Fall back to original path (will likely fail later, but better than crashing)
                        CompilationResult::Success {
                            wasm_path,
                            build_time_ms: 0,
                            output,
                        }
                    }
                }
            }
            other => other,
        };

        // Log workspace path on failure for debugging
        if matches!(result, CompilationResult::Failure { .. }) {
            // Copy generated source to persistent debug location
            let debug_dir = std::env::temp_dir().join("wasmflow-debug");
            let _ = fs::create_dir_all(&debug_dir);

            let source_file = match config.language {
                Language::Rust => workspace.join("src/lib.rs"),
                Language::Python => workspace.join("app.py"),
                Language::JavaScript => workspace.join("app.js"),
            };

            let debug_file = debug_dir.join(format!(
                "failed_{}.{}",
                config.component_name,
                config.language.file_extension()
            ));

            if let Err(e) = fs::copy(&source_file, &debug_file) {
                log::error!("Failed to copy source for debugging: {}", e);
            } else {
                log::error!("Generated source saved to: {}", debug_file.display());
            }

            log::error!("Compilation failed. Workspace was at: {}", workspace.display());
            log::error!("Inspect the generated files to debug the issue.");
        }

        // Always cleanup workspace
        if let Err(e) = self.cleanup_workspace(&workspace) {
            log::warn!("Failed to cleanup workspace: {}", e);
        }

        // T080: Add build time to result and log slow compilations
        match result {
            CompilationResult::Success {
                wasm_path,
                output,
                ..
            } => {
                let build_time_ms = start.elapsed().as_millis() as u64;

                // T080: Log slow compilations (>30s)
                if build_time_ms > 30_000 {
                    log::warn!(
                        "Slow compilation detected for '{}': {}ms (>30s). \
                        Consider simplifying code or checking system resources.",
                        config.component_name,
                        build_time_ms
                    );
                } else {
                    log::info!(
                        "Compilation completed for '{}' in {}ms",
                        config.component_name,
                        build_time_ms
                    );
                }

                Ok(CompilationResult::Success {
                    wasm_path,
                    build_time_ms,
                    output,
                })
            }
            other => Ok(other),
        }
    }

    /// Create workspace directory with all necessary files (T013)
    fn create_workspace(&self, config: &CompilationConfig) -> Result<PathBuf, String> {
        // Create unique workspace directory
        let workspace_id = Uuid::new_v4();
        let workspace = self
            .workspace_root
            .join(format!("wasmflow-build-{}", workspace_id));

        fs::create_dir_all(&workspace)
            .map_err(|e| format!("Failed to create workspace directory: {}", e))?;

        // Create language-specific files
        match config.language {
            Language::Rust => {
                // Create Cargo.toml
                let cargo_path = workspace.join("Cargo.toml");
                fs::write(&cargo_path, &config.cargo_toml)
                    .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

                // Create src directory and lib.rs
                let src_dir = workspace.join("src");
                fs::create_dir_all(&src_dir)
                    .map_err(|e| format!("Failed to create src directory: {}", e))?;

                let lib_path = src_dir.join("lib.rs");
                fs::write(&lib_path, &config.source_code)
                    .map_err(|e| format!("Failed to write lib.rs: {}", e))?;
            }
            Language::Python => {
                // Create app.py (main Python file)
                let app_path = workspace.join("app.py");
                fs::write(&app_path, &config.source_code)
                    .map_err(|e| format!("Failed to write app.py: {}", e))?;
            }
            Language::JavaScript => {
                // Create app.js (main JavaScript file)
                let app_path = workspace.join("app.js");
                fs::write(&app_path, &config.source_code)
                    .map_err(|e| format!("Failed to write app.js: {}", e))?;
            }
        }

        // Create wit directory and world.wit (needed for all languages)
        let wit_dir = workspace.join("wit");
        fs::create_dir_all(&wit_dir)
            .map_err(|e| format!("Failed to create wit directory: {}", e))?;

        let wit_path = wit_dir.join("world.wit");
        fs::write(&wit_path, &config.wit_definition)
            .map_err(|e| format!("Failed to write world.wit: {}", e))?;

        Ok(workspace)
    }

    /// Invoke cargo-component with timeout monitoring (T014, T015)
    fn invoke_cargo(
        &self,
        workspace: &Path,
        component_name: &str,
        timeout: Duration,
    ) -> Result<CompilationResult, String> {
        // T079: Check if cargo-component is available with helpful error message
        let mut child = Command::new("cargo")
            .args(&["component", "build", "--release"])
            .current_dir(workspace)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    format!(
                        "cargo-component not found or not in PATH\n\
                        \n\
                        To compile custom WASM components, you need cargo-component installed.\n\
                        \n\
                        Installation instructions:\n\
                        1. Install cargo-component:\n\
                           cargo install cargo-component\n\
                        \n\
                        2. Install wasm32-wasip2 target:\n\
                           rustup target add wasm32-wasip2\n\
                        \n\
                        3. Restart the application\n\
                        \n\
                        For more information, visit:\n\
                        https://github.com/bytecodealliance/cargo-component\n\
                        \n\
                        Original error: {}", e
                    )
                } else {
                    format!("Failed to spawn cargo-component: {}", e)
                }
            })?;

        let start = Instant::now();

        // Poll for completion with timeout
        loop {
            if start.elapsed() > timeout {
                // Kill the process
                let _ = child.kill();
                return Ok(CompilationResult::Timeout {
                    elapsed: start.elapsed(),
                });
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process completed
                    let output = child
                        .wait_with_output()
                        .map_err(|e| format!("Failed to read cargo output: {}", e))?;

                    if status.success() {
                        // Find the compiled .wasm file
                        // Try multiple possible locations
                        let crate_name = Self::component_name_to_crate_name(component_name);
                        let possible_paths = vec![
                            workspace.join("target/wasm32-wasip2/release").join(format!("{}.wasm", crate_name)),
                            workspace.join("target/wasm32-wasi/release").join(format!("{}.wasm", crate_name)),
                            workspace.join("target/release").join(format!("{}.wasm", crate_name)),
                        ];

                        let wasm_path = possible_paths.iter()
                            .find(|p| p.exists())
                            .cloned();

                        let wasm_path = match wasm_path {
                            Some(path) => path,
                            None => {
                                // Search the entire target directory for .wasm files
                                let target_dir = workspace.join("target");
                                let stderr_output = String::from_utf8_lossy(&output.stderr).to_string();

                                if let Ok(found_wasm) = Self::find_wasm_file(&target_dir, &crate_name) {
                                    found_wasm
                                } else {
                                    return Ok(CompilationResult::Failure {
                                        error_message: format!(
                                            "Compilation succeeded but .wasm file not found.\n\
                                            Expected one of:\n\
                                            - target/wasm32-wasip2/release/{}.wasm\n\
                                            - target/wasm32-wasi/release/{}.wasm\n\
                                            - target/release/{}.wasm\n\
                                            \n\
                                            Build output:\n{}",
                                            crate_name, crate_name, crate_name, stderr_output
                                        ),
                                        line_number: None,
                                        stderr: stderr_output,
                                    });
                                }
                            }
                        };

                        return Ok(CompilationResult::Success {
                            wasm_path,
                            build_time_ms: 0, // Will be set by caller
                            output: String::from_utf8_lossy(&output.stdout).to_string(),
                        });
                    } else {
                        // Compilation failed - parse error
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let (error_message, line_number) = Self::parse_cargo_error(&stderr);

                        return Ok(CompilationResult::Failure {
                            error_message,
                            line_number,
                            stderr,
                        });
                    }
                }
                Ok(None) => {
                    // Still running, wait a bit
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(format!("Failed to wait for cargo process: {}", e)),
            }
        }
    }

    /// Parse cargo error output to extract message and line number (T016, T045)
    ///
    /// Parses cargo's output to extract:
    /// - Error message
    /// - Line number where error occurred
    /// - Optional error code (e.g., E0308)
    ///
    /// Handles both plain text and JSON error formats (legacy support).
    fn parse_cargo_error(stderr: &str) -> (String, Option<usize>) {
        let mut error_message = None;
        let mut line_number = None;

        // Look for common error patterns
        for line in stderr.lines() {
            // Try to parse JSON message format (T045)
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                // Only process compiler messages
                if json["reason"] == "compiler-message" {
                    // Only process errors, not warnings
                    if let Some(level) = json["message"]["level"].as_str() {
                        if level != "error" {
                            continue; // Skip warnings
                        }
                    }

                    // Extract error message
                    if let Some(message) = json["message"]["message"].as_str() {
                        error_message = Some(message.to_string());

                        // Extract line number from first primary span
                        if let Some(spans) = json["message"]["spans"].as_array() {
                            for span in spans {
                                // Only look at primary spans
                                if span["is_primary"].as_bool().unwrap_or(false) {
                                    if let Some(line_num) = span["line_start"].as_u64() {
                                        line_number = Some(line_num as usize);
                                        break;
                                    }
                                }
                            }

                            // If no primary span, try first span
                            if line_number.is_none() && !spans.is_empty() {
                                if let Some(line_num) = spans[0]["line_start"].as_u64() {
                                    line_number = Some(line_num as usize);
                                }
                            }
                        }

                        // If we found both message and line, return immediately
                        if error_message.is_some() {
                            return (error_message.unwrap(), line_number);
                        }
                    }
                }
            }

            // Fallback: Look for error message in plain text format
            if error_message.is_none() && (line.contains("error:") || line.contains("error[")) {
                // Extract just the error message part
                let msg = if line.contains("error[") {
                    // Format: "error[E0308]: mismatched types"
                    line.split("error[")
                        .nth(1)
                        .and_then(|s| s.split("]: ").nth(1))
                        .unwrap_or(line)
                        .to_string()
                } else if line.contains("error: ") {
                    // Format: "error: expected expression"
                    line.split("error: ")
                        .nth(1)
                        .unwrap_or(line)
                        .to_string()
                } else {
                    line.to_string()
                };
                error_message = Some(msg);
            }

            // Look for file location pattern: "--> src/lib.rs:42:10"
            if line.trim_start().starts_with("-->") {
                // Extract line number from path like "src/lib.rs:42:10"
                if let Some(path_part) = line.split("-->").nth(1) {
                    let parts: Vec<&str> = path_part.trim().split(':').collect();
                    if parts.len() >= 2 {
                        if let Ok(num) = parts[1].parse::<usize>() {
                            line_number = Some(num);
                        }
                    }
                }
            }

            // If we found both in plain text format, return early
            if error_message.is_some() && line_number.is_some() {
                return (error_message.unwrap(), line_number);
            }
        }

        // Return what we found, or whole stderr
        match error_message {
            Some(msg) => (msg, line_number),
            None => {
                // If no specific error found, return first few lines of stderr
                let fallback = stderr.lines().take(5).collect::<Vec<_>>().join("\n");
                (
                    if fallback.is_empty() {
                        "Compilation failed with no error message".to_string()
                    } else {
                        fallback
                    },
                    None,
                )
            }
        }
    }

    /// Clean up workspace directory (T017)
    pub fn cleanup_workspace(&self, workspace: &Path) -> Result<(), String> {
        if workspace.exists() {
            fs::remove_dir_all(workspace)
                .map_err(|e| format!("Failed to remove workspace: {}", e))?;
        }
        Ok(())
    }

    /// Convert component name to crate name (PascalCase -> snake_case)
    fn component_name_to_crate_name(component_name: &str) -> String {
        component_name
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
            .collect()
    }

    /// Recursively search for .wasm file in target directory
    fn find_wasm_file(dir: &Path, crate_name: &str) -> Result<PathBuf, String> {
        let target_filename = format!("{}.wasm", crate_name);

        // Walk through the directory tree
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check if this is the file we're looking for
                if path.is_file() {
                    if let Some(filename) = path.file_name() {
                        if filename == target_filename.as_str() {
                            log::info!("Found .wasm file at: {}", path.display());
                            return Ok(path);
                        }
                    }
                }

                // Recursively search subdirectories
                if path.is_dir() {
                    // Skip some directories for performance
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name != "deps" && dir_name != "build" && dir_name != "incremental" {
                        if let Ok(found) = Self::find_wasm_file(&path, crate_name) {
                            return Ok(found);
                        }
                    }
                }
            }
        }

        Err(format!("Could not find {}.wasm in target directory", crate_name))
    }

    /// Invoke componentize-py to compile Python component
    fn invoke_componentize_py(
        &self,
        workspace: &Path,
        component_name: &str,
        timeout: Duration,
    ) -> Result<CompilationResult, String> {
        // Check if componentize-py is installed
        let check_result = Command::new("componentize-py")
            .arg("--version")
            .output();

        if check_result.is_err() {
            return Ok(CompilationResult::Failure {
                error_message: format!(
                    "componentize-py not found or not in PATH\n\
                    \n\
                    To compile Python WASM components, you need componentize-py installed.\n\
                    \n\
                    Installation instructions:\n\
                    1. Install componentize-py:\n\
                       pip install componentize-py\n\
                    \n\
                    2. Install wasm-tools:\n\
                       cargo install wasm-tools\n\
                    \n\
                    3. Restart the application\n\
                    \n\
                    For more information, visit:\n\
                    https://github.com/bytecodealliance/componentize-py\n\
                    "
                ),
                line_number: None,
                stderr: "componentize-py not found".to_string(),
            });
        }

        // Invoke componentize-py to build the component
        // Note: We define types directly in app.py instead of generating bindings
        // componentize-py will automatically convert between Python types and WIT types
        let output_file = format!("{}.wasm", Self::component_name_to_crate_name(component_name));
        let mut child = Command::new("componentize-py")
            .args(&[
                "-d", "wit",
                "-w", "component",
                "componentize",
                "app",
                "-o", &output_file
            ])
            .current_dir(workspace)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn componentize-py: {}", e))?;

        let start = Instant::now();

        // Poll for completion with timeout
        loop {
            if start.elapsed() > timeout {
                let _ = child.kill();
                return Ok(CompilationResult::Timeout {
                    elapsed: start.elapsed(),
                });
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child
                        .wait_with_output()
                        .map_err(|e| format!("Failed to read componentize-py output: {}", e))?;

                    if status.success() {
                        // Find the compiled .wasm file
                        let crate_name = Self::component_name_to_crate_name(component_name);
                        let wasm_path = workspace.join(format!("{}.wasm", crate_name));

                        if !wasm_path.exists() {
                            return Ok(CompilationResult::Failure {
                                error_message: "Compilation succeeded but .wasm file not found".to_string(),
                                line_number: None,
                                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                            });
                        }

                        return Ok(CompilationResult::Success {
                            wasm_path,
                            build_time_ms: 0,
                            output: String::from_utf8_lossy(&output.stdout).to_string(),
                        });
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        return Ok(CompilationResult::Failure {
                            error_message: stderr.clone(),
                            line_number: None,
                            stderr,
                        });
                    }
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(format!("Failed to wait for componentize-py process: {}", e)),
            }
        }
    }

    /// Invoke componentize-js to compile JavaScript component
    fn invoke_componentize_js(
        &self,
        workspace: &Path,
        component_name: &str,
        timeout: Duration,
    ) -> Result<CompilationResult, String> {
        // Check if componentize-js is installed
        let check_result = Command::new("componentize-js")
            .arg("--version")
            .output();

        if check_result.is_err() {
            return Ok(CompilationResult::Failure {
                error_message: format!(
                    "componentize-js not found or not in PATH\n\
                    \n\
                    To compile JavaScript WASM components, you need componentize-js installed.\n\
                    \n\
                    Installation instructions:\n\
                    1. Install Node.js (v18 or later)\n\
                    2. Install componentize-js:\n\
                       npm install -g @bytecodealliance/componentize-js\n\
                    \n\
                    3. Install wasm-tools:\n\
                       cargo install wasm-tools\n\
                    \n\
                    4. Restart the application\n\
                    \n\
                    For more information, visit:\n\
                    https://github.com/bytecodealliance/componentize-js\n\
                    "
                ),
                line_number: None,
                stderr: "componentize-js not found".to_string(),
            });
        }

        // Invoke componentize-js
        let output_file = format!("{}.wasm", Self::component_name_to_crate_name(component_name));
        let mut child = Command::new("componentize-js")
            .args(&[
                "-w", "wit",
                "-n", "component",
                "-o", &output_file,
                "app.js"
            ])
            .current_dir(workspace)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn componentize-js: {}", e))?;

        let start = Instant::now();

        // Poll for completion with timeout
        loop {
            if start.elapsed() > timeout {
                let _ = child.kill();
                return Ok(CompilationResult::Timeout {
                    elapsed: start.elapsed(),
                });
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child
                        .wait_with_output()
                        .map_err(|e| format!("Failed to read componentize-js output: {}", e))?;

                    if status.success() {
                        // Find the compiled .wasm file
                        let crate_name = Self::component_name_to_crate_name(component_name);
                        let wasm_path = workspace.join(format!("{}.wasm", crate_name));

                        if !wasm_path.exists() {
                            return Ok(CompilationResult::Failure {
                                error_message: "Compilation succeeded but .wasm file not found".to_string(),
                                line_number: None,
                                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                            });
                        }

                        return Ok(CompilationResult::Success {
                            wasm_path,
                            build_time_ms: 0,
                            output: String::from_utf8_lossy(&output.stdout).to_string(),
                        });
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        return Ok(CompilationResult::Failure {
                            error_message: stderr.clone(),
                            line_number: None,
                            stderr,
                        });
                    }
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(format!("Failed to wait for componentize-js process: {}", e)),
            }
        }
    }
}

// Implement Drop for automatic cleanup
impl Drop for ComponentCompiler {
    fn drop(&mut self) {
        // Best effort cleanup of workspace root
        if self.workspace_root.exists() {
            let _ = fs::remove_dir_all(&self.workspace_root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_name_to_crate_name() {
        assert_eq!(
            ComponentCompiler::component_name_to_crate_name("TripleNumber"),
            "triple_number"
        );
        assert_eq!(
            ComponentCompiler::component_name_to_crate_name("HTTPFetcher"),
            "h_t_t_p_fetcher"
        );
    }

    #[test]
    fn test_parse_cargo_error_with_json() {
        let json_error = r#"{"reason":"compiler-message","message":{"message":"expected `;`","spans":[{"line_start":42}]}}"#;
        let (msg, line) = ComponentCompiler::parse_cargo_error(json_error);

        assert!(msg.contains("expected `;`"));
        assert_eq!(line, Some(42));
    }

    #[test]
    fn test_parse_cargo_error_fallback() {
        let error = "error: expected `;`\n  --> src/lib.rs:42:10";
        let (msg, line) = ComponentCompiler::parse_cargo_error(error);

        assert!(msg.contains("error"));
        assert_eq!(line, Some(42));
    }
}
