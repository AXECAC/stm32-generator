use eframe::egui;
use std::net::Ipv4Addr;
use std::str::FromStr;
use strum::VariantNames;

use crate::core::{
    config::{Config, SpiConfig, SpiMode},
    gpio::{ChosenPin, ChosenSpiBus, f4::f401::{StmF401Pin, StmF401SpiBus}},
    peripherals::{Peripheral, ethernet::w5500::{W5500Config, NetworkConfig, SocketMode}},
};
use crate::gui::pages::Page;

pub struct PeripheralsState {
    pub w5500_spi: usize,
    pub w5500_cs: usize,
    pub w5500_rst: usize,
    pub w5500_mac: String,
    pub w5500_ip: String,
    pub w5500_subnet: String,
    pub w5500_gateway: String,
    pub w5500_port: String,
    pub w5500_error: Option<String>,
}

impl Default for PeripheralsState {
    fn default() -> Self {
        Self {
            w5500_spi: 0,
            w5500_cs: 0,
            w5500_rst: 0,
            w5500_mac: "00:08:DC:AB:CD:EF".to_string(),
            w5500_ip: "192.168.1.100".to_string(),
            w5500_subnet: "255.255.255.0".to_string(),
            w5500_gateway: "192.168.1.1".to_string(),
            w5500_port: "8080".to_string(),
            w5500_error: None,
        }
    }
}

impl PeripheralsState {
    pub fn render(&mut self, ui: &mut egui::Ui, config: &mut Config, page: &mut Page) {
        ui.heading("Peripherals Configuration");
        ui.label("Add up to two W5500 modules.");

        let w5500_count = config.peripherals().iter().filter(|(_, p)| matches!(p, Peripheral::W5500(_))).count();

        if w5500_count < 2 {
            ui.group(|ui| {
                ui.label("Add New W5500 (TCP Server)");
                
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

                egui::Grid::new("w5500_form").show(ui, |ui| {
                    ui.label("SPI Bus:");
                    egui::ComboBox::from_id_salt("w5500_spi")
                        .selected_text(*spi_buses.get(self.w5500_spi).unwrap_or(&""))
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            for (i, name) in spi_buses.iter().enumerate() {
                                ui.selectable_value(&mut self.w5500_spi, i, *name);
                            }
                        });
                    ui.end_row();

                    let mut pin_combo = |id: &str, label: &str, selected: &mut usize| {
                        ui.label(label);
                        if available_pins.iter().find(|(orig_i, _)| *orig_i == *selected).is_none() && !available_pins.is_empty() {
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
                        ui.end_row();
                    };

                    pin_combo("w5500_cs", "CS Pin:", &mut self.w5500_cs);
                    pin_combo("w5500_rst", "RST Pin:", &mut self.w5500_rst);

                    ui.label("MAC Address:");
                    ui.text_edit_singleline(&mut self.w5500_mac);
                    ui.end_row();

                    ui.label("IP Address:");
                    ui.text_edit_singleline(&mut self.w5500_ip);
                    ui.end_row();
                    
                    ui.label("Subnet Mask:");
                    ui.text_edit_singleline(&mut self.w5500_subnet);
                    ui.end_row();
                    
                    ui.label("Gateway:");
                    ui.text_edit_singleline(&mut self.w5500_gateway);
                    ui.end_row();

                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.w5500_port);
                    ui.end_row();
                });

                if let Some(err) = &self.w5500_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                if ui.button("Add W5500").clicked() {
                    self.w5500_error = None;
                    
                    let spi_bus_name = spi_buses.get(self.w5500_spi).unwrap();
                    let cs_name = all_pins.get(self.w5500_cs).unwrap();
                    let rst_name = all_pins.get(self.w5500_rst).unwrap();
                    
                    let spi_bus_val = StmF401SpiBus::from_str(spi_bus_name).unwrap();
                    let cs_val = StmF401Pin::from_str(cs_name).unwrap();
                    let rst_val = StmF401Pin::from_str(rst_name).unwrap();
                    
                    let ip = Ipv4Addr::from_str(&self.w5500_ip);
                    let subnet = Ipv4Addr::from_str(&self.w5500_subnet);
                    let gateway = Ipv4Addr::from_str(&self.w5500_gateway);
                    let port = self.w5500_port.parse::<u16>();

                    if ip.is_err() || subnet.is_err() || gateway.is_err() || port.is_err() {
                        self.w5500_error = Some("Invalid IP or Port format".to_string());
                    } else {
                        let spi_cfg = SpiConfig {
                            bus: ChosenSpiBus::StmF401(spi_bus_val),
                            frequency_mhz: 10,
                            mode: SpiMode::Mode0,
                            sck: ChosenPin::StmF401(StmF401Pin::A5),
                            miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
                            mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
                        };

                        let _ = config.add_spi_bus(spi_cfg);

                        let w5500_cfg = W5500Config {
                            spi_bus: ChosenSpiBus::StmF401(spi_bus_val),
                            cs: ChosenPin::StmF401(cs_val),
                            rst: ChosenPin::StmF401(rst_val),
                            network: NetworkConfig {
                                mac: [0, 8, 220, 171, 205, 239],
                                ip: ip.unwrap().octets(),
                                subnet: subnet.unwrap().octets(),
                                gateway: gateway.unwrap().octets(),
                            },
                            socket_mode: SocketMode::TcpServer { port: port.unwrap(), socket_num: 0 },
                        };

                        if let Err(e) = config.add_peripheral(Peripheral::W5500(w5500_cfg)) {
                            self.w5500_error = Some(format!("{:?}", e));
                        }
                    }
                }
            });
        } else {
            ui.label("Maximum number of W5500 modules reached (2).");
        }

        ui.separator();
        ui.heading("Current Peripherals");
        let mut to_remove = None;
        for (id, periph) in config.peripherals() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", periph));
                if ui.button("Remove").clicked() {
                    to_remove = Some(*id);
                }
            });
        }
        if let Some(id) = to_remove {
            config.remove_peripheral(id);
        }

        ui.add_space(20.0);
        if ui.button("Next: GPIO Pins ->").clicked() {
            *page = Page::Pins;
        }
    }
}
