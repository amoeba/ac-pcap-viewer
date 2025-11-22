//! Web UI for Asheron's Call PCAP Parser
//!
//! A drag-and-drop web interface built with egui for parsing AC PCAP files.

use eframe::egui;
use egui_json_tree::JsonTree;
use ac_parser::{PacketParser, ParsedPacket, messages::ParsedMessage};
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

    // Dropped file data
    dropped_file_data: Option<Vec<u8>>,

    // Async loaded data (from fetch)
    fetched_data: SharedData,
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
            dropped_file_data: None,
            fetched_data: Arc::new(Mutex::new(None)),
        }
    }
}

impl PcapViewerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
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
                self.selected_message = if self.messages.is_empty() { None } else { Some(0) };
                self.selected_packet = if self.packets.is_empty() { None } else { Some(0) };
            }
            Err(e) => {
                self.status_message = format!("Error parsing PCAP: {}", e);
            }
        }
        self.is_loading = false;
    }

    #[cfg(target_arch = "wasm32")]
    fn load_example(&mut self, ctx: &egui::Context) {
        if self.is_loading {
            return;
        }

        self.is_loading = true;
        self.status_message = "Loading example PCAP...".to_string();

        let fetched_data = self.fetched_data.clone();
        let ctx = ctx.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let url = "./example.pcap";

            match fetch_bytes(url).await {
                Ok(bytes) => {
                    if let Ok(mut data) = fetched_data.lock() {
                        *data = Some(bytes);
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    log::error!("Failed to fetch example: {}", e);
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_example(&mut self, _ctx: &egui::Context) {
        // Native: just read from file
        if let Ok(data) = std::fs::read("pkt_2025-11-18_1763490291_log.pcap") {
            self.parse_pcap_data(&data);
        }
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

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "Response cast error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let array_buffer = JsFuture::from(
        resp.array_buffer().map_err(|e| format!("ArrayBuffer error: {:?}", e))?
    )
    .await
    .map_err(|e| format!("ArrayBuffer await error: {:?}", e))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    Ok(bytes)
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

        // Preview dropped files
        preview_files_being_dropped(ctx);

        // Top panel with tabs and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("AC PCAP Parser");
                ui.separator();

                // Tab buttons
                if ui.selectable_label(self.current_tab == Tab::Messages, "Messages").clicked() {
                    self.current_tab = Tab::Messages;
                }
                if ui.selectable_label(self.current_tab == Tab::Fragments, "Fragments").clicked() {
                    self.current_tab = Tab::Fragments;
                }

                ui.separator();

                // Search box
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);

                ui.separator();

                // Sort controls
                ui.label("Sort:");
                egui::ComboBox::from_label("")
                    .selected_text(match self.sort_field {
                        SortField::Id => "ID",
                        SortField::Type => "Type/Seq",
                        SortField::Direction => "Direction",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sort_field, SortField::Id, "ID");
                        ui.selectable_value(&mut self.sort_field, SortField::Type, "Type/Seq");
                        ui.selectable_value(&mut self.sort_field, SortField::Direction, "Direction");
                    });

                if ui.button(if self.sort_ascending { "↑" } else { "↓" }).clicked() {
                    self.sort_ascending = !self.sort_ascending;
                }
            });
        });

        // Bottom panel with status
        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.is_loading {
                    ui.spinner();
                }
                ui.label(&self.status_message);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // "Made with Claude" badge
                    ui.hyperlink_to(
                        egui::RichText::new("Made with Claude")
                            .small()
                            .color(egui::Color32::from_rgb(217, 119, 87)),
                        "https://claude.ai",
                    );
                    ui.separator();

                    // Git info
                    let git_sha = option_env!("GIT_SHA").unwrap_or("dev");
                    let short_sha = if git_sha.len() > 7 { &git_sha[..7] } else { git_sha };
                    ui.hyperlink_to(
                        egui::RichText::new(format!("#{}", short_sha)).small(),
                        format!("https://github.com/amoeba/ac-pcap-parser/commit/{}", git_sha),
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
                });
            });
        });

        // Right panel with detail view
        egui::SidePanel::right("detail_panel")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Detail");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.current_tab {
                        Tab::Messages => {
                            if let Some(idx) = self.selected_message {
                                if idx < self.messages.len() {
                                    // Use JsonTree to display the message data
                                    let tree_id = format!("message_tree_{}", idx);
                                    JsonTree::new(&tree_id, &self.messages[idx].data)
                                        .show(ui);
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
                                    // Convert packet to serde_json::Value for tree display
                                    if let Ok(value) = serde_json::to_value(&self.packets[idx]) {
                                        let tree_id = format!("packet_tree_{}", idx);
                                        JsonTree::new(&tree_id, &value)
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
                    }
                });
            });

        // Central panel with list
        let mut should_load_example = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.messages.is_empty() && self.packets.is_empty() {
                // Show drop zone with Load Example button
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);

                    let rect = ui.available_rect_before_wrap();
                    let drop_rect = egui::Rect::from_center_size(
                        rect.center(),
                        egui::vec2(400.0, 200.0),
                    );
                    ui.painter().rect_stroke(
                        drop_rect,
                        10.0,
                        egui::Stroke::new(2.0, egui::Color32::GRAY),
                    );

                    ui.heading("Drop PCAP file here");
                    ui.add_space(20.0);
                    ui.label("or");
                    ui.add_space(20.0);

                    if ui.add_sized([200.0, 40.0], egui::Button::new("Load Example")).clicked() {
                        should_load_example = true;
                    }

                    if self.is_loading {
                        ui.add_space(20.0);
                        ui.spinner();
                    }
                });
            } else {
                match self.current_tab {
                    Tab::Messages => self.show_messages_list(ui),
                    Tab::Fragments => self.show_packets_list(ui),
                }
            }
        });

        if should_load_example {
            self.load_example(ctx);
        }
    }
}

impl PcapViewerApp {
    fn show_messages_list(&mut self, ui: &mut egui::Ui) {
        // Pre-collect data to avoid borrow issues
        let search = self.search_query.to_lowercase();
        let sort_field = self.sort_field;
        let sort_ascending = self.sort_ascending;
        let total = self.messages.len();

        let mut filtered: Vec<(usize, usize, String, String, String)> = self.messages.iter()
            .enumerate()
            .filter(|(_, m)| search.is_empty() || m.message_type.to_lowercase().contains(&search))
            .map(|(idx, m)| (idx, m.id, m.message_type.clone(), m.direction.clone(), m.opcode.clone()))
            .collect();

        filtered.sort_by(|a, b| {
            let cmp = match sort_field {
                SortField::Id => a.1.cmp(&b.1),
                SortField::Type => a.2.cmp(&b.2),
                SortField::Direction => a.3.cmp(&b.3),
            };
            if sort_ascending { cmp } else { cmp.reverse() }
        });

        ui.horizontal(|ui| {
            ui.label(format!("Showing {} of {} messages", filtered.len(), total));
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
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
                        if ui.selectable_label(is_selected, egui::RichText::new(direction).color(dir_color)).clicked() {
                            self.selected_message = Some(*original_idx);
                        }

                        if ui.selectable_label(is_selected, opcode).clicked() {
                            self.selected_message = Some(*original_idx);
                        }

                        ui.end_row();
                    }
                });
        });
    }

    fn show_packets_list(&mut self, ui: &mut egui::Ui) {
        // Pre-collect data to avoid borrow issues
        let sort_field = self.sort_field;
        let sort_ascending = self.sort_ascending;
        let total = self.packets.len();

        let mut filtered: Vec<(usize, usize, u32, String, u32, u16)> = self.packets.iter()
            .enumerate()
            .map(|(idx, p)| (
                idx,
                p.id,
                p.header.sequence,
                format!("{:?}", p.direction),
                p.header.flags.bits(),
                p.header.size,
            ))
            .collect();

        filtered.sort_by(|a, b| {
            let cmp = match sort_field {
                SortField::Id => a.1.cmp(&b.1),
                SortField::Type => a.2.cmp(&b.2),
                SortField::Direction => a.3.cmp(&b.3),
            };
            if sort_ascending { cmp } else { cmp.reverse() }
        });

        ui.horizontal(|ui| {
            ui.label(format!("Showing {} of {} packets", filtered.len(), total));
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("packets_grid")
                .num_columns(5)
                .striped(true)
                .min_col_width(50.0)
                .show(ui, |ui| {
                    // Header
                    ui.strong("ID");
                    ui.strong("Sequence");
                    ui.strong("Dir");
                    ui.strong("Flags");
                    ui.strong("Size");
                    ui.end_row();

                    for (original_idx, id, sequence, direction, flags, size) in &filtered {
                        let is_selected = self.selected_packet == Some(*original_idx);

                        if ui.selectable_label(is_selected, id.to_string()).clicked() {
                            self.selected_packet = Some(*original_idx);
                        }

                        if ui.selectable_label(is_selected, sequence.to_string()).clicked() {
                            self.selected_packet = Some(*original_idx);
                        }

                        let dir_color = if direction == "Send" {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else {
                            egui::Color32::from_rgb(100, 255, 150)
                        };
                        if ui.selectable_label(is_selected, egui::RichText::new(direction).color(dir_color)).clicked() {
                            self.selected_packet = Some(*original_idx);
                        }

                        if ui.selectable_label(is_selected, format!("{:08X}", flags)).clicked() {
                            self.selected_packet = Some(*original_idx);
                        }

                        if ui.selectable_label(is_selected, size.to_string()).clicked() {
                            self.selected_packet = Some(*original_idx);
                        }

                        ui.end_row();
                    }
                });
        });
    }
}

/// Preview hovering files
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = "Drop PCAP file to load";

        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

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
