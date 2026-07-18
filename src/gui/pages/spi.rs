use eframe::egui;
use std::str::FromStr;
use strum::VariantNames;

use crate::core::{
    config::{Config, SpiConfig, SpiMode},
    gpio::{
        ChosenPin, ChosenSpiBus,
        f4::f401::{StmF401Pin, StmF401SpiBus},
    },
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
            ui.heading(egui::RichText::new("Конфигурация шин SPI").size(20.0));
            ui.add_space(5.0);
            ui.label("Настройте шины SPI перед добавлением периферии (например, W5500).");
            ui.add_space(20.0);

            ui.allocate_ui_with_layout(
                egui::vec2(500.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.group(|ui| {
                        ui.label("Добавить новую шину SPI");

                        let spi_buses = StmF401SpiBus::VARIANTS;
                        let all_pins = StmF401Pin::VARIANTS;
                        let used_pins = config.all_uses_pins();

                        let available_pins: Vec<_> = all_pins
                            .iter()
                            .enumerate()
                            .filter(|(_, name)| {
                                if let Ok(pin_val) = StmF401Pin::from_str(name) {
                                    !used_pins.contains(&ChosenPin::StmF401(pin_val))
                                } else {
                                    false
                                }
                            })
                            .collect();

                        if available_pins.is_empty() {
                            ui.label("Нет доступных пинов.");
                        } else {
                            egui::Grid::new("spi_form").show(ui, |ui| {
                                ui.label("Шина:");
                                egui::ComboBox::from_id_salt("spi_bus")
                                    .selected_text(spi_buses[self.spi_bus_idx])
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        for (i, name) in spi_buses.iter().enumerate() {
                                            ui.selectable_value(&mut self.spi_bus_idx, i, *name);
                                        }
                                    });
                                ui.end_row();

                                ui.label("Частота (МГц):");
                                ui.text_edit_singleline(&mut self.frequency_mhz);
                                ui.end_row();

                                ui.label("Режим SPI:");
                                let modes = ["Режим 0", "Режим 1", "Режим 2", "Режим 3"];
                                egui::ComboBox::from_id_salt("spi_mode")
                                    .selected_text(modes[self.mode_idx])
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        for (i, name) in modes.iter().enumerate() {
                                            ui.selectable_value(&mut self.mode_idx, i, *name);
                                        }
                                    });
                                ui.end_row();

                                let pin_combo =
                                    |ui: &mut egui::Ui,
                                     id: &str,
                                     label: &str,
                                     selected: &mut usize,
                                     local_used: &[usize]| {
                                        ui.label(label);

                                        let filtered_pins: Vec<_> = available_pins
                                            .iter()
                                            .filter(|(orig_i, _)| !local_used.contains(orig_i))
                                            .collect();

                                        if filtered_pins
                                            .iter()
                                            .find(|&&(orig_i, _)| *orig_i == *selected)
                                            .is_none()
                                            && !filtered_pins.is_empty()
                                        {
                                            *selected = filtered_pins[0].0;
                                        }
                                        let selected_name = all_pins.get(*selected).unwrap_or(&"");
                                        egui::ComboBox::from_id_salt(id)
                                            .selected_text(*selected_name)
                                            .show_ui(ui, |ui: &mut egui::Ui| {
                                                for &&(orig_i, name) in &filtered_pins {
                                                    ui.selectable_value(selected, orig_i, *name);
                                                }
                                            });
                                    };

                                pin_combo(ui, "sck_pin", "Пин SCK:", &mut self.sck_pin_idx, &[]);
                                ui.end_row();

                                ui.horizontal(|ui| {
                                    ui.label("Пин MISO:");
                                    ui.checkbox(&mut self.use_miso, "Включить");
                                });
                                if self.use_miso {
                                    pin_combo(
                                        ui,
                                        "miso_pin",
                                        "",
                                        &mut self.miso_pin_idx,
                                        &[self.sck_pin_idx],
                                    );
                                }
                                ui.end_row();

                                ui.horizontal(|ui| {
                                    ui.label("Пин MOSI:");
                                    ui.checkbox(&mut self.use_mosi, "Включить");
                                });
                                if self.use_mosi {
                                    let mut used_for_mosi = vec![self.sck_pin_idx];
                                    if self.use_miso {
                                        used_for_mosi.push(self.miso_pin_idx);
                                    }
                                    pin_combo(
                                        ui,
                                        "mosi_pin",
                                        "",
                                        &mut self.mosi_pin_idx,
                                        &used_for_mosi,
                                    );
                                }
                                ui.end_row();
                            });

                            if let Some(err) = &self.spi_error {
                                ui.colored_label(egui::Color32::RED, err);
                            }

                            if ui.button("Добавить шину SPI").clicked() {
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
                                        Some(ChosenPin::StmF401(
                                            StmF401Pin::from_str(name).unwrap(),
                                        ))
                                    } else {
                                        None
                                    };

                                    let mosi = if self.use_mosi {
                                        let name = all_pins[self.mosi_pin_idx];
                                        Some(ChosenPin::StmF401(
                                            StmF401Pin::from_str(name).unwrap(),
                                        ))
                                    } else {
                                        None
                                    };

                                    let spi_cfg_res = SpiConfig::new(
                                        ChosenSpiBus::StmF401(bus_val),
                                        freq,
                                        mode,
                                        ChosenPin::StmF401(sck_val),
                                        miso,
                                        mosi,
                                    );

                                    match spi_cfg_res {
                                        Ok(spi_cfg) => {
                                            if let Err(e) = config.add_spi_bus(spi_cfg) {
                                                self.spi_error = Some(format!("{:?}", e));
                                            }
                                        }
                                        Err(e) => {
                                            self.spi_error = Some(format!("{:?}", e));
                                        }
                                    }
                                } else {
                                    self.spi_error = Some("Неверная частота".to_string());
                                }
                            }
                        }
                    });

                    ui.separator();
                    ui.heading("Сконфигурированные шины SPI");
                    ui.add_space(10.0);

                    let mut to_remove = None;
                    for spi_config in config.spi() {
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_gray(35))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{:?}", spi_config.bus))
                                            .strong()
                                            .size(16.0),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("Удалить").clicked() {
                                                to_remove = Some(spi_config.bus);
                                            }
                                        },
                                    );
                                });

                                ui.separator();

                                ui.horizontal(|ui| {
                                    ui.label(format!("Частота: {} МГц", spi_config.frequency_mhz));
                                    ui.label("|");
                                    ui.label(format!("Режим: {:?}", spi_config.mode));
                                });

                                ui.add_space(3.0);

                                ui.horizontal(|ui| {
                                    ui.label(format!("SCK: {:?}", spi_config.sck));

                                    ui.label("|");
                                    if let Some(miso) = &spi_config.miso {
                                        ui.label(format!("MISO: {:?}", miso));
                                    } else {
                                        ui.label("MISO: Выкл.");
                                    }

                                    ui.label("|");
                                    if let Some(mosi) = &spi_config.mosi {
                                        ui.label(format!("MOSI: {:?}", mosi));
                                    } else {
                                        ui.label("MOSI: Выкл.");
                                    }
                                });
                            });
                        ui.add_space(5.0);
                    }
                    if let Some(bus) = to_remove {
                        let _ = config.remove_spi(&bus);
                    }

                    ui.add_space(20.0);
                    if ui.button("Далее: Периферия ->").clicked() {
                        *page = Page::Peripherals;
                    }
                },
            );
        });
    }
}
