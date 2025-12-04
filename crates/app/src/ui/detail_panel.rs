//! Detail panel UI components for displaying message/packet details

use crate::ui::hyper_tree::AcJsonTree;
use crate::{PcapViewerApp, Tab, ViewMode};
use eframe::egui;
use lib::messages::ParsedMessage;

/// Show detail content in the detail panel
pub fn show_detail_content(app: &mut PcapViewerApp, ui: &mut egui::Ui) {
    // For Weenies tab, show custom weenie detail view
    if app.current_tab == Tab::Weenies {
        super::weenie_panel::show_weenie_detail(app, ui);
        return;
    }

    // View mode toggle buttons
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.view_mode, ViewMode::Tree, "Tree");
        ui.selectable_value(&mut app.view_mode, ViewMode::JSON, "JSON");
        ui.selectable_value(&mut app.view_mode, ViewMode::Binary, "Binary");
    });
    ui.separator();

    // Track filter clicks to update after the match block
    let mut filter_value: Option<String> = None;

    match app.view_mode {
        ViewMode::JSON => {
            if let Some(idx) = app.selected_message {
                if idx < app.messages.len() {
                    show_pretty_json(ui, &app.messages[idx].data);
                } else {
                    ui.label("No message selected");
                }
            } else {
                ui.label("No message selected");
            }
        }
        ViewMode::Tree => {
            if let Some(idx) = app.selected_message {
                if idx < app.messages.len() {
                    let tree_id = format!("message_tree_{idx}");
                    let response = AcJsonTree::new(&tree_id).show(ui, &app.messages[idx].data);
                    if let Some(value) = response.filter_clicked {
                        filter_value = Some(value);
                    }
                } else {
                    ui.label("No message selected");
                }
            } else {
                ui.label("No message selected");
            }
        }
        ViewMode::Binary => {
            if let Some(idx) = app.selected_message {
                if idx < app.messages.len() {
                    show_hex_dump(ui, &app.messages[idx]);
                } else {
                    ui.label("No message selected");
                }
            } else {
                ui.label("No message selected");
            }
        }
    }

    // Handle filter click - update search query
    if let Some(value) = filter_value {
        app.search_query = value;
    }
}

/// Extract binary data from a message
fn extract_message_binary(message: &ParsedMessage) -> Option<Vec<u8>> {
    // Use the raw_bytes field which contains the original message bytes
    if !message.raw_bytes.is_empty() {
        return Some(message.raw_bytes.clone());
    }
    None
}

/// Display hex dump for a message
fn show_hex_dump(ui: &mut egui::Ui, message: &ParsedMessage) {
    if let Some(data) = extract_message_binary(message) {
        render_hex_dump(ui, &data);
    } else {
        ui.label("No binary data available for this message");
    }
}

/// Render a hex dump view of binary data
fn render_hex_dump(ui: &mut egui::Ui, data: &[u8]) {
    use egui::text::LayoutJob;
    use egui::{Color32, FontId, TextFormat};

    let bytes_per_line = 16;
    let mut job = LayoutJob::default();

    // Use monospace font for the entire hex dump
    let font_id = FontId::monospace(12.0);
    let offset_color = if ui.visuals().dark_mode {
        Color32::from_rgb(128, 128, 255)
    } else {
        Color32::from_rgb(64, 64, 200)
    };
    let hex_color = ui.visuals().text_color();
    let ascii_color = if ui.visuals().dark_mode {
        Color32::from_rgb(128, 255, 128)
    } else {
        Color32::from_rgb(64, 150, 64)
    };

    for (i, chunk) in data.chunks(bytes_per_line).enumerate() {
        let offset = i * bytes_per_line;

        // Offset column
        job.append(
            &format!("{offset:08x}  "),
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: offset_color,
                ..Default::default()
            },
        );

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            job.append(
                &format!("{byte:02x} "),
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: hex_color,
                    ..Default::default()
                },
            );

            // Add extra space after 8 bytes for readability
            if j == 7 {
                job.append(
                    " ",
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: hex_color,
                        ..Default::default()
                    },
                );
            }
        }

        // Pad if line is not full
        if chunk.len() < bytes_per_line {
            let padding_bytes = bytes_per_line - chunk.len();
            let extra_space = if chunk.len() < 8 { 1 } else { 0 };
            let padding = " ".repeat(padding_bytes * 3 + extra_space);
            job.append(
                &padding,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: hex_color,
                    ..Default::default()
                },
            );
        }

        // ASCII representation
        job.append(
            " |",
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: hex_color,
                ..Default::default()
            },
        );

        for byte in chunk.iter() {
            let ch = if *byte >= 32 && *byte < 127 {
                *byte as char
            } else {
                '.'
            };
            job.append(
                &ch.to_string(),
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: ascii_color,
                    ..Default::default()
                },
            );
        }

        job.append(
            "|\n",
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: hex_color,
                ..Default::default()
            },
        );
    }

    // Wrap in horizontal and vertical scroll area
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Set layout to prevent wrapping
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                ui.add(egui::Label::new(job).extend());
            });
        });
}

/// Show pretty-printed JSON
fn show_pretty_json(ui: &mut egui::Ui, value: &serde_json::Value) {
    use egui::text::LayoutJob;
    use egui::{FontId, TextFormat};

    let json_str = match serde_json::to_string_pretty(value) {
        Ok(s) => s,
        Err(e) => {
            ui.label(format!("Error formatting JSON: {e}"));
            return;
        }
    };

    let mut job = LayoutJob::default();
    let font_id = FontId::monospace(12.0);
    let text_color = ui.visuals().text_color();

    // Simple syntax highlighting
    for line in json_str.lines() {
        let trimmed = line.trim_start();

        // Determine color based on line content
        let color = if trimmed.starts_with('"') && trimmed.contains(':') {
            // Field names (keys)
            if ui.visuals().dark_mode {
                egui::Color32::from_rgb(156, 220, 254) // Light blue
            } else {
                egui::Color32::from_rgb(0, 92, 197) // Dark blue
            }
        } else if trimmed.starts_with('"') {
            // String values
            if ui.visuals().dark_mode {
                egui::Color32::from_rgb(206, 145, 120) // Peach
            } else {
                egui::Color32::from_rgb(163, 21, 21) // Dark red
            }
        } else if trimmed.starts_with(|c: char| c.is_ascii_digit())
            || trimmed.starts_with("true")
            || trimmed.starts_with("false")
            || trimmed.starts_with("null")
        {
            // Numbers and literals
            if ui.visuals().dark_mode {
                egui::Color32::from_rgb(181, 206, 168) // Light green
            } else {
                egui::Color32::from_rgb(9, 134, 88) // Dark green
            }
        } else {
            text_color
        };

        job.append(
            line,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
        job.append(
            "\n",
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: text_color,
                ..Default::default()
            },
        );
    }

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                ui.add(egui::Label::new(job).extend());
            });
        });
}
