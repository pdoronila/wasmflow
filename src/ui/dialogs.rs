//! UI dialogs
//!
//! This module contains various dialog windows for user interaction.

use crate::runtime::capabilities::{CapabilityGrant, CapabilitySet, RiskLevel};
use eframe::egui;

/// Result of the unsaved changes dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChangesAction {
    /// User chose to save the changes
    Save,
    /// User chose to discard the changes
    Discard,
    /// User chose to cancel the operation
    Cancel,
}

/// Dialog shown when closing with unsaved changes
pub struct UnsavedChangesDialog {
    /// Whether the dialog is open
    is_open: bool,
    /// The result of the dialog (if any)
    result: Option<UnsavedChangesAction>,
}

impl UnsavedChangesDialog {
    /// Create a new unsaved changes dialog
    pub fn new() -> Self {
        Self {
            is_open: false,
            result: None,
        }
    }

    /// Open the dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.result = None;
    }

    /// Check if the dialog is open
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the result of the dialog
    #[allow(dead_code)]
    pub fn result(&self) -> Option<UnsavedChangesAction> {
        self.result
    }

    /// Reset the dialog
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.is_open = false;
        self.result = None;
    }

    /// Show the dialog and return the user's choice
    pub fn show(&mut self, ctx: &egui::Context) -> Option<UnsavedChangesAction> {
        if !self.is_open {
            return None;
        }

        let mut result = None;

        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label("You have unsaved changes.");
                    ui.label("Do you want to save before closing?");
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            result = Some(UnsavedChangesAction::Save);
                            self.is_open = false;
                        }
                        if ui.button("Discard").clicked() {
                            result = Some(UnsavedChangesAction::Discard);
                            self.is_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            result = Some(UnsavedChangesAction::Cancel);
                            self.is_open = false;
                        }
                    });
                    ui.add_space(10.0);
                });
            });

        self.result = result;
        result
    }
}

impl Default for UnsavedChangesDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// T072: Result of the permission dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionAction {
    /// User approved the requested permissions
    Approve,
    /// User approved with Full (unrestricted) access override
    ApproveAsFull,
    /// User denied the requested permissions
    Deny,
}

/// T072: Dialog for requesting user approval of component permissions
pub struct PermissionDialog {
    /// Whether the dialog is open
    is_open: bool,
    /// Component name requesting permissions
    component_name: String,
    /// Component description
    component_description: String,
    /// Requested capability set
    requested_capabilities: CapabilitySet,
    /// The result of the dialog (if any)
    result: Option<PermissionAction>,
    /// T081: Full access warning checkbox state
    full_access_acknowledged: bool,
    /// Index of currently selected element for keyboard navigation
    selected_index: Option<usize>,
}

impl PermissionDialog {
    /// Create a new permission dialog
    pub fn new() -> Self {
        Self {
            is_open: false,
            component_name: String::new(),
            component_description: String::new(),
            requested_capabilities: CapabilitySet::none(),
            result: None,
            full_access_acknowledged: false,
            selected_index: None,
        }
    }

    /// Open the dialog with component information
    pub fn open(&mut self, name: String, description: String, capabilities: CapabilitySet) {
        self.is_open = true;
        self.component_name = name;
        self.component_description = description;
        self.requested_capabilities = capabilities;
        self.result = None;
        self.full_access_acknowledged = false;
        self.selected_index = Some(0); // Default to first button (Approve)
    }

    /// Check if the dialog is open
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the result of the dialog
    #[allow(dead_code)]
    pub fn result(&self) -> Option<PermissionAction> {
        self.result
    }

    /// Reset the dialog
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.is_open = false;
        self.component_name.clear();
        self.component_description.clear();
        self.requested_capabilities = CapabilitySet::none();
        self.result = None;
        self.full_access_acknowledged = false;
        self.selected_index = None;
    }

    /// Show the dialog and return the user's choice
    /// T072: Permission dialog UI implementation
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PermissionAction> {
        if !self.is_open {
            return None;
        }

        let mut result = None;

        egui::Window::new("Permission Request")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Determine button visibility
                    let is_full_access = matches!(self.requested_capabilities, CapabilitySet::Full);

                    // Navigation order:
                    // 0: Approve button
                    // 1: Approve as Full Access button (only if !is_full_access)
                    // 2: Deny button
                    // 3: Checkbox
                    let total_elements = if is_full_access { 3 } else { 4 };

                    // Keyboard navigation handling (before UI rendering)
                    let mut handle_navigation = false;
                    let mut nav_forward = false;
                    let mut nav_backward = false;
                    let mut handle_enter = false;
                    let mut handle_space = false;
                    let mut handle_escape = false;

                    ui.input_mut(|i| {
                        if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                            handle_escape = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                            || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) {
                            handle_navigation = true;
                            nav_forward = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                            || i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab) {
                            handle_navigation = true;
                            nav_backward = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                            handle_enter = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Space) {
                            handle_space = true;
                        }
                    });

                    // Apply navigation
                    if handle_escape {
                        result = Some(PermissionAction::Deny);
                        self.is_open = false;
                    }

                    if handle_navigation {
                        if nav_forward {
                            if let Some(idx) = self.selected_index {
                                self.selected_index = Some((idx + 1) % total_elements);
                            } else {
                                self.selected_index = Some(0);
                            }
                        } else if nav_backward {
                            if let Some(idx) = self.selected_index {
                                self.selected_index = Some(if idx == 0 {
                                    total_elements - 1
                                } else {
                                    idx - 1
                                });
                            } else {
                                self.selected_index = Some(0);
                            }
                        }
                    }

                    // Handle Enter key - activate selected button
                    if handle_enter {
                        if let Some(idx) = self.selected_index {
                            let can_approve = !is_full_access || self.full_access_acknowledged;
                            match idx {
                                0 => {
                                    // Approve button
                                    if can_approve {
                                        result = Some(PermissionAction::Approve);
                                        self.is_open = false;
                                    }
                                }
                                1 => {
                                    if is_full_access {
                                        // Deny button (no Approve as Full button)
                                        result = Some(PermissionAction::Deny);
                                        self.is_open = false;
                                    } else {
                                        // Approve as Full Access button
                                        if self.full_access_acknowledged {
                                            result = Some(PermissionAction::ApproveAsFull);
                                            self.is_open = false;
                                        }
                                    }
                                }
                                2 => {
                                    if is_full_access {
                                        // Checkbox
                                        self.full_access_acknowledged = !self.full_access_acknowledged;
                                    } else {
                                        // Deny button
                                        result = Some(PermissionAction::Deny);
                                        self.is_open = false;
                                    }
                                }
                                3 => {
                                    // Checkbox (only when !is_full_access)
                                    self.full_access_acknowledged = !self.full_access_acknowledged;
                                }
                                _ => {}
                            }
                        }
                    }

                    // Handle Space key - toggle checkbox if selected
                    if handle_space {
                        if let Some(idx) = self.selected_index {
                            let checkbox_idx = if is_full_access { 2 } else { 3 };
                            if idx == checkbox_idx {
                                self.full_access_acknowledged = !self.full_access_acknowledged;
                            }
                        }
                    }

                    ui.add_space(10.0);

                    // Component info header
                    ui.heading(&self.component_name);
                    ui.label(&self.component_description);
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Permission request message
                    ui.label("This component is requesting the following permissions:");
                    ui.add_space(10.0);

                    // Show requested capabilities with risk indicator
                    self.show_capabilities(ui);

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Warning message based on risk level
                    if let Some(risk) = self.requested_capabilities.max_risk_level() {
                        self.show_risk_warning(ui, risk);
                        ui.add_space(10.0);
                    }

                    // T081: Special warning for Full access with explicit acknowledgment
                    // Show Full Access warning for either:
                    // 1. Components requesting Full access
                    // 2. Users wanting to override with Full access
                    if is_full_access || !matches!(self.requested_capabilities, CapabilitySet::Full)
                    {
                        ui.separator();
                        ui.add_space(10.0);

                        // Red warning box
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(80, 20, 20))
                            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 80, 80)))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 100, 100),
                                        "⚠ CRITICAL SECURITY WARNING ⚠",
                                    );
                                    ui.add_space(5.0);
                                    if is_full_access {
                                        ui.label(
                                            "This component requests UNRESTRICTED system access.",
                                        );
                                    } else {
                                        ui.label("Full Access grants UNRESTRICTED system access.");
                                    }
                                    ui.label(
                                        "It can read/write ANY files, access ANY network, and",
                                    );
                                    ui.label("read environment variables. Only approve if you");
                                    ui.label("FULLY TRUST this component's author and source.");
                                });
                            });

                        ui.add_space(10.0);

                        // Explicit acknowledgment checkbox
                        let checkbox_idx = if is_full_access { 2 } else { 3 };
                        let is_checkbox_selected = self.selected_index == Some(checkbox_idx);

                        ui.horizontal(|ui| {
                            // Add visual highlight for selected checkbox
                            if is_checkbox_selected {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(
                                    rect,
                                    egui::CornerRadius::same(4),
                                    ui.visuals().selection.bg_fill,
                                );
                            }

                            ui.checkbox(
                                &mut self.full_access_acknowledged,
                                "I understand the security risks and trust this component",
                            );
                        });

                        ui.add_space(10.0);
                    }

                    // Action buttons
                    ui.horizontal(|ui| {
                        ui.add_space(50.0);

                        // Disable Approve button if Full access not acknowledged
                        let can_approve = !is_full_access || self.full_access_acknowledged;

                        // Approve button (index 0)
                        let is_approve_selected = self.selected_index == Some(0);
                        ui.add_enabled_ui(can_approve, |ui| {
                            let button = egui::Button::new("✓ Approve");
                            let button_response = if is_approve_selected {
                                ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                            } else {
                                ui.add(button)
                            };

                            if button_response.clicked() {
                                result = Some(PermissionAction::Approve);
                                self.is_open = false;
                            }
                        });

                        // Add "Approve as Full" button for advanced users who want unrestricted access
                        if !is_full_access {
                            // Approve as Full button (index 1)
                            let is_approve_full_selected = self.selected_index == Some(1);
                            ui.add_enabled_ui(self.full_access_acknowledged, |ui| {
                                let button = egui::Button::new("✓ Approve as Full Access");
                                let button_response = if is_approve_full_selected {
                                    ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                                } else {
                                    ui.add(button)
                                };

                                if button_response.clicked() {
                                    result = Some(PermissionAction::ApproveAsFull);
                                    self.is_open = false;
                                }
                            });
                            if !self.full_access_acknowledged {
                                ui.label("↑").on_hover_text(
                                    "Check the box above to enable Full Access override",
                                );
                            }
                        }

                        // Deny button (index 1 if is_full_access, index 2 otherwise)
                        let deny_idx = if is_full_access { 1 } else { 2 };
                        let is_deny_selected = self.selected_index == Some(deny_idx);
                        let button = egui::Button::new("✗ Deny");
                        let button_response = if is_deny_selected {
                            ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                        } else {
                            ui.add(button)
                        };

                        if button_response.clicked() {
                            result = Some(PermissionAction::Deny);
                            self.is_open = false;
                        }
                    });
                    ui.add_space(10.0);
                });
            });

        self.result = result;
        result
    }

    /// Show the capabilities list with descriptions
    fn show_capabilities(&self, ui: &mut egui::Ui) {
        let _description = self.requested_capabilities.description();

        ui.group(|ui| {
            ui.set_min_width(400.0);

            match &self.requested_capabilities {
                CapabilitySet::None => {
                    ui.label("• No system access (pure computation)");
                }
                CapabilitySet::FileRead { paths } => {
                    ui.label("• Read files from:");
                    for path in paths {
                        ui.label(format!("  📁 {}", path.display()));
                    }
                }
                CapabilitySet::FileWrite { paths } => {
                    ui.label("• Write files to:");
                    for path in paths {
                        ui.label(format!("  📁 {}", path.display()));
                    }
                }
                CapabilitySet::FileReadWrite { paths } => {
                    ui.label("• Read and write files in:");
                    for path in paths {
                        ui.label(format!("  📁 {}", path.display()));
                    }
                }
                CapabilitySet::Network { allowed_hosts } => {
                    ui.label("• Network access to:");
                    for host in allowed_hosts {
                        ui.label(format!("  🌐 {}", host));
                    }
                }
                CapabilitySet::Full => {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        "⚠ UNRESTRICTED SYSTEM ACCESS",
                    );
                    ui.label("This component can:");
                    ui.label("  • Read and write any files");
                    ui.label("  • Access any network resources");
                    ui.label("  • Read environment variables");
                }
            }
        });
    }

    /// Show risk warning based on capability risk level
    fn show_risk_warning(&self, ui: &mut egui::Ui, risk: RiskLevel) {
        match risk {
            RiskLevel::High => {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 80, 80),
                    "⚠ High Risk: This component can modify system files or has unrestricted access."
                );
            }
            RiskLevel::Medium => {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 180, 0),
                    "⚡ Medium Risk: This component can read files or access network resources.",
                );
            }
            RiskLevel::Low => {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 100),
                    "✓ Low Risk: This component has limited system access.",
                );
            }
        }
    }
}

impl Default for PermissionDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Action from the permissions view dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionViewAction {
    /// User wants to revoke permissions
    Revoke,
    /// User wants to upgrade to Full access
    UpgradeToFull,
}

/// T078: Dialog for viewing and managing node permissions
pub struct PermissionsViewDialog {
    /// Whether the dialog is open
    is_open: bool,
    /// Node ID being viewed
    node_id: Option<uuid::Uuid>,
    /// Node name
    node_name: String,
    /// Current capability grant (if any)
    capability_grant: Option<CapabilityGrant>,
    /// Action requested by user
    requested_action: Option<PermissionViewAction>,
    /// Acknowledgment for Full access upgrade
    full_access_acknowledged: bool,
    /// Index of currently selected element for keyboard navigation
    selected_index: Option<usize>,
}

impl PermissionsViewDialog {
    /// Create a new permissions view dialog
    pub fn new() -> Self {
        Self {
            is_open: false,
            node_id: None,
            node_name: String::new(),
            capability_grant: None,
            requested_action: None,
            full_access_acknowledged: false,
            selected_index: None,
        }
    }

    /// Open the dialog with node permissions
    pub fn open(&mut self, node_id: uuid::Uuid, node_name: String, grant: Option<CapabilityGrant>) {
        self.is_open = true;
        self.node_id = Some(node_id);
        self.node_name = node_name;
        self.capability_grant = grant;
        self.requested_action = None;
        self.full_access_acknowledged = false;
        self.selected_index = Some(0); // Default to first button
    }

    /// Check if the dialog is open
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the requested action (if any)
    pub fn take_action(&mut self) -> Option<PermissionViewAction> {
        self.requested_action.take()
    }

    /// Get the node ID
    pub fn node_id(&self) -> Option<uuid::Uuid> {
        self.node_id
    }

    /// Reset the dialog
    pub fn reset(&mut self) {
        self.is_open = false;
        self.node_id = None;
        self.node_name.clear();
        self.capability_grant = None;
        self.requested_action = None;
        self.full_access_acknowledged = false;
        self.selected_index = None;
    }

    /// Show the dialog
    /// T078: Permissions view dialog UI implementation
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut close_dialog = false;

        egui::Window::new("Node Permissions")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Determine dialog state and navigation elements
                    let has_grant = self.capability_grant.is_some();
                    let is_full_access = if let Some(ref grant) = self.capability_grant {
                        matches!(grant.capability_set, CapabilitySet::Full)
                    } else {
                        false
                    };

                    // Navigation order:
                    // With grant, !is_full_access: 0=Upgrade button, 1=Revoke button, 2=Close button, 3=Checkbox
                    // With grant, is_full_access: 0=Revoke button, 1=Close button
                    // No grant: 0=Close button
                    let total_elements = if has_grant {
                        if is_full_access {
                            2 // Revoke, Close
                        } else {
                            4 // Upgrade, Revoke, Close, Checkbox
                        }
                    } else {
                        1 // Just Close
                    };

                    // Keyboard navigation handling (before UI rendering)
                    let mut handle_navigation = false;
                    let mut nav_forward = false;
                    let mut nav_backward = false;
                    let mut handle_enter = false;
                    let mut handle_space = false;
                    let mut handle_escape = false;

                    ui.input_mut(|i| {
                        if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                            handle_escape = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                            || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) {
                            handle_navigation = true;
                            nav_forward = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                            || i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab) {
                            handle_navigation = true;
                            nav_backward = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                            handle_enter = true;
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Space) {
                            handle_space = true;
                        }
                    });

                    // Apply navigation
                    if handle_escape {
                        close_dialog = true;
                    }

                    if handle_navigation {
                        if nav_forward {
                            if let Some(idx) = self.selected_index {
                                self.selected_index = Some((idx + 1) % total_elements);
                            } else {
                                self.selected_index = Some(0);
                            }
                        } else if nav_backward {
                            if let Some(idx) = self.selected_index {
                                self.selected_index = Some(if idx == 0 {
                                    total_elements - 1
                                } else {
                                    idx - 1
                                });
                            } else {
                                self.selected_index = Some(0);
                            }
                        }
                    }

                    // Handle Enter key - activate selected button
                    if handle_enter {
                        if let Some(idx) = self.selected_index {
                            if has_grant {
                                if is_full_access {
                                    // Navigation: 0=Revoke, 1=Close
                                    match idx {
                                        0 => {
                                            // Revoke button
                                            self.requested_action = Some(PermissionViewAction::Revoke);
                                            close_dialog = true;
                                        }
                                        1 => {
                                            // Close button
                                            close_dialog = true;
                                        }
                                        _ => {}
                                    }
                                } else {
                                    // Navigation: 0=Upgrade, 1=Revoke, 2=Close, 3=Checkbox
                                    match idx {
                                        0 => {
                                            // Upgrade button
                                            if self.full_access_acknowledged {
                                                self.requested_action = Some(PermissionViewAction::UpgradeToFull);
                                                close_dialog = true;
                                            }
                                        }
                                        1 => {
                                            // Revoke button
                                            self.requested_action = Some(PermissionViewAction::Revoke);
                                            close_dialog = true;
                                        }
                                        2 => {
                                            // Close button
                                            close_dialog = true;
                                        }
                                        3 => {
                                            // Checkbox
                                            self.full_access_acknowledged = !self.full_access_acknowledged;
                                        }
                                        _ => {}
                                    }
                                }
                            } else {
                                // No grant - just close button at index 0
                                if idx == 0 {
                                    close_dialog = true;
                                }
                            }
                        }
                    }

                    // Handle Space key - toggle checkbox if selected
                    if handle_space {
                        if let Some(idx) = self.selected_index {
                            // Checkbox is at index 3 when !is_full_access
                            if has_grant && !is_full_access && idx == 3 {
                                self.full_access_acknowledged = !self.full_access_acknowledged;
                            }
                        }
                    }

                    ui.add_space(10.0);

                    // Node info header
                    ui.heading(&self.node_name);
                    if let Some(node_id) = self.node_id {
                        ui.label(format!("Node ID: {}", node_id));
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Show current permissions
                    if let Some(ref grant) = self.capability_grant {
                        ui.label("Current Permissions:");
                        ui.add_space(5.0);

                        // Show capability details
                        self.show_capability_details(ui, &grant.capability_set);

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(5.0);

                        // Show grant metadata
                        ui.label(format!("Granted: {}", grant.granted_at));
                        ui.label(format!("Scope: {}", grant.scope));

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Show upgrade option if not already Full access
                        let is_full_access = matches!(grant.capability_set, CapabilitySet::Full);
                        if !is_full_access {
                            // Warning box for upgrading to Full access
                            egui::Frame::new()
                                .fill(egui::Color32::from_rgb(80, 20, 20))
                                .stroke(egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgb(255, 80, 80),
                                ))
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 100, 100),
                                            "⚠ UPGRADE TO FULL ACCESS ⚠",
                                        );
                                        ui.add_space(5.0);
                                        ui.label("Full Access grants UNRESTRICTED system access.");
                                        ui.label(
                                            "It can read/write ANY files, access ANY network, and",
                                        );
                                        ui.label("read environment variables. Only approve if you");
                                        ui.label("FULLY TRUST this component's author and source.");
                                    });
                                });

                            ui.add_space(10.0);

                            // Explicit acknowledgment checkbox
                            let is_checkbox_selected = self.selected_index == Some(3);
                            ui.horizontal(|ui| {
                                // Add visual highlight for selected checkbox
                                if is_checkbox_selected {
                                    let rect = ui.available_rect_before_wrap();
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::CornerRadius::same(4),
                                        ui.visuals().selection.bg_fill,
                                    );
                                }

                                ui.checkbox(
                                    &mut self.full_access_acknowledged,
                                    "I understand the security risks and trust this component",
                                );
                            });

                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(10.0);
                        }

                        // Action buttons
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);

                            // Upgrade to Full button (if not already Full)
                            if !is_full_access {
                                // Upgrade button is at index 0
                                let is_upgrade_selected = self.selected_index == Some(0);
                                ui.add_enabled_ui(self.full_access_acknowledged, |ui| {
                                    let button = egui::Button::new("🔓 Upgrade to Full Access");
                                    let button_response = if is_upgrade_selected {
                                        ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                                    } else {
                                        ui.add(button)
                                    };

                                    if button_response.clicked() {
                                        self.requested_action =
                                            Some(PermissionViewAction::UpgradeToFull);
                                        close_dialog = true;
                                    }
                                });
                            }

                            // Revoke button (index 0 if is_full_access, index 1 otherwise)
                            let revoke_idx = if is_full_access { 0 } else { 1 };
                            let is_revoke_selected = self.selected_index == Some(revoke_idx);
                            let button = egui::Button::new("🔒 Revoke Permissions");
                            let button_response = if is_revoke_selected {
                                ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                            } else {
                                ui.add(button)
                            };

                            if button_response.clicked() {
                                self.requested_action = Some(PermissionViewAction::Revoke);
                                close_dialog = true;
                            }

                            // Close button (index 1 if is_full_access, index 2 otherwise)
                            let close_idx = if is_full_access { 1 } else { 2 };
                            let is_close_selected = self.selected_index == Some(close_idx);
                            let button = egui::Button::new("Close");
                            let button_response = if is_close_selected {
                                ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                            } else {
                                ui.add(button)
                            };

                            if button_response.clicked() {
                                close_dialog = true;
                            }
                        });
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 150, 50),
                            "⚠ No permissions granted",
                        );
                        ui.add_space(5.0);
                        ui.label("This node does not have any capability grants.");
                        ui.label("It will not be able to execute.");

                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.add_space(150.0);

                            // Close button is at index 0 when no grant
                            let is_close_selected = self.selected_index == Some(0);
                            let button = egui::Button::new("Close");
                            let button_response = if is_close_selected {
                                ui.add(button.stroke(egui::Stroke::new(2.0, ui.visuals().selection.stroke.color)))
                            } else {
                                ui.add(button)
                            };

                            if button_response.clicked() {
                                close_dialog = true;
                            }
                        });
                    }

                    ui.add_space(10.0);
                });
            });

        if close_dialog {
            self.is_open = false;
        }
    }

    /// Show capability details
    fn show_capability_details(&self, ui: &mut egui::Ui, capabilities: &CapabilitySet) {
        ui.group(|ui| {
            ui.set_min_width(400.0);

            match capabilities {
                CapabilitySet::None => {
                    ui.label("• No system access (pure computation)");
                }
                CapabilitySet::FileRead { paths } => {
                    ui.label("• Read files from:");
                    for path in paths {
                        ui.label(format!("  📁 {}", path.display()));
                    }
                }
                CapabilitySet::FileWrite { paths } => {
                    ui.label("• Write files to:");
                    for path in paths {
                        ui.label(format!("  📁 {}", path.display()));
                    }
                }
                CapabilitySet::FileReadWrite { paths } => {
                    ui.label("• Read and write files in:");
                    for path in paths {
                        ui.label(format!("  📁 {}", path.display()));
                    }
                }
                CapabilitySet::Network { allowed_hosts } => {
                    ui.label("• Network access to:");
                    for host in allowed_hosts {
                        ui.label(format!("  🌐 {}", host));
                    }
                }
                CapabilitySet::Full => {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        "⚠ UNRESTRICTED SYSTEM ACCESS",
                    );
                    ui.label("This component can:");
                    ui.label("  • Read and write any files");
                    ui.label("  • Access any network resources");
                    ui.label("  • Read environment variables");
                }
            }
        });
    }
}

impl Default for PermissionsViewDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// T100: About dialog showing application information
pub struct AboutDialog {
    /// Whether the dialog is open
    is_open: bool,
    /// Cached texture for WASM composition image
    wasm_image_texture: Option<egui::TextureHandle>,
}

impl AboutDialog {
    /// Create a new about dialog
    pub fn new() -> Self {
        Self {
            is_open: false,
            wasm_image_texture: None,
        }
    }

    /// Open the dialog
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Check if the dialog is open
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Show the dialog
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut close_dialog = false;

        egui::Window::new("About WasmFlow")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    // Application name and version
                    ui.heading("WasmFlow");
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(10.0);

                    // Description
                    ui.label("WebAssembly Node-Based Visual Composition System");
                    ui.add_space(15.0);

                    // WASM Composition image
                    if self.wasm_image_texture.is_none() {
                        // Load and decode the image
                        let image_bytes = include_bytes!("../../assets/wasm_composition.png");
                        if let Ok(image) = image::load_from_memory(image_bytes) {
                            let size = [image.width() as usize, image.height() as usize];
                            let image_buffer = image.to_rgba8();
                            let pixels = image_buffer.as_flat_samples();
                            let color_image =
                                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                            self.wasm_image_texture = Some(ctx.load_texture(
                                "wasm_composition",
                                color_image,
                                Default::default(),
                            ));
                        }
                    }

                    if let Some(texture) = &self.wasm_image_texture {
                        ui.add(egui::Image::new(texture).max_width(400.0));
                    }
                    ui.add_space(15.0);

                    // Build information
                    ui.group(|ui| {
                        ui.set_min_width(400.0);
                        ui.label("Build Information:");
                        ui.add_space(5.0);
                        ui.label(format!("Rust Version: {}", env!("CARGO_PKG_RUST_VERSION")));
                        ui.label(format!(
                            "Build Profile: {}",
                            if cfg!(debug_assertions) {
                                "Debug"
                            } else {
                                "Release"
                            }
                        ));
                        ui.label(format!("Target: {}", std::env::consts::ARCH));
                    });

                    ui.add_space(15.0);

                    // Key dependencies
                    ui.group(|ui| {
                        ui.set_min_width(400.0);
                        ui.label("Key Dependencies:");
                        ui.add_space(5.0);
                        ui.label("• egui - Immediate mode GUI");
                        ui.label("• wasmtime - WebAssembly runtime");
                        ui.label("• petgraph - Graph data structures");
                        ui.label("• serde - Serialization framework");
                        ui.label("• WAC - WebAssembly Composition");
                    });

                    ui.add_space(15.0);

                    // Links
                    ui.horizontal(|ui| {
                        if ui.link("GitHub Repository").clicked() {
                            // TODO: Open browser to GitHub repo
                            log::info!("GitHub link clicked");
                        }
                        ui.separator();
                        if ui.link("Documentation").clicked() {
                            // TODO: Open browser to docs
                            log::info!("Documentation link clicked");
                        }
                    });

                    ui.add_space(15.0);

                    // Copyright
                    ui.label("© 2025 WasmFlow Project");

                    ui.add_space(15.0);

                    // Close button
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }

                    ui.add_space(10.0);
                });
            });

        if close_dialog {
            self.is_open = false;
        }
    }
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// T092: Dialog for editing graph metadata
pub struct GraphMetadataDialog {
    /// Whether the dialog is open
    is_open: bool,
    /// Edited graph name
    name: String,
    /// Edited author
    author: String,
    /// Edited description
    description: String,
    /// Created timestamp (read-only)
    created_at: String,
    /// Modified timestamp (read-only)
    modified_at: String,
    /// Whether the user saved changes
    saved: bool,
}

impl GraphMetadataDialog {
    /// Create a new graph metadata dialog
    pub fn new() -> Self {
        Self {
            is_open: false,
            name: String::new(),
            author: String::new(),
            description: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
            saved: false,
        }
    }

    /// Open the dialog with current graph metadata
    pub fn open(
        &mut self,
        name: String,
        author: String,
        description: String,
        created_at: String,
        modified_at: String,
    ) {
        self.is_open = true;
        self.name = name;
        self.author = author;
        self.description = description;
        self.created_at = created_at;
        self.modified_at = modified_at;
        self.saved = false;
    }

    /// Check if the dialog is open
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Check if changes were saved
    pub fn saved(&self) -> bool {
        self.saved
    }

    /// Get the edited metadata (name, author, description)
    pub fn get_metadata(&self) -> (String, String, String) {
        (
            self.name.clone(),
            self.author.clone(),
            self.description.clone(),
        )
    }

    /// Reset the dialog
    pub fn reset(&mut self) {
        self.is_open = false;
        self.name.clear();
        self.author.clear();
        self.description.clear();
        self.created_at.clear();
        self.modified_at.clear();
        self.saved = false;
    }

    /// Show the dialog
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut close_dialog = false;

        egui::Window::new("Graph Metadata")
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);

                    // Editable fields
                    ui.label("Graph Name:");
                    ui.text_edit_singleline(&mut self.name);
                    ui.add_space(10.0);

                    ui.label("Author:");
                    ui.text_edit_singleline(&mut self.author);
                    ui.add_space(10.0);

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.description);
                    ui.add_space(10.0);

                    ui.separator();
                    ui.add_space(10.0);

                    // Read-only timestamps
                    ui.label("Metadata (Read-only):");
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("Created:");
                        ui.label(&self.created_at);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Modified:");
                        ui.label(&self.modified_at);
                    });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        ui.add_space(120.0);

                        if ui.button("💾 Save").clicked() {
                            self.saved = true;
                            close_dialog = true;
                        }

                        if ui.button("Cancel").clicked() {
                            self.saved = false;
                            close_dialog = true;
                        }
                    });

                    ui.add_space(10.0);
                });
            });

        if close_dialog {
            self.is_open = false;
        }
    }
}

impl Default for GraphMetadataDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Dialog mode for CompositeNameDialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogMode {
    /// Creating a new composite node
    Create,
    /// Renaming an existing composite node
    Rename,
}

/// Action result from the CompositeNameDialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeNameAction {
    /// User confirmed with the entered name
    Confirmed(String),
    /// User cancelled the operation
    Cancelled,
}

/// Dialog for naming/renaming composite nodes
pub struct CompositeNameDialog {
    /// Whether the dialog is open
    is_open: bool,
    /// Editable name
    name: String,
    /// Dialog mode (Create or Rename)
    mode: DialogMode,
    /// Validation error message
    validation_error: Option<String>,
    /// The result of the dialog (if any)
    result: Option<CompositeNameAction>,
}

impl CompositeNameDialog {
    /// Create a new composite name dialog
    pub fn new() -> Self {
        Self {
            is_open: false,
            name: String::new(),
            mode: DialogMode::Create,
            validation_error: None,
            result: None,
        }
    }

    /// Open the dialog for creating a new composite
    pub fn open_for_creation(&mut self, default_name: String) {
        self.is_open = true;
        self.name = default_name;
        self.mode = DialogMode::Create;
        self.validation_error = None;
        self.result = None;
    }

    /// Open the dialog for renaming an existing composite
    pub fn open_for_rename(&mut self, current_name: String) {
        self.is_open = true;
        self.name = current_name;
        self.mode = DialogMode::Rename;
        self.validation_error = None;
        self.result = None;
    }

    /// Check if the dialog is open
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the result of the dialog
    #[allow(dead_code)]
    pub fn result(&self) -> Option<CompositeNameAction> {
        self.result.clone()
    }

    /// Reset the dialog
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.is_open = false;
        self.name.clear();
        self.validation_error = None;
        self.result = None;
    }

    /// Show the dialog and return the user's choice
    pub fn show(&mut self, ctx: &egui::Context) -> Option<CompositeNameAction> {
        if !self.is_open {
            return None;
        }

        let mut result = None;

        let title = match self.mode {
            DialogMode::Create => "Name Your Composite",
            DialogMode::Rename => "Rename Composite Node",
        };

        let button_label = match self.mode {
            DialogMode::Create => "Create",
            DialogMode::Rename => "Rename",
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);

                    // Name input field
                    ui.label("Name:");
                    let text_edit = ui.text_edit_singleline(&mut self.name);

                    // Handle Enter key to confirm
                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let trimmed = self.name.trim();
                        if trimmed.is_empty() {
                            self.validation_error = Some("Name cannot be empty".to_string());
                        } else {
                            result = Some(CompositeNameAction::Confirmed(trimmed.to_string()));
                            self.is_open = false;
                        }
                    }

                    // Handle Escape key to cancel
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        result = Some(CompositeNameAction::Cancelled);
                        self.is_open = false;
                    }

                    ui.add_space(5.0);

                    // Show validation error if present
                    if let Some(ref error) = self.validation_error {
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), error);
                        ui.add_space(5.0);
                    }

                    ui.add_space(10.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        ui.add_space(80.0);

                        if ui.button(button_label).clicked() {
                            let trimmed = self.name.trim();
                            if trimmed.is_empty() {
                                self.validation_error = Some("Name cannot be empty".to_string());
                            } else {
                                result = Some(CompositeNameAction::Confirmed(trimmed.to_string()));
                                self.is_open = false;
                                self.validation_error = None;
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            result = Some(CompositeNameAction::Cancelled);
                            self.is_open = false;
                            self.validation_error = None;
                        }
                    });

                    ui.add_space(10.0);
                });
            });

        self.result = result.clone();
        result
    }
}

impl Default for CompositeNameDialog {
    fn default() -> Self {
        Self::new()
    }
}
