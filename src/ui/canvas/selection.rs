//! Rectangle selection functionality for canvas
//!
//! This module handles the logic for finding nodes within a selection rectangle
//! during rectangle selection mode.

use crate::graph::graph::NodeGraph;
use egui_snarl::{NodeId, Snarl};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::node_data::SnarlNodeData;

/// Selection helper methods
pub(super) struct SelectionHelper;

impl SelectionHelper {
    /// T012: Find nodes whose centers fall within a given rectangle
    ///
    /// This is used for rectangle selection to determine which nodes
    /// should be selected when the user drags a selection box.
    ///
    /// Note: This assumes no viewport transform (scale=1, offset=0).
    /// Works best when canvas hasn't been panned/zoomed before entering Selection mode.
    pub(super) fn find_nodes_in_rect(
        rect: egui::Rect,
        graph: &NodeGraph,
        snarl: &Snarl<SnarlNodeData>,
        uuid_to_snarl: &HashMap<Uuid, NodeId>,
        viewport_scale: f32,
        viewport_offset: egui::Vec2,
    ) -> HashSet<Uuid> {
        let mut selected = HashSet::new();

        for (uuid, graph_node) in &graph.nodes {
            // Get node position from snarl if available (it may have been moved)
            let graph_pos = if let Some(&snarl_id) = uuid_to_snarl.get(uuid) {
                if let Some(node_info) = snarl.get_node_info(snarl_id) {
                    node_info.pos
                } else {
                    graph_node.position
                }
            } else {
                graph_node.position
            };

            // Transform to screen space using stored viewport
            let screen_pos = egui::pos2(
                graph_pos.x * viewport_scale + viewport_offset.x,
                graph_pos.y * viewport_scale + viewport_offset.y
            );

            // Calculate node center in screen space
            let screen_size = egui::vec2(200.0 * viewport_scale, 100.0 * viewport_scale);
            let node_center = screen_pos + screen_size / 2.0;

            if rect.contains(node_center) {
                selected.insert(*uuid);
            }
        }

        selected
    }
}
