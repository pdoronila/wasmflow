//! T029-T030, T083: Syntax-highlighted code editor for WASM Creator Node
//!
//! Provides a code editor with syntax highlighting for Rust and JavaScript.
//! Supports theme selection for different color schemes.

use crate::graph::node::Language;
use egui::{Response, Ui};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};

/// Syntax-highlighted code editor supporting multiple languages
///
/// Provides:
/// - Syntax highlighting for Rust and JavaScript
/// - Multiple color themes (dark/light)
/// - Line numbers
/// - Monospace font
///
/// T083: Enhanced from basic TextEdit to full syntax highlighting
pub struct CodeEditorWidget {
    /// Number of rows to display
    rows: usize,
    /// Font size
    font_size: f32,
    /// Color theme for syntax highlighting
    theme: CodeTheme,
}

/// Available color themes for the code editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeTheme {
    /// Catppuccin Mocha (dark, default)
    Mocha,
    /// Catppuccin Macchiato (dark)
    Macchiato,
    /// Catppuccin Frappé (medium dark)
    Frappe,
    /// Catppuccin Latte (light)
    Latte,
}

impl CodeTheme {
    /// Get the display name for this theme
    pub fn display_name(&self) -> &'static str {
        match self {
            CodeTheme::Mocha => "Catppuccin Mocha",
            CodeTheme::Macchiato => "Catppuccin Macchiato",
            CodeTheme::Frappe => "Catppuccin Frappé",
            CodeTheme::Latte => "Catppuccin Latte",
        }
    }

    /// Convert to egui_code_editor ColorTheme
    fn to_color_theme(&self) -> ColorTheme {
        match self {
            CodeTheme::Mocha => Self::catppuccin_mocha(),
            CodeTheme::Macchiato => Self::catppuccin_macchiato(),
            CodeTheme::Frappe => Self::catppuccin_frappe(),
            CodeTheme::Latte => Self::catppuccin_latte(),
        }
    }

    /// Catppuccin Mocha theme (dark)
    fn catppuccin_mocha() -> ColorTheme {
        ColorTheme {
            name: "Catppuccin Mocha",
            dark: true,
            bg: "#1e1e2e",        // Base
            cursor: "#f5e0dc",    // Rosewater
            selection: "#f5c2e7", // Pink
            comments: "#6c7086",  // Overlay0
            functions: "#89b4fa", // Blue
            keywords: "#cba6f7",  // Mauve
            literals: "#fab387",  // Peach
            numerics: "#fab387",  // Peach
            punctuation: "#cdd6f4", // Text
            strs: "#a6e3a1",      // Green
            types: "#f9e2af",     // Yellow
            special: "#f5c2e7",   // Pink
        }
    }

    /// Catppuccin Macchiato theme (dark)
    fn catppuccin_macchiato() -> ColorTheme {
        ColorTheme {
            name: "Catppuccin Macchiato",
            dark: true,
            bg: "#24273a",        // Base
            cursor: "#f4dbd6",    // Rosewater
            selection: "#f5bde6", // Pink
            comments: "#6e738d",  // Overlay0
            functions: "#8aadf4", // Blue
            keywords: "#c6a0f6",  // Mauve
            literals: "#f5a97f",  // Peach
            numerics: "#f5a97f",  // Peach
            punctuation: "#cad3f5", // Text
            strs: "#a6da95",      // Green
            types: "#eed49f",     // Yellow
            special: "#f5bde6",   // Pink
        }
    }

    /// Catppuccin Frappé theme (medium dark)
    fn catppuccin_frappe() -> ColorTheme {
        ColorTheme {
            name: "Catppuccin Frappé",
            dark: true,
            bg: "#303446",        // Base
            cursor: "#f2d5cf",    // Rosewater
            selection: "#f4b8e4", // Pink
            comments: "#737994",  // Overlay0
            functions: "#8caaee", // Blue
            keywords: "#ca9ee6",  // Mauve
            literals: "#ef9f76",  // Peach
            numerics: "#ef9f76",  // Peach
            punctuation: "#c6d0f5", // Text
            strs: "#a6d189",      // Green
            types: "#e5c890",     // Yellow
            special: "#f4b8e4",   // Pink
        }
    }

    /// Catppuccin Latte theme (light)
    fn catppuccin_latte() -> ColorTheme {
        ColorTheme {
            name: "Catppuccin Latte",
            dark: false,
            bg: "#eff1f5",        // Base
            cursor: "#dc8a78",    // Rosewater
            selection: "#ea76cb", // Pink
            comments: "#9ca0b0",  // Overlay0
            functions: "#1e66f5", // Blue
            keywords: "#8839ef",  // Mauve
            literals: "#fe640b",  // Peach
            numerics: "#fe640b",  // Peach
            punctuation: "#4c4f69", // Text
            strs: "#40a02b",      // Green
            types: "#df8e1d",     // Yellow
            special: "#ea76cb",   // Pink
        }
    }

    /// Get all available themes
    pub fn all() -> &'static [CodeTheme] {
        &[
            CodeTheme::Mocha,
            CodeTheme::Macchiato,
            CodeTheme::Frappe,
            CodeTheme::Latte,
        ]
    }
}

impl Default for CodeTheme {
    fn default() -> Self {
        CodeTheme::Mocha
    }
}

impl CodeEditorWidget {
    /// Create JavaScript syntax highlighting
    fn javascript_syntax() -> Syntax {
        Syntax::new("javascript")
            .with_comment("//")
            .with_comment_multiline(["/*", "*/"])
            .with_keywords([
                "async", "await", "break", "case", "catch", "class", "const", "continue",
                "debugger", "default", "delete", "do", "else", "export", "extends", "finally",
                "for", "function", "if", "import", "in", "instanceof", "let", "new", "of",
                "return", "static", "super", "switch", "this", "throw", "try", "typeof",
                "var", "void", "while", "with", "yield",
            ])
            .with_types([
                "Array", "Boolean", "Date", "Error", "Function", "JSON", "Map", "Math",
                "Number", "Object", "Promise", "Proxy", "RegExp", "Set", "String", "Symbol",
                "WeakMap", "WeakSet",
            ])
            .with_special([
                "console", "document", "window", "global", "process", "undefined", "null",
                "true", "false", "NaN", "Infinity",
            ])
    }

    /// Create a new code editor with default settings
    ///
    /// Defaults:
    /// - 20 rows visible
    /// - 14.0 font size
    /// - Dark theme
    pub fn new() -> Self {
        Self {
            rows: 20,
            font_size: 14.0,
            theme: CodeTheme::default(),
        }
    }

    /// Set the number of visible rows
    pub fn with_rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// Set the font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the color theme
    pub fn with_theme(mut self, theme: CodeTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Display the code editor with syntax highlighting
    ///
    /// # Arguments
    /// * `ui` - The egui UI context
    /// * `code` - Mutable reference to the code string
    /// * `language` - The programming language for syntax highlighting
    ///
    /// # Returns
    /// The Response from the editor widget (for detecting changes)
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut code = String::from("fn main() {}");
    /// let editor = CodeEditorWidget::new();
    /// let response = editor.show(ui, &mut code, Language::Rust);
    /// if response.changed() {
    ///     // Code was modified
    /// }
    /// ```
    pub fn show(&self, ui: &mut Ui, code: &mut String, language: Language) -> Response {
        let syntax = match language {
            Language::Rust => Syntax::rust(),
            Language::JavaScript => Self::javascript_syntax(),
            Language::Python => Syntax::python(),
        };

        let theme = self.theme.to_color_theme();

        CodeEditor::default()
            .id_source("wasm_creator_code_editor")
            .with_rows(self.rows)
            .with_fontsize(self.font_size)
            .with_theme(theme)
            .with_syntax(syntax)
            .with_numlines(true)
            .show(ui, code)
            .response
    }

    /// Display the code editor with a theme selector
    ///
    /// Shows a dropdown to select the color theme, followed by the code editor
    ///
    /// # Arguments
    /// * `ui` - The egui UI context
    /// * `code` - Mutable reference to the code string
    /// * `language` - The programming language for syntax highlighting
    /// * `theme` - Mutable reference to the current theme (can be changed by user)
    ///
    /// # Returns
    /// The Response from the editor widget (for detecting changes)
    pub fn show_with_theme_selector(
        &self,
        ui: &mut Ui,
        code: &mut String,
        language: Language,
        theme: &mut CodeTheme,
    ) -> Response {
        ui.horizontal(|ui| {
            ui.label("Theme:");
            egui::ComboBox::from_id_salt("code_editor_theme")
                .selected_text(theme.display_name())
                .show_ui(ui, |ui| {
                    for available_theme in CodeTheme::all() {
                        ui.selectable_value(theme, *available_theme, available_theme.display_name());
                    }
                });
        });

        ui.add_space(5.0);

        // Create a temporary editor with the selected theme
        let editor_with_theme = Self {
            rows: self.rows,
            font_size: self.font_size,
            theme: *theme,
        };

        editor_with_theme.show(ui, code, language)
    }

    /// Get the current line count from the code string
    ///
    /// # Arguments
    /// * `code` - The code string to count lines in
    ///
    /// # Returns
    /// Number of lines in the code
    pub fn line_count(code: &str) -> usize {
        if code.is_empty() {
            0
        } else {
            code.lines().count()
        }
    }

    /// Get the number of visible rows
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Get the font size
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Get the current theme
    pub fn theme(&self) -> CodeTheme {
        self.theme
    }
}

impl Default for CodeEditorWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_editor_has_defaults() {
        let editor = CodeEditorWidget::new();
        assert_eq!(editor.rows(), 20);
        assert_eq!(editor.font_size(), 14.0);
        assert_eq!(editor.theme(), CodeTheme::Mocha);
    }

    #[test]
    fn test_with_rows() {
        let editor = CodeEditorWidget::new().with_rows(30);
        assert_eq!(editor.rows(), 30);
    }

    #[test]
    fn test_with_font_size() {
        let editor = CodeEditorWidget::new().with_font_size(16.0);
        assert_eq!(editor.font_size(), 16.0);
    }

    #[test]
    fn test_with_theme() {
        let editor = CodeEditorWidget::new().with_theme(CodeTheme::Latte);
        assert_eq!(editor.theme(), CodeTheme::Latte);
    }

    #[test]
    fn test_line_count_empty() {
        assert_eq!(CodeEditorWidget::line_count(""), 0);
    }

    #[test]
    fn test_line_count_single_line() {
        assert_eq!(CodeEditorWidget::line_count("fn main() {}"), 1);
    }

    #[test]
    fn test_line_count_multiple_lines() {
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(CodeEditorWidget::line_count(code), 3);
    }

    #[test]
    fn test_line_count_trailing_newline() {
        let code = "line 1\nline 2\n";
        assert_eq!(CodeEditorWidget::line_count(code), 2);
    }

    #[test]
    fn test_builder_pattern() {
        let editor = CodeEditorWidget::new()
            .with_rows(25)
            .with_font_size(16.0)
            .with_theme(CodeTheme::Macchiato);

        assert_eq!(editor.rows(), 25);
        assert_eq!(editor.font_size(), 16.0);
        assert_eq!(editor.theme(), CodeTheme::Macchiato);
    }

    #[test]
    fn test_theme_display_names() {
        assert_eq!(CodeTheme::Mocha.display_name(), "Catppuccin Mocha");
        assert_eq!(CodeTheme::Macchiato.display_name(), "Catppuccin Macchiato");
        assert_eq!(CodeTheme::Frappe.display_name(), "Catppuccin Frappé");
        assert_eq!(CodeTheme::Latte.display_name(), "Catppuccin Latte");
    }

    #[test]
    fn test_all_themes() {
        let themes = CodeTheme::all();
        assert_eq!(themes.len(), 4);
        assert!(themes.contains(&CodeTheme::Mocha));
        assert!(themes.contains(&CodeTheme::Latte));
    }
}
