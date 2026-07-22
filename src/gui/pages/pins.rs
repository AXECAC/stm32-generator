use eframe::egui::{self};
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
use crate::gui::components::chip_canvas::{ChipCanvas, PinState, compute_pin_state};

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



impl PinsState {
    pub fn render(&mut self, ui: &mut egui::Ui, config: &mut Config, page: &mut Page) {
        // Pre-compute to avoid repeated borrow conflicts later
        let all_used: Vec<ChosenPin> = config.all_uses_pins();
        let gpio_configured: Vec<ChosenPin> = config.gpio().iter().map(|pc| pc.pin.pin()).collect();
        let gpio_list: Vec<(ChosenPin, String)> = config.gpio().iter()
            .map(|pc| (pc.pin.pin(), pc.label()))
            .collect();

        ui.heading("Конфигурация пинов GPIO");
        ui.label("Нажмите на пин на схеме платы для его настройки.");
        ui.add_space(6.0);

        // === Legend row ===
        ui.horizontal(|ui| {
            let legend = [
                ("Доступно", egui::Color32::from_rgb(140, 140, 155)),
                ("Сконфигурирован (GPIO)", egui::Color32::from_rgb(40, 190, 60)),
                ("Используется периферией", egui::Color32::from_rgb(220, 140, 20)),
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

                if let Some(ref pin_name) = self.selected_pin {
                    let chosen = ChosenPin::StmF401(StmF401Pin::from_str(pin_name).unwrap());
                    let state = compute_pin_state(&chosen, &all_used, &gpio_configured);
                    match StmF401Pin::from_str(pin_name) {
                        Err(_) => {
                            ui.group(|ui| {
                                ui.label(format!("P{} — не GPIO", pin_name));
                            });
                        }
                        Ok(pin_val) => {
                            ui.group(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("Пин P{}", pin_name))
                                        .strong()
                                        .size(16.0),
                                );
                                ui.separator();

                                match state {
                                    PinState::UsedByPeripheral => {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 165, 0),
                                            "Используется периферией",
                                        );
                                        ui.label("Освободите пин в разделе Peripherals.");
                                    }
                                    PinState::Configured => {
                                        ui.colored_label(egui::Color32::from_rgb(40, 190, 60), "Сконфигурирован как GPIO");
                                        ui.add_space(6.0);
                                        if ui.button("Удалить").clicked() {
                                            action_remove = Some(ChosenPin::StmF401(pin_val));
                                            new_selected = Some(None);
                                        }
                                    }
                                    PinState::Available => {
                                        let modes = [
                                            "Вход Floating",
                                            "Вход Pull-Up",
                                            "Вход Pull-Down",
                                            "Выход Push-Pull",
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

            let available_width = ui.available_width();
            let pad = (available_width - 350.0).max(0.0) / 2.0;
            if pad > 0.0 {
                ui.add_space(pad);
            }

            ui.vertical(|ui| {
                let board_pins = config.board.build_pins();
                let clicked_pin = ChipCanvas::new(config.board.chip_label(), board_pins)
                    .with_used_pins(&all_used, &gpio_configured)
                    .with_selected(self.selected_pin.as_deref())
                    .show(ui);

                if let Some(clicked) = clicked_pin {
                    new_selected = Some(if self.selected_pin.as_deref() == Some(&clicked) {
                        None // deselect
                    } else {
                        Some(clicked)
                    });
                }
            });
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
