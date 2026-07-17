use eframe::egui;
use std::str::FromStr;
use strum::VariantNames;

use crate::core::{
    config::{Config, SpiConfig, SpiMode},
    gpio::{ChosenPin, f4::f401::{StmF401Pin, StmF401SpiBus}, ChosenSpiBus},
};
use crate::gui::pages::Page;

pub struct SpiState {
    pub spi_bus_idx: usize,
    pub sck_pin_idx: usize,
    pub miso_pin_idx: usize,
    pub mosi_pin_idx: usize,
    pub use_miso: bool,
    pub use_mosi: bool,
    pub frequency_mhz: String,
    pub mode_idx: usize,
    pub spi_error: Option<String>,
}

impl Default for SpiState {
    fn default() -> Self {
        Self {
            spi_bus_idx: 0,
            sck_pin_idx: 0,
            miso_pin_idx: 0,
            mosi_pin_idx: 0,
            use_miso: true,
            use_mosi: true,
            frequency_mhz: "10".to_string(),
            mode_idx: 0,
            spi_error: None,
        }
    }
}

impl SpiState {
    pub fn render(&mut self, ui: &mut egui::Ui, config: &mut Config, page: &mut Page) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("SPI Buses Configuration").size(20.0));
            ui.add_space(5.0);
            ui.label("Configure SPI buses before attaching peripherals like W5500.");
            ui.add_space(20.0);

            ui.allocate_ui_with_layout(egui::vec2(500.0, ui.available_height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.group(|ui| {
                    ui.label("Add New SPI Bus");
                    
                    let spi_buses = StmF401SpiBus::VARIANTS;
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
                        egui::Grid::new("spi_form").show(ui, |ui| {
                            ui.label("Bus:");
                            egui::ComboBox::from_id_salt("spi_bus")
                                .selected_text(spi_buses[self.spi_bus_idx])
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    for (i, name) in spi_buses.iter().enumerate() {
                                        ui.selectable_value(&mut self.spi_bus_idx, i, *name);
                                    }
                                });
                            ui.end_row();

                            ui.label("Frequency (MHz):");
                            ui.text_edit_singleline(&mut self.frequency_mhz);
                            ui.end_row();

                            ui.label("SPI Mode:");
                            let modes = ["Mode 0", "Mode 1", "Mode 2", "Mode 3"];
                            egui::ComboBox::from_id_salt("spi_mode")
                                .selected_text(modes[self.mode_idx])
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    for (i, name) in modes.iter().enumerate() {
                                        ui.selectable_value(&mut self.mode_idx, i, *name);
                                    }
                                });
                            ui.end_row();

                            let mut pin_combo = |ui: &mut egui::Ui, id: &str, label: &str, selected: &mut usize| {
                                ui.label(label);
                                if available_pins.iter().find(|(orig_i, _)| *orig_i == *selected).is_none() {
                                    *selected = available_pins[0].0;
                                }
                                let selected_name = all_pins.get(*selected).unwrap_or(&"");
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(*selected_name)
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        for (orig_i, name) in &available_pins {
                                            ui.selectable_value(selected, *orig_i, **name);
                                        }
                                    });
                            };

                            pin_combo(ui, "sck_pin", "SCK Pin:", &mut self.sck_pin_idx);
                            ui.end_row();

                            ui.horizontal(|ui| {
                                ui.label("MISO Pin:");
                                ui.checkbox(&mut self.use_miso, "Enable");
                            });
                            if self.use_miso {
                                pin_combo(ui, "miso_pin", "", &mut self.miso_pin_idx);
                            }
                            ui.end_row();

                            ui.horizontal(|ui| {
                                ui.label("MOSI Pin:");
                                ui.checkbox(&mut self.use_mosi, "Enable");
                            });
                            if self.use_mosi {
                                pin_combo(ui, "mosi_pin", "", &mut self.mosi_pin_idx);
                            }
                            ui.end_row();
                        });

                        if let Some(err) = &self.spi_error {
                            ui.colored_label(egui::Color32::RED, err);
                        }

                        if ui.button("Add SPI Bus").clicked() {
                            self.spi_error = None;
                            
                            let bus_name = spi_buses[self.spi_bus_idx];
                            let bus_val = StmF401SpiBus::from_str(bus_name).unwrap();

                            if let Ok(freq) = self.frequency_mhz.parse::<u32>() {
                                let mode = match self.mode_idx {
                                    0 => SpiMode::Mode0,
                                    1 => SpiMode::Mode1,
                                    2 => SpiMode::Mode2,
                                    3 => SpiMode::Mode3,
                                    _ => SpiMode::Mode0,
                                };

                                let sck_name = all_pins[self.sck_pin_idx];
                                let sck_val = StmF401Pin::from_str(sck_name).unwrap();

                                let miso = if self.use_miso {
                                    let name = all_pins[self.miso_pin_idx];
                                    Some(ChosenPin::StmF401(StmF401Pin::from_str(name).unwrap()))
                                } else {
                                    None
                                };

                                let mosi = if self.use_mosi {
                                    let name = all_pins[self.mosi_pin_idx];
                                    Some(ChosenPin::StmF401(StmF401Pin::from_str(name).unwrap()))
                                } else {
                                    None
                                };

                                let spi_cfg = SpiConfig {
                                    bus: ChosenSpiBus::StmF401(bus_val),
                                    frequency_mhz: freq,
                                    mode,
                                    sck: ChosenPin::StmF401(sck_val),
                                    miso,
                                    mosi,
                                };

                                if let Err(e) = config.add_spi_bus(spi_cfg) {
                                    self.spi_error = Some(format!("{:?}", e));
                                }
                            } else {
                                self.spi_error = Some("Invalid frequency".to_string());
                            }
                        }
                    }
                });

                ui.separator();
                ui.heading("Configured SPI Buses");
                let mut to_remove = None;
                for spi_config in config.spi() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:?}", spi_config));
                        if ui.button("Remove").clicked() {
                            to_remove = Some(spi_config.bus.clone());
                        }
                    });
                }
                if let Some(bus) = to_remove {
                    let _ = config.remove_spi(&bus);
                }

                ui.add_space(20.0);
                if ui.button("Next: Peripherals ->").clicked() {
                    *page = Page::Peripherals;
                }
            });
        });
    }
}
