//! UI for Asheron's Call PCAP Parser
//!
//! Shared egui-based interface for both web and desktop applications.

pub mod filter;
pub mod state;
pub mod time_scrubber;
pub mod ui;

use eframe::egui;
use lib::{messages::ParsedMessage, ParsedPacket};
use std::sync::{Arc, Mutex};
use time_scrubber::TimeScrubber;

// Re-export state types for convenience
pub use lib::{SortField, Tab, ViewMode};
use state::{MOBILE_BREAKPOINT, MOBILE_SCALE, TABLET_BREAKPOINT};

// Shared state for async loading
pub type SharedData = Arc<Mutex<Option<Vec<u8>>>>;
pub type SharedError = Arc<Mutex<Option<String>>>;

pub struct PcapViewerApp {
    // Data
    pub messages: Vec<ParsedMessage>,
    pub packets: Vec<ParsedPacket>,
    pub weenie_db: lib::weenie::WeenieDatabase,

    // UI State
    pub current_tab: Tab,
    pub selected_message: Option<usize>,
    pub selected_packet: Option<usize>,
    pub selected_weenie: Option<usize>,
    pub search_query: String,
    pub sort_field: SortField,
    pub sort_ascending: bool,
    pub view_mode: ViewMode,

    // Status
    pub status_message: String,
    pub is_loading: bool,

    // Theme
    pub dark_mode: bool,

    // Responsive layout state
    pub show_detail_panel: bool,

    // Dropped file data
    pub dropped_file_data: Option<Vec<u8>>,

    // Async loaded data (from fetch)
    pub fetched_data: SharedData,
    pub fetched_error: SharedError,

    // Initial URL to load from query params (consumed on first update)
    pub initial_url: Option<String>,

    // Initial Discord load flag (consumed on first update)
    pub initial_discord_load: bool,

    // Base pixels_per_point for scaling calculations (set on first frame)
    pub base_pixels_per_point: Option<f32>,

    // Menu dialog state
    pub show_url_dialog: bool,
    pub url_input: String,
    pub url_load_error: Option<String>,
    pub show_settings: bool,
    pub show_about: bool,

    // Discord loading state
    pub discord_channel_id: String,
    pub discord_message_id: String,
    pub discord_load_error: Option<String>,

    // Time scrubbers (separate for messages and fragments)
    pub messages_scrubber: TimeScrubber,
    pub fragments_scrubber: TimeScrubber,

    // Marking state for filtered items
    pub marked_messages: std::collections::HashSet<usize>,
    pub marked_packets: std::collections::HashSet<usize>,

    // Desktop: pending file from file dialog
    #[cfg(feature = "desktop")]
    pub pending_file_path: Option<std::path::PathBuf>,
}

impl Default for PcapViewerApp {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            packets: Vec::new(),
            weenie_db: lib::weenie::WeenieDatabase::new(),
            current_tab: Tab::Messages,
            selected_message: None,
            selected_packet: None,
            selected_weenie: None,
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
            initial_discord_load: false,
            base_pixels_per_point: None,
            show_url_dialog: false,
            url_input: String::new(),
            url_load_error: None,
            show_settings: false,
            show_about: false,
            discord_channel_id: String::new(),
            discord_message_id: String::new(),
            discord_load_error: None,
            messages_scrubber: TimeScrubber::new(),
            fragments_scrubber: TimeScrubber::new(),
            marked_messages: std::collections::HashSet::new(),
            marked_packets: std::collections::HashSet::new(),
            #[cfg(feature = "desktop")]
            pending_file_path: None,
        }
    }
}

impl PcapViewerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut app = Self::default();

        app
    }

    /// Mark all currently filtered items for visual tracking (replaces previous marks)
    fn mark_filtered_items(&mut self) {
        let search = self.search_query.to_lowercase();

        match self.current_tab {
            Tab::Messages => {
                // Clear previous marks before setting new ones
                self.marked_messages.clear();

                let time_filter = self.messages_scrubber.get_selected_range().cloned();

                // Filter messages based on search and time
                let filtered_indices: Vec<usize> = self
                    .messages
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| {
                        // Apply search filter with rich filter support
                        let matches_search = if search.is_empty() {
                            true
                        } else {
                            // Parse search string into rich filters (supports hex, decimal, and text)
                            let filters = crate::filter::parse_filter_string(&search);

                            // Check if any filter matches
                            let mut matches = false;

                            // Search in message ID
                            if crate::filter::matches_any_filter(&filters, &m.id.to_string()) {
                                matches = true;
                            }

                            // Check opcode match
                            if !matches && crate::filter::matches_any_filter(&filters, &m.opcode) {
                                matches = true;
                            }

                            // Check direction match
                            if !matches && crate::filter::matches_any_filter(&filters, &m.direction)
                            {
                                matches = true;
                            }

                            // Check if filter matches in data fields
                            if !matches {
                                let data_str = serde_json::to_string(&m.data).unwrap_or_default();
                                if crate::filter::matches_any_filter(&filters, &data_str) {
                                    matches = true;
                                }
                            }

                            // Always also do text search (type and data)
                            if !matches {
                                let type_matches = m.message_type.to_lowercase().contains(&search);
                                let data_matches =
                                    crate::state::json_contains_string(&m.data, &search);
                                matches = type_matches || data_matches;
                            }

                            matches
                        };

                        // Apply time filter
                        let matches_time = if let Some(ref range) = time_filter {
                            range.contains(m.timestamp)
                        } else {
                            true
                        };

                        matches_search && matches_time
                    })
                    .map(|(idx, _)| idx)
                    .collect();

                // Set marked_messages to only the filtered indices
                self.marked_messages = filtered_indices.into_iter().collect();

                // Update scrubber with marked timestamps
                let marked_timestamps: Vec<f64> = self
                    .messages
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| self.marked_messages.contains(idx))
                    .map(|(_, m)| m.timestamp)
                    .collect();
                self.messages_scrubber
                    .set_marked_timestamps(marked_timestamps);
            }
            Tab::Weenies => {
                // TODO: Implement weenie marking (weenies don't have timestamps yet)
            }
        }
    }
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
        #[cfg(feature = "desktop")]
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
            self.url_load_error = Some(error.clone());
            self.discord_load_error = Some(error);
            self.is_loading = false;
        }

        // Handle initial URL from query params (auto-load on first frame)
        if let Some(url) = self.initial_url.take() {
            ui::file_panel::load_from_url(self, url, ctx);
        }

        // Handle initial Discord load from query params (auto-load on first frame)
        if self.initial_discord_load {
            log::info!("initial_discord_load triggered!");
            self.initial_discord_load = false;
            let channel = self.discord_channel_id.clone();
            let msg = self.discord_message_id.clone();
            log::info!("Discord IDs: channel={channel}, msg={msg}");
            if !channel.is_empty() && !msg.is_empty() {
                log::info!("Calling load_from_discord...");
                ui::file_panel::load_from_discord(self, channel, msg, ctx);
            } else {
                log::warn!("Channel or message ID is empty!");
            }
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

                    // Quit option
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        quit_clicked = true;
                        ui.close_menu();
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
                            .selectable_label(self.current_tab == Tab::Weenies, "Obj")
                            .clicked()
                        {
                            self.current_tab = Tab::Weenies;
                        }

                        ui.separator();

                        // Search section
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Filter...")
                                .desired_width(60.0),
                        );

                        // Reset search button
                        ui.add_enabled_ui(!self.search_query.is_empty(), |ui| {
                            if ui.button("✕").on_hover_text("Clear filter").clicked() {
                                self.search_query.clear();
                            }
                        });

                        ui.separator();

                        // Mark section
                        ui.label("Mark:");

                        // Mark button (enabled when filter is active)
                        ui.add_enabled_ui(!self.search_query.is_empty(), |ui| {
                            if ui
                                .button("◉")
                                .on_hover_text("Mark filtered items")
                                .clicked()
                            {
                                self.mark_filtered_items();
                            }
                        });

                        // Reset marks button (enabled when there are marks)
                        let has_marks = match self.current_tab {
                            Tab::Messages => !self.marked_messages.is_empty(),
                            Tab::Weenies => false, // TODO: Implement weenie marking
                        };
                        ui.add_enabled_ui(has_marks, |ui| {
                            if ui.button("✕").on_hover_text("Clear marks").clicked() {
                                match self.current_tab {
                                    Tab::Messages => {
                                        self.marked_messages.clear();
                                        self.messages_scrubber.clear_marked_timestamps();
                                    }
                                    Tab::Weenies => {
                                        // TODO: Implement weenie marking
                                    }
                                }
                            }
                        });

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
                        .selectable_label(self.current_tab == Tab::Weenies, "Weenies")
                        .clicked()
                    {
                        self.current_tab = Tab::Weenies;
                    }

                    ui.separator();

                    // Search section
                    if !is_tablet {
                        ui.label("Search:");
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Filter...")
                            .desired_width(if is_tablet { 100.0 } else { 120.0 }),
                    );

                    // Reset search button
                    ui.add_enabled_ui(!self.search_query.is_empty(), |ui| {
                        if ui.button("Reset").on_hover_text("Clear filter").clicked() {
                            self.search_query.clear();
                        }
                    });

                    ui.separator();

                    // Mark section
                    ui.label("Mark");

                    // Mark button (enabled when filter is active)
                    ui.add_enabled_ui(!self.search_query.is_empty(), |ui| {
                        if ui
                            .button("Mark")
                            .on_hover_text("Mark filtered items")
                            .clicked()
                        {
                            self.mark_filtered_items();
                        }
                    });

                    // Reset marks button (enabled when there are marks)
                    let has_marks = match self.current_tab {
                        Tab::Messages => !self.marked_messages.is_empty(),
                        Tab::Weenies => false, // TODO: Implement weenie marking
                    };
                    ui.add_enabled_ui(has_marks, |ui| {
                        if ui.button("Reset").on_hover_text("Clear marks").clicked() {
                            match self.current_tab {
                                Tab::Messages => {
                                    self.marked_messages.clear();
                                    self.messages_scrubber.clear_marked_timestamps();
                                }
                                Tab::Weenies => {
                                    // TODO: Implement weenie marking
                                }
                            }
                        }
                    });

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
                // Mobile: Bottom panel (stacked layout) - 25% of screen height
                let default_height = (screen_height * 0.25).max(150.0);
                let min_height = 100.0;
                let max_height = screen_height * 0.75;

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
        // This panel is shown BELOW the central panel (list/detail pane)
        let mut clicked_time: Option<f64> = None;
        if has_data {
            // Check which scrubber has data
            let scrubber_has_data = match self.current_tab {
                Tab::Messages => self.messages_scrubber.has_data(),
                Tab::Weenies => false, // Weenies don't have time scrubbers
            };

            if scrubber_has_data {
                egui::TopBottomPanel::bottom("time_scrubber_panel")
                    .resizable(false)
                    .show(ctx, |ui| {
                        // Show appropriate scrubber
                        let result = match self.current_tab {
                            Tab::Messages => self.messages_scrubber.show(ui),
                            Tab::Weenies => unreachable!("Weenies don't have time scrubbers"),
                        };

                        // Check if user clicked
                        if result.clicked_index.is_some() {
                            clicked_time = match self.current_tab {
                                Tab::Messages => self.messages_scrubber.get_hover_time(),
                                Tab::Weenies => unreachable!("Weenies don't have time scrubbers"),
                            };
                        }

                        // Handle reset marks button
                        if result.reset_marks_clicked {
                            match self.current_tab {
                                Tab::Messages => {
                                    self.marked_messages.clear();
                                    self.messages_scrubber.clear_marked_timestamps();
                                }
                                Tab::Weenies => {
                                    // TODO: Implement weenie marking
                                }
                            }
                        }
                    });
            }
        }

        // Handle click-to-scroll from time scrubber
        if let Some(time) = clicked_time {
            // Find the closest message to the clicked time
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
                    #[cfg(feature = "desktop")]
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
                    Tab::Weenies => ui::weenie_panel::show_weenie_panel(self, ui, is_mobile),
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
