//! Weenie (game object) list and detail display
//!
//! This module provides UI components for displaying weenies (game objects)
//! aggregated from PCAP messages.

use crate::PcapViewerApp;
use eframe::egui;
use lib::weenie::Weenie;

/// Show the weenie list with search and detail panel
pub fn show_weenie_list(app: &mut PcapViewerApp, ui: &mut egui::Ui, is_mobile: bool) {
    let weenies = app.weenie_db.sorted_weenies();

    if weenies.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("No weenies found in PCAP");
        });
        return;
    }

    // Filter weenies based on search query
    let search = app.search_query.to_lowercase();
    let filtered_weenies: Vec<&Weenie> = if search.is_empty() {
        weenies
    } else {
        weenies
            .into_iter()
            .filter(|w| {
                // Search by object ID
                if w.object_id.to_string().contains(&search) {
                    return true;
                }

                // Search by name
                if let Some(ref name) = w.name {
                    if name.to_lowercase().contains(&search) {
                        return true;
                    }
                }

                // Search in properties (convert to JSON and search)
                let json = serde_json::to_string(w).unwrap_or_default();
                json.to_lowercase().contains(&search)
            })
            .collect()
    };

    // Create a table with weenie information
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                if is_mobile {
                    ui.label(egui::RichText::new("ID").strong());
                    ui.separator();
                    ui.label(egui::RichText::new("Name").strong());
                } else {
                    ui.label(
                        egui::RichText::new(format!("{:>10}", "Object ID"))
                            .strong()
                            .monospace(),
                    );
                    ui.separator();
                    ui.label(egui::RichText::new("Name").strong());
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("{:>6}", "Props"))
                            .strong()
                            .monospace(),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("{:>6}", "Msgs"))
                            .strong()
                            .monospace(),
                    );
                }
            });

            ui.separator();

            // Rows
            for (idx, weenie) in filtered_weenies.iter().enumerate() {
                let is_selected = app.selected_message == Some(idx); // Reuse selected_message for weenie selection

                let response = ui.selectable_label(is_selected, "");

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    if is_mobile {
                        // Mobile: compact layout
                        ui.label(
                            egui::RichText::new(format!("{}", weenie.object_id))
                                .monospace()
                                .color(if is_selected {
                                    ui.visuals().strong_text_color()
                                } else {
                                    ui.visuals().text_color()
                                }),
                        );
                        ui.separator();
                        ui.label(weenie.name.as_deref().unwrap_or("<unknown>").to_string());
                    } else {
                        // Desktop: full layout
                        ui.label(
                            egui::RichText::new(format!("{:>10}", weenie.object_id))
                                .monospace()
                                .color(if is_selected {
                                    ui.visuals().strong_text_color()
                                } else {
                                    ui.visuals().text_color()
                                }),
                        );
                        ui.separator();

                        let name = weenie.name.as_deref().unwrap_or("<unknown>");
                        ui.label(truncate_string(name, 40));
                        ui.separator();

                        // Count total properties
                        let prop_count = weenie.int_properties.len()
                            + weenie.int64_properties.len()
                            + weenie.bool_properties.len()
                            + weenie.float_properties.len()
                            + weenie.string_properties.len()
                            + weenie.data_id_properties.len()
                            + weenie.instance_id_properties.len();

                        ui.label(egui::RichText::new(format!("{:>6}", prop_count)).monospace());
                        ui.separator();

                        ui.label(
                            egui::RichText::new(format!("{:>6}", weenie.message_count)).monospace(),
                        );
                    }
                });

                if response.clicked() {
                    app.selected_message = Some(idx);
                    if is_mobile {
                        app.show_detail_panel = true;
                    }
                }
            }
        });
}

/// Show detailed information about the selected weenie
pub fn show_weenie_detail(app: &PcapViewerApp, ui: &mut egui::Ui) {
    if let Some(selected_idx) = app.selected_message {
        let weenies = app.weenie_db.sorted_weenies();

        // Apply search filter
        let search = app.search_query.to_lowercase();
        let filtered_weenies: Vec<&Weenie> = if search.is_empty() {
            weenies
        } else {
            weenies
                .into_iter()
                .filter(|w| {
                    if w.object_id.to_string().contains(&search) {
                        return true;
                    }
                    if let Some(ref name) = w.name {
                        if name.to_lowercase().contains(&search) {
                            return true;
                        }
                    }
                    let json = serde_json::to_string(w).unwrap_or_default();
                    json.to_lowercase().contains(&search)
                })
                .collect()
        };

        if let Some(weenie) = filtered_weenies.get(selected_idx) {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.heading("Weenie Details");
                    ui.separator();

                    // Basic info
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Object ID:").strong());
                        ui.label(egui::RichText::new(format!("{}", weenie.object_id)).monospace());
                    });

                    if let Some(ref name) = weenie.name {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Name:").strong());
                            ui.label(name);
                        });
                    }

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Messages:").strong());
                        ui.label(format!("{}", weenie.message_count));
                    });

                    ui.separator();

                    // Properties sections
                    if !weenie.int_properties.is_empty() {
                        ui.collapsing(
                            format!("Int Properties ({})", weenie.int_properties.len()),
                            |ui| {
                                for (key, value) in &weenie.int_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(
                                            egui::RichText::new(format!("{value}")).monospace(),
                                        );
                                    });
                                }
                            },
                        );
                    }

                    if !weenie.int64_properties.is_empty() {
                        ui.collapsing(
                            format!("Int64 Properties ({})", weenie.int64_properties.len()),
                            |ui| {
                                for (key, value) in &weenie.int64_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(
                                            egui::RichText::new(format!("{value}")).monospace(),
                                        );
                                    });
                                }
                            },
                        );
                    }

                    if !weenie.bool_properties.is_empty() {
                        ui.collapsing(
                            format!("Bool Properties ({})", weenie.bool_properties.len()),
                            |ui| {
                                for (key, value) in &weenie.bool_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(
                                            egui::RichText::new(format!("{value}")).monospace(),
                                        );
                                    });
                                }
                            },
                        );
                    }

                    if !weenie.float_properties.is_empty() {
                        ui.collapsing(
                            format!("Float Properties ({})", weenie.float_properties.len()),
                            |ui| {
                                for (key, value) in &weenie.float_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(
                                            egui::RichText::new(format!("{value}")).monospace(),
                                        );
                                    });
                                }
                            },
                        );
                    }

                    if !weenie.string_properties.is_empty() {
                        ui.collapsing(
                            format!("String Properties ({})", weenie.string_properties.len()),
                            |ui| {
                                for (key, value) in &weenie.string_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(value);
                                    });
                                }
                            },
                        );
                    }

                    if !weenie.data_id_properties.is_empty() {
                        ui.collapsing(
                            format!("DataId Properties ({})", weenie.data_id_properties.len()),
                            |ui| {
                                for (key, value) in &weenie.data_id_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(
                                            egui::RichText::new(format!("0x{:08X}", value))
                                                .monospace(),
                                        );
                                    });
                                }
                            },
                        );
                    }

                    if !weenie.instance_id_properties.is_empty() {
                        ui.collapsing(
                            format!(
                                "InstanceId Properties ({})",
                                weenie.instance_id_properties.len()
                            ),
                            |ui| {
                                for (key, value) in &weenie.instance_id_properties {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{key}:")).weak());
                                        ui.label(
                                            egui::RichText::new(format!("0x{:08X}", value))
                                                .monospace(),
                                        );
                                    });
                                }
                            },
                        );
                    }

                    ui.separator();

                    // Metadata
                    ui.collapsing("Metadata", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("First Seen:").weak());
                            ui.label(format!("{:.3}s", weenie.first_seen));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Last Updated:").weak());
                            ui.label(format!("{:.3}s", weenie.last_updated));
                        });
                    });
                });
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a weenie to view details");
        });
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
