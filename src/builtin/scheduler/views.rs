//! Scheduler footer view - Gantt chart visualization
//!
//! Renders execution history as a visual timeline showing which tasks ran when.

use crate::graph::node::{GraphNode, NodeValue};
use crate::ui::component_view::ComponentFooterView;
use std::sync::Arc;

/// Footer view for the Time-Partitioned Scheduler
///
/// Displays a Gantt chart visualization of task execution history,
/// showing which tasks executed, when, and whether they violated budgets or deadlines.
pub struct SchedulerFooterView;

impl SchedulerFooterView {
    /// Create a new scheduler footer view
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl ComponentFooterView for SchedulerFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Get the schedule_state output which contains execution history
        let schedule_state = node
            .outputs
            .iter()
            .find(|output| output.name == "schedule_state")
            .and_then(|output| output.current_value.as_ref())
            .ok_or_else(|| "No schedule_state output available".to_string())?;

        // Extract the schedule state record
        let state_record = match schedule_state {
            NodeValue::Record(r) => r,
            _ => return Err("schedule_state is not a Record".to_string()),
        };

        // Extract statistics for display
        let iterations = match state_record.get("iterations") {
            Some(NodeValue::U32(n)) => *n as u64,
            _ => 0,
        };

        let context_switches = match state_record.get("context_switches") {
            Some(NodeValue::U32(n)) => *n,
            _ => 0,
        };

        let cpu_utilization = match state_record.get("cpu_utilization") {
            Some(NodeValue::F32(f)) => *f,
            _ => 0.0,
        };

        let total_violations = match state_record.get("total_violations") {
            Some(NodeValue::U32(n)) => *n,
            _ => 0,
        };

        // Extract execution history for Gantt chart
        let execution_history = match state_record.get("execution_history") {
            Some(NodeValue::List(items)) => items,
            _ => return Ok(()), // No history yet, render basic stats only
        };

        // Render header with statistics
        ui.vertical(|ui| {
            ui.separator();

            ui.heading(egui::RichText::new("Schedule Visualization").size(14.0).strong());
            ui.add_space(5.0);

            // Statistics grid
            egui::Grid::new(format!("scheduler_stats_{}", node.id))
                .num_columns(4)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    // Row 1: Labels
                    ui.label(egui::RichText::new("Iterations:").color(egui::Color32::from_rgb(150, 150, 150)));
                    ui.label(egui::RichText::new("Context Switches:").color(egui::Color32::from_rgb(150, 150, 150)));
                    ui.label(egui::RichText::new("CPU Utilization:").color(egui::Color32::from_rgb(150, 150, 150)));
                    ui.label(egui::RichText::new("Violations:").color(egui::Color32::from_rgb(150, 150, 150)));
                    ui.end_row();

                    // Row 2: Values
                    ui.label(egui::RichText::new(format!("{}", iterations)).strong());
                    ui.label(egui::RichText::new(format!("{}", context_switches)).strong());

                    // Color-code utilization
                    let util_color = if cpu_utilization > 0.95 {
                        egui::Color32::from_rgb(255, 100, 100) // Red if near saturation
                    } else if cpu_utilization > 0.80 {
                        egui::Color32::from_rgb(255, 200, 100) // Yellow if high
                    } else {
                        egui::Color32::from_rgb(100, 255, 100) // Green if healthy
                    };
                    ui.label(egui::RichText::new(format!("{:.1}%", cpu_utilization * 100.0)).color(util_color).strong());

                    // Color-code violations
                    let violation_color = if total_violations > 0 {
                        egui::Color32::from_rgb(255, 100, 100) // Red if any violations
                    } else {
                        egui::Color32::from_rgb(100, 255, 100) // Green if none
                    };
                    ui.label(egui::RichText::new(format!("{}", total_violations)).color(violation_color).strong());
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            // Gantt Chart Header
            ui.heading(egui::RichText::new("Execution Timeline (Last 20 Iterations)").size(13.0));
            ui.add_space(8.0);

            // Legend
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Legend:").size(11.0).color(egui::Color32::from_rgb(150, 150, 150)));
                ui.add_space(10.0);

                // Success box
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "█");
                ui.label(egui::RichText::new("Success").size(11.0));
                ui.add_space(10.0);

                // Budget exceeded box
                ui.colored_label(egui::Color32::from_rgb(255, 150, 100), "█");
                ui.label(egui::RichText::new("Budget Exceeded").size(11.0));
                ui.add_space(10.0);

                // Deadline missed box
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "█");
                ui.label(egui::RichText::new("Deadline Missed").size(11.0));
            });

            ui.add_space(8.0);

            // Render Gantt chart in a scrollable area
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if let Err(e) = render_gantt_chart(ui, execution_history, &node.id) {
                        ui.colored_label(egui::Color32::RED, format!("Error rendering chart: {}", e));
                    }
                });
        });

        Ok(())
    }
}

/// Render the Gantt chart from execution history
fn render_gantt_chart(
    ui: &mut egui::Ui,
    execution_history: &[NodeValue],
    _node_id: &uuid::Uuid,
) -> Result<(), String> {
    if execution_history.is_empty() {
        ui.label(egui::RichText::new("No execution history yet...").color(egui::Color32::from_rgb(150, 150, 150)).italics());
        return Ok(());
    }

    // Parse history entries
    let mut entries = Vec::new();
    for entry in execution_history {
        if let NodeValue::Record(record) = entry {
            let task_name = match record.get("task_name") {
                Some(NodeValue::String(s)) => s.clone(),
                _ => "Unknown".to_string(),
            };

            let start_time_ms = match record.get("start_time_ms") {
                Some(NodeValue::U32(n)) => *n as u64,
                _ => 0,
            };

            let duration_ms = match record.get("duration_ms") {
                Some(NodeValue::U32(n)) => *n,
                _ => 0,
            };

            let budget_exceeded = match record.get("budget_exceeded") {
                Some(NodeValue::Bool(b)) => *b,
                _ => false,
            };

            let deadline_missed = match record.get("deadline_missed") {
                Some(NodeValue::Bool(b)) => *b,
                _ => false,
            };

            entries.push(HistoryEntry {
                task_name,
                start_time_ms,
                duration_ms,
                budget_exceeded,
                deadline_missed,
            });
        }
    }

    if entries.is_empty() {
        ui.label("No valid history entries");
        return Ok(());
    }

    // Calculate time range for scaling
    let min_time = entries.iter().map(|e| e.start_time_ms).min().unwrap_or(0);
    let max_time = entries.iter().map(|e| e.start_time_ms + e.duration_ms as u64).max().unwrap_or(100);
    let time_range = (max_time - min_time).max(1); // Avoid division by zero

    // Available width for timeline
    let available_width = ui.available_width() - 150.0; // Reserve space for task names

    // Render each execution as a horizontal bar
    for (_idx, entry) in entries.iter().enumerate() {
        ui.horizontal(|ui| {
            // Task name (fixed width)
            ui.allocate_ui_with_layout(
                egui::vec2(140.0, 20.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let text = if entry.task_name.len() > 18 {
                        format!("{}...", &entry.task_name[..15])
                    } else {
                        entry.task_name.clone()
                    };
                    ui.label(egui::RichText::new(text).size(10.0));
                },
            );

            // Timeline bar
            let relative_start = ((entry.start_time_ms - min_time) as f32 / time_range as f32) * available_width;
            let bar_width = (entry.duration_ms as f32 / time_range as f32) * available_width;

            // Add spacing to position the bar
            ui.add_space(relative_start);

            // Choose color based on status
            let color = if entry.deadline_missed {
                egui::Color32::from_rgb(255, 100, 100) // Red for deadline miss
            } else if entry.budget_exceeded {
                egui::Color32::from_rgb(255, 150, 100) // Orange for budget exceeded
            } else {
                egui::Color32::from_rgb(100, 200, 100) // Green for success
            };

            // Draw colored bar with hover tooltip
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(bar_width.max(2.0), 16.0),
                egui::Sense::hover(),
            );
            ui.painter().rect_filled(rect, 2.0, color);

            // Tooltip with details on hover
            response.on_hover_ui(|ui| {
                ui.label(format!("Task: {}", entry.task_name));
                ui.label(format!("Start: {}ms", entry.start_time_ms));
                ui.label(format!("Duration: {}ms", entry.duration_ms));
                if entry.budget_exceeded {
                    ui.colored_label(egui::Color32::RED, "⚠ Budget Exceeded");
                }
                if entry.deadline_missed {
                    ui.colored_label(egui::Color32::RED, "⚠ Deadline Missed");
                }
            });
        });
        ui.add_space(2.0);
    }

    // Time scale at bottom
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(140.0); // Align with timeline
        ui.label(egui::RichText::new(format!("{}ms", min_time)).size(9.0).color(egui::Color32::from_rgb(120, 120, 120)));
        ui.add_space(available_width - 100.0);
        ui.label(egui::RichText::new(format!("{}ms", max_time)).size(9.0).color(egui::Color32::from_rgb(120, 120, 120)));
    });

    Ok(())
}

/// Local representation of history entry (parsed from NodeValue::Record)
struct HistoryEntry {
    task_name: String,
    start_time_ms: u64,
    duration_ms: u32,
    budget_exceeded: bool,
    deadline_missed: bool,
}
