//! Splash screen UI for component loading
//!
//! Displays a loading screen with progress bar while components are being loaded
//! in the background during application startup.

use crate::ui::ComponentLoadProgress;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Splash screen for displaying component loading progress
///
/// Shows a branded loading screen with:
/// - WasmFlow title and version
/// - Progress bar (0-100%)
/// - Component count ("Loaded X/Y components")
/// - Current component being loaded
/// - Error summary (if any failures)
/// - Animated loading spinner
pub struct SplashScreen {
    /// Shared progress tracker from loading thread
    progress: Arc<Mutex<ComponentLoadProgress>>,
    /// Animation start time for spinner
    animation_start: Instant,
}

impl SplashScreen {
    /// Create a new splash screen
    ///
    /// # Arguments
    ///
    /// * `progress` - Shared progress tracker from the background loading thread
    pub fn new(progress: Arc<Mutex<ComponentLoadProgress>>) -> Self {
        Self {
            progress,
            animation_start: Instant::now(),
        }
    }

    /// Render the splash screen
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui context for rendering
    ///
    /// # Returns
    ///
    /// Returns `true` when loading is complete and the splash screen should be dismissed
    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        let mut loading_complete = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);

                // Title and branding
                self.render_title(ui);

                ui.add_space(40.0);

                // Progress bar and status
                let progress = self.progress.lock().unwrap();
                loading_complete = progress.is_complete();

                self.render_progress_bar(ui, &progress);
                ui.add_space(20.0);

                self.render_component_count(ui, &progress);
                ui.add_space(10.0);

                self.render_current_component(ui, &progress);
                ui.add_space(10.0);

                self.render_error_summary(ui, &progress);
                ui.add_space(30.0);

                // Loading spinner (only show if not complete)
                if !loading_complete {
                    self.render_spinner(ui);
                }

                drop(progress);
            });
        });

        loading_complete
    }

    /// Render the title and branding
    fn render_title(&self, ui: &mut egui::Ui) {
        // WasmFlow title with wave emoji
        ui.heading(
            egui::RichText::new("🌊 WasmFlow")
                .size(56.0)
                .color(egui::Color32::from_rgb(100, 150, 255))
                .strong(),
        );

        ui.add_space(10.0);

        // Subtitle
        ui.label(
            egui::RichText::new("Visual Programming with WebAssembly")
                .size(18.0)
                .color(egui::Color32::from_rgb(180, 180, 180)),
        );

        ui.add_space(5.0);

        // Version
        ui.label(
            egui::RichText::new("v0.1.0")
                .size(14.0)
                .color(egui::Color32::from_rgb(150, 150, 150)),
        );
    }

    /// Render the progress bar
    fn render_progress_bar(&self, ui: &mut egui::Ui, progress: &ComponentLoadProgress) {
        let fraction = progress.percentage();

        let progress_bar = egui::ProgressBar::new(fraction)
            .desired_width(500.0)
            .desired_height(24.0)
            .show_percentage()
            .animate(true); // Animate the progress bar

        ui.add(progress_bar);
    }

    /// Render component count display
    fn render_component_count(&self, ui: &mut egui::Ui, progress: &ComponentLoadProgress) {
        let text = if progress.total_components == 0 {
            "Initializing...".to_string()
        } else {
            format!(
                "Loading components: {} / {} ({:.0}%)",
                progress.loaded_count,
                progress.total_components,
                progress.percentage() * 100.0
            )
        };

        ui.label(
            egui::RichText::new(text)
                .size(16.0)
                .color(egui::Color32::from_rgb(200, 200, 200)),
        );
    }

    /// Render current component being loaded
    fn render_current_component(&self, ui: &mut egui::Ui, progress: &ComponentLoadProgress) {
        if let Some(ref current) = progress.current_component {
            ui.label(
                egui::RichText::new(format!("Current: {}", current))
                    .size(14.0)
                    .color(egui::Color32::from_rgb(150, 200, 255))
                    .italics(),
            );
        } else if progress.is_complete() {
            ui.label(
                egui::RichText::new("✓ Loading complete!")
                    .size(14.0)
                    .color(egui::Color32::from_rgb(100, 255, 100)),
            );
        } else {
            // Show placeholder to maintain consistent spacing
            ui.label(
                egui::RichText::new(" ")
                    .size(14.0),
            );
        }
    }

    /// Render error summary if there are errors
    fn render_error_summary(&self, ui: &mut egui::Ui, progress: &ComponentLoadProgress) {
        if !progress.errors.is_empty() {
            let error_text = if progress.errors.len() == 1 {
                "⚠ 1 component failed to load".to_string()
            } else {
                format!("⚠ {} components failed to load", progress.errors.len())
            };

            let error_label = egui::Label::new(
                egui::RichText::new(error_text)
                    .size(14.0)
                    .color(egui::Color32::from_rgb(255, 200, 100)),
            )
            .sense(egui::Sense::hover());

            let response = ui.add(error_label);

            // Show tooltip with first few errors
            if response.hovered() {
                response.on_hover_ui(|ui| {
                    ui.label(
                        egui::RichText::new("Failed components:")
                            .size(12.0)
                            .strong(),
                    );
                    ui.add_space(5.0);

                    // Show first 5 errors
                    for (i, error) in progress.errors.iter().take(5).enumerate() {
                        ui.label(
                            egui::RichText::new(format!("{}. {}", i + 1, error))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(255, 180, 180)),
                        );
                    }

                    if progress.errors.len() > 5 {
                        ui.label(
                            egui::RichText::new(format!(
                                "... and {} more",
                                progress.errors.len() - 5
                            ))
                            .size(11.0)
                            .italics(),
                        );
                    }
                });
            }
        }
    }

    /// Render animated loading spinner
    fn render_spinner(&self, ui: &mut egui::Ui) {
        let elapsed = self.animation_start.elapsed().as_secs_f32();

        // Rotating circle animation
        let angle = elapsed * 2.0 * std::f32::consts::PI;
        let radius = 20.0;
        let center = ui.cursor().left_top() + egui::vec2(250.0, 30.0);

        let painter = ui.painter();

        // Draw spinning arc
        for i in 0..8 {
            let i_angle = angle + (i as f32 * std::f32::consts::PI / 4.0);
            let alpha = 255 - (i * 25) as u8; // Fade effect
            let color = egui::Color32::from_rgba_premultiplied(100, 150, 255, alpha);

            let start = center + egui::vec2(i_angle.cos() * radius, i_angle.sin() * radius);
            let end = center
                + egui::vec2(i_angle.cos() * (radius + 5.0), i_angle.sin() * (radius + 5.0));

            painter.line_segment(
                [start, end],
                egui::Stroke::new(3.0, color),
            );
        }

        // Allocate space for the spinner
        ui.allocate_space(egui::vec2(500.0, 60.0));

        // Request repaint for animation
        ui.ctx().request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_screen_creation() {
        let progress = Arc::new(Mutex::new(ComponentLoadProgress::new()));
        let splash = SplashScreen::new(progress);

        // Should be created successfully
        assert!(splash.animation_start.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_completion_detection() {
        let progress = Arc::new(Mutex::new(ComponentLoadProgress::new()));

        {
            let mut p = progress.lock().unwrap();
            p.total_components = 10;
            p.loaded_count = 10;
        }

        // Progress shows complete
        let p = progress.lock().unwrap();
        assert!(p.is_complete());
    }
}
