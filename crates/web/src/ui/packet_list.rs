//! Packet and message list UI components

use crate::state::json_contains_string;
use crate::{PcapViewerApp, SortField};
// TODO: Re-enable this import when needed
// use ac_parser::messages::ParsedMessage;
use eframe::egui;

/// Draw sort button
pub fn draw_sort_button(app: &mut PcapViewerApp, ui: &mut egui::Ui) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
    if response.clicked() {
        return true;
    }
    response.on_hover_text(if app.sort_ascending {
        "Sort descending"
    } else {
        "Sort ascending"
    });

    let painter = ui.painter();
    let center = rect.center();
    let color = ui.visuals().text_color();

    if app.sort_ascending {
        // Draw up arrow
        let points = [
            center + egui::vec2(-4.0, 2.0),
            center + egui::vec2(0.0, -2.0),
            center + egui::vec2(4.0, 2.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            points.into(),
            color,
            egui::Stroke::NONE,
        ));
    } else {
        // Draw down arrow
        let points = [
            center + egui::vec2(-4.0, -2.0),
            center + egui::vec2(0.0, 2.0),
            center + egui::vec2(4.0, -2.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            points.into(),
            color,
            egui::Stroke::NONE,
        ));
    }

    false
}

/// Draw theme toggle button
pub fn draw_theme_toggle(app: &mut PcapViewerApp, ui: &mut egui::Ui) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
    if response.clicked() {
        app.dark_mode = !app.dark_mode;
    }
    response.on_hover_text("Toggle dark/light mode");

    let painter = ui.painter();
    let center = rect.center();

    if app.dark_mode {
        // Draw sun icon (switch to light mode)
        let sun_color = egui::Color32::from_rgb(255, 200, 50);
        painter.circle_filled(center, 6.0, sun_color);
        // Draw rays
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::PI / 4.0;
            let inner = 7.5;
            let outer = 9.5;
            let start = center + egui::vec2(angle.cos() * inner, angle.sin() * inner);
            let end = center + egui::vec2(angle.cos() * outer, angle.sin() * outer);
            painter.line_segment([start, end], egui::Stroke::new(1.5, sun_color));
        }
    } else {
        // Draw moon icon (switch to dark mode)
        let moon_color = egui::Color32::from_rgb(100, 150, 255);
        painter.circle_filled(center, 7.0, moon_color);
        // Cut out crescent with background color
        let bg_color = ui.visuals().panel_fill;
        painter.circle_filled(center + egui::vec2(4.0, -3.0), 5.5, bg_color);
    }
}

/// Column definition for mobile table
#[derive(Clone)]
struct MobileColumn {
    header: &'static str,
    width_pct: f32,
    right_align: bool,
}

/// Mobile header row
fn mobile_header(ui: &mut egui::Ui, columns: &[MobileColumn], available_width: f32) {
    for col in columns {
        let width = available_width * col.width_pct;
        ui.allocate_ui(egui::vec2(width, ui.spacing().interact_size.y), |ui| {
            ui.with_layout(
                if col.right_align {
                    egui::Layout::right_to_left(egui::Align::Center)
                } else {
                    egui::Layout::left_to_right(egui::Align::Center)
                },
                |ui| {
                    ui.strong(col.header);
                },
            );
        });
    }
    ui.end_row();
}

/// Mobile table cell
fn mobile_cell(
    ui: &mut egui::Ui,
    width: f32,
    right_align: bool,
    selected: bool,
    text: impl Into<egui::WidgetText>,
) -> egui::Response {
    ui.allocate_ui(egui::vec2(width, ui.spacing().interact_size.y), |ui| {
        ui.with_layout(
            if right_align {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            },
            |ui| {
                if selected {
                    ui.visuals_mut().override_text_color =
                        Some(egui::Color32::from_rgb(255, 255, 0));
                }
                ui.selectable_label(selected, text)
            },
        )
        .inner
    })
    .inner
}

/// Show messages list
pub fn show_messages_list(app: &mut PcapViewerApp, ui: &mut egui::Ui, is_mobile: bool) {
    // Pre-collect data to avoid borrow issues
    let search = app.search_query.to_lowercase();
    let sort_field = app.sort_field;
    let sort_ascending = app.sort_ascending;
    let total = app.messages.len();
    let time_filter = app.messages_scrubber.get_selected_range().cloned();

    // Collect timestamps of messages matching search (for highlighting on scrubber)
    if !search.is_empty() {
        let search_matched_timestamps: Vec<f64> = app
            .messages
            .iter()
            .filter(|m| {
                let type_matches = m.message_type.to_lowercase().contains(&search);
                let data_matches = json_contains_string(&m.data, &search);
                type_matches || data_matches
            })
            .map(|m| m.timestamp)
            .collect();
        app.messages_scrubber
            .set_highlighted_timestamps(search_matched_timestamps);
    } else {
        app.messages_scrubber.set_highlighted_timestamps(Vec::new());
    }

    let mut filtered: Vec<(usize, usize, String, String, String)> = app
        .messages
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            // Apply search filter (search both message type and data)
            let matches_search = if search.is_empty() {
                true
            } else {
                // Search in message type
                let type_matches = m.message_type.to_lowercase().contains(&search);
                // Search in message data (deep search)
                let data_matches = json_contains_string(&m.data, &search);
                // Match if either type or data contains the search string
                type_matches || data_matches
            };

            // Apply time filter
            let matches_time = if let Some(ref range) = time_filter {
                range.contains(m.timestamp)
            } else {
                true
            };

            matches_search && matches_time
        })
        .map(|(idx, m)| {
            (
                idx,
                m.id,
                m.message_type.clone(),
                m.direction.clone(),
                m.opcode.clone(),
            )
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort_field {
            SortField::Id => a.1.cmp(&b.1),
            SortField::Type => a.2.cmp(&b.2),
            SortField::Direction => a.3.cmp(&b.3),
        };
        if sort_ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });

    ui.horizontal(|ui| {
        ui.label(format!("{}/{} messages", filtered.len(), total));
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();

        if is_mobile {
            ui.set_min_width(available_width);
            let columns = [
                MobileColumn {
                    header: "ID",
                    width_pct: 0.12,
                    right_align: false,
                },
                MobileColumn {
                    header: "Type",
                    width_pct: 0.76,
                    right_align: false,
                },
                MobileColumn {
                    header: "Dir",
                    width_pct: 0.12,
                    right_align: true,
                },
            ];
            let widths: Vec<f32> = columns
                .iter()
                .map(|c| available_width * c.width_pct)
                .collect();

            egui::Grid::new("messages_grid")
                .num_columns(3)
                .striped(true)
                .spacing(egui::vec2(4.0, 4.0))
                .show(ui, |ui| {
                    mobile_header(ui, &columns, available_width);

                    for (original_idx, id, msg_type, direction, _opcode) in &filtered {
                        let is_selected = app.selected_message == Some(*original_idx);

                        if mobile_cell(ui, widths[0], false, is_selected, id.to_string()).clicked()
                        {
                            app.selected_message = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        let display_type = if msg_type.len() > 25 {
                            format!("{}…", &msg_type[..24])
                        } else {
                            msg_type.clone()
                        };
                        if mobile_cell(ui, widths[1], false, is_selected, display_type).clicked() {
                            app.selected_message = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        let dir_color = if direction == "Send" {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else {
                            egui::Color32::from_rgb(100, 255, 150)
                        };
                        let dir_text = if direction == "Send" { "S" } else { "R" };
                        if mobile_cell(
                            ui,
                            widths[2],
                            true,
                            is_selected,
                            egui::RichText::new(dir_text).color(dir_color),
                        )
                        .clicked()
                        {
                            app.selected_message = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        ui.end_row();
                    }
                });
        } else {
            // Desktop layout
            egui::Grid::new("messages_grid")
                .num_columns(4)
                .striped(true)
                .min_col_width(50.0)
                .show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("Type");
                    ui.strong("Dir");
                    ui.strong("OpCode");
                    ui.end_row();

                    for (original_idx, id, msg_type, direction, opcode) in &filtered {
                        let is_selected = app.selected_message == Some(*original_idx);

                        if ui.selectable_label(is_selected, id.to_string()).clicked() {
                            app.selected_message = Some(*original_idx);
                        }
                        if ui.selectable_label(is_selected, msg_type).clicked() {
                            app.selected_message = Some(*original_idx);
                        }
                        let dir_color = if direction == "Send" {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else {
                            egui::Color32::from_rgb(100, 255, 150)
                        };
                        if ui
                            .selectable_label(
                                is_selected,
                                egui::RichText::new(direction).color(dir_color),
                            )
                            .clicked()
                        {
                            app.selected_message = Some(*original_idx);
                        }
                        if ui.selectable_label(is_selected, opcode).clicked() {
                            app.selected_message = Some(*original_idx);
                        }
                        ui.end_row();
                    }
                });
        }
    });
}

/// Show packets list
pub fn show_packets_list(app: &mut PcapViewerApp, ui: &mut egui::Ui, is_mobile: bool) {
    // Pre-collect data to avoid borrow issues
    let sort_field = app.sort_field;
    let sort_ascending = app.sort_ascending;
    let total = app.packets.len();
    let time_filter = app.fragments_scrubber.get_selected_range().cloned();

    let mut filtered: Vec<(usize, usize, u32, String, u32, u16)> = app
        .packets
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            // Apply time filter
            if let Some(ref range) = time_filter {
                range.contains(p.timestamp)
            } else {
                true
            }
        })
        .map(|(idx, p)| {
            (
                idx,
                p.id,
                p.header.sequence,
                format!("{:?}", p.direction),
                p.header.flags.bits(),
                p.header.size,
            )
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort_field {
            SortField::Id => a.1.cmp(&b.1),
            SortField::Type => a.2.cmp(&b.2),
            SortField::Direction => a.3.cmp(&b.3),
        };
        if sort_ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });

    ui.horizontal(|ui| {
        ui.label(format!("{}/{} packets", filtered.len(), total));
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();

        if is_mobile {
            ui.set_min_width(available_width);
            let columns = [
                MobileColumn {
                    header: "ID",
                    width_pct: 0.15,
                    right_align: false,
                },
                MobileColumn {
                    header: "Seq",
                    width_pct: 0.20,
                    right_align: false,
                },
                MobileColumn {
                    header: "Dir",
                    width_pct: 0.15,
                    right_align: true,
                },
                MobileColumn {
                    header: "Flags",
                    width_pct: 0.25,
                    right_align: false,
                },
                MobileColumn {
                    header: "Size",
                    width_pct: 0.25,
                    right_align: false,
                },
            ];
            let widths: Vec<f32> = columns
                .iter()
                .map(|c| available_width * c.width_pct)
                .collect();

            egui::Grid::new("packets_grid")
                .num_columns(5)
                .striped(true)
                .spacing(egui::vec2(4.0, 4.0))
                .show(ui, |ui| {
                    mobile_header(ui, &columns, available_width);

                    for (original_idx, id, seq, direction, flags, size) in &filtered {
                        let is_selected = app.selected_packet == Some(*original_idx);

                        if mobile_cell(ui, widths[0], false, is_selected, id.to_string()).clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        if mobile_cell(ui, widths[1], false, is_selected, seq.to_string()).clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        let dir_color = if direction == "ClientToServer" {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else {
                            egui::Color32::from_rgb(100, 255, 150)
                        };
                        let dir_text = if direction == "ClientToServer" {
                            "C→S"
                        } else {
                            "S→C"
                        };
                        if mobile_cell(
                            ui,
                            widths[2],
                            true,
                            is_selected,
                            egui::RichText::new(dir_text).color(dir_color),
                        )
                        .clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        if mobile_cell(ui, widths[3], false, is_selected, format!("{flags:08X}"))
                            .clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        if mobile_cell(ui, widths[4], false, is_selected, size.to_string())
                            .clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                            app.show_detail_panel = true;
                        }

                        ui.end_row();
                    }
                });
        } else {
            // Desktop layout
            egui::Grid::new("packets_grid")
                .num_columns(6)
                .striped(true)
                .min_col_width(50.0)
                .show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("Sequence");
                    ui.strong("Direction");
                    ui.strong("Flags");
                    ui.strong("Size");
                    ui.strong("Data Size");
                    ui.end_row();

                    for (original_idx, id, seq, direction, flags, size) in &filtered {
                        let is_selected = app.selected_packet == Some(*original_idx);

                        if ui.selectable_label(is_selected, id.to_string()).clicked() {
                            app.selected_packet = Some(*original_idx);
                        }
                        if ui.selectable_label(is_selected, seq.to_string()).clicked() {
                            app.selected_packet = Some(*original_idx);
                        }
                        let dir_color = if direction == "ClientToServer" {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else {
                            egui::Color32::from_rgb(100, 255, 150)
                        };
                        if ui
                            .selectable_label(
                                is_selected,
                                egui::RichText::new(direction).color(dir_color),
                            )
                            .clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                        }
                        if ui
                            .selectable_label(is_selected, format!("{flags:08X}"))
                            .clicked()
                        {
                            app.selected_packet = Some(*original_idx);
                        }
                        if ui.selectable_label(is_selected, size.to_string()).clicked() {
                            app.selected_packet = Some(*original_idx);
                        }
                        ui.end_row();
                    }
                });
        }
    });
}
