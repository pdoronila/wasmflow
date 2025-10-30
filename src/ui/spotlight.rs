//! Spotlight search for quick node creation
//!
//! Provides a macOS Spotlight-like search dialog for quickly finding and adding
//! components to the canvas. Triggered by double-space, with fuzzy search and
//! keyboard navigation.

use crate::graph::node::{ComponentRegistry, ComponentSpec};
use crate::runtime::wasm_host::ComponentManager;
use eframe::egui;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

/// Action returned by the spotlight search
#[derive(Debug, Clone)]
pub enum SpotlightAction {
    /// User selected a component to add
    AddComponent {
        spec: ComponentSpec,
        position: egui::Pos2,
    },
}

/// Spotlight search widget
pub struct SpotlightSearch {
    /// Whether the dialog is visible
    visible: bool,
    /// Search query text
    query: String,
    /// Currently selected index in filtered results
    selected_index: Option<usize>,
    /// Whether to scroll to selected item
    scroll_to_selected: bool,
    /// Request focus on the search input
    request_focus: bool,
    /// Last 3 components used (most recent first)
    recent_components: VecDeque<String>,
}

impl SpotlightSearch {
    /// Create a new spotlight search widget
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
            selected_index: None,
            scroll_to_selected: false,
            request_focus: false,
            recent_components: VecDeque::new(),
        }
    }

    /// Open the spotlight search dialog
    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = None;
        self.scroll_to_selected = false;
        self.request_focus = true;
    }

    /// Close the spotlight search dialog
    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.selected_index = None;
        self.scroll_to_selected = false;
        self.request_focus = false;
    }

    /// Check if the dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Record that a component was used
    /// Keeps track of the last 3 unique components
    pub fn record_component_usage(&mut self, component_id: String) {
        // Remove the component if it already exists (to move it to front)
        self.recent_components.retain(|id| id != &component_id);

        // Add to front
        self.recent_components.push_front(component_id);

        // Keep only the last 3
        while self.recent_components.len() > 3 {
            self.recent_components.pop_back();
        }
    }

    /// Show the spotlight search dialog
    /// Returns an action if the user selected a component
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        registry: &ComponentRegistry,
        component_manager: &Arc<Mutex<ComponentManager>>,
        mouse_pos: Option<egui::Pos2>,
    ) -> Option<SpotlightAction> {
        if !self.visible {
            return None;
        }

        let mut action = None;
        let mut close_dialog = false;

        egui::Window::new("Quick Add")
            .id(egui::Id::new("spotlight_search"))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(600.0)
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                stroke: egui::Stroke::new(2.0, ctx.style().visuals.widgets.active.bg_fill),
                corner_radius: egui::CornerRadius::same(10),
                inner_margin: egui::Margin::same(16),
                outer_margin: egui::Margin::same(0),
                shadow: egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 16,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(100),
                },
            })
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Handle keyboard input BEFORE TextEdit to prevent it from consuming events
                    // We need to use input_mut to consume the events
                    let mut handle_navigation = false;
                    let mut nav_down = false;
                    let mut nav_up = false;
                    let mut handle_enter = false;
                    let mut handle_escape = false;

                    ui.input_mut(|i| {
                        if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                            handle_escape = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                            handle_navigation = true;
                            nav_down = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                            handle_navigation = true;
                            nav_up = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                            handle_enter = true;
                        }
                    });

                    // Apply navigation after consuming the input
                    if handle_escape {
                        close_dialog = true;
                    }

                    if handle_navigation {
                        // Get combined list to calculate bounds
                        let (recent, filtered) = self.get_recent_and_filtered_components(registry, component_manager);
                        let total_count = recent.len() + filtered.len();

                        if nav_down {
                            if let Some(idx) = self.selected_index {
                                if idx + 1 < total_count {
                                    self.selected_index = Some(idx + 1);
                                    self.scroll_to_selected = true;
                                }
                            } else if total_count > 0 {
                                self.selected_index = Some(0);
                                self.scroll_to_selected = true;
                            }
                        } else if nav_up {
                            if let Some(idx) = self.selected_index {
                                if idx > 0 {
                                    self.selected_index = Some(idx - 1);
                                    self.scroll_to_selected = true;
                                }
                            } else if total_count > 0 {
                                self.selected_index = Some(0);
                                self.scroll_to_selected = true;
                            }
                        }
                    }

                    if handle_enter {
                        // Add the selected component at mouse position
                        if let Some(idx) = self.selected_index {
                            let (recent, filtered) = self.get_recent_and_filtered_components(registry, component_manager);
                            let all_components: Vec<_> = recent.iter().chain(filtered.iter()).collect();
                            if let Some(spec) = all_components.get(idx) {
                                let position = mouse_pos.unwrap_or(egui::Pos2::new(400.0, 300.0));
                                self.record_component_usage(spec.id.clone());
                                action = Some(SpotlightAction::AddComponent {
                                    spec: (*spec).clone(),
                                    position,
                                });
                                close_dialog = true;
                            }
                        }
                    }

                    // Search input box
                    let search_response = ui.add_sized(
                        egui::vec2(ui.available_width(), 30.0),
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Type to search components..."),
                    );

                    // Auto-focus the search input when opened
                    if self.request_focus {
                        search_response.request_focus();
                        self.request_focus = false;
                    }

                    ui.add_space(8.0);

                    // Results list
                    let (recent, filtered) = self.get_recent_and_filtered_components(registry, component_manager);
                    let total_count = recent.len() + filtered.len();

                    // Update selected index if out of bounds
                    if let Some(idx) = self.selected_index {
                        if idx >= total_count {
                            self.selected_index = if total_count == 0 {
                                None
                            } else {
                                Some(total_count - 1)
                            };
                        }
                    }

                    // Show results in scrollable area
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .id_salt("spotlight_results")
                        .show(ui, |ui| {
                            if total_count == 0 {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(30.0);
                                    ui.label("No components found");
                                    ui.add_space(30.0);
                                });
                            } else {
                                let mut current_idx = 0;

                                // Show recent components first
                                if !recent.is_empty() {
                                    ui.label(
                                        egui::RichText::new("Last Used")
                                            .size(12.0)
                                            .color(ctx.style().visuals.weak_text_color()),
                                    );
                                    ui.add_space(4.0);

                                    for spec in recent.iter() {
                                        let is_selected = self.selected_index == Some(current_idx);

                                        // Create a selectable row
                                        let row_response = ui.horizontal(|ui| {
                                            ui.set_min_width(ui.available_width());

                                            // Background highlight for selected item
                                            if is_selected {
                                                let rect = ui.available_rect_before_wrap();
                                                ui.painter().rect_filled(
                                                    rect,
                                                    egui::CornerRadius::same(4),
                                                    ctx.style().visuals.selection.bg_fill,
                                                );
                                            }

                                            ui.vertical(|ui| {
                                                // Component name (bold)
                                                let name_text = if is_selected {
                                                    egui::RichText::new(&spec.name).strong().size(14.0)
                                                } else {
                                                    egui::RichText::new(&spec.name).size(14.0)
                                                };
                                                ui.label(name_text);

                                                // Category and description
                                                let category = spec
                                                    .category
                                                    .clone()
                                                    .unwrap_or_else(|| "Other".to_string());
                                                let meta_text =
                                                    format!("{} · {}", category, spec.description);
                                                ui.label(
                                                    egui::RichText::new(meta_text)
                                                        .size(11.0)
                                                        .color(ctx.style().visuals.weak_text_color()),
                                                );
                                            });
                                        });

                                        // Scroll to selected item if needed
                                        if is_selected && self.scroll_to_selected {
                                            row_response
                                                .response
                                                .scroll_to_me(Some(egui::Align::Center));
                                            self.scroll_to_selected = false;
                                        }

                                        // Handle click
                                        if row_response.response.clicked() {
                                            let position =
                                                mouse_pos.unwrap_or(egui::Pos2::new(400.0, 300.0));
                                            self.record_component_usage(spec.id.clone());
                                            action = Some(SpotlightAction::AddComponent {
                                                spec: (*spec).clone(),
                                                position,
                                            });
                                            close_dialog = true;
                                        }

                                        // Hover effect
                                        if row_response.response.hovered() {
                                            self.selected_index = Some(current_idx);
                                        }

                                        ui.add_space(4.0);
                                        current_idx += 1;
                                    }

                                    // Separator between recent and search results
                                    ui.add_space(4.0);
                                    ui.separator();
                                    ui.add_space(4.0);
                                }

                                // Show filtered search results
                                for spec in filtered.iter() {
                                    let is_selected = self.selected_index == Some(current_idx);

                                    // Create a selectable row
                                    let row_response = ui.horizontal(|ui| {
                                        ui.set_min_width(ui.available_width());

                                        // Background highlight for selected item
                                        if is_selected {
                                            let rect = ui.available_rect_before_wrap();
                                            ui.painter().rect_filled(
                                                rect,
                                                egui::CornerRadius::same(4),
                                                ctx.style().visuals.selection.bg_fill,
                                            );
                                        }

                                        ui.vertical(|ui| {
                                            // Component name (bold)
                                            let name_text = if is_selected {
                                                egui::RichText::new(&spec.name).strong().size(14.0)
                                            } else {
                                                egui::RichText::new(&spec.name).size(14.0)
                                            };
                                            ui.label(name_text);

                                            // Category and description
                                            let category = spec
                                                .category
                                                .clone()
                                                .unwrap_or_else(|| "Other".to_string());
                                            let meta_text =
                                                format!("{} · {}", category, spec.description);
                                            ui.label(
                                                egui::RichText::new(meta_text)
                                                    .size(11.0)
                                                    .color(ctx.style().visuals.weak_text_color()),
                                            );
                                        });
                                    });

                                    // Scroll to selected item if needed
                                    if is_selected && self.scroll_to_selected {
                                        row_response
                                            .response
                                            .scroll_to_me(Some(egui::Align::Center));
                                        self.scroll_to_selected = false;
                                    }

                                    // Handle click
                                    if row_response.response.clicked() {
                                        let position =
                                            mouse_pos.unwrap_or(egui::Pos2::new(400.0, 300.0));
                                        self.record_component_usage(spec.id.clone());
                                        action = Some(SpotlightAction::AddComponent {
                                            spec: (*spec).clone(),
                                            position,
                                        });
                                        close_dialog = true;
                                    }

                                    // Hover effect
                                    if row_response.response.hovered() {
                                        self.selected_index = Some(current_idx);
                                    }

                                    ui.add_space(4.0);
                                    current_idx += 1;
                                }
                            }
                        });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Footer with count and hint
                    ui.horizontal(|ui| {
                        let count = total_count;
                        ui.label(
                            egui::RichText::new(format!(
                                "{} component{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ))
                            .size(11.0)
                            .color(ctx.style().visuals.weak_text_color()),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("↑↓ Navigate · Enter Add · Esc Close")
                                    .size(10.0)
                                    .color(ctx.style().visuals.weak_text_color()),
                            );
                        });
                    });
                });
            });

        if close_dialog {
            self.close();
        }

        action
    }

    /// Get recent components and filtered search results
    /// Returns (recent_components, filtered_results)
    /// Recent components are excluded from filtered results to avoid duplication
    fn get_recent_and_filtered_components<'a>(
        &self,
        registry: &'a ComponentRegistry,
        component_manager: &Arc<Mutex<ComponentManager>>,
    ) -> (Vec<&'a ComponentSpec>, Vec<&'a ComponentSpec>) {
        // Get recent components that are still available
        let recent: Vec<&ComponentSpec> = self
            .recent_components
            .iter()
            .filter_map(|id| registry.get_by_id(id))
            .filter(|spec| {
                // Check if component is still available (builtin or loaded in ComponentManager)
                if spec.id.starts_with("builtin:") {
                    true
                } else {
                    let cm = match component_manager.lock() {
                        Ok(cm) => cm,
                        Err(_) => return false,
                    };
                    cm.has_component(&spec.id)
                }
            })
            .collect();

        // Get filtered results
        let mut filtered = self.get_filtered_components(registry, component_manager);

        // Remove recent components from filtered results to avoid duplication
        let recent_ids: HashSet<_> = recent.iter().map(|spec| &spec.id).collect();
        filtered.retain(|spec| !recent_ids.contains(&spec.id));

        (recent, filtered)
    }

    /// Get filtered and sorted components based on search query
    /// Only returns components that are actually loaded in ComponentManager
    fn get_filtered_components<'a>(
        &self,
        registry: &'a ComponentRegistry,
        component_manager: &Arc<Mutex<ComponentManager>>,
    ) -> Vec<&'a ComponentSpec> {
        // Get available component IDs from ComponentManager
        let available_components = {
            let cm = match component_manager.lock() {
                Ok(cm) => cm,
                Err(e) => {
                    log::error!("Failed to lock ComponentManager in spotlight: {}", e);
                    return Vec::new();
                }
            };

            // Filter to only components that are actually loaded in ComponentManager
            registry
                .list_all()
                .into_iter()
                .filter(|spec| {
                    // Builtin components don't need to be in ComponentManager
                    if spec.id.starts_with("builtin:") {
                        true
                    } else {
                        // User components must be loaded in ComponentManager
                        cm.has_component(&spec.id)
                    }
                })
                .collect::<Vec<_>>()
        };

        if self.query.is_empty() {
            // Show all available components sorted by name when no query
            let mut all = available_components;
            all.sort_by(|a, b| a.name.cmp(&b.name));
            return all;
        }

        let query = self.query.to_lowercase();
        let mut results: Vec<(&ComponentSpec, i32)> = available_components
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
    /// Searches across name, description, and category
    /// Higher score = better match
    fn fuzzy_match_score(&self, spec: &ComponentSpec, query: &str) -> i32 {
        let mut score = 0;

        let name = spec.name.to_lowercase();
        let description = spec.description.to_lowercase();
        let category = spec.category.clone().unwrap_or_default().to_lowercase();

        // Name matching (highest priority)
        if name == query {
            score += 1000; // Exact match
        } else if name.starts_with(query) {
            score += 500; // Starts with query
        } else if name.contains(query) {
            score += 250; // Contains query
        }

        // Category matching (medium priority)
        if category == query {
            score += 200; // Exact category match
        } else if category.contains(query) {
            score += 100; // Category contains query
        }

        // Description matching (lower priority)
        if description.contains(query) {
            score += 100;
        }

        // Fuzzy character sequence matching (lowest priority)
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
}

impl Default for SpotlightSearch {
    fn default() -> Self {
        Self::new()
    }
}
