//! Time scrubber component for filtering packets by timestamp
//!
//! Provides a visual timeline with density visualization and interactive time range selection.

use eframe::egui;

/// Time range selection state
#[derive(Clone, Debug)]
pub struct TimeRange {
    pub min: f64,
    pub max: f64,
}

impl TimeRange {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, timestamp: f64) -> bool {
        timestamp >= self.min && timestamp <= self.max
    }

    pub fn is_full_range(&self, data_min: f64, data_max: f64) -> bool {
        (self.min - data_min).abs() < 0.001 && (self.max - data_max).abs() < 0.001
    }
}

/// Time scrubber state
pub struct TimeScrubber {
    /// Current selected time range
    pub selected_range: Option<TimeRange>,
    /// Overall data time range
    pub data_range: Option<TimeRange>,
    /// Density estimation data points (time, density)
    density_data: Vec<(f64, f32)>,
    /// Dragging state
    drag_start: Option<f64>,
    /// Hover position
    hover_time: Option<f64>,
    /// Highlighted timestamps (e.g., search results)
    highlighted_timestamps: Vec<f64>,
}

impl Default for TimeScrubber {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeScrubber {
    pub fn new() -> Self {
        Self {
            selected_range: None,
            data_range: None,
            density_data: Vec::new(),
            drag_start: None,
            hover_time: None,
            highlighted_timestamps: Vec::new(),
        }
    }

    /// Update density data from timestamps using histogram binning
    pub fn update_density(&mut self, timestamps: &[f64]) {
        if timestamps.is_empty() {
            self.density_data.clear();
            self.data_range = None;
            self.selected_range = None;
            return;
        }

        let min_time = timestamps.iter().copied().fold(f64::INFINITY, f64::min);
        let max_time = timestamps.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        self.data_range = Some(TimeRange::new(min_time, max_time));

        // Calculate histogram bins
        let num_bins = 100;
        let bin_width = (max_time - min_time) / num_bins as f64;

        // Count packets in each bin
        let mut bins = vec![0u32; num_bins];
        for &timestamp in timestamps {
            let bin_index = ((timestamp - min_time) / bin_width).floor() as usize;
            let bin_index = bin_index.min(num_bins - 1); // Clamp to last bin
            bins[bin_index] += 1;
        }

        // Store bin data as (time, count)
        self.density_data.clear();
        for (i, &count) in bins.iter().enumerate() {
            let t = min_time + (i as f64 + 0.5) * bin_width; // Center of bin
            self.density_data.push((t, count as f32));
        }

        // Initialize selected range to full range
        if self.selected_range.is_none() {
            self.selected_range = Some(TimeRange::new(min_time, max_time));
        }
    }

    /// Reset selection to show all data
    pub fn reset_selection(&mut self) {
        if let Some(ref range) = self.data_range {
            self.selected_range = Some(range.clone());
        }
    }

    /// Check if we have data
    pub fn has_data(&self) -> bool {
        self.data_range.is_some() && !self.density_data.is_empty()
    }

    /// Set highlighted timestamps (e.g., search results)
    pub fn set_highlighted_timestamps(&mut self, timestamps: Vec<f64>) {
        self.highlighted_timestamps = timestamps;
    }

    /// Render the time scrubber UI
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize> {
        if !self.has_data() {
            ui.label("No data to display");
            return None;
        }

        let data_range = self.data_range.clone().unwrap();
        let selected_range = self.selected_range.clone().unwrap();

        let mut clicked_index: Option<usize> = None;

        ui.vertical(|ui| {
            // Header with info and reset button
            ui.horizontal(|ui| {
                ui.label("Time Scrubber");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset").clicked() {
                        self.reset_selection();
                    }

                    // Show time range
                    let range_text = if selected_range.is_full_range(data_range.min, data_range.max)
                    {
                        "All data".to_string()
                    } else {
                        format!(
                            "Range: {:.3}s - {:.3}s ({:.3}s)",
                            selected_range.min - data_range.min,
                            selected_range.max - data_range.min,
                            selected_range.max - selected_range.min
                        )
                    };
                    ui.label(range_text);
                });
            });

            ui.separator();

            // Density visualization
            let height = 80.0;
            let (response, painter) = ui.allocate_painter(
                egui::vec2(ui.available_width(), height),
                egui::Sense::click_and_drag(),
            );

            let rect = response.rect;
            let time_range = data_range.max - data_range.min;

            // Background
            painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);

            // Calculate max density for normalization
            let max_density = self
                .density_data
                .iter()
                .map(|(_, d)| *d)
                .fold(0.0f32, f32::max);

            if max_density > 0.0 {
                // Draw histogram bars
                let bar_width = rect.width() / self.density_data.len() as f32;

                let fill_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(100, 150, 255, 120)
                } else {
                    egui::Color32::from_rgba_unmultiplied(50, 100, 200, 120)
                };

                let stroke_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(100, 150, 255, 200)
                } else {
                    egui::Color32::from_rgba_unmultiplied(50, 100, 200, 200)
                };

                for (time, density) in &self.density_data {
                    let x =
                        rect.min.x + ((*time - data_range.min) / time_range) as f32 * rect.width();
                    let normalized_density = density / max_density;
                    let bar_height = normalized_density * height;

                    // Draw bar from bottom up
                    let bar_rect = egui::Rect::from_min_size(
                        egui::pos2(x - bar_width * 0.4, rect.max.y - bar_height),
                        egui::vec2(bar_width * 0.8, bar_height),
                    );

                    painter.rect_filled(bar_rect, 1.0, fill_color);
                    painter.rect_stroke(bar_rect, 1.0, egui::Stroke::new(0.5, stroke_color));
                }
            }

            // Draw highlighted timestamps (search results) as yellow vertical lines
            if !self.highlighted_timestamps.is_empty() {
                let highlight_color = egui::Color32::from_rgb(255, 220, 0); // Bright yellow
                for &timestamp in &self.highlighted_timestamps {
                    let x = rect.min.x
                        + ((timestamp - data_range.min) / time_range) as f32 * rect.width();
                    painter.vline(x, rect.y_range(), egui::Stroke::new(2.0, highlight_color));
                }
            }

            // Draw selected range overlay
            if !selected_range.is_full_range(data_range.min, data_range.max) {
                let sel_start_x = rect.min.x
                    + ((selected_range.min - data_range.min) / time_range) as f32 * rect.width();
                let sel_end_x = rect.min.x
                    + ((selected_range.max - data_range.min) / time_range) as f32 * rect.width();

                let selection_rect = egui::Rect::from_min_max(
                    egui::pos2(sel_start_x, rect.min.y),
                    egui::pos2(sel_end_x, rect.max.y),
                );

                let selection_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(255, 200, 100, 60)
                } else {
                    egui::Color32::from_rgba_unmultiplied(255, 200, 100, 80)
                };
                painter.rect_filled(selection_rect, 0.0, selection_color);

                // Draw range borders
                let border_color = egui::Color32::from_rgb(255, 200, 100);
                painter.vline(
                    sel_start_x,
                    rect.y_range(),
                    egui::Stroke::new(2.0, border_color),
                );
                painter.vline(
                    sel_end_x,
                    rect.y_range(),
                    egui::Stroke::new(2.0, border_color),
                );
            }

            // Handle interactions
            let mut show_tooltip = false;
            let mut tooltip_text = String::new();

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if rect.contains(pointer_pos) {
                    let x_ratio = (pointer_pos.x - rect.min.x) / rect.width();
                    let hover_time_val = data_range.min + x_ratio as f64 * time_range;
                    self.hover_time = Some(hover_time_val);

                    // Draw hover line
                    let hover_color = ui.visuals().text_color();
                    painter.vline(
                        pointer_pos.x,
                        rect.y_range(),
                        egui::Stroke::new(1.0, hover_color.gamma_multiply(0.5)),
                    );

                    // Prepare tooltip
                    show_tooltip = true;
                    tooltip_text = format!("Time: {:.3}s", hover_time_val - data_range.min);
                }
            } else {
                self.hover_time = None;
            }

            // Handle dragging for range selection
            if response.drag_started() {
                if let Some(hover) = self.hover_time {
                    self.drag_start = Some(hover);
                }
            }

            if response.dragged() {
                if let (Some(start), Some(current)) = (self.drag_start, self.hover_time) {
                    let min = start.min(current);
                    let max = start.max(current);
                    self.selected_range = Some(TimeRange::new(min, max));
                }
            }

            if response.drag_stopped() {
                self.drag_start = None;
            }

            // Handle single click
            if response.clicked() {
                if let Some(clicked_time) = self.hover_time {
                    // If there's a selection active, clear it
                    if !selected_range.is_full_range(data_range.min, data_range.max) {
                        self.reset_selection();
                    } else {
                        // Otherwise scroll to time
                        clicked_index = Some(0); // Placeholder - will be computed by caller
                        self.hover_time = Some(clicked_time);
                    }
                }
            }

            // Handle ESC key to clear selection
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.reset_selection();
            }

            // Show tooltip after all interactions (this consumes response)
            if show_tooltip {
                response.on_hover_text(tooltip_text);
            }
        });

        clicked_index
    }

    /// Get the current selected range
    pub fn get_selected_range(&self) -> Option<&TimeRange> {
        self.selected_range.as_ref()
    }

    /// Get the last hover time (used for click-to-scroll)
    pub fn get_hover_time(&self) -> Option<f64> {
        self.hover_time
    }
}
