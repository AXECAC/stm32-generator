use eframe::egui::{self, StrokeKind};
use std::str::FromStr;

use crate::core::{
    config::{Config, PinConfig},
    gpio::{
        ChosenPin, ChosenPinWithMode,
        f4::{StmF4PinMode, StmF4InputMode, StmF4OutputMode, StmF4OutputSpeed},
        f4::f401::StmF401Pin,
    },
};
use crate::gui::pages::Page;

/// Раскладка пинов STM32F401 Black Pill
/// Поля: (variant_name, display_label, col[0=left,1=right], row, is_gpio)
/// Пины B11, H0, H1 недоступны на Black Pill (QFN-48, F401CC)
const BOARD_PINS: &[(&str, &str, u8, u8, bool)] = &[
    // Левая колонка (USB сверху, смотрим на плату)
    ("",    "3V3",  0,  0, false),
    ("",    "GND",  0,  1, false),
    ("",    "3V3",  0,  2, false),
    ("",    "GND",  0,  3, false),
    ("B9",  "PB9",  0,  4, true),
    ("B8",  "PB8",  0,  5, true),
    ("B7",  "PB7",  0,  6, true),
    ("B6",  "PB6",  0,  7, true),
    ("B5",  "PB5",  0,  8, true),
    ("B4",  "PB4",  0,  9, true),
    ("B3",  "PB3",  0, 10, true),
    ("A15", "PA15", 0, 11, true),
    ("A12", "PA12", 0, 12, true),
    ("A11", "PA11", 0, 13, true),
    ("A10", "PA10", 0, 14, true),
    ("A9",  "PA9",  0, 15, true),
    ("A8",  "PA8",  0, 16, true),
    ("B15", "PB15", 0, 17, true),
    ("B14", "PB14", 0, 18, true),
    ("B13", "PB13", 0, 19, true),
    // Правая колонка
    ("",    "VBAT", 1,  0, false),
    ("C13", "PC13", 1,  1, true),
    ("C14", "PC14", 1,  2, true),
    ("C15", "PC15", 1,  3, true),
    ("A0",  "PA0",  1,  4, true),
    ("A1",  "PA1",  1,  5, true),
    ("A2",  "PA2",  1,  6, true),
    ("A3",  "PA3",  1,  7, true),
    ("A4",  "PA4",  1,  8, true),
    ("A5",  "PA5",  1,  9, true),
    ("A6",  "PA6",  1, 10, true),
    ("A7",  "PA7",  1, 11, true),
    ("B0",  "PB0",  1, 12, true),
    ("B1",  "PB1",  1, 13, true),
    ("B2",  "PB2",  1, 14, true),
    ("B10", "PB10", 1, 15, true),
    ("B12", "PB12", 1, 16, true),
    ("",    "5V",   1, 17, false),
    ("",    "GND",  1, 18, false),
    ("",    "GND",  1, 19, false),
];

const NUM_ROWS: usize = 20;
const PIN_R: f32 = 7.0;
const PIN_SPACING: f32 = 24.0;
const BOARD_W: f32 = 100.0;
const LABEL_W: f32 = 44.0;

#[derive(PartialEq)]
enum PinState {
    Available,
    Configured,
    UsedByPeripheral,
}

pub struct PinsState {
    pub selected_pin: Option<String>,
    pub gpio_mode_idx: usize,
    pub gpio_label: String,
    pub gpio_error: Option<String>,
}

impl Default for PinsState {
    fn default() -> Self {
        Self {
            selected_pin: None,
            gpio_mode_idx: 0,
            gpio_label: String::new(),
            gpio_error: None,
        }
    }
}

fn compute_pin_state(pin_key: &str, all_used: &[ChosenPin], gpio_configured: &[ChosenPin]) -> PinState {
    let Ok(pin_val) = StmF401Pin::from_str(pin_key) else {
        return PinState::Available;
    };
    let chosen = ChosenPin::StmF401(pin_val);
    if gpio_configured.contains(&chosen) {
        PinState::Configured
    } else if all_used.contains(&chosen) {
        PinState::UsedByPeripheral
    } else {
        PinState::Available
    }
}

fn pin_circle_color(state: &PinState, is_gpio: bool, label: &str) -> egui::Color32 {
    if !is_gpio {
        return match label {
            "GND"  => egui::Color32::from_rgb(30, 30, 30),
            "3V3"  => egui::Color32::from_rgb(200, 50, 50),
            "5V"   => egui::Color32::from_rgb(230, 110, 0),
            "VBAT" => egui::Color32::from_rgb(160, 40, 40),
            _      => egui::Color32::DARK_GRAY,
        };
    }
    match state {
        PinState::Configured      => egui::Color32::from_rgb(40, 190, 60),
        PinState::UsedByPeripheral => egui::Color32::from_rgb(220, 140, 20),
        PinState::Available       => egui::Color32::from_rgb(140, 140, 155),
    }
}

impl PinsState {
    pub fn render(&mut self, ui: &mut egui::Ui, config: &mut Config, page: &mut Page) {
        // Pre-compute to avoid repeated borrow conflicts later
        let all_used: Vec<ChosenPin> = config.all_uses_pins();
        let gpio_configured: Vec<ChosenPin> = config.gpio().iter().map(|pc| pc.pin.pin()).collect();
        let gpio_list: Vec<(ChosenPin, String)> = config.gpio().iter()
            .map(|pc| (pc.pin.pin(), pc.label()))
            .collect();

        ui.heading("GPIO Pins Configuration");
        ui.label("Нажмите на пин на схеме платы для его настройки.");
        ui.add_space(6.0);

        // === Legend row ===
        ui.horizontal(|ui| {
            let legend = [
                ("Available", egui::Color32::from_rgb(140, 140, 155)),
                ("GPIO configured", egui::Color32::from_rgb(40, 190, 60)),
                ("Used by peripheral", egui::Color32::from_rgb(220, 140, 20)),
            ];
            for (label, color) in legend {
                let (resp, painter) = ui.allocate_painter(egui::Vec2::splat(14.0), egui::Sense::hover());
                painter.circle_filled(resp.rect.center(), 6.0, color);
                painter.circle_stroke(resp.rect.center(), 6.0, egui::Stroke::new(0.8, egui::Color32::BLACK));
                ui.label(label);
                ui.add_space(12.0);
            }
        });
        ui.add_space(6.0);

        // === Main layout: left config panel + board canvas ===
        let board_canvas_w = LABEL_W + PIN_R * 2.5 + BOARD_W + PIN_R * 2.5 + LABEL_W;
        let board_canvas_h = NUM_ROWS as f32 * PIN_SPACING + 40.0;

        // Collect mutations we need to perform after rendering
        let mut action_add: Option<(StmF401Pin, StmF4PinMode, Option<String>)> = None;
        let mut action_remove: Option<ChosenPin> = None;
        let mut new_selected: Option<Option<String>> = None; // Some(Some(pin)) = select, Some(None) = deselect
        let mut go_next = false;

        ui.horizontal(|ui| {
            // ---- Left config panel ----
            ui.vertical(|ui| {
                ui.set_min_width(240.0);
                ui.set_max_width(240.0);

                if let Some(ref pin_name) = self.selected_pin.clone() {
                    let state = compute_pin_state(pin_name, &all_used, &gpio_configured);
                    match StmF401Pin::from_str(pin_name) {
                        Err(_) => {
                            ui.group(|ui| {
                                ui.label(format!("P{} — не GPIO", pin_name));
                            });
                        }
                        Ok(pin_val) => {
                            ui.group(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("Pin  P{}", pin_name))
                                        .strong()
                                        .size(16.0),
                                );
                                ui.separator();

                                match state {
                                    PinState::UsedByPeripheral => {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 165, 0),
                                            "⚠  Используется периферией",
                                        );
                                        ui.label("Освободите пин в разделе Peripherals.");
                                    }
                                    PinState::Configured => {
                                        ui.colored_label(egui::Color32::from_rgb(40, 190, 60), "✓  Сконфигурирован как GPIO");
                                        ui.add_space(6.0);
                                        if ui.button("🗑  Удалить").clicked() {
                                            action_remove = Some(ChosenPin::StmF401(pin_val));
                                            new_selected = Some(None);
                                        }
                                    }
                                    PinState::Available => {
                                        let modes = [
                                            "Input Floating",
                                            "Input PullUp",
                                            "Input PullDown",
                                            "Output PushPull",
                                        ];
                                        ui.label("Режим:");
                                        egui::ComboBox::from_id_salt("pin_mode_combo")
                                            .width(200.0)
                                            .selected_text(modes[self.gpio_mode_idx])
                                            .show_ui(ui, |ui| {
                                                for (i, name) in modes.iter().enumerate() {
                                                    ui.selectable_value(&mut self.gpio_mode_idx, i, *name);
                                                }
                                            });

                                        ui.add_space(4.0);
                                        ui.label("Метка (опционально):");
                                        ui.text_edit_singleline(&mut self.gpio_label);

                                        if let Some(ref err) = self.gpio_error.clone() {
                                            ui.colored_label(egui::Color32::RED, err);
                                        }

                                        ui.add_space(8.0);
                                        if ui.button("💾  Сохранить").clicked() {
                                            let mode = match self.gpio_mode_idx {
                                                0 => StmF4PinMode::Input(StmF4InputMode::Floating),
                                                1 => StmF4PinMode::Input(StmF4InputMode::PullUp),
                                                2 => StmF4PinMode::Input(StmF4InputMode::PullDown),
                                                _ => StmF4PinMode::Output(StmF4OutputMode::PushPull, StmF4OutputSpeed::Low),
                                            };
                                            let lbl = if self.gpio_label.is_empty() {
                                                None
                                            } else {
                                                Some(self.gpio_label.clone())
                                            };
                                            action_add = Some((pin_val, mode, lbl));
                                        }
                                    }
                                }
                            });
                        }
                    }
                } else {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Пин не выбран").italics());
                        ui.add_space(4.0);
                        ui.label("← Нажмите на пин на схеме");
                    });
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new("Сконфигурированные GPIO:").strong());

                let mut remove_from_list: Option<ChosenPin> = None;
                if gpio_list.is_empty() {
                    ui.label(egui::RichText::new("(пусто)").italics().weak());
                }
                for (chosen_pin, label) in &gpio_list {
                    let pin_key = match chosen_pin {
                        ChosenPin::StmF401(p) => {
                            let s: &'static str = (*p).into();
                            s.to_string()
                        }
                    };
                    let is_sel = self.selected_pin.as_deref() == Some(&pin_key);

                    ui.horizontal(|ui| {
                        let text = egui::RichText::new(format!("P{}  ({})", pin_key, label));
                        let text = if is_sel { text.color(egui::Color32::from_rgb(40, 200, 60)).strong() } else { text };
                        ui.label(text);
                        if ui.small_button("✕").clicked() {
                            remove_from_list = Some(*chosen_pin);
                        }
                    });
                }
                if let Some(p) = remove_from_list {
                    action_remove = Some(p);
                }

                ui.add_space(12.0);
                if ui.button("Далее: SPI Buses →").clicked() {
                    go_next = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // ---- Board schematic canvas ----
            let (board_resp, painter) = ui.allocate_painter(
                egui::Vec2::new(board_canvas_w, board_canvas_h),
                egui::Sense::click(),
            );

            let origin = board_resp.rect.min;
            let board_x = origin.x + LABEL_W + PIN_R * 2.5;

            // PCB body
            let pcb_rect = egui::Rect::from_min_size(
                egui::pos2(board_x, origin.y + 18.0),
                egui::Vec2::new(BOARD_W, board_canvas_h - 28.0),
            );
            painter.rect_filled(pcb_rect, 6.0, egui::Color32::from_rgb(18, 72, 18));
            painter.rect_stroke(
                pcb_rect,
                6.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(50, 130, 50)),
                StrokeKind::Outside,
            );

            // USB connector at top
            let usb_w = 30.0;
            let usb_rect = egui::Rect::from_min_size(
                egui::pos2(board_x + BOARD_W / 2.0 - usb_w / 2.0, origin.y),
                egui::Vec2::new(usb_w, 20.0),
            );
            painter.rect_filled(usb_rect, 3.0, egui::Color32::from_rgb(200, 200, 220));
            painter.rect_stroke(
                usb_rect,
                3.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 130, 150)),
                StrokeKind::Outside,
            );
            painter.text(
                usb_rect.center(),
                egui::Align2::CENTER_CENTER,
                "USB",
                egui::FontId::proportional(8.0),
                egui::Color32::BLACK,
            );

            // MCU chip outline in center
            let chip_margin = 16.0;
            let chip_rect = egui::Rect::from_min_size(
                egui::pos2(board_x + chip_margin, pcb_rect.min.y + 50.0),
                egui::Vec2::new(BOARD_W - chip_margin * 2.0, pcb_rect.height() - 100.0),
            );
            painter.rect_filled(chip_rect, 2.0, egui::Color32::from_rgb(30, 30, 35));
            painter.rect_stroke(
                chip_rect,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                StrokeKind::Outside,
            );
            painter.text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                "STM32\nF401",
                egui::FontId::monospace(10.0),
                egui::Color32::from_rgb(160, 220, 160),
            );

            // Interaction state
            let hover_pos = board_resp.hover_pos();
            let click_pos = if board_resp.clicked() {
                board_resp.interact_pointer_pos()
            } else {
                None
            };

            let mut clicked_pin: Option<String> = None;

            for &(pin_key, label, col, row, is_gpio) in BOARD_PINS {
                let pin_y = origin.y + 28.0 + row as f32 * PIN_SPACING;

                let (pin_cx, label_x, align) = if col == 0 {
                    let px = board_x - PIN_R * 1.5;
                    (px, px - PIN_R - 2.0, egui::Align2::RIGHT_CENTER)
                } else {
                    let px = board_x + BOARD_W + PIN_R * 1.5;
                    (px, px + PIN_R + 2.0, egui::Align2::LEFT_CENTER)
                };

                let pin_center = egui::pos2(pin_cx, pin_y);

                // Wire from pin to board edge
                let board_edge_x = if col == 0 { board_x } else { board_x + BOARD_W };
                painter.line_segment(
                    [pin_center, egui::pos2(board_edge_x, pin_y)],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(160, 160, 100)),
                );

                // Pin state and color
                let state = if is_gpio {
                    compute_pin_state(pin_key, &all_used, &gpio_configured)
                } else {
                    PinState::Available // irrelevant for non-gpio
                };
                let color = pin_circle_color(&state, is_gpio, label);

                let is_selected = is_gpio && self.selected_pin.as_deref() == Some(pin_key);
                let is_hovered = is_gpio
                    && hover_pos.map(|p| (p - pin_center).length() < PIN_R + 4.0).unwrap_or(false);

                // Glow effects
                if is_selected {
                    painter.circle_filled(
                        pin_center,
                        PIN_R + 5.0,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 45),
                    );
                    painter.circle_stroke(
                        pin_center,
                        PIN_R + 3.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                } else if is_hovered {
                    painter.circle_filled(
                        pin_center,
                        PIN_R + 4.0,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 0, 35),
                    );
                    painter.circle_stroke(
                        pin_center,
                        PIN_R + 2.0,
                        egui::Stroke::new(1.5, egui::Color32::YELLOW),
                    );
                }

                // Pin circle
                painter.circle_filled(pin_center, PIN_R, color);
                painter.circle_stroke(
                    pin_center,
                    PIN_R,
                    egui::Stroke::new(0.8, egui::Color32::from_rgb(0, 0, 0)),
                );

                // Label text
                let text_color = if is_gpio {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(140, 140, 140)
                };
                painter.text(
                    egui::pos2(label_x, pin_y),
                    align,
                    label,
                    egui::FontId::monospace(9.0),
                    text_color,
                );

                // Click detection
                if is_gpio {
                    if let Some(cpos) = click_pos {
                        if (cpos - pin_center).length() < PIN_R + 4.0 {
                            clicked_pin = Some(pin_key.to_string());
                        }
                    }
                }
            }

            // Process board click
            if let Some(clicked) = clicked_pin {
                new_selected = Some(if self.selected_pin.as_deref() == Some(&clicked) {
                    None // deselect
                } else {
                    Some(clicked)
                });
            }
        });

        // Apply deferred mutations
        if let Some(new_sel) = new_selected {
            if new_sel.as_deref() != self.selected_pin.as_deref() {
                self.gpio_mode_idx = 0;
                self.gpio_label.clear();
                self.gpio_error = None;
            }
            self.selected_pin = new_sel;
        }

        if let Some((pin_val, mode, lbl)) = action_add {
            let cfg = PinConfig {
                pin: ChosenPinWithMode::StmF401(pin_val, mode),
                label: lbl,
            };
            match config.add_gpio_pin(cfg) {
                Ok(_) => {
                    self.gpio_label.clear();
                    self.gpio_error = None;
                }
                Err(e) => {
                    self.gpio_error = Some(format!("{:?}", e));
                }
            }
        }

        if let Some(pin) = action_remove {
            config.remove_gpio_pin(&pin);
        }

        if go_next {
            *page = Page::Spi;
        }
    }
}
