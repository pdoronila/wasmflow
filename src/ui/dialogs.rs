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
                    let is_full_access = matches!(self.requested_capabilities, CapabilitySet::Full);

                    // Show Full Access warning for either:
                    // 1. Components requesting Full access
                    // 2. Users wanting to override with Full access
                    if is_full_access || !matches!(self.requested_capabilities, CapabilitySet::Full) {
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
                                        ui.label("This component requests UNRESTRICTED system access.");
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
                        ui.horizontal(|ui| {
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

                        ui.add_enabled_ui(can_approve, |ui| {
                            if ui.button("✓ Approve").clicked() {
                                result = Some(PermissionAction::Approve);
                                self.is_open = false;
                            }
                        });

                        // Add "Approve as Full" button for advanced users who want unrestricted access
                        if !is_full_access {
                            ui.add_enabled_ui(self.full_access_acknowledged, |ui| {
                                if ui.button("✓ Approve as Full Access").clicked() {
                                    result = Some(PermissionAction::ApproveAsFull);
                                    self.is_open = false;
                                }
                            });
                            if !self.full_access_acknowledged {
                                ui.label("↑").on_hover_text("Check the box above to enable Full Access override");
                            }
                        }

                        if ui.button("✗ Deny").clicked() {
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
                                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 80, 80)))
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 100, 100),
                                            "⚠ UPGRADE TO FULL ACCESS ⚠",
                                        );
                                        ui.add_space(5.0);
                                        ui.label("Full Access grants UNRESTRICTED system access.");
                                        ui.label("It can read/write ANY files, access ANY network, and");
                                        ui.label("read environment variables. Only approve if you");
                                        ui.label("FULLY TRUST this component's author and source.");
                                    });
                                });

                            ui.add_space(10.0);

                            // Explicit acknowledgment checkbox
                            ui.horizontal(|ui| {
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
                                ui.add_enabled_ui(self.full_access_acknowledged, |ui| {
                                    if ui.button("🔓 Upgrade to Full Access").clicked() {
                                        self.requested_action = Some(PermissionViewAction::UpgradeToFull);
                                        close_dialog = true;
                                    }
                                });
                            }

                            // Revoke button
                            if ui.button("🔒 Revoke Permissions").clicked() {
                                self.requested_action = Some(PermissionViewAction::Revoke);
                                close_dialog = true;
                            }

                            if ui.button("Close").clicked() {
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
                            if ui.button("Close").clicked() {
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
