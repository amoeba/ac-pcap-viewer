use crate::PcapViewerApp;
use common::Tab;
use eframe::egui;
use egui::ScrollArea;
use egui_extras::{Column, TableBuilder};

pub fn show_weenie_panel(app: &mut PcapViewerApp, ui: &mut egui::Ui, is_mobile: bool) {
    // Clone weenies to avoid borrow checker issues
    let weenies: Vec<common::weenie::Weenie> = app
        .weenie_db
        .sorted_weenies()
        .into_iter()
        .cloned()
        .collect();

    ui.horizontal(|ui| {
        ui.heading("Weenies");
        ui.label(format!("({} objects)", weenies.len()));
    });

    ui.separator();

    // Filter input
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.text_edit_singleline(&mut app.search_query);
        if ui.button("Clear").clicked() {
            app.search_query.clear();
        }
    });

    ui.separator();

    // Filter weenies
    let filter_lower = app.search_query.to_lowercase();
    let filtered_weenies: Vec<&common::weenie::Weenie> = weenies
        .iter()
        .filter(|w| {
            if filter_lower.is_empty() {
                true
            } else {
                w.object_id.to_string().contains(&filter_lower)
                    || w.name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&filter_lower))
                        .unwrap_or(false)
            }
        })
        .collect();

    if is_mobile {
        show_mobile_weenie_list(app, ui, &filtered_weenies);
    } else {
        show_desktop_weenie_table(app, ui, &filtered_weenies);
    }
}

fn show_mobile_weenie_list(
    app: &mut PcapViewerApp,
    ui: &mut egui::Ui,
    weenies: &[&common::weenie::Weenie],
) {
    ScrollArea::vertical().show(ui, |ui| {
        for (idx, weenie) in weenies.iter().enumerate() {
            let is_selected = app.selected_weenie == Some(idx);

            if ui
                .selectable_label(
                    is_selected,
                    format!(
                        "{} - {}",
                        weenie.object_id,
                        weenie.name.as_deref().unwrap_or("<unknown>")
                    ),
                )
                .clicked()
            {
                app.selected_weenie = Some(idx);
            }
        }
    });
}

fn show_desktop_weenie_table(
    app: &mut PcapViewerApp,
    ui: &mut egui::Ui,
    weenies: &[&common::weenie::Weenie],
) {
    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(100.0)) // ObjectID
        .column(Column::remainder().at_least(200.0)) // Name
        .column(Column::auto().at_least(60.0)) // Props
        .column(Column::auto().at_least(60.0)) // Msgs
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("ObjectID");
            });
            header.col(|ui| {
                ui.strong("Name");
            });
            header.col(|ui| {
                ui.strong("Props");
            });
            header.col(|ui| {
                ui.strong("Msgs");
            });
        })
        .body(|body| {
            body.rows(20.0, weenies.len(), |mut row| {
                let row_index = row.index();
                let weenie = weenies[row_index];
                let is_selected = app.selected_weenie == Some(row_index);

                let prop_count = weenie.int_properties.len()
                    + weenie.int64_properties.len()
                    + weenie.bool_properties.len()
                    + weenie.float_properties.len()
                    + weenie.string_properties.len()
                    + weenie.data_id_properties.len()
                    + weenie.instance_id_properties.len();

                row.set_selected(is_selected);

                row.col(|ui| {
                    if ui
                        .selectable_label(is_selected, format!("{}", weenie.object_id))
                        .clicked()
                    {
                        app.selected_weenie = Some(row_index);
                    }
                });

                row.col(|ui| {
                    let name = weenie.name.as_deref().unwrap_or("<unknown>");
                    if ui.selectable_label(is_selected, name).clicked() {
                        app.selected_weenie = Some(row_index);
                    }
                });

                row.col(|ui| {
                    if ui
                        .selectable_label(is_selected, format!("{}", prop_count))
                        .clicked()
                    {
                        app.selected_weenie = Some(row_index);
                    }
                });

                row.col(|ui| {
                    if ui
                        .selectable_label(is_selected, format!("{}", weenie.message_count))
                        .clicked()
                    {
                        app.selected_weenie = Some(row_index);
                    }
                });
            });
        });
}

pub fn show_weenie_detail(app: &mut PcapViewerApp, ui: &mut egui::Ui) {
    // Clone weenies to avoid borrow checker issues
    let weenies: Vec<common::weenie::Weenie> = app
        .weenie_db
        .sorted_weenies()
        .into_iter()
        .cloned()
        .collect();

    // Filter weenies
    let filter_lower = app.search_query.to_lowercase();
    let filtered_weenies: Vec<&common::weenie::Weenie> = weenies
        .iter()
        .filter(|w| {
            if filter_lower.is_empty() {
                true
            } else {
                w.object_id.to_string().contains(&filter_lower)
                    || w.name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&filter_lower))
                        .unwrap_or(false)
            }
        })
        .collect();

    if let Some(idx) = app.selected_weenie {
        if let Some(weenie) = filtered_weenies.get(idx) {
            ScrollArea::vertical().show(ui, |ui| {
                ui.heading(format!(
                    "Weenie: {}",
                    weenie.name.as_deref().unwrap_or("<unknown>")
                ));
                ui.separator();

                egui::Grid::new("weenie_detail_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Object ID:");
                        ui.label(format!("{}", weenie.object_id));
                        ui.end_row();

                        if let Some(name) = &weenie.name {
                            ui.label("Name:");
                            ui.label(name);
                            ui.end_row();
                        }

                        ui.label("First Seen:");
                        ui.label(format!("{:.3}s", weenie.first_seen));
                        ui.end_row();

                        ui.label("Last Updated:");
                        ui.label(format!("{:.3}s", weenie.last_updated));
                        ui.end_row();

                        ui.label("Message Count:");
                        ui.label(format!("{}", weenie.message_count));
                        ui.end_row();
                    });

                ui.separator();

                // Message IDs section with clickable links
                if !weenie.message_ids.is_empty() {
                    ui.heading("Referenced in Messages:");
                    ui.horizontal_wrapped(|ui| {
                        for &msg_id in &weenie.message_ids {
                            if ui.link(format!("#{}", msg_id)).clicked() {
                                // Switch to Messages tab and select this message
                                app.current_tab = Tab::Messages;
                                app.selected_message = Some(msg_id);
                            }
                        }
                    });
                    ui.separator();
                }

                // Properties sections
                show_property_section(ui, "Int Properties", &weenie.int_properties);
                show_property_section(ui, "Int64 Properties", &weenie.int64_properties);
                show_property_section(ui, "Bool Properties", &weenie.bool_properties);
                show_property_section(ui, "Float Properties", &weenie.float_properties);
                show_property_section(ui, "String Properties", &weenie.string_properties);
                show_property_section(ui, "DataId Properties", &weenie.data_id_properties);
                show_property_section(ui, "InstanceId Properties", &weenie.instance_id_properties);
            });
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.label("Select a weenie to view details");
        });
    }
}

fn show_property_section<K, V>(
    ui: &mut egui::Ui,
    title: &str,
    properties: &std::collections::HashMap<K, V>,
) where
    K: std::fmt::Display + std::cmp::Ord,
    V: std::fmt::Display,
{
    if !properties.is_empty() {
        ui.heading(title);

        egui::Grid::new(title)
            .num_columns(2)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                let mut sorted: Vec<_> = properties.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));

                for (key, value) in sorted {
                    ui.label(format!("{}", key));
                    ui.label(format!("{}", value));
                    ui.end_row();
                }
            });

        ui.separator();
    }
}
