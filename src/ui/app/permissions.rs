//! Permission and capability management
//!
//! This module handles permission dialogs and capability parsing for user-defined components.

use super::{PendingPermissionRequest, WasmFlowApp};
use crate::runtime::capabilities::{CapabilityGrant, CapabilitySet};
use std::path::PathBuf;

impl WasmFlowApp {
    /// T078: Handle permission view dialog
    pub(super) fn handle_permissions_view_dialog(&mut self, ctx: &egui::Context) {
        // Show the dialog
        self.permissions_view_dialog.show(ctx);

        // Check if an action was requested
        if let Some(action) = self.permissions_view_dialog.take_action() {
            if let Some(node_id) = self.permissions_view_dialog.node_id() {
                match action {
                    crate::ui::dialogs::PermissionViewAction::Revoke => {
                        // Revoke the capability grant
                        self.graph.revoke_capability(node_id);
                        self.status_message =
                            "Permissions revoked - node will fail to execute without re-approval"
                                .to_string();
                        self.dirty = true;
                    }
                    crate::ui::dialogs::PermissionViewAction::UpgradeToFull => {
                        // Upgrade to Full access
                        let grant = CapabilityGrant {
                            node_id,
                            capability_set: CapabilitySet::Full,
                            granted_at: chrono::Utc::now().to_rfc3339(),
                            scope: "graph".to_string(),
                        };
                        self.graph.grant_capability(grant);
                        self.status_message = "Upgraded to FULL ACCESS (unrestricted)".to_string();
                        self.dirty = true;
                    }
                }
            }
            self.permissions_view_dialog.reset();
        }
    }

    /// T073: Handle permission dialog response
    pub(super) fn handle_permission_dialog(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.permission_dialog.show(ctx) {
            match action {
                crate::ui::dialogs::PermissionAction::Approve => {
                    // User approved the permissions - create the node with grant
                    if let Some(pending) = self.pending_permission_request.take() {
                        // Create a capability grant
                        let node = pending.component_spec.create_node(pending.position);
                        let grant = CapabilityGrant {
                            node_id: node.id,
                            capability_set: pending.capabilities,
                            granted_at: chrono::Utc::now().to_rfc3339(),
                            scope: "graph".to_string(),
                        };

                        // Add the grant to the graph
                        self.graph.grant_capability(grant);

                        // Add the node to the graph through command history
                        let cmd = crate::graph::command::Command::AddNode { node };
                        if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                            self.error_message = Some(format!("Failed to add node: {}", e));
                        } else {
                            self.status_message = format!(
                                "Added {} node with approved permissions",
                                pending.component_spec.name
                            );
                            self.error_message = None;
                            self.dirty = true;
                        }
                    }
                }
                crate::ui::dialogs::PermissionAction::ApproveAsFull => {
                    // User approved with Full access override - create the node with Full capabilities
                    if let Some(pending) = self.pending_permission_request.take() {
                        let node = pending.component_spec.create_node(pending.position);
                        let grant = CapabilityGrant {
                            node_id: node.id,
                            capability_set: CapabilitySet::Full, // Override with Full access
                            granted_at: chrono::Utc::now().to_rfc3339(),
                            scope: "graph".to_string(),
                        };

                        // Add the grant to the graph
                        self.graph.grant_capability(grant);

                        // Add the node to the graph through command history
                        let cmd = crate::graph::command::Command::AddNode { node };
                        if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                            self.error_message = Some(format!("Failed to add node: {}", e));
                        } else {
                            self.status_message = format!(
                                "Added {} node with FULL ACCESS (unrestricted)",
                                pending.component_spec.name
                            );
                            self.error_message = None;
                            self.dirty = true;
                        }
                    }
                }
                crate::ui::dialogs::PermissionAction::Deny => {
                    // User denied the permissions - don't create the node
                    if let Some(pending) = self.pending_permission_request.take() {
                        self.status_message = format!(
                            "Permission denied for {} - node not created",
                            pending.component_spec.name
                        );
                    }
                }
            }
        }
    }

    /// Parse capability strings into a CapabilitySet
    /// Supports formats:
    /// - "file-read:/path"  -> FileRead { paths }
    /// - "file-write:/path" -> FileWrite { paths }
    /// - "network:host.com" -> Network { allowed_hosts }
    /// - Empty vec          -> None
    pub(super) fn parse_capabilities(capabilities: &[String]) -> CapabilitySet {
        if capabilities.is_empty() {
            return CapabilitySet::None;
        }

        let mut file_read_paths = Vec::new();
        let mut file_write_paths = Vec::new();
        let mut network_hosts = Vec::new();

        for cap in capabilities {
            if let Some(path) = cap.strip_prefix("file-read:") {
                file_read_paths.push(PathBuf::from(path));
            } else if let Some(path) = cap.strip_prefix("file-write:") {
                file_write_paths.push(PathBuf::from(path));
            } else if let Some(host) = cap.strip_prefix("network:") {
                network_hosts.push(host.to_string());
            }
        }

        // Determine the most appropriate capability set
        if !file_read_paths.is_empty() && !file_write_paths.is_empty() {
            // Combine read and write paths
            let mut all_paths = file_read_paths;
            all_paths.extend(file_write_paths);
            CapabilitySet::FileReadWrite { paths: all_paths }
        } else if !file_read_paths.is_empty() {
            CapabilitySet::FileRead {
                paths: file_read_paths,
            }
        } else if !file_write_paths.is_empty() {
            CapabilitySet::FileWrite {
                paths: file_write_paths,
            }
        } else if !network_hosts.is_empty() {
            CapabilitySet::Network {
                allowed_hosts: network_hosts,
            }
        } else {
            // Unknown capability format, default to None for safety
            CapabilitySet::None
        }
    }

    /// Handle palette action for adding components with permission checks
    pub(super) fn handle_add_component_with_permissions(
        &mut self,
        spec: crate::graph::node::ComponentSpec,
        position: egui::Pos2,
    ) {
        // T073: Check if this is a user-defined component that needs permission
        if let crate::graph::node::ComponentType::UserDefined(_) = spec.component_type {
            // Extract capabilities from component metadata
            let requested_capabilities = Self::parse_capabilities(&spec.required_capabilities);

            // T080: Check for capability escalation
            // Look for existing grants for this component_id
            let existing_grant = self
                .graph
                .nodes
                .values()
                .find(|node| node.component_id == spec.id)
                .and_then(|node| self.graph.get_capability_grant(node.id));

            let needs_approval = if let Some(grant) = existing_grant {
                // Compare requested vs granted capabilities
                // If they differ, we need re-approval
                grant.capability_set != requested_capabilities
            } else {
                // No existing grant - always need approval
                true
            };

            if needs_approval {
                // Show permission dialog for approval
                self.permission_dialog.open(
                    spec.name.clone(),
                    spec.description.clone(),
                    requested_capabilities.clone(),
                );

                // Store pending request for later approval
                self.pending_permission_request = Some(PendingPermissionRequest {
                    component_spec: spec,
                    capabilities: requested_capabilities,
                    position,
                });
            } else {
                // Reuse existing grant - add node directly
                let node = spec.create_node(position);
                let grant = existing_grant.unwrap(); // Safe because we checked above

                // Create a new grant for this node instance
                let new_grant = CapabilityGrant {
                    node_id: node.id,
                    capability_set: grant.capability_set.clone(),
                    granted_at: chrono::Utc::now().to_rfc3339(),
                    scope: grant.scope.clone(),
                };
                self.graph.grant_capability(new_grant);

                // Add the node
                let cmd = crate::graph::command::Command::AddNode { node };
                if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                    self.error_message = Some(format!("Failed to add node: {}", e));
                } else {
                    self.status_message = format!(
                        "Added {} node (reusing existing permissions)",
                        spec.name
                    );
                    self.error_message = None;
                    self.dirty = true;
                }
            }
        } else {
            // Builtin component - add directly without permission dialog
            let node = spec.create_node(position);
            let cmd = crate::graph::command::Command::AddNode { node };

            if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                self.error_message = Some(format!("Failed to add node: {}", e));
            } else {
                self.status_message = format!("Added {} node", spec.name);
                self.error_message = None;
                self.dirty = true;
            }
        }
    }
}
