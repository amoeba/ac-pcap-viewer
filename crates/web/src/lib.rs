//! Web UI for Asheron's Call PCAP Parser
//!
//! A drag-and-drop web interface built with egui for parsing AC PCAP files.

use ac_parser::{messages::ParsedMessage, PacketParser, ParsedPacket};
use eframe::egui;
use egui_json_tree::JsonTree;
use std::sync::{Arc, Mutex};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum Tab {
    #[default]
    Messages,
    Fragments,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum SortField {
    #[default]
    Id,
    Type,
    Direction,
}

// Responsive breakpoints
const MOBILE_BREAKPOINT: f32 = 768.0;
const TABLET_BREAKPOINT: f32 = 1024.0;

// Mobile UI scaling factor
const MOBILE_SCALE: f32 = 1.5;

// Shared state for async loading
type SharedData = Arc<Mutex<Option<Vec<u8>>>>;

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

    // Initial URL to load from query params (consumed on first update)
    initial_url: Option<String>,

    // Base pixels_per_point for scaling calculations (set on first frame)
    base_pixels_per_point: Option<f32>,

    // Menu dialog state
    show_url_dialog: bool,
    url_input: String,
    show_settings: bool,
    show_about: bool,
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
            status_message: "Drag & drop a PCAP file or click 'Load Example'".to_string(),
            is_loading: false,
            dark_mode: true,
            show_detail_panel: false,
            dropped_file_data: None,
            fetched_data: Arc::new(Mutex::new(None)),
            initial_url: None,
            base_pixels_per_point: None,
            show_url_dialog: false,
            url_input: String::new(),
            show_settings: false,
            show_about: false,
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

    fn parse_pcap_data(&mut self, data: &[u8]) {
        self.is_loading = true;
        self.status_message = "Parsing PCAP file...".to_string();

        let mut parser = PacketParser::new();
        match parser.parse_pcap_bytes(data) {
            Ok((packets, messages)) => {
                self.status_message = format!(
                    "Loaded {} packets, {} messages",
                    packets.len(),
                    messages.len()
                );
                self.packets = packets;
                self.messages = messages;
                self.selected_message = if self.messages.is_empty() {
                    None
                } else {
                    Some(0)
                };
                self.selected_packet = if self.packets.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            Err(e) => {
                self.status_message = format!("Error parsing PCAP: {}", e);
            }
        }
        self.is_loading = false;
    }

    #[cfg(target_arch = "wasm32")]
    fn load_example(&mut self, ctx: &egui::Context) {
        self.load_from_url("./example.pcap".to_string(), ctx);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_example(&mut self, _ctx: &egui::Context) {
        // Native: just read from file
        if let Ok(data) = std::fs::read("pkt_2025-11-18_1763490291_log.pcap") {
            self.parse_pcap_data(&data);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn load_from_url(&mut self, url: String, ctx: &egui::Context) {
        if self.is_loading {
            return;
        }

        self.is_loading = true;
        self.status_message = format!("Loading PCAP from {}...", url);

        let fetched_data = self.fetched_data.clone();
        let ctx = ctx.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match fetch_bytes(&url).await {
                Ok(bytes) => {
                    if let Ok(mut data) = fetched_data.lock() {
                        *data = Some(bytes);
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    log::error!("Failed to fetch PCAP from URL: {}", e);
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_url(&mut self, _url: String, _ctx: &egui::Context) {
        // Native: URL loading not supported
        self.status_message = "URL loading not supported in native mode".to_string();
    }

    #[cfg(target_arch = "wasm32")]
    fn trigger_file_picker(&mut self, ctx: &egui::Context) {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let document = match web_sys::window().and_then(|w| w.document()) {
            Some(d) => d,
            None => return,
        };

        // Create a hidden file input element
        let input: web_sys::HtmlInputElement = match document.create_element("input") {
            Ok(el) => match el.dyn_into() {
                Ok(input) => input,
                Err(_) => return,
            },
            Err(_) => return,
        };

        input.set_type("file");
        input.set_accept(".pcap,.pcapng");
        input.style().set_property("display", "none").ok();

        // Add to document temporarily
        let body = match document.body() {
            Some(b) => b,
            None => return,
        };
        if body.append_child(&input).is_err() {
            return;
        }

        // Set up the change handler
        let fetched_data = self.fetched_data.clone();
        let ctx_clone = ctx.clone();
        let input_clone = input.clone();

        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let files = match input_clone.files() {
                Some(f) => f,
                None => return,
            };

            let file = match files.get(0) {
                Some(f) => f,
                None => return,
            };

            let fetched_data = fetched_data.clone();
            let ctx = ctx_clone.clone();
            let input_to_remove = input_clone.clone();

            let reader = match web_sys::FileReader::new() {
                Ok(r) => r,
                Err(_) => return,
            };

            let reader_clone = reader.clone();
            let onload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                if let Ok(result) = reader_clone.result() {
                    if let Some(array_buffer) = result.dyn_ref::<js_sys::ArrayBuffer>() {
                        let uint8_array = js_sys::Uint8Array::new(array_buffer);
                        let bytes = uint8_array.to_vec();

                        if let Ok(mut data) = fetched_data.lock() {
                            *data = Some(bytes);
                        }
                        ctx.request_repaint();
                    }
                }
                // Clean up the input element
                input_to_remove.remove();
            }) as Box<dyn FnMut(_)>);

            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();

            reader.read_as_array_buffer(&file).ok();
        }) as Box<dyn FnMut(_)>);

        input.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        // Trigger the file picker
        input.click();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn trigger_file_picker(&mut self, _ctx: &egui::Context) {
        // Native: Use drag-and-drop or show a message
        self.status_message = "Please drag and drop a PCAP file to open it".to_string();
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let window = web_sys::window().ok_or("No window")?;

    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into().map_err(|_| "Response cast error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let array_buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| format!("ArrayBuffer error: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("ArrayBuffer await error: {:?}", e))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    Ok(bytes)
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
            self.parse_pcap_data(&data);
        }

        // Check for async fetched data
        let fetched_data = if let Ok(mut fetched) = self.fetched_data.try_lock() {
            fetched.take()
        } else {
            None
        };
        if let Some(data) = fetched_data {
            self.parse_pcap_data(&data);
        }

        // Handle initial URL from query params (auto-load on first frame)
        if let Some(url) = self.initial_url.take() {
            self.load_from_url(url, ctx);
        }

        // Preview dropped files
        preview_files_being_dropped(ctx);

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
        let mut open_file_clicked = false;
        let mut open_url_clicked = false;
        #[cfg(not(target_arch = "wasm32"))]
        let mut quit_clicked = false;

        // Menu bar panel
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.menu_button("Open", |ui| {
                        if ui.button("From File...").clicked() {
                            open_file_clicked = true;
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
                            self.draw_theme_toggle(ui);

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
                        if self.draw_sort_button(ui) {
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

                    if self.draw_sort_button(ui) {
                        self.sort_ascending = !self.sort_ascending;
                    }

                    // Theme toggle on far right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.draw_theme_toggle(ui);
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
                            egui::RichText::new(format!("#{}", short_sha)).small(),
                            format!(
                                "https://github.com/amoeba/ac-pcap-parser/commit/{}",
                                git_sha
                            ),
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
                let panel_height = (screen_height * 0.45).max(200.0);

                egui::TopBottomPanel::bottom("detail_panel_bottom")
                    .default_height(panel_height)
                    .height_range(panel_height..=panel_height)
                    .resizable(false)
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
                                self.show_detail_content(ui);
                            });
                    });
            } else {
                // Desktop/Tablet: Right side panel
                let panel_width = if is_tablet {
                    (screen_width * 0.35).max(200.0)
                } else {
                    (screen_width * 0.35).clamp(300.0, 500.0)
                };

                egui::SidePanel::right("detail_panel_right")
                    .default_width(panel_width)
                    .width_range(panel_width..=panel_width)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading("Detail");
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                self.show_detail_content(ui);
                            });
                    });
            }
        }

        // Central panel with list - responsive
        let mut should_load_example = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.messages.is_empty() && self.packets.is_empty() {
                // Show drop zone with Load Example button - responsive
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);

                    let rect = ui.available_rect_before_wrap();
                    // Responsive drop zone size
                    let drop_size = if is_mobile {
                        egui::vec2(rect.width() * 0.9, 150.0)
                    } else {
                        egui::vec2(400.0, 200.0)
                    };
                    let drop_rect = egui::Rect::from_center_size(rect.center(), drop_size);
                    ui.painter().rect_stroke(
                        drop_rect,
                        10.0,
                        egui::Stroke::new(2.0, egui::Color32::GRAY),
                    );

                    ui.heading(if is_mobile {
                        "Drop PCAP here"
                    } else {
                        "Drop PCAP file here"
                    });
                    ui.add_space(if is_mobile { 10.0 } else { 20.0 });
                    ui.label("or");
                    ui.add_space(if is_mobile { 10.0 } else { 20.0 });

                    let button_size = if is_mobile {
                        [150.0, 35.0]
                    } else {
                        [200.0, 40.0]
                    };
                    if ui
                        .add_sized(button_size, egui::Button::new("Load Example"))
                        .clicked()
                    {
                        should_load_example = true;
                    }

                    if self.is_loading {
                        ui.add_space(if is_mobile { 10.0 } else { 20.0 });
                        ui.spinner();
                    }
                });
            } else {
                // On mobile, auto-show detail when selecting an item
                match self.current_tab {
                    Tab::Messages => self.show_messages_list(ui, is_mobile),
                    Tab::Fragments => self.show_packets_list(ui, is_mobile),
                }
            }
        });

        if should_load_example {
            self.load_example(ctx);
        }

        // Handle file picker action
        if open_file_clicked {
            self.trigger_file_picker(ctx);
        }

        // URL input dialog
        if self.show_url_dialog {
            let mut close_dialog = false;
            let mut load_url = false;

            egui::Window::new("Open from URL")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.url_input)
                                .hint_text("https://example.com/file.pcap")
                                .desired_width(300.0),
                        );
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked() {
                            load_url = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_dialog = true;
                        }
                    });
                });

            if load_url && !self.url_input.is_empty() {
                let url = self.url_input.clone();
                self.url_input.clear();
                self.show_url_dialog = false;
                self.load_from_url(url, ctx);
            } else if close_dialog {
                self.url_input.clear();
                self.show_url_dialog = false;
            }
        }

        // Settings window
        if self.show_settings {
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
                        if ui.selectable_label(self.dark_mode, "Dark").clicked() {
                            self.dark_mode = true;
                        }
                        if ui.selectable_label(!self.dark_mode, "Light").clicked() {
                            self.dark_mode = false;
                        }
                    });

                    ui.add_space(10.0);

                    ui.heading("Default View");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Default Tab:");
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
                    });

                    ui.horizontal(|ui| {
                        ui.label("Sort Order:");
                        if ui
                            .selectable_label(self.sort_ascending, "Ascending")
                            .clicked()
                        {
                            self.sort_ascending = true;
                        }
                        if ui
                            .selectable_label(!self.sort_ascending, "Descending")
                            .clicked()
                        {
                            self.sort_ascending = false;
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
                self.show_settings = false;
            }
        }

        // About window
        if self.show_about {
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
                        ui.label(format!("Version: {}", short_sha));

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("A web-based parser for Asheron's Call");
                        ui.label("PCAP network traffic files.");

                        ui.add_space(10.0);

                        ui.hyperlink_to(
                            "View on GitHub",
                            "https://github.com/amoeba/ac-pcap-parser",
                        );

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Claude branding
                        let claude_color = egui::Color32::from_rgb(217, 119, 87);
                        ui.horizontal(|ui| {
                            // Claude logo
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
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
                self.show_about = false;
            }
        }
    }
}

/// Column definition for mobile table
struct MobileColumn {
    header: &'static str,
    width_pct: f32,
    right_align: bool,
}

impl PcapViewerApp {
    /// Render a mobile-optimized table row cell
    fn mobile_cell(
        ui: &mut egui::Ui,
        width: f32,
        right_align: bool,
        is_selected: bool,
        text: impl Into<egui::WidgetText>,
    ) -> egui::Response {
        let layout = if right_align {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };
        ui.allocate_ui_with_layout(egui::vec2(width, 20.0), layout, |ui| {
            ui.selectable_label(is_selected, text)
        })
        .inner
    }

    /// Render mobile table header
    fn mobile_header(ui: &mut egui::Ui, columns: &[MobileColumn], available_width: f32) {
        let widths: Vec<f32> = columns
            .iter()
            .map(|c| available_width * c.width_pct)
            .collect();

        for (i, col) in columns.iter().enumerate() {
            let layout = if col.right_align {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            };
            ui.allocate_ui_with_layout(egui::vec2(widths[i], 20.0), layout, |ui| {
                ui.strong(col.header);
            });
        }
        ui.end_row();
    }

    /// Show the detail panel content (shared between bottom and side panel)
    fn show_detail_content(&self, ui: &mut egui::Ui) {
        match self.current_tab {
            Tab::Messages => {
                if let Some(idx) = self.selected_message {
                    if idx < self.messages.len() {
                        let tree_id = format!("message_tree_{}", idx);
                        JsonTree::new(&tree_id, &self.messages[idx].data).show(ui);
                    } else {
                        ui.label("No message selected");
                    }
                } else {
                    ui.label("No message selected");
                }
            }
            Tab::Fragments => {
                if let Some(idx) = self.selected_packet {
                    if idx < self.packets.len() {
                        if let Ok(value) = serde_json::to_value(&self.packets[idx]) {
                            let tree_id = format!("packet_tree_{}", idx);
                            JsonTree::new(&tree_id, &value).show(ui);
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
        }
    }

    /// Draw a sort direction button with custom painted arrow icon
    fn draw_sort_button(&mut self, ui: &mut egui::Ui) -> bool {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
        let clicked = response.clicked();
        response.on_hover_text(if self.sort_ascending {
            "Sort descending"
        } else {
            "Sort ascending"
        });

        let painter = ui.painter();
        let center = rect.center();
        let arrow_color = ui.visuals().text_color();

        if self.sort_ascending {
            // Draw up arrow ↑
            let top = center + egui::vec2(0.0, -6.0);
            let bottom = center + egui::vec2(0.0, 6.0);
            let left = center + egui::vec2(-5.0, -1.0);
            let right = center + egui::vec2(5.0, -1.0);

            // Stem
            painter.line_segment([top, bottom], egui::Stroke::new(2.0, arrow_color));
            // Arrowhead
            painter.line_segment([top, left], egui::Stroke::new(2.0, arrow_color));
            painter.line_segment([top, right], egui::Stroke::new(2.0, arrow_color));
        } else {
            // Draw down arrow ↓
            let top = center + egui::vec2(0.0, -6.0);
            let bottom = center + egui::vec2(0.0, 6.0);
            let left = center + egui::vec2(-5.0, 1.0);
            let right = center + egui::vec2(5.0, 1.0);

            // Stem
            painter.line_segment([top, bottom], egui::Stroke::new(2.0, arrow_color));
            // Arrowhead
            painter.line_segment([bottom, left], egui::Stroke::new(2.0, arrow_color));
            painter.line_segment([bottom, right], egui::Stroke::new(2.0, arrow_color));
        }

        clicked
    }

    /// Draw the theme toggle (sun/moon icon)
    fn draw_theme_toggle(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
        if response.clicked() {
            self.dark_mode = !self.dark_mode;
        }
        response.on_hover_text("Toggle dark/light mode");

        let painter = ui.painter();
        let center = rect.center();

        if self.dark_mode {
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

    fn show_messages_list(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        // Pre-collect data to avoid borrow issues
        let search = self.search_query.to_lowercase();
        let sort_field = self.sort_field;
        let sort_ascending = self.sort_ascending;
        let total = self.messages.len();

        let mut filtered: Vec<(usize, usize, String, String, String)> = self
            .messages
            .iter()
            .enumerate()
            .filter(|(_, m)| search.is_empty() || m.message_type.to_lowercase().contains(&search))
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
                        Self::mobile_header(ui, &columns, available_width);

                        for (original_idx, id, msg_type, direction, _opcode) in &filtered {
                            let is_selected = self.selected_message == Some(*original_idx);

                            if Self::mobile_cell(ui, widths[0], false, is_selected, id.to_string())
                                .clicked()
                            {
                                self.selected_message = Some(*original_idx);
                                self.show_detail_panel = true;
                            }

                            let display_type = if msg_type.len() > 25 {
                                format!("{}…", &msg_type[..24])
                            } else {
                                msg_type.clone()
                            };
                            if Self::mobile_cell(ui, widths[1], false, is_selected, display_type)
                                .clicked()
                            {
                                self.selected_message = Some(*original_idx);
                                self.show_detail_panel = true;
                            }

                            let dir_color = if direction == "Send" {
                                egui::Color32::from_rgb(100, 200, 255)
                            } else {
                                egui::Color32::from_rgb(100, 255, 150)
                            };
                            let dir_text = if direction == "Send" { "S" } else { "R" };
                            if Self::mobile_cell(
                                ui,
                                widths[2],
                                true,
                                is_selected,
                                egui::RichText::new(dir_text).color(dir_color),
                            )
                            .clicked()
                            {
                                self.selected_message = Some(*original_idx);
                                self.show_detail_panel = true;
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
                            let is_selected = self.selected_message == Some(*original_idx);

                            if ui.selectable_label(is_selected, id.to_string()).clicked() {
                                self.selected_message = Some(*original_idx);
                            }
                            if ui.selectable_label(is_selected, msg_type).clicked() {
                                self.selected_message = Some(*original_idx);
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
                                self.selected_message = Some(*original_idx);
                            }
                            if ui.selectable_label(is_selected, opcode).clicked() {
                                self.selected_message = Some(*original_idx);
                            }
                            ui.end_row();
                        }
                    });
            }
        });
    }

    fn show_packets_list(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        // Pre-collect data to avoid borrow issues
        let sort_field = self.sort_field;
        let sort_ascending = self.sort_ascending;
        let total = self.packets.len();

        let mut filtered: Vec<(usize, usize, u32, String, u32, u16)> = self
            .packets
            .iter()
            .enumerate()
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
                        width_pct: 0.70,
                        right_align: false,
                    },
                    MobileColumn {
                        header: "Dir",
                        width_pct: 0.15,
                        right_align: true,
                    },
                ];
                let widths: Vec<f32> = columns
                    .iter()
                    .map(|c| available_width * c.width_pct)
                    .collect();

                egui::Grid::new("packets_grid")
                    .num_columns(3)
                    .striped(true)
                    .spacing(egui::vec2(4.0, 4.0))
                    .show(ui, |ui| {
                        Self::mobile_header(ui, &columns, available_width);

                        for (original_idx, id, sequence, direction, _flags, _size) in &filtered {
                            let is_selected = self.selected_packet == Some(*original_idx);

                            if Self::mobile_cell(ui, widths[0], false, is_selected, id.to_string())
                                .clicked()
                            {
                                self.selected_packet = Some(*original_idx);
                                self.show_detail_panel = true;
                            }

                            if Self::mobile_cell(
                                ui,
                                widths[1],
                                false,
                                is_selected,
                                sequence.to_string(),
                            )
                            .clicked()
                            {
                                self.selected_packet = Some(*original_idx);
                                self.show_detail_panel = true;
                            }

                            let dir_color = if direction == "Send" {
                                egui::Color32::from_rgb(100, 200, 255)
                            } else {
                                egui::Color32::from_rgb(100, 255, 150)
                            };
                            let dir_text = if direction == "Send" { "S" } else { "R" };
                            if Self::mobile_cell(
                                ui,
                                widths[2],
                                true,
                                is_selected,
                                egui::RichText::new(dir_text).color(dir_color),
                            )
                            .clicked()
                            {
                                self.selected_packet = Some(*original_idx);
                                self.show_detail_panel = true;
                            }

                            ui.end_row();
                        }
                    });
            } else {
                // Desktop layout
                egui::Grid::new("packets_grid")
                    .num_columns(5)
                    .striped(true)
                    .min_col_width(50.0)
                    .spacing(egui::vec2(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.strong("ID");
                        ui.strong("Seq");
                        ui.strong("Dir");
                        ui.strong("Flags");
                        ui.strong("Size");
                        ui.end_row();

                        for (original_idx, id, sequence, direction, flags, size) in &filtered {
                            let is_selected = self.selected_packet == Some(*original_idx);

                            if ui.selectable_label(is_selected, id.to_string()).clicked() {
                                self.selected_packet = Some(*original_idx);
                            }
                            if ui
                                .selectable_label(is_selected, sequence.to_string())
                                .clicked()
                            {
                                self.selected_packet = Some(*original_idx);
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
                                self.selected_packet = Some(*original_idx);
                            }
                            if ui
                                .selectable_label(is_selected, format!("{:08X}", flags))
                                .clicked()
                            {
                                self.selected_packet = Some(*original_idx);
                            }
                            if ui.selectable_label(is_selected, size.to_string()).clicked() {
                                self.selected_packet = Some(*original_idx);
                            }
                            ui.end_row();
                        }
                    });
            }
        });
    }
}

/// Preview hovering files
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = "Drop PCAP file to load";

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

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
