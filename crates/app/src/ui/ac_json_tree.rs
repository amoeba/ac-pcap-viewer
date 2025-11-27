//! AC Protocol-aware JSON tree viewer with hex display and click-to-filter

use eframe::egui;
use serde_json::Value;
use std::collections::HashSet;

/// Field types that determine display format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayFormat {
    Decimal,
    Hex,
    Auto, // Decide based on context
}

/// Result of tree interaction
#[derive(Default)]
pub struct TreeResponse {
    pub filter_clicked: Option<String>,
}

/// AC Protocol-aware JSON tree viewer
pub struct AcJsonTree {
    id: String,
    expanded_paths: HashSet<String>,
    response: TreeResponse,
}

impl AcJsonTree {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            expanded_paths: HashSet::new(),
            response: TreeResponse::default(),
        }
    }

    /// Show the tree and return interaction result
    pub fn show(mut self, ui: &mut egui::Ui, value: &Value) -> TreeResponse {
        self.show_value(ui, value, "", 0);
        self.response
    }

    /// Recursively show a JSON value
    fn show_value(&mut self, ui: &mut egui::Ui, value: &Value, path: &str, depth: usize) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let item_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };

                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            // Collapsible header for nested structures
                            let header_id = egui::Id::new(format!("{}_{}", self.id, item_path));
                            let is_expanded = self.expanded_paths.contains(&item_path) || depth < 1; // Auto-expand first level

                            let header_response = egui::CollapsingHeader::new(key)
                                .id_salt(header_id)
                                .default_open(is_expanded)
                                .show(ui, |ui| {
                                    self.show_value(ui, val, &item_path, depth + 1);
                                });

                            if header_response.header_response.clicked() {
                                if self.expanded_paths.contains(&item_path) {
                                    self.expanded_paths.remove(&item_path);
                                } else {
                                    self.expanded_paths.insert(item_path.clone());
                                }
                            }
                        }
                        _ => {
                            // Leaf value - show key and clickable value
                            ui.horizontal(|ui| {
                                ui.label(format!("{key}:"));
                                self.show_leaf_value(ui, key, val);
                            });
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (idx, val) in arr.iter().enumerate() {
                    let item_path = format!("{path}[{idx}]");

                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            let header_id = egui::Id::new(format!("{}_{}", self.id, item_path));
                            let is_expanded = self.expanded_paths.contains(&item_path);

                            let header_response = egui::CollapsingHeader::new(format!("[{idx}]"))
                                .id_salt(header_id)
                                .default_open(is_expanded)
                                .show(ui, |ui| {
                                    self.show_value(ui, val, &item_path, depth + 1);
                                });

                            if header_response.header_response.clicked() {
                                if self.expanded_paths.contains(&item_path) {
                                    self.expanded_paths.remove(&item_path);
                                } else {
                                    self.expanded_paths.insert(item_path.clone());
                                }
                            }
                        }
                        _ => {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{idx}]:"));
                                self.show_leaf_value(ui, &idx.to_string(), val);
                            });
                        }
                    }
                }
            }
            _ => {
                // Top-level primitive (unusual, but handle it)
                self.show_leaf_value(ui, "", value);
            }
        }
    }

    /// Show a leaf value with appropriate formatting and click handling
    fn show_leaf_value(&mut self, ui: &mut egui::Ui, key: &str, value: &Value) {
        let format = Self::determine_format(key, value);

        match value {
            Value::Number(num) => {
                if let Some(u) = num.as_u64() {
                    self.show_number_value(ui, u, format);
                } else if let Some(i) = num.as_i64() {
                    self.show_signed_number_value(ui, i, format);
                } else if let Some(f) = num.as_f64() {
                    self.show_float_value(ui, f);
                }
            }
            Value::String(s) => {
                // Check if it's a hex string (like OpCode "F7B0")
                if Self::is_hex_string(s) {
                    if let Ok(num) = u64::from_str_radix(s, 16) {
                        ui.horizontal(|ui| {
                            let response = ui
                                .selectable_label(
                                    false,
                                    egui::RichText::new(format!("0x{s}"))
                                        .color(ui.visuals().hyperlink_color),
                                )
                                .on_hover_text("Click to filter")
                                .on_hover_cursor(egui::CursorIcon::PointingHand);

                            if response.clicked() {
                                self.response.filter_clicked = Some(format!("0x{s}"));
                            }

                            ui.label(egui::RichText::new(format!("({})", num)).weak().small());
                        });
                    } else {
                        // Just a regular string that happens to look hex-like
                        let response = ui
                            .selectable_label(
                                false,
                                egui::RichText::new(format!("\"{s}\""))
                                    .color(ui.visuals().hyperlink_color),
                            )
                            .on_hover_text("Click to filter")
                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                        if response.clicked() {
                            self.response.filter_clicked = Some(s.clone());
                        }
                    }
                } else {
                    // Regular string
                    let response = ui
                        .selectable_label(
                            false,
                            egui::RichText::new(format!("\"{s}\""))
                                .color(ui.visuals().hyperlink_color),
                        )
                        .on_hover_text("Click to filter")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    if response.clicked() {
                        self.response.filter_clicked = Some(s.clone());
                    }
                }
            }
            Value::Bool(b) => {
                let color = if *b {
                    egui::Color32::from_rgb(100, 200, 100)
                } else {
                    egui::Color32::from_rgb(200, 100, 100)
                };
                ui.label(egui::RichText::new(b.to_string()).color(color));
            }
            Value::Null => {
                ui.label(egui::RichText::new("null").weak());
            }
            _ => {
                ui.label(value.to_string());
            }
        }
    }

    /// Show a numeric value with hex/decimal display
    fn show_number_value(&mut self, ui: &mut egui::Ui, num: u64, format: DisplayFormat) {
        match format {
            DisplayFormat::Hex => {
                ui.horizontal(|ui| {
                    let response = ui
                        .selectable_label(
                            false,
                            egui::RichText::new(format!("0x{num:X}"))
                                .color(ui.visuals().hyperlink_color),
                        )
                        .on_hover_text("Click to filter")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    if response.clicked() {
                        self.response.filter_clicked = Some(format!("0x{num:X}"));
                    }

                    ui.label(egui::RichText::new(format!("({})", num)).weak().small());
                });
            }
            DisplayFormat::Decimal => {
                let response = ui
                    .selectable_label(
                        false,
                        egui::RichText::new(num.to_string()).color(ui.visuals().hyperlink_color),
                    )
                    .on_hover_text("Click to filter")
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                if response.clicked() {
                    self.response.filter_clicked = Some(num.to_string());
                }
            }
            DisplayFormat::Auto => {
                // Show both if the number is large enough to be interesting in hex
                if num > 255 {
                    ui.horizontal(|ui| {
                        let response = ui
                            .selectable_label(
                                false,
                                egui::RichText::new(format!("0x{num:X}"))
                                    .color(ui.visuals().hyperlink_color),
                            )
                            .on_hover_text("Click to filter")
                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                        if response.clicked() {
                            self.response.filter_clicked = Some(format!("0x{num:X}"));
                        }

                        ui.label(egui::RichText::new(format!("({})", num)).weak().small());
                    });
                } else {
                    let response = ui
                        .selectable_label(
                            false,
                            egui::RichText::new(num.to_string())
                                .color(ui.visuals().hyperlink_color),
                        )
                        .on_hover_text("Click to filter")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    if response.clicked() {
                        self.response.filter_clicked = Some(num.to_string());
                    }
                }
            }
        }
    }

    /// Show a signed number value
    fn show_signed_number_value(&mut self, ui: &mut egui::Ui, num: i64, format: DisplayFormat) {
        match format {
            DisplayFormat::Hex if num >= 0 => {
                ui.horizontal(|ui| {
                    let response = ui
                        .selectable_label(
                            false,
                            egui::RichText::new(format!("0x{num:X}"))
                                .color(ui.visuals().hyperlink_color),
                        )
                        .on_hover_text("Click to filter")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    if response.clicked() {
                        self.response.filter_clicked = Some(format!("0x{num:X}"));
                    }

                    ui.label(egui::RichText::new(format!("({})", num)).weak().small());
                });
            }
            _ => {
                let response = ui
                    .selectable_label(
                        false,
                        egui::RichText::new(num.to_string()).color(ui.visuals().hyperlink_color),
                    )
                    .on_hover_text("Click to filter")
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                if response.clicked() {
                    self.response.filter_clicked = Some(num.to_string());
                }
            }
        }
    }

    /// Show a float value
    fn show_float_value(&mut self, ui: &mut egui::Ui, num: f64) {
        ui.label(egui::RichText::new(format!("{num}")).weak());
    }

    /// Determine display format based on field name and value
    fn determine_format(key: &str, value: &Value) -> DisplayFormat {
        // Fields that should always show as hex
        let hex_fields = [
            "ObjectId",
            "object_id",
            "OpCode",
            "opcode",
            "Flags",
            "flags",
            "SpellId",
            "spell_id",
            "Id", // SpellId, EnchantmentId, etc.
            "CasterId",
            "caster_id",
            "Sequence",
            "sequence",
            "ItemType",
            "item_type",
            "DamageType",
            "damage_type",
            "ContainerId",
            "container_id",
            "WielderId",
            "wielder_id",
            "TargetId",
            "target_id",
            "Key", // Property keys
            "key",
        ];

        // Check if it's a hex field
        if hex_fields.iter().any(|&field| {
            key.eq_ignore_ascii_case(field)
                || key.ends_with("Id")
                || key.ends_with("_id")
                || key.contains("Sequence")
        }) {
            return DisplayFormat::Hex;
        }

        // Check if value is large enough to warrant hex display
        if let Some(num) = value.as_u64() {
            if num > 0xFFFF {
                // Large numbers likely IDs
                return DisplayFormat::Auto;
            }
        }

        DisplayFormat::Decimal
    }

    /// Check if a string looks like a hex value (e.g., "F7B0", "02CD")
    fn is_hex_string(s: &str) -> bool {
        if s.is_empty() || s.len() > 16 {
            return false;
        }
        s.chars().all(|c| c.is_ascii_hexdigit())
    }
}
