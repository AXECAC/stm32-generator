use eframe::egui;
use std::str::FromStr;
use strum::VariantNames;

use crate::core::{
    config::{Config, PinConfig},
    gpio::{ChosenPin, ChosenPinWithMode, f4::{StmF4PinMode, StmF4InputMode, StmF4OutputMode, StmF4OutputSpeed}, f4::f401::StmF401Pin},
};
use crate::gui::pages::Page;

pub struct PinsState {
    pub gpio_pin_idx: usize,
    pub gpio_mode_idx: usize,
    pub gpio_label: String,
    pub gpio_error: Option<String>,
}

impl Default for PinsState {
    fn default() -> Self {
        Self {
            gpio_pin_idx: 0,
            gpio_mode_idx: 0,
            gpio_label: "".to_string(),
            gpio_error: None,
        }
    }
}

impl PinsState {
    pub fn render(&mut self, ui: &mut egui::Ui, config: &mut Config, page: &mut Page) {
        ui.heading("GPIO Pins Configuration");
        ui.label("Configure general purpose IO pins. Pins used by peripherals are not available here.");

        ui.group(|ui| {
            ui.label("Add New GPIO Pin");
            
            let all_pins = StmF401Pin::VARIANTS;
            let used_pins = config.all_uses_pins();

            let available_pins: Vec<_> = all_pins.iter().enumerate().filter(|(_, name)| {
                if let Ok(pin_val) = StmF401Pin::from_str(name) {
                    !used_pins.contains(&ChosenPin::StmF401(pin_val))
                } else {
                    false
                }
            }).collect();

            if available_pins.is_empty() {
                ui.label("No available pins left.");
            } else {
                egui::Grid::new("gpio_form").show(ui, |ui| {
                    ui.label("Pin:");
                    if available_pins.iter().find(|(orig_i, _)| *orig_i == self.gpio_pin_idx).is_none() {
                        self.gpio_pin_idx = available_pins[0].0;
                    }
                    let selected_name = all_pins.get(self.gpio_pin_idx).unwrap_or(&"");
                    egui::ComboBox::from_id_salt("gpio_pin")
                        .selected_text(*selected_name)
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            for (orig_i, name) in &available_pins {
                                ui.selectable_value(&mut self.gpio_pin_idx, *orig_i, **name);
                            }
                        });
                    ui.end_row();

                    ui.label("Mode:");
                    let modes = ["Input Floating", "Input PullUp", "Input PullDown", "Output PushPull Low"];
                    egui::ComboBox::from_id_salt("gpio_mode")
                        .selected_text(modes[self.gpio_mode_idx])
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            for (i, name) in modes.iter().enumerate() {
                                ui.selectable_value(&mut self.gpio_mode_idx, i, *name);
                            }
                        });
                    ui.end_row();

                    ui.label("Label (optional):");
                    ui.text_edit_singleline(&mut self.gpio_label);
                    ui.end_row();
                });

                if let Some(err) = &self.gpio_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                if ui.button("Add Pin").clicked() {
                    self.gpio_error = None;
                    let pin_name = all_pins.get(self.gpio_pin_idx).unwrap();
                    let pin_val = StmF401Pin::from_str(pin_name).unwrap();

                    let mode = match self.gpio_mode_idx {
                        0 => StmF4PinMode::Input(StmF4InputMode::Floating),
                        1 => StmF4PinMode::Input(StmF4InputMode::PullUp),
                        2 => StmF4PinMode::Input(StmF4InputMode::PullDown),
                        3 => StmF4PinMode::Output(StmF4OutputMode::PushPull, StmF4OutputSpeed::Low),
                        _ => StmF4PinMode::Input(StmF4InputMode::Floating),
                    };

                    let pin_cfg = PinConfig {
                        pin: ChosenPinWithMode::StmF401(pin_val, mode),
                        label: if self.gpio_label.is_empty() { None } else { Some(self.gpio_label.clone()) },
                    };

                    if let Err(e) = config.add_gpio_pin(pin_cfg) {
                        self.gpio_error = Some(format!("{:?}", e));
                    } else {
                        self.gpio_label.clear();
                    }
                }
            }
        });

        ui.separator();
        ui.heading("Configured GPIO Pins");
        let mut to_remove = None;
        for pin_config in config.gpio() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", pin_config));
                if ui.button("Remove").clicked() {
                    to_remove = Some(pin_config.pin.into());
                }
            });
        }
        if let Some(pin) = to_remove {
            config.remove_gpio_pin(&pin);
        }

        ui.add_space(20.0);
        if ui.button("Next: Run ->").clicked() {
            *page = Page::Run;
        }
    }
}
