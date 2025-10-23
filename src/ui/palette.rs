//! Component palette with search functionality
//!
//! T091: Provides a searchable, categorized list of available components
//! with fuzzy search and keyboard navigation support.

use crate::graph::node::{ComponentRegistry, ComponentSpec};
use eframe::egui;

/// Component palette widget with search functionality
pub struct Palette {
    /// Search filter text
    search_filter: String,
    /// Currently selected index in the filtered results
    selected_index: Option<usize>,
    /// Whether to scroll to selected item
    scroll_to_selected: bool,
}

impl Palette {
    /// Create a new palette widget
    pub fn new() -> Self {
        Self {
            search_filter: String::new(),
            selected_index: None,
            scroll_to_selected: false,
        }
    }

    /// Render the palette and return the component the user wants to add (if any)
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        registry: &ComponentRegistry,
        theme: &crate::ui::theme::Theme,
    ) -> Option<PaletteAction> {
        let mut action = None;

        egui::SidePanel::left("palette")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Components");
                ui.separator();

                // T091: Search filter text box
                let response = ui.text_edit_singleline(&mut self.search_filter);

                // Handle keyboard navigation in search box
                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        self.move_selection_down();
                        self.scroll_to_selected = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.move_selection_up();
                        self.scroll_to_selected = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Add the selected component
                        if let Some(idx) = self.selected_index {
                            let filtered = self.get_filtered_components(registry);
                            if let Some(spec) = filtered.get(idx) {
                                action = Some(PaletteAction::AddComponent {
                                    spec: (*spec).clone(),
                                    position: egui::Pos2::new(400.0, 300.0),
                                });
                                self.search_filter.clear();
                                self.selected_index = None;
                            }
                        }
                    }
                }

                ui.add_space(5.0);

                // Show filtered results in a scrollable area
                egui::ScrollArea::vertical()
                    .id_salt("palette_scroll")
                    .show(ui, |ui| {
                        if self.search_filter.is_empty() {
                            // Show categorized view when no search filter
                            if let Some(new_action) = self.render_categorized(ui, registry, theme) {
                                action = Some(new_action);
                            }
                        } else {
                            // Show flat filtered list when searching
                            if let Some(new_action) = self.render_filtered(ui, registry, theme) {
                                action = Some(new_action);
                            }
                        }
                    });

                // Show search hint
                if self.search_filter.is_empty() {
                    ui.separator();
                    ui.label("💡 Type to search components");
                } else {
                    let count = self.get_filtered_components(registry).len();
                    ui.separator();
                    ui.label(format!("Found {} component{}", count, if count == 1 { "" } else { "s" }));
                }
            });

        action
    }

    /// Render categorized component list (when no search filter)
    /// T024: Separate user-defined components into their own category
    fn render_categorized(&mut self, ui: &mut egui::Ui, registry: &ComponentRegistry, theme: &crate::ui::theme::Theme) -> Option<PaletteAction> {
        let mut action = None;

        // Separate builtin and user-defined components
        let mut builtin_categories: std::collections::HashMap<String, Vec<&ComponentSpec>> =
            std::collections::HashMap::new();
        let mut user_defined_components: Vec<&ComponentSpec> = Vec::new();

        for spec in registry.list_all() {
            match &spec.component_type {
                crate::graph::node::ComponentType::Builtin => {
                    let category = spec.category.clone().unwrap_or_else(|| "Other".to_string());
                    builtin_categories.entry(category).or_default().push(spec);
                }
                crate::graph::node::ComponentType::UserDefined(_) => {
                    user_defined_components.push(spec);
                }
                crate::graph::node::ComponentType::Composed { .. } => {
                    // Composite nodes appear in user-defined section
                    user_defined_components.push(spec);
                }
            }
        }

        // T024: Show user-defined components first if any exist
        if !user_defined_components.is_empty() {
            ui.collapsing("User-Defined", |ui| {
                for spec in user_defined_components {
                    if let Some(new_action) = self.render_component_button(ui, spec, theme) {
                        action = Some(new_action);
                    }
                }
            });
        }

        // Show builtin categories sorted alphabetically
        let mut sorted_categories: Vec<_> = builtin_categories.iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

        for (category, specs) in sorted_categories {
            ui.collapsing(category, |ui| {
                for spec in specs {
                    if let Some(new_action) = self.render_component_button(ui, spec, theme) {
                        action = Some(new_action);
                    }
                }
            });
        }

        action
    }

    /// Render filtered component list (when searching)
    fn render_filtered(&mut self, ui: &mut egui::Ui, registry: &ComponentRegistry, theme: &crate::ui::theme::Theme) -> Option<PaletteAction> {
        let mut action = None;

        // Clone the filtered specs to avoid borrow checker issues
        let filtered: Vec<ComponentSpec> = self.get_filtered_components(registry)
            .into_iter().cloned()
            .collect();

        // Update selected index if out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= filtered.len() {
                self.selected_index = if filtered.is_empty() { None } else { Some(filtered.len() - 1) };
            }
        }

        for (idx, spec) in filtered.iter().enumerate() {
            let is_selected = self.selected_index == Some(idx);

            // Choose color based on component type
            let bg_color = match &spec.component_type {
                crate::graph::node::ComponentType::Builtin => theme.palette_colors.builtin_component,
                crate::graph::node::ComponentType::UserDefined(_) | crate::graph::node::ComponentType::Composed { .. } => theme.palette_colors.user_defined_component,
            };

            // Highlight selected item
            let button = if is_selected {
                ui.add(
                    egui::Button::new(egui::RichText::new(&spec.name).strong())
                        .fill(bg_color)
                )
            } else {
                ui.add(
                    egui::Button::new(&spec.name)
                        .fill(bg_color)
                )
            };

            // Scroll to selected item if needed
            if is_selected && self.scroll_to_selected {
                button.scroll_to_me(Some(egui::Align::Center));
                self.scroll_to_selected = false;
            }

            if button.clicked() {
                action = Some(PaletteAction::AddComponent {
                    spec: spec.clone(),
                    position: egui::Pos2::new(400.0, 300.0),
                });
                self.search_filter.clear();
                self.selected_index = None;
            }

            // Show category and description in tooltip
            let category = spec.category.clone().unwrap_or_else(|| "Other".to_string());
            let tooltip_text = format!(
                "{}\n\nCategory: {}\nAuthor: {}\nVersion: {}\nType: {}",
                spec.description,
                category,
                spec.author,
                spec.version,
                match &spec.component_type {
                    crate::graph::node::ComponentType::Builtin => "Built-in",
                    crate::graph::node::ComponentType::UserDefined(_) => "User-defined",
                    crate::graph::node::ComponentType::Composed { .. } => "Composite",
                }
            );
            button.on_hover_text(tooltip_text);
        }

        action
    }

    /// Render a component button with tooltip
    fn render_component_button(&self, ui: &mut egui::Ui, spec: &ComponentSpec, theme: &crate::ui::theme::Theme) -> Option<PaletteAction> {
        // Choose color based on component type
        let bg_color = match &spec.component_type {
            crate::graph::node::ComponentType::Builtin => theme.palette_colors.builtin_component,
            crate::graph::node::ComponentType::UserDefined(_) | crate::graph::node::ComponentType::Composed { .. } => theme.palette_colors.user_defined_component,
        };

        let button = ui.add(
            egui::Button::new(&spec.name)
                .fill(bg_color)
        );

        if button.clicked() {
            return Some(PaletteAction::AddComponent {
                spec: spec.clone(),
                position: egui::Pos2::new(400.0, 300.0),
            });
        }

        // Show detailed tooltip with component info
        let tooltip_text = format!(
            "{}\n\nAuthor: {}\nVersion: {}\nType: {}",
            spec.description,
            spec.author,
            spec.version,
            match &spec.component_type {
                crate::graph::node::ComponentType::Builtin => "Built-in",
                crate::graph::node::ComponentType::UserDefined(_) => "User-defined",
                crate::graph::node::ComponentType::Composed { .. } => "Composite",
            }
        );
        button.on_hover_text(tooltip_text);

        None
    }

    /// Get filtered components based on search query
    /// Uses fuzzy matching on name, category, and description
    fn get_filtered_components<'a>(&self, registry: &'a ComponentRegistry) -> Vec<&'a ComponentSpec> {
        if self.search_filter.is_empty() {
            return registry.list_all();
        }

        let query = self.search_filter.to_lowercase();
        let mut results: Vec<(&ComponentSpec, i32)> = registry
            .list_all()
            .into_iter()
            .filter_map(|spec| {
                let score = self.fuzzy_match_score(spec, &query);
                if score > 0 {
                    Some((spec, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(spec, _)| spec).collect()
    }

    /// Calculate fuzzy match score for a component
    /// Higher score = better match
    fn fuzzy_match_score(&self, spec: &ComponentSpec, query: &str) -> i32 {
        let mut score = 0;

        let name = spec.name.to_lowercase();
        let description = spec.description.to_lowercase();
        let category = spec.category.clone().unwrap_or_default().to_lowercase();

        // Exact name match gets highest score
        if name == query {
            score += 1000;
        }
        // Name starts with query
        else if name.starts_with(query) {
            score += 500;
        }
        // Name contains query
        else if name.contains(query) {
            score += 250;
        }

        // Category matches
        if category == query {
            score += 100;
        } else if category.contains(query) {
            score += 50;
        }

        // Description contains query
        if description.contains(query) {
            score += 25;
        }

        // Fuzzy character sequence matching
        if self.fuzzy_contains(&name, query) {
            score += 10;
        }

        score
    }

    /// Check if target contains all characters from query in order (fuzzy match)
    /// e.g., "add" matches "Addition", "ct" matches "Constant"
    fn fuzzy_contains(&self, target: &str, query: &str) -> bool {
        let mut target_chars = target.chars();

        for query_char in query.chars() {
            if !target_chars.any(|c| c == query_char) {
                return false;
            }
        }

        true
    }

    /// Move selection down in filtered results
    fn move_selection_down(&mut self) {
        if let Some(idx) = self.selected_index {
            self.selected_index = Some(idx + 1);
        } else {
            self.selected_index = Some(0);
        }
    }

    /// Move selection up in filtered results
    fn move_selection_up(&mut self) {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
            }
        } else {
            self.selected_index = Some(0);
        }
    }
}

/// Actions that can be triggered from the palette
#[derive(Debug, Clone)]
pub enum PaletteAction {
    /// User wants to add a component
    AddComponent {
        spec: ComponentSpec,
        position: egui::Pos2,
    },
}
