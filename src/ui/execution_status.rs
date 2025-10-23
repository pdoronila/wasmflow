//! Visual indicators for continuous node execution states
//!
//! Provides color mapping and animation helpers for displaying
//! continuous node runtime states in the UI.

use egui::Color32;
use std::time::Instant;

use crate::graph::node::ContinuousExecutionState;

/// Get the color for a given execution state
pub fn state_color(state: &ContinuousExecutionState) -> Color32 {
    match state {
        ContinuousExecutionState::Idle => Color32::from_rgb(128, 128, 128),    // Gray
        ContinuousExecutionState::Starting => Color32::from_rgb(255, 200, 0),  // Yellow
        ContinuousExecutionState::Running => Color32::from_rgb(0, 200, 0),     // Green
        ContinuousExecutionState::Stopping => Color32::from_rgb(255, 140, 0),  // Orange
        ContinuousExecutionState::Stopped => Color32::from_rgb(128, 128, 128), // Gray
        ContinuousExecutionState::Error => Color32::from_rgb(220, 0, 0),       // Red
    }
}

/// Calculate pulsing alpha value for running state animation
///
/// Returns a value between 0.4 and 1.0 that oscillates over time,
/// creating a subtle pulsing effect for running nodes.
///
/// # Arguments
/// * `started_at` - When the node started running
/// * `pulse_speed` - How fast to pulse (higher = faster), default 2.0
pub fn pulsing_alpha(started_at: Option<Instant>, pulse_speed: f32) -> f32 {
    if let Some(start_time) = started_at {
        let elapsed = start_time.elapsed().as_secs_f32();
        let phase = (elapsed * pulse_speed).sin();
        // Map from [-1, 1] to [0.4, 1.0]
        0.7 + (phase * 0.3)
    } else {
        1.0 // Full opacity if no start time
    }
}

/// Get a human-readable status string for display
pub fn state_display_name(state: &ContinuousExecutionState) -> &'static str {
    match state {
        ContinuousExecutionState::Idle => "Idle",
        ContinuousExecutionState::Starting => "Starting...",
        ContinuousExecutionState::Running => "Running",
        ContinuousExecutionState::Stopping => "Stopping...",
        ContinuousExecutionState::Stopped => "Stopped",
        ContinuousExecutionState::Error => "Error",
    }
}

/// Format elapsed time for display (e.g., "5.2s", "1m 23s", "1h 5m")
pub fn format_duration(started_at: Option<Instant>) -> String {
    if let Some(start_time) = started_at {
        let elapsed = start_time.elapsed();
        let total_seconds = elapsed.as_secs();

        if total_seconds < 60 {
            format!("{:.1}s", elapsed.as_secs_f32())
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            format!("{}m {}s", minutes, seconds)
        } else {
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            format!("{}h {}m", hours, minutes)
        }
    } else {
        "N/A".to_string()
    }
}
