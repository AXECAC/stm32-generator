use eframe::egui;
use std::net::Ipv4Addr;
use std::str::FromStr;
use strum::VariantNames;

use crate::core::{
    config::Config,
    gpio::{ChosenPin, f4::f401::StmF401Pin},
    peripherals::{Peripheral, ethernet::w5500::{W5500Config, NetworkConfig, SocketMode}},
};
use crate::gui::pages::Page;

pub struct PeripheralsState {
    pub w5500_spi_idx: usize,
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
            w5500_spi_idx: 0,
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

        let configured_spis = config.spi().to_vec();
        if configured_spis.is_empty() {
            ui.colored_label(egui::Color32::RED, "Please configure at least one SPI bus in the 'SPI Buses' page first.");
            ui.add_space(20.0);
            if ui.button("<- Go back to SPI Buses").clicked() {
                *page = Page::Spi;
            }
            return;
        }

        let w5500_count = config.peripherals().iter().filter(|(_, p)| matches!(p, Peripheral::W5500(_))).count();

        if w5500_count < 2 {
            ui.group(|ui| {
                ui.label("Add New W5500 (TCP Server)");
                
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
                    
                    if self.w5500_spi_idx >= configured_spis.len() {
                        self.w5500_spi_idx = 0;
                    }
                    
                    let selected_spi_name = format!("{:?}", configured_spis[self.w5500_spi_idx].bus);
                    
                    egui::ComboBox::from_id_salt("w5500_spi")
                        .selected_text(selected_spi_name)
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            for (i, spi) in configured_spis.iter().enumerate() {
                                ui.selectable_value(&mut self.w5500_spi_idx, i, format!("{:?}", spi.bus));
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
                    
                    let cs_name = all_pins.get(self.w5500_cs).unwrap();
                    let rst_name = all_pins.get(self.w5500_rst).unwrap();
                    
                    let cs_val = StmF401Pin::from_str(cs_name).unwrap();
                    let rst_val = StmF401Pin::from_str(rst_name).unwrap();
                    
                    let ip = Ipv4Addr::from_str(&self.w5500_ip);
                    let subnet = Ipv4Addr::from_str(&self.w5500_subnet);
                    let gateway = Ipv4Addr::from_str(&self.w5500_gateway);
                    let port = self.w5500_port.parse::<u16>();

                    // Basic MAC parsing since we removed MacAddr string parsing easily available
                    let mac_parts: Vec<&str> = self.w5500_mac.split(':').collect();
                    let mut parsed_mac = [0u8; 6];
                    let mut mac_valid = mac_parts.len() == 6;
                    
                    if mac_valid {
                        for (i, part) in mac_parts.iter().enumerate() {
                            if let Ok(byte) = u8::from_str_radix(part, 16) {
                                parsed_mac[i] = byte;
                            } else {
                                mac_valid = false;
                                break;
                            }
                        }
                    }

                    if !mac_valid {
                        self.w5500_error = Some("Invalid MAC Address format (expected XX:XX:XX:XX:XX:XX)".to_string());
                    } else if ip.is_err() || subnet.is_err() || gateway.is_err() || port.is_err() {
                        self.w5500_error = Some("Invalid IP, Subnet, Gateway, or Port format".to_string());
                    } else {
                        let chosen_spi = configured_spis[self.w5500_spi_idx].bus;

                        let w5500_cfg = W5500Config {
                            spi_bus: chosen_spi,
                            cs: ChosenPin::StmF401(cs_val),
                            rst: ChosenPin::StmF401(rst_val),
                            network: NetworkConfig {
                                mac: parsed_mac,
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
        if ui.button("Next: Run ->").clicked() {
            *page = Page::Run;
        }
    }
}
