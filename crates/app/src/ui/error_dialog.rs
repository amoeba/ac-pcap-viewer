//! Error dialog UI component

use crate::PcapViewerApp;
use eframe::egui;

/// Show error dialog if there's an error to display
pub fn show_error_dialog(app: &mut PcapViewerApp, ctx: &egui::Context) {
    if !app.show_error_dialog {
        return;
    }

    let mut close_dialog = false;

    egui::Window::new("Error")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(&app.error_dialog_message);
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            });
        });

    if close_dialog {
        app.show_error_dialog = false;
    }
}
