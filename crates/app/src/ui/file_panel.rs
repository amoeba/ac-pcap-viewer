//! File loading and management UI components

use crate::PcapViewerApp;
use common::PacketParser;
use eframe::egui;

#[allow(dead_code)]
static BOT_BASE_URL: &str = env!("BOT_BASE_URL");

/// Parse PCAP data and update the app state
pub fn parse_pcap_data(app: &mut PcapViewerApp, data: &[u8]) {
    app.is_loading = true;
    app.status_message = "Parsing PCAP file...".to_string();

    let mut parser = PacketParser::new();
    match parser.parse_pcap_bytes(data) {
        Ok((packets, messages, weenie_db)) => {
            app.status_message = format!(
                "Loaded {} packets, {} messages, {} weenies",
                packets.len(),
                messages.len(),
                weenie_db.count()
            );
            app.packets = packets;
            app.messages = messages;
            app.weenie_db = weenie_db;

            // Clear any URL load errors on success
            app.url_load_error = None;
            app.selected_message = if app.messages.is_empty() {
                None
            } else {
                Some(0)
            };
            app.selected_packet = if app.packets.is_empty() {
                None
            } else {
                Some(0)
            };

            // Update time scrubbers
            // Messages scrubber uses message timestamps
            let message_timestamps: Vec<f64> = app.messages.iter().map(|m| m.timestamp).collect();
            app.messages_scrubber.update_density(&message_timestamps);

            // Fragments scrubber uses packet timestamps
            let packet_timestamps: Vec<f64> = app.packets.iter().map(|p| p.timestamp).collect();
            app.fragments_scrubber.update_density(&packet_timestamps);
        }
        Err(e) => {
            app.show_error(format!("Error parsing PCAP: {e}"));
        }
    }
    app.is_loading = false;
}

/// Load example PCAP file
#[cfg(target_arch = "wasm32")]
pub fn load_example(app: &mut PcapViewerApp, ctx: &egui::Context) {
    load_from_url(app, "./example.pcap".to_string(), ctx);
}

/// Load example PCAP file (native)
#[cfg(not(target_arch = "wasm32"))]
pub fn load_example(app: &mut PcapViewerApp, _ctx: &egui::Context) {
    match std::fs::read("static/example.pcap") {
        Ok(data) => parse_pcap_data(app, &data),
        Err(e) => app.show_error(format!("Failed to load example PCAP: {e}")),
    }
}

/// Load PCAP from URL (WASM)
#[cfg(target_arch = "wasm32")]
pub fn load_from_url(app: &mut PcapViewerApp, url: String, ctx: &egui::Context) {
    if app.is_loading {
        return;
    }

    app.is_loading = true;
    app.status_message = format!("Loading PCAP from {}...", url);

    // Clear any previous errors
    if let Ok(mut error) = app.fetched_error.lock() {
        *error = None;
    }
    app.url_load_error = None;

    let fetched_data = app.fetched_data.clone();
    let fetched_error = app.fetched_error.clone();
    let ctx = ctx.clone();

    wasm_bindgen_futures::spawn_local(async move {
        match fetch_bytes(&url).await {
            Ok(bytes) => {
                if let Ok(mut data) = fetched_data.lock() {
                    *data = Some(bytes);
                }
                // Clear error on success
                if let Ok(mut error) = fetched_error.lock() {
                    *error = None;
                }
                ctx.request_repaint();
            }
            Err(e) => {
                log::error!("Failed to fetch PCAP from URL: {}", e);
                // Store error for display
                if let Ok(mut error) = fetched_error.lock() {
                    *error = Some(e);
                }
                ctx.request_repaint();
            }
        }
    });
}

/// Load PCAP from URL (native - not supported)
#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_url(app: &mut PcapViewerApp, _url: String, _ctx: &egui::Context) {
    // Native: URL loading not supported
    app.status_message = "URL loading not supported in native mode".to_string();
}

/// Load PCAP from Discord (WASM)
#[cfg(target_arch = "wasm32")]
pub fn load_from_discord(
    app: &mut PcapViewerApp,
    channel_id: String,
    message_id: String,
    ctx: &egui::Context,
) {
    if app.is_loading {
        return;
    }

    app.is_loading = true;
    app.status_message = format!(
        "Loading PCAP from Discord ({}:{})...",
        channel_id, message_id
    );

    // Clear any previous errors
    if let Ok(mut error) = app.fetched_error.lock() {
        *error = None;
    }
    app.discord_load_error = None;

    let fetched_data = app.fetched_data.clone();
    let fetched_error = app.fetched_error.clone();
    let ctx = ctx.clone();

    wasm_bindgen_futures::spawn_local(async move {
        match fetch_discord_pcap(&channel_id, &message_id).await {
            Ok(bytes) => {
                if let Ok(mut data) = fetched_data.lock() {
                    *data = Some(bytes);
                }
                // Clear error on success
                if let Ok(mut error) = fetched_error.lock() {
                    *error = None;
                }
                ctx.request_repaint();
            }
            Err(e) => {
                log::error!("Failed to fetch PCAP from Discord: {}", e);
                // Store error for display
                if let Ok(mut error) = fetched_error.lock() {
                    *error = Some(e);
                }
                ctx.request_repaint();
            }
        }
    });
}

/// Load PCAP from Discord (native - not supported)
#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_discord(
    app: &mut PcapViewerApp,
    _channel_id: String,
    _message_id: String,
    _ctx: &egui::Context,
) {
    // Native: Discord loading not supported
    app.status_message = "Discord loading not supported in native mode".to_string();
}

/// Fetch PCAP from Discord API (WASM only)
#[cfg(target_arch = "wasm32")]
async fn fetch_discord_pcap(channel_id: &str, message_id: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let url = format!(
        "{}/api/discord/channels/{}/messages/{}/attachments",
        BOT_BASE_URL, channel_id, message_id
    );

    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or_else(|| "No window object".to_string())?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not Response".to_string())?;

    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(format!("HTTP {}: {}", status, status_text));
    }

    let array_buffer = JsFuture::from(resp.array_buffer().unwrap())
        .await
        .map_err(|e| format!("Failed to get array buffer: {:?}", e))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}

/// Fetch bytes from URL (WASM only)
#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or_else(|| "No window object".to_string())?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not Response".to_string())?;

    if !resp.ok() {
        return Err(format!("HTTP {}: {}", resp.status(), resp.status_text()));
    }

    let array_buffer = JsFuture::from(resp.array_buffer().unwrap())
        .await
        .map_err(|e| format!("Failed to get array buffer: {:?}", e))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}

/// Open file dialog (desktop only)
#[cfg(feature = "desktop")]
pub fn open_file_dialog(app: &mut PcapViewerApp) {
    use rfd::FileDialog;

    if let Some(path) = FileDialog::new()
        .add_filter("PCAP files", &["pcap", "pcapng"])
        .pick_file()
    {
        app.pending_file_path = Some(path);
    }
}

/// Preview files being dropped
pub fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));

        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            "Drop PCAP file here",
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

/// Show URL input dialog
pub fn show_url_dialog(app: &mut PcapViewerApp, ctx: &egui::Context) {
    let mut close_dialog = false;

    egui::Window::new("Load from URL")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Enter PCAP file URL:");
            ui.add_space(5.0);

            let response = ui.text_edit_singleline(&mut app.url_input);
            if response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !app.url_input.is_empty()
            {
                load_from_url(app, app.url_input.clone(), ctx);
                close_dialog = true;
            }

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                    if ui.button("Load").clicked() && !app.url_input.is_empty() {
                        load_from_url(app, app.url_input.clone(), ctx);
                        close_dialog = true;
                    }
                });
            });

            // Display error if URL load failed
            if let Some(ref error) = app.url_load_error {
                ui.add_space(5.0);
                ui.colored_label(egui::Color32::RED, error);
            }
        });

    if close_dialog {
        app.show_url_dialog = false;
        app.url_load_error = None;
    }
}

/// Show settings dialog
pub fn show_settings_dialog(app: &mut PcapViewerApp, ctx: &egui::Context) {
    let mut close_settings = false;

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Appearance");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Theme:");
                if ui.selectable_label(app.dark_mode, "Dark").clicked() {
                    app.dark_mode = true;
                }
                if ui.selectable_label(!app.dark_mode, "Light").clicked() {
                    app.dark_mode = false;
                }
            });

            ui.add_space(10.0);

            ui.heading("Default View");
            ui.separator();

            // Default tab is always Messages (no selection needed)

            ui.horizontal(|ui| {
                ui.label("Sort Order:");
                if ui
                    .selectable_label(app.sort_ascending, "Ascending")
                    .clicked()
                {
                    app.sort_ascending = true;
                }
                if ui
                    .selectable_label(!app.sort_ascending, "Descending")
                    .clicked()
                {
                    app.sort_ascending = false;
                }
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        close_settings = true;
                    }
                });
            });
        });

    if close_settings {
        app.show_settings = false;
    }
}

/// Show about dialog
pub fn show_about_dialog(app: &mut PcapViewerApp, ctx: &egui::Context) {
    let mut close_about = false;

    egui::Window::new("About AC PCAP Parser")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("AC PCAP Parser");
                ui.add_space(5.0);

                let git_sha = option_env!("GIT_SHA").unwrap_or("dev");
                let short_sha = if git_sha.len() > 7 {
                    &git_sha[..7]
                } else {
                    git_sha
                };
                ui.label(format!("Version: {short_sha}"));

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("A web-based parser for Asheron's Call");
                ui.label("PCAP network traffic files.");

                ui.add_space(10.0);

                ui.hyperlink_to("View on GitHub", "https://github.com/amoeba/ac-pcap-parser");

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Claude branding
                let claude_color = egui::Color32::from_rgb(217, 119, 87);
                ui.horizontal(|ui| {
                    // Claude logo
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 6.0, claude_color);
                    ui.hyperlink_to(
                        egui::RichText::new("Made with Claude").color(claude_color),
                        "https://claude.ai",
                    );
                });

                ui.add_space(20.0);

                if ui.button("Close").clicked() {
                    close_about = true;
                }
            });
        });

    if close_about {
        app.show_about = false;
    }
}
