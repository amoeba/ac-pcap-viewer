//! Detail panel UI components for displaying message/packet details

use crate::{PcapViewerApp, Tab, ViewMode};
use eframe::egui;
use egui_json_tree::JsonTree;
use lib::{messages::ParsedMessage, ParsedPacket};

/// Show detail content in the detail panel
pub fn show_detail_content(app: &mut PcapViewerApp, ui: &mut egui::Ui) {
    // View mode toggle buttons
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.view_mode, ViewMode::Tree, "Tree");
        ui.selectable_value(&mut app.view_mode, ViewMode::Binary, "Binary");
    });
    ui.separator();

    match app.view_mode {
        ViewMode::Tree => match app.current_tab {
            Tab::Messages => {
                if let Some(idx) = app.selected_message {
                    if idx < app.messages.len() {
                        let tree_id = format!("message_tree_{idx}");
                        JsonTree::new(&tree_id, &app.messages[idx].data)
                            .default_expand(egui_json_tree::DefaultExpand::ToLevel(1))
                            .show(ui);
                    } else {
                        ui.label("No message selected");
                    }
                } else {
                    ui.label("No message selected");
                }
            }
            Tab::Fragments => {
                if let Some(idx) = app.selected_packet {
                    if idx < app.packets.len() {
                        if let Ok(value) = serde_json::to_value(&app.packets[idx]) {
                            let tree_id = format!("packet_tree_{idx}");
                            JsonTree::new(&tree_id, &value)
                                .default_expand(egui_json_tree::DefaultExpand::ToLevel(1))
                                .show(ui);
                        } else {
                            ui.label("Error displaying packet");
                        }
                    } else {
                        ui.label("No packet selected");
                    }
                } else {
                    ui.label("No packet selected");
                }
            }
        },
        ViewMode::Binary => match app.current_tab {
            Tab::Messages => {
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
            Tab::Fragments => {
                if let Some(idx) = app.selected_packet {
                    if idx < app.packets.len() {
                        show_hex_dump_packet(ui, &app.packets[idx]);
                    } else {
                        ui.label("No packet selected");
                    }
                } else {
                    ui.label("No packet selected");
                }
            }
        },
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

/// Extract binary data from a packet fragment
fn extract_packet_binary(packet: &ParsedPacket) -> Option<Vec<u8>> {
    // First, try to use the raw payload which has all packet data
    if !packet.raw_payload.is_empty() {
        return Some(packet.raw_payload.clone());
    }

    // Fall back to fragment data (base64-encoded) if available
    if let Some(ref fragment) = packet.fragment {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        if let Ok(bytes) = BASE64.decode(&fragment.data) {
            return Some(bytes);
        }
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

/// Display hex dump for a packet
fn show_hex_dump_packet(ui: &mut egui::Ui, packet: &ParsedPacket) {
    if let Some(data) = extract_packet_binary(packet) {
        render_hex_dump(ui, &data);
    } else {
        ui.label("No binary data available for this packet");
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
