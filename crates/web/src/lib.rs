//! Web UI for Asheron's Call PCAP Parser
//!
//! A drag-and-drop web interface built with egui for parsing AC PCAP files.

mod state;
mod time_scrubber;
mod ui;

use ac_parser::{messages::ParsedMessage, ParsedPacket};
use eframe::egui;
use std::sync::{Arc, Mutex};
use time_scrubber::TimeScrubber;

// Re-export state types for convenience
pub use ac_pcap_lib::{SortField, Tab, ViewMode};
use state::{MOBILE_BREAKPOINT, MOBILE_SCALE, TABLET_BREAKPOINT};

// Shared state for async loading
type SharedData = Arc<Mutex<Option<Vec<u8>>>>;
type SharedError = Arc<Mutex<Option<String>>>;

pub struct PcapViewerApp {
    // Data
    messages: Vec<ParsedMessage>,
    packets: Vec<ParsedPacket>,

    // UI State
    current_tab: Tab,
    selected_message: Option<usize>,
    selected_packet: Option<usize>,
    search_query: String,
    sort_field: SortField,
    sort_ascending: bool,
    view_mode: ViewMode,

    // Status
    status_message: String,
    is_loading: bool,

    // Theme
    dark_mode: bool,

    // Responsive layout state
    show_detail_panel: bool,

    // Dropped file data
    dropped_file_data: Option<Vec<u8>>,

    // Async loaded data (from fetch)
    fetched_data: SharedData,
    fetched_error: SharedError,

    // Initial URL to load from query params (consumed on first update)
    initial_url: Option<String>,

    // Base pixels_per_point for scaling calculations (set on first frame)
    base_pixels_per_point: Option<f32>,

    // Menu dialog state
    show_url_dialog: bool,
    url_input: String,
    url_load_error: Option<String>,
    show_settings: bool,
    show_about: bool,

    // Time scrubbers (separate for messages and fragments)
    messages_scrubber: TimeScrubber,
    fragments_scrubber: TimeScrubber,

    // Desktop: pending file from file dialog
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pending_file_path: Option<std::path::PathBuf>,
}

impl Default for PcapViewerApp {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            packets: Vec::new(),
            current_tab: Tab::Messages,
            selected_message: None,
            selected_packet: None,
            search_query: String::new(),
            sort_field: SortField::Id,
            sort_ascending: true,
            view_mode: ViewMode::Tree,
            status_message: "Drag & drop a PCAP file or click 'Load Example'".to_string(),
            is_loading: false,
            dark_mode: true,
            show_detail_panel: false,
            dropped_file_data: None,
            fetched_data: Arc::new(Mutex::new(None)),
            fetched_error: Arc::new(Mutex::new(None)),
            initial_url: None,
            base_pixels_per_point: None,
            show_url_dialog: false,
            url_input: String::new(),
            url_load_error: None,
            show_settings: false,
            show_about: false,
            messages_scrubber: TimeScrubber::new(),
            fragments_scrubber: TimeScrubber::new(),
            #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
            pending_file_path: None,
        }
    }
}

impl PcapViewerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut app = Self::default();

        // Check for URL query parameter on WASM
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(url) = get_url_from_query_params() {
                log::info!("Found URL in query params: {}", url);
                app.initial_url = Some(url);
                app.status_message = "Loading PCAP from URL...".to_string();
            }
        }

        app
    }
}

/// Get the URL from query parameters (?url=...)
#[cfg(target_arch = "wasm32")]
fn get_url_from_query_params() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok()?;

    if search.is_empty() {
        return None;
    }

    // Remove the leading '?' and parse
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get("url")
}

impl eframe::App for PcapViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(bytes) = &file.bytes {
                        self.dropped_file_data = Some(bytes.to_vec());
                    }
                }
            }
        });

        // Process dropped file data outside the input closure
        if let Some(data) = self.dropped_file_data.take() {
            ui::file_panel::parse_pcap_data(self, &data);
        }

        // Desktop: process file from file dialog
        #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
        if let Some(path) = self.pending_file_path.take() {
            self.status_message = format!("Loading {}...", path.display());
            match std::fs::read(&path) {
                Ok(data) => ui::file_panel::parse_pcap_data(self, &data),
                Err(e) => self.status_message = format!("Error reading file: {e}"),
            }
        }

        // Check for async fetched data
        let fetched_data = if let Ok(mut fetched) = self.fetched_data.try_lock() {
            fetched.take()
        } else {
            None
        };
        if let Some(data) = fetched_data {
            ui::file_panel::parse_pcap_data(self, &data);
        }

        // Check for async fetch errors
        let fetched_error = if let Ok(mut error) = self.fetched_error.try_lock() {
            error.take()
        } else {
            None
        };
        if let Some(error) = fetched_error {
            self.url_load_error = Some(error);
            self.is_loading = false;
        }

        // Handle initial URL from query params (auto-load on first frame)
        if let Some(url) = self.initial_url.take() {
            ui::file_panel::load_from_url(self, url, ctx);
        }

        // Preview dropped files
        ui::file_panel::preview_files_being_dropped(ctx);

        // Apply theme
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        // Mobile scaling: store base pixels_per_point and apply scale factor
        let base_ppp = *self
            .base_pixels_per_point
            .get_or_insert_with(|| ctx.pixels_per_point());
        let current_ppp = ctx.pixels_per_point();

        // Calculate viewport width in base units (before our custom scaling)
        let screen_rect = ctx.screen_rect();
        let viewport_width = screen_rect.width() * current_ppp / base_ppp;
        let is_mobile_viewport = viewport_width < MOBILE_BREAKPOINT;

        // Apply appropriate scale factor
        let desired_ppp = if is_mobile_viewport {
            base_ppp * MOBILE_SCALE
        } else {
            base_ppp
        };
        if (current_ppp - desired_ppp).abs() > 0.01 {
            ctx.set_pixels_per_point(desired_ppp);
        }

        // Determine responsive layout mode (using scaled screen rect)
        let screen_rect = ctx.screen_rect();
        let screen_width = screen_rect.width();
        let screen_height = screen_rect.height();
        let is_mobile = is_mobile_viewport;
        let is_tablet = (MOBILE_BREAKPOINT..TABLET_BREAKPOINT).contains(&viewport_width);
        let has_data = !self.messages.is_empty() || !self.packets.is_empty();

        // Debug mode string
        let mode_str = if is_mobile {
            "M"
        } else if is_tablet {
            "T"
        } else {
            "D"
        };

        // Track menu actions to execute after borrow ends
        let mut open_url_clicked = false;
        #[cfg(not(target_arch = "wasm32"))]
        let mut quit_clicked = false;

        // Menu bar panel
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.menu_button("Open", |ui| {
                        if ui.button("From File...").clicked() {
                            #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
                            ui::file_panel::open_file_dialog(self);
                            ui.close_menu();
                        }
                        if ui.button("From URL...").clicked() {
                            open_url_clicked = true;
                            ui.close_menu();
                        }
                    });

                    ui.separator();

                    if ui.button("Settings...").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }

                    // Only show Quit on desktop
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            quit_clicked = true;
                            ui.close_menu();
                        }
                    }
                });

                ui.menu_button("About", |ui| {
                    if ui.button("About AC PCAP Parser").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Handle menu actions
        if open_url_clicked {
            self.show_url_dialog = true;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if quit_clicked {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Top panel with tabs and controls - responsive
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if is_mobile {
                // Mobile: Two-row compact layout
                ui.vertical(|ui| {
                    // First row: Title + theme toggle
                    ui.horizontal(|ui| {
                        ui.heading("AC PCAP");

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Theme toggle
                            ui::packet_list::draw_theme_toggle(self, ui);

                            // Detail panel toggle (only when we have data)
                            if has_data {
                                ui.separator();
                                let detail_icon = if self.show_detail_panel { "×" } else { "≡" };
                                if ui
                                    .button(detail_icon)
                                    .on_hover_text("Toggle detail panel")
                                    .clicked()
                                {
                                    self.show_detail_panel = !self.show_detail_panel;
                                }
                            }
                        });
                    });

                    // Second row: Tabs + minimal controls
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.current_tab == Tab::Messages, "Msg")
                            .clicked()
                        {
                            self.current_tab = Tab::Messages;
                        }
                        if ui
                            .selectable_label(self.current_tab == Tab::Fragments, "Frag")
                            .clicked()
                        {
                            self.current_tab = Tab::Fragments;
                        }

                        ui.separator();

                        // Compact search
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Search...")
                                .desired_width(ui.available_width() - 40.0),
                        );

                        // Sort direction only
                        if ui::packet_list::draw_sort_button(self, ui) {
                            self.sort_ascending = !self.sort_ascending;
                        }
                    });
                });
            } else {
                // Desktop/Tablet: Single row layout
                ui.horizontal(|ui| {
                    ui.heading(if is_tablet {
                        "AC PCAP"
                    } else {
                        "AC PCAP Parser"
                    });
                    ui.separator();

                    // Desktop: Open File button
                    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
                    {
                        if ui.button("Open...").clicked() {
                            ui::file_panel::open_file_dialog(self);
                        }
                        ui.separator();
                    }

                    // Tab buttons
                    if ui
                        .selectable_label(self.current_tab == Tab::Messages, "Messages")
                        .clicked()
                    {
                        self.current_tab = Tab::Messages;
                    }
                    if ui
                        .selectable_label(self.current_tab == Tab::Fragments, "Fragments")
                        .clicked()
                    {
                        self.current_tab = Tab::Fragments;
                    }

                    ui.separator();

                    // Search box
                    if !is_tablet {
                        ui.label("Search:");
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search...")
                            .desired_width(if is_tablet { 120.0 } else { 150.0 }),
                    );

                    ui.separator();

                    // Sort controls
                    if !is_tablet {
                        ui.label("Sort:");
                    }
                    egui::ComboBox::from_label("")
                        .selected_text(match self.sort_field {
                            SortField::Id => "ID",
                            SortField::Type => {
                                if is_tablet {
                                    "Type"
                                } else {
                                    "Type/Seq"
                                }
                            }
                            SortField::Direction => "Dir",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_field, SortField::Id, "ID");
                            ui.selectable_value(&mut self.sort_field, SortField::Type, "Type/Seq");
                            ui.selectable_value(
                                &mut self.sort_field,
                                SortField::Direction,
                                "Direction",
                            );
                        });

                    if ui::packet_list::draw_sort_button(self, ui) {
                        self.sort_ascending = !self.sort_ascending;
                    }

                    // Theme toggle on far right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui::packet_list::draw_theme_toggle(self, ui);
                    });
                });
            }
        });

        // Bottom panel with status - responsive
        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.is_loading {
                    ui.spinner();
                }

                // Debug info: mode, width x height, show_detail state
                let debug_info = format!(
                    "[{}:{}x{} d:{}]",
                    mode_str,
                    screen_width as i32,
                    screen_height as i32,
                    if self.show_detail_panel { "1" } else { "0" }
                );

                if is_mobile {
                    ui.label(format!("{} msgs {}", self.messages.len(), debug_info));
                } else {
                    ui.label(format!("{} {}", &self.status_message, debug_info));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // "Made with Claude" badge with logo
                    let claude_color = egui::Color32::from_rgb(217, 119, 87);
                    if is_mobile {
                        // Mobile: Just the logo
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::click());
                        ui.painter().circle_filled(rect.center(), 6.0, claude_color);
                        if response.clicked() {
                            ui.ctx()
                                .open_url(egui::OpenUrl::new_tab("https://claude.ai"));
                        }
                    } else {
                        ui.hyperlink_to(
                            egui::RichText::new("Made with Claude").color(claude_color),
                            "https://claude.ai",
                        );
                        // Claude logo (painted orange circle)
                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 6.0, claude_color);
                        ui.separator();

                        // Git info
                        let git_sha = option_env!("GIT_SHA").unwrap_or("dev");
                        let short_sha = if git_sha.len() > 7 {
                            &git_sha[..7]
                        } else {
                            git_sha
                        };
                        ui.hyperlink_to(
                            egui::RichText::new(format!("#{short_sha}")).small(),
                            format!("https://github.com/amoeba/ac-pcap-parser/commit/{git_sha}"),
                        );
                        ui.hyperlink_to(
                            egui::RichText::new("GitHub").small(),
                            "https://github.com/amoeba/ac-pcap-parser",
                        );
                        ui.separator();

                        ui.label(format!(
                            "Messages: {} | Packets: {}",
                            self.messages.len(),
                            self.packets.len()
                        ));
                    }
                });
            });
        });

        // Detail panel - responsive layout:
        // Mobile: Bottom panel (stacked vertically below list)
        // Desktop/Tablet: Right side panel (side by side)
        let show_detail = if is_mobile {
            self.show_detail_panel && has_data
        } else {
            has_data
        };

        if show_detail {
            if is_mobile {
                // Mobile: Bottom panel (stacked layout)
                let default_height = (screen_height * 0.45).max(200.0);
                let min_height = 150.0;
                let max_height = screen_height * 0.8;

                egui::TopBottomPanel::bottom("detail_panel_bottom")
                    .default_height(default_height)
                    .height_range(min_height..=max_height)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Detail");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("×").clicked() {
                                        self.show_detail_panel = false;
                                    }
                                },
                            );
                        });
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui::detail_panel::show_detail_content(self, ui);
                            });
                    });
            } else {
                // Desktop/Tablet: Right side panel
                let default_width = if is_tablet {
                    (screen_width * 0.35).max(200.0)
                } else {
                    (screen_width * 0.35).clamp(300.0, 500.0)
                };
                let min_width = 200.0;
                let max_width = screen_width * 0.7;

                egui::SidePanel::right("detail_panel_right")
                    .default_width(default_width)
                    .width_range(min_width..=max_width)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.heading("Detail");
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui::detail_panel::show_detail_content(self, ui);
                            });
                    });
            }
        }

        // Time scrubber panel (only show if we have data)
        // Show appropriate scrubber based on current tab
        // This panel is shown BELOW the central panel (list/detail pane)
        let mut clicked_time: Option<f64> = None;
        if has_data {
            // Check which scrubber has data
            let scrubber_has_data = match self.current_tab {
                Tab::Messages => self.messages_scrubber.has_data(),
                Tab::Fragments => self.fragments_scrubber.has_data(),
            };

            if scrubber_has_data {
                egui::TopBottomPanel::bottom("time_scrubber_panel")
                    .resizable(false)
                    .show(ctx, |ui| {
                        // Show appropriate scrubber
                        let result = match self.current_tab {
                            Tab::Messages => self.messages_scrubber.show(ui),
                            Tab::Fragments => self.fragments_scrubber.show(ui),
                        };

                        // Check if user clicked
                        if result.clicked_index.is_some() {
                            clicked_time = match self.current_tab {
                                Tab::Messages => self.messages_scrubber.get_hover_time(),
                                Tab::Fragments => self.fragments_scrubber.get_hover_time(),
                            };
                        }
                    });
            }
        }

        // Handle click-to-scroll from time scrubber
        if let Some(time) = clicked_time {
            // Find the closest packet/message to the clicked time
            match self.current_tab {
                Tab::Messages => {
                    let closest_idx = self
                        .messages
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            let dist_a = (a.timestamp - time).abs();
                            let dist_b = (b.timestamp - time).abs();
                            dist_a
                                .partial_cmp(&dist_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(idx, _)| idx);

                    if let Some(idx) = closest_idx {
                        self.selected_message = Some(idx);
                    }
                }
                Tab::Fragments => {
                    let closest_idx = self
                        .packets
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            let dist_a = (a.timestamp - time).abs();
                            let dist_b = (b.timestamp - time).abs();
                            dist_a
                                .partial_cmp(&dist_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(idx, _)| idx);

                    if let Some(idx) = closest_idx {
                        self.selected_packet = Some(idx);
                    }
                }
            }
        }

        // Central panel with list - responsive
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.messages.is_empty() && self.packets.is_empty() {
                // Show drop zone with Load Example button - responsive
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);

                    ui.label(if is_mobile {
                        "Drop a PCAP anywhere\nin the window"
                    } else {
                        "Drop a PCAP anywhere in the window"
                    });
                    ui.add_space(10.0);
                    ui.label("or");
                    ui.add_space(10.0);

                    let button_size = if is_mobile {
                        [150.0, 35.0]
                    } else {
                        [200.0, 40.0]
                    };

                    // Desktop: Open File button
                    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
                    {
                        if ui
                            .add_sized(button_size, egui::Button::new("Open File..."))
                            .clicked()
                        {
                            ui::file_panel::open_file_dialog(self);
                        }
                        ui.add_space(10.0);
                        ui.label("or");
                        ui.add_space(10.0);
                    }

                    if ui
                        .add_sized(button_size, egui::Button::new("Load Example"))
                        .clicked()
                    {
                        ui::file_panel::load_example(self, ctx);
                    }

                    // Add URL loading option
                    ui.add_space(10.0);
                    ui.label("or");
                    ui.add_space(10.0);

                    ui.label("Load from URL");
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        let input_width = if is_mobile { 200.0 } else { 300.0 };
                        let button_width = 50.0; // approximate button width
                        let spacing = ui.spacing().item_spacing.x;
                        let total_content_width = input_width + spacing + button_width;
                        let available_width = ui.available_width();

                        // Add left padding to center the content
                        if total_content_width < available_width {
                            let left_padding = (available_width - total_content_width) / 2.0;
                            ui.add_space(left_padding);
                        }

                        ui.add(
                            egui::TextEdit::singleline(&mut self.url_input)
                                .hint_text("https://example.com/file.pcap")
                                .desired_width(input_width),
                        );
                        if ui.button("Load").clicked() && !self.url_input.is_empty() {
                            let url = self.url_input.clone();
                            ui::file_panel::load_from_url(self, url, ctx);
                        }
                    });

                    // Show example URL link
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        // Get the full absolute URL for the example
                        #[cfg(target_arch = "wasm32")]
                        let example_url = {
                            if let Some(window) = web_sys::window() {
                                if let Some(location) = window.location().href().ok() {
                                    // Build absolute URL from current location
                                    if let Ok(url) =
                                        web_sys::Url::new_with_base("example.pcap", &location)
                                    {
                                        url.href()
                                    } else {
                                        "./example.pcap".to_string()
                                    }
                                } else {
                                    "./example.pcap".to_string()
                                }
                            } else {
                                "./example.pcap".to_string()
                            }
                        };
                        #[cfg(not(target_arch = "wasm32"))]
                        let example_url = "./example.pcap".to_string();

                        let prefix_text = "Example: ";
                        let full_text = format!("{prefix_text}{example_url}");

                        // Calculate width for centering the entire line
                        let total_width = ui.fonts(|f| {
                            f.layout_no_wrap(
                                full_text.clone(),
                                egui::FontId::default(),
                                egui::Color32::PLACEHOLDER,
                            )
                            .rect
                            .width()
                        });
                        let available_width = ui.available_width();

                        // Add left padding to center
                        if total_width < available_width {
                            let left_padding = (available_width - total_width) / 2.0;
                            ui.add_space(left_padding);
                        }

                        // Show "Example: " as plain text
                        ui.label(prefix_text);

                        // Show the URL as a clickable link
                        if ui.link(&example_url).clicked() {
                            self.url_input = example_url.clone();
                            ui::file_panel::load_from_url(self, example_url, ctx);
                        }
                    });

                    // Display error if URL load failed
                    if let Some(ref error) = self.url_load_error {
                        ui.add_space(5.0);
                        ui.colored_label(egui::Color32::RED, error);
                    }

                    if self.is_loading {
                        ui.add_space(if is_mobile { 10.0 } else { 20.0 });
                        ui.spinner();
                    }
                });
            } else {
                // On mobile, auto-show detail when selecting an item
                match self.current_tab {
                    Tab::Messages => ui::packet_list::show_messages_list(self, ui, is_mobile),
                    Tab::Fragments => ui::packet_list::show_packets_list(self, ui, is_mobile),
                }
            }
        });

        // URL input dialog
        if self.show_url_dialog {
            ui::file_panel::show_url_dialog(self, ctx);
        }

        // Settings window
        if self.show_settings {
            ui::file_panel::show_settings_dialog(self, ctx);
        }

        // About window
        if self.show_about {
            ui::file_panel::show_about_dialog(self, ctx);
        }
    }
}

impl PcapViewerApp {}

// WASM entry point
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Redirect panics to console.error
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("ac_pcap_canvas")
            .expect("Failed to find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(PcapViewerApp::new(cc)))),
            )
            .await;

        // Remove loading text and spinner
        if let Some(loading) = document.get_element_by_id("loading") {
            loading.remove();
        }

        if let Err(e) = start_result {
            log::error!("Failed to start eframe: {:?}", e);
        }
    });

    Ok(())
}

// Native entry point for testing
#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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
