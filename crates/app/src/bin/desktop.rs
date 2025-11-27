//! Desktop application entry point for AC PCAP Parser
//!
//! This binary provides a native desktop GUI with features like
//! native file dialogs and keyboard shortcuts.

use app::PcapViewerApp;

fn main() -> eframe::Result<()> {
    // Initialize logging for desktop
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "AC PCAP Parser",
        native_options,
        Box::new(|cc| Ok(Box::new(PcapViewerApp::new(cc)))),
    )
}
