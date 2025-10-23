//! T087: Application theme with consistent color palette
//!
//! Provides a centralized theme system for the WasmFlow application,
//! ensuring consistent colors across nodes, ports, connections, and UI elements.

use egui::Color32;

/// WasmFlow application theme
pub struct Theme {
    /// Port colors by data type
    #[allow(dead_code)]
    pub port_colors: PortColors,
    /// Node visual appearance
    #[allow(dead_code)]
    pub node_colors: NodeColors,
    /// Connection line colors
    #[allow(dead_code)]
    pub connection_colors: ConnectionColors,
    /// UI element colors
    #[allow(dead_code)]
    pub ui_colors: UiColors,
    /// Component palette colors
    pub palette_colors: PaletteColors,
    /// Selection visual theme
    #[allow(dead_code)]
    pub selection_colors: SelectionTheme,
    /// T046: Composite node visual theme
    #[allow(dead_code)]
    pub composite_colors: CompositeNodeTheme,
}

/// Port colors based on data type
#[allow(dead_code)]
pub struct PortColors {
    pub f32_color: Color32,
    pub i32_color: Color32,
    pub u32_color: Color32,
    pub string_color: Color32,
    pub bool_color: Color32,
    pub binary_color: Color32,
    pub list_color: Color32,
    pub record_color: Color32,
    pub any_color: Color32,
}

/// Node appearance colors
#[allow(dead_code)]
pub struct NodeColors {
    /// Node background color
    pub background: Color32,
    /// Node header background
    pub header_background: Color32,
    /// Node border color (normal state)
    pub border: Color32,
    /// Node border color (selected)
    pub border_selected: Color32,
    /// Node border color (executing)
    pub border_executing: Color32,
    /// Node border color (completed)
    pub border_completed: Color32,
    /// Node border color (failed)
    pub border_failed: Color32,
}

/// Connection line colors
#[allow(dead_code)]
pub struct ConnectionColors {
    /// Valid connection (type-compatible)
    pub valid: Color32,
    /// Invalid connection (type mismatch)
    pub invalid: Color32,
    /// Connection being dragged
    pub dragging: Color32,
}

/// UI element colors
#[allow(dead_code)]
pub struct UiColors {
    /// Error messages
    pub error: Color32,
    /// Warning messages
    pub warning: Color32,
    /// Success messages
    pub success: Color32,
    /// Info messages
    pub info: Color32,
    /// Permission dialog - danger zone background
    pub danger_background: Color32,
    /// Permission dialog - danger zone border
    pub danger_border: Color32,
}

/// Component palette colors
pub struct PaletteColors {
    /// Built-in component background
    pub builtin_component: Color32,
    /// User-defined (WASM) component background
    pub user_defined_component: Color32,
}

/// T018: Selection visual theme colors
#[allow(dead_code)]
pub struct SelectionTheme {
    /// Selection rectangle fill color (during drag)
    pub rectangle_fill: Color32,
    /// Selection rectangle stroke color (during drag)
    pub rectangle_stroke: Color32,
    /// Preview highlight color (nodes under selection rectangle)
    pub preview_highlight: Color32,
    /// Selected node border color (persistent after selection)
    pub selected_border: Color32,
    /// Selected node border width
    pub selected_border_width: f32,
}

/// T046: Composite node visual theme colors
#[allow(dead_code)]
pub struct CompositeNodeTheme {
    /// Background color (darker/distinct from regular nodes)
    pub background: Color32,
    /// Border color (unique to composite nodes)
    pub border: Color32,
    /// Border width (slightly thicker)
    pub border_width: f32,
    /// Badge/icon color (for composition symbol)
    pub badge_color: Color32,
    /// Footer background color
    pub footer_background: Color32,
    /// Footer text color
    pub footer_text_color: Color32,
}

impl Theme {
    /// Create default dark theme for WasmFlow
    pub fn dark() -> Self {
        Self {
            port_colors: PortColors {
                f32_color: Color32::from_rgb(100, 150, 255),    // Blue
                i32_color: Color32::from_rgb(150, 100, 255),    // Purple
                u32_color: Color32::from_rgb(255, 150, 100),    // Orange
                string_color: Color32::from_rgb(100, 255, 150), // Green
                bool_color: Color32::from_rgb(100, 255, 255),   // Cyan
                binary_color: Color32::from_rgb(200, 200, 200), // Gray
                list_color: Color32::from_rgb(255, 200, 100),   // Yellow
                record_color: Color32::from_rgb(255, 100, 150), // Pink
                any_color: Color32::WHITE,
            },
            node_colors: NodeColors {
                background: Color32::from_rgb(45, 45, 50),
                header_background: Color32::from_rgb(35, 35, 40),
                border: Color32::from_rgb(100, 100, 110),
                border_selected: Color32::from_rgb(100, 150, 255),
                border_executing: Color32::from_rgb(255, 200, 100),
                border_completed: Color32::from_rgb(100, 200, 100),
                border_failed: Color32::from_rgb(255, 80, 80),
            },
            connection_colors: ConnectionColors {
                valid: Color32::from_rgb(100, 200, 100),
                invalid: Color32::from_rgb(255, 80, 80),
                dragging: Color32::from_rgb(150, 150, 200),
            },
            ui_colors: UiColors {
                error: Color32::from_rgb(255, 100, 100),
                warning: Color32::from_rgb(255, 180, 0),
                success: Color32::from_rgb(100, 200, 100),
                info: Color32::from_rgb(200, 150, 50),
                danger_background: Color32::from_rgb(80, 20, 20),
                danger_border: Color32::from_rgb(255, 80, 80),
            },
            palette_colors: PaletteColors {
                builtin_component: Color32::from_rgb(65, 65, 70), // Gray
                user_defined_component: Color32::from_rgb(180, 100, 220), // Purple (T023)
            },
            selection_colors: SelectionTheme {
                rectangle_fill: Color32::from_rgba_unmultiplied(100, 150, 200, 50),
                rectangle_stroke: Color32::from_rgb(100, 150, 200),
                preview_highlight: Color32::from_rgba_unmultiplied(100, 200, 100, 80),
                selected_border: Color32::from_rgb(100, 200, 255),
                selected_border_width: 2.5,
            },
            composite_colors: CompositeNodeTheme {
                // T046: Darker background to distinguish from regular nodes
                background: Color32::from_rgb(45, 50, 65),
                // T046: Unique teal/cyan border color
                border: Color32::from_rgb(80, 180, 180),
                // T046: Thicker border (2.5px vs normal 1.5px)
                border_width: 2.5,
                // T046: Gold badge color for composition symbol
                badge_color: Color32::from_rgb(255, 200, 80),
                // T046: Slightly darker footer background
                footer_background: Color32::from_rgb(35, 40, 50),
                // T046: Lighter footer text for readability
                footer_text_color: Color32::from_rgb(180, 180, 190),
            },
        }
    }

    /// Create light theme for WasmFlow
    #[allow(dead_code)]
    pub fn light() -> Self {
        Self {
            port_colors: PortColors {
                f32_color: Color32::from_rgb(50, 100, 200),     // Blue
                i32_color: Color32::from_rgb(100, 50, 200),     // Purple
                u32_color: Color32::from_rgb(200, 100, 50),     // Orange
                string_color: Color32::from_rgb(50, 180, 100),  // Green
                bool_color: Color32::from_rgb(50, 180, 180),    // Cyan
                binary_color: Color32::from_rgb(120, 120, 120), // Gray
                list_color: Color32::from_rgb(200, 150, 50),    // Yellow
                record_color: Color32::from_rgb(200, 50, 100),  // Pink
                any_color: Color32::BLACK,
            },
            node_colors: NodeColors {
                background: Color32::from_rgb(245, 245, 248),
                header_background: Color32::from_rgb(230, 230, 235),
                border: Color32::from_rgb(180, 180, 190),
                border_selected: Color32::from_rgb(50, 100, 200),
                border_executing: Color32::from_rgb(200, 150, 50),
                border_completed: Color32::from_rgb(50, 150, 50),
                border_failed: Color32::from_rgb(200, 50, 50),
            },
            connection_colors: ConnectionColors {
                valid: Color32::from_rgb(50, 150, 50),
                invalid: Color32::from_rgb(200, 50, 50),
                dragging: Color32::from_rgb(100, 100, 150),
            },
            ui_colors: UiColors {
                error: Color32::from_rgb(200, 50, 50),
                warning: Color32::from_rgb(200, 140, 0),
                success: Color32::from_rgb(50, 150, 50),
                info: Color32::from_rgb(150, 100, 30),
                danger_background: Color32::from_rgb(255, 220, 220),
                danger_border: Color32::from_rgb(200, 50, 50),
            },
            palette_colors: PaletteColors {
                builtin_component: Color32::from_rgb(200, 200, 205), // Light gray
                user_defined_component: Color32::from_rgb(200, 140, 230), // Light purple (T023)
            },
            selection_colors: SelectionTheme {
                rectangle_fill: Color32::from_rgba_unmultiplied(150, 180, 220, 50),
                rectangle_stroke: Color32::from_rgb(50, 100, 200),
                preview_highlight: Color32::from_rgba_unmultiplied(50, 180, 100, 80),
                selected_border: Color32::from_rgb(50, 150, 255),
                selected_border_width: 2.5,
            },
            composite_colors: CompositeNodeTheme {
                // Light theme composite colors
                background: Color32::from_rgb(235, 240, 250),
                border: Color32::from_rgb(80, 160, 180),
                border_width: 2.5,
                badge_color: Color32::from_rgb(200, 140, 40),
                footer_background: Color32::from_rgb(220, 225, 235),
                footer_text_color: Color32::from_rgb(60, 60, 70),
            },
        }
    }

    /// Apply theme to egui::Style (affects global UI appearance)
    #[allow(dead_code)]
    pub fn apply_to_style(&self, style: &mut egui::Style) {
        // Configure visuals
        let visuals = &mut style.visuals;

        // Use dark theme as base if we're using dark theme
        // egui will handle the rest based on dark_mode flag
        // We mainly want to ensure our custom colors are used where needed

        // Window background
        visuals.window_fill = Color32::from_rgb(30, 30, 35);
        visuals.panel_fill = Color32::from_rgb(35, 35, 40);

        // Widget colors
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(50, 50, 55);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(55, 55, 60);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 65, 70);
        visuals.widgets.active.bg_fill = Color32::from_rgb(75, 75, 80);

        // Text colors
        visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(200, 200, 200);
        visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(220, 220, 220);
        visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(240, 240, 240);
        visuals.widgets.active.fg_stroke.color = Color32::WHITE;

        // Selection color
        visuals.selection.bg_fill = Color32::from_rgb(100, 150, 255).linear_multiply(0.3);
        visuals.selection.stroke.color = Color32::from_rgb(100, 150, 255);

        // Hyperlink color
        visuals.hyperlink_color = Color32::from_rgb(100, 150, 255);

        // Error text color
        visuals.error_fg_color = self.ui_colors.error;
        visuals.warn_fg_color = self.ui_colors.warning;

        // Window rounding (using modern CornerRadius API)
        // Note: egui 0.32+ renamed Rounding to CornerRadius and changed field names
        // visuals.window_rounding and visuals.menu_rounding removed in favor of widgets config

        // Spacing
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(8);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
    }

    /// Get port color for a given data type
    #[allow(dead_code)]
    pub fn port_color(&self, data_type: &crate::graph::node::DataType) -> Color32 {
        use crate::graph::node::DataType;
        match data_type {
            DataType::F32 => self.port_colors.f32_color,
            DataType::I32 => self.port_colors.i32_color,
            DataType::U32 => self.port_colors.u32_color,
            DataType::String => self.port_colors.string_color,
            DataType::Bool => self.port_colors.bool_color,
            DataType::Binary => self.port_colors.binary_color,
            DataType::List(_) => self.port_colors.list_color,
            DataType::Record(_) => self.port_colors.record_color,
            DataType::Any => self.port_colors.any_color,
        }
    }

    /// Get node border color based on execution state
    #[allow(dead_code)]
    pub fn node_border_color(
        &self,
        execution_state: crate::graph::node::ExecutionState,
    ) -> Color32 {
        use crate::graph::node::ExecutionState;
        match execution_state {
            ExecutionState::Idle => self.node_colors.border,
            ExecutionState::Running => self.node_colors.border_executing,
            ExecutionState::Completed => self.node_colors.border_completed,
            ExecutionState::Failed => self.node_colors.border_failed,
        }
    }

    /// Get connection color based on validity
    #[allow(dead_code)]
    pub fn connection_color(&self, is_valid: bool) -> Color32 {
        if is_valid {
            self.connection_colors.valid
        } else {
            self.connection_colors.invalid
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme_creation() {
        let theme = Theme::dark();
        assert_eq!(
            theme.port_colors.f32_color,
            Color32::from_rgb(100, 150, 255)
        );
        assert_eq!(theme.ui_colors.error, Color32::from_rgb(255, 100, 100));
    }

    #[test]
    fn test_light_theme_creation() {
        let theme = Theme::light();
        assert_eq!(theme.port_colors.f32_color, Color32::from_rgb(50, 100, 200));
        assert_eq!(theme.ui_colors.error, Color32::from_rgb(200, 50, 50));
    }

    #[test]
    fn test_port_color_mapping() {
        use crate::graph::node::DataType;
        let theme = Theme::dark();

        assert_eq!(
            theme.port_color(&DataType::F32),
            theme.port_colors.f32_color
        );
        assert_eq!(
            theme.port_color(&DataType::String),
            theme.port_colors.string_color
        );
        assert_eq!(
            theme.port_color(&DataType::Any),
            theme.port_colors.any_color
        );
    }

    #[test]
    fn test_node_border_color_by_state() {
        use crate::graph::node::ExecutionState;
        let theme = Theme::dark();

        assert_eq!(
            theme.node_border_color(ExecutionState::Idle),
            theme.node_colors.border
        );
        assert_eq!(
            theme.node_border_color(ExecutionState::Running),
            theme.node_colors.border_executing
        );
        assert_eq!(
            theme.node_border_color(ExecutionState::Completed),
            theme.node_colors.border_completed
        );
        assert_eq!(
            theme.node_border_color(ExecutionState::Failed),
            theme.node_colors.border_failed
        );
    }

    #[test]
    fn test_connection_color() {
        let theme = Theme::dark();
        assert_eq!(theme.connection_color(true), theme.connection_colors.valid);
        assert_eq!(
            theme.connection_color(false),
            theme.connection_colors.invalid
        );
    }
}
