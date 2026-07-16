use std::path::PathBuf;
use std::net::Ipv4Addr;
use std::str::FromStr;
use crossbeam_channel::Receiver;
use eframe::egui;

use crate::core::{
    config::{Config, PinConfig, SpiConfig, SpiMode},
    gpio::{ChosenPin, ChosenPinWithMode, ChosenSpiBus, f4::StmF4PinMode, f4::f401::{StmF401Pin, StmF401SpiBus}, f4::StmF4InputMode, f4::StmF4OutputMode, f4::StmF4OutputSpeed},
    peripherals::{Peripheral, ethernet::{w5500::{W5500Config, NetworkConfig, SocketMode}, MacAddr}},
    worker::{WorkerMessage, start_generation},
};
use strum::VariantNames;

#[derive(PartialEq)]
enum Page {
    Start,
    Peripherals,
    Pins,
    Run,
}

pub struct GeneratorApp {
    page: Page,
    config: Config,
    output_path: String,
    
    receiver: Option<Receiver<WorkerMessage>>,
    gen_status: String,
    gen_percent: f32,
    gen_finished: bool,
    gen_error: Option<String>,

    w5500_spi: usize,
    w5500_cs: usize,
    w5500_rst: usize,
    w5500_mac: String,
    w5500_ip: String,
    w5500_subnet: String,
    w5500_gateway: String,
    w5500_port: String,
    w5500_error: Option<String>,

    gpio_pin_idx: usize,
    gpio_mode_idx: usize,
    gpio_label: String,
    gpio_error: Option<String>,
}

impl Default for GeneratorApp {
    fn default() -> Self {
        Self {
            page: Page::Start,
            config: Config::new(),
            output_path: "/home/aragami3070/projects/".to_string(),
            
            receiver: None,
            gen_status: "".to_string(),
            gen_percent: 0.0,
            gen_finished: false,
            gen_error: None,

            w5500_spi: 0,
            w5500_cs: 0,
            w5500_rst: 0,
            w5500_mac: "00:08:DC:AB:CD:EF".to_string(),
            w5500_ip: "192.168.1.100".to_string(),
            w5500_subnet: "255.255.255.0".to_string(),
            w5500_gateway: "192.168.1.1".to_string(),
            w5500_port: "8080".to_string(),
            w5500_error: None,

            gpio_pin_idx: 0,
            gpio_mode_idx: 0,
            gpio_label: "".to_string(),
            gpio_error: None,
        }
    }
}

impl GeneratorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.page, Page::Start, "Start");
            ui.selectable_value(&mut self.page, Page::Peripherals, "1. Peripherals");
            ui.selectable_value(&mut self.page, Page::Pins, "2. GPIO Pins");
            ui.selectable_value(&mut self.page, Page::Run, "3. Run");
        });
        ui.separator();
    }

    fn render_start(&mut self, ui: &mut egui::Ui) {
        ui.heading("STM32 Code Generator (Black-Pill)");
        ui.label("Welcome to the STM32 generator. Follow the steps in the top bar to configure and generate your project.");
        
        ui.add_space(20.0);
        if ui.button("Begin Configuration ->").clicked() {
            self.page = Page::Peripherals;
        }
    }

    fn render_peripherals(&mut self, ui: &mut egui::Ui) {
        ui.heading("Peripherals Configuration");
        ui.label("Add up to two W5500 modules.");

        let w5500_count = self.config.peripherals().iter().filter(|(_, p)| matches!(p, Peripheral::W5500(_))).count();

        if w5500_count < 2 {
            ui.group(|ui| {
                ui.label("Add New W5500 (TCP Server)");
                
                let spi_buses = StmF401SpiBus::VARIANTS;
                let all_pins = StmF401Pin::VARIANTS;
                let used_pins = self.config.all_uses_pins();

                // Filter out used pins for CS and RST
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
                        // Fix selected if it points to an invalid index due to previous removal
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
                        // Dummy MAC for now since we don't have a string parser in MacAddr easily available
                        let mac = MacAddr([0, 8, 220, 171, 205, 239]); 

                        // Also need to add SPI config to core first before peripheral
                        // We will add a dummy SPI config for the chosen bus
                        let spi_cfg = SpiConfig {
                            bus: ChosenSpiBus::StmF401(spi_bus_val),
                            frequency_mhz: 10,
                            mode: SpiMode::Mode0,
                            // W5500 usually needs these pins, but we can't select them here in UI yet.
                            // Assuming default SPI pins for the bus, we just pick some for compilation.
                            // But actually they must be ChosenPin. We will use dummy pins just to pass logic.
                            sck: ChosenPin::StmF401(StmF401Pin::A5),
                            miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
                            mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
                        };

                        // Ignore error if it's already added or pin conflicts for SPI dummy pins
                        let _ = self.config.add_spi_bus(spi_cfg);

                        let w5500_cfg = W5500Config {
                            spi_bus: ChosenSpiBus::StmF401(spi_bus_val),
                            cs: ChosenPin::StmF401(cs_val),
                            rst: ChosenPin::StmF401(rst_val),
                            network: NetworkConfig {
                                mac_addr: mac,
                                ip: ip.unwrap(),
                                subnet: subnet.unwrap(),
                                gateway: gateway.unwrap(),
                            },
                            socket_mode: SocketMode::TcpServer { port: port.unwrap(), socket_num: 0 },
                        };

                        if let Err(e) = self.config.add_peripheral(Peripheral::W5500(w5500_cfg)) {
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
        for (id, periph) in self.config.peripherals() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", periph));
                if ui.button("Remove").clicked() {
                    to_remove = Some(*id);
                }
            });
        }
        if let Some(id) = to_remove {
            self.config.remove_peripheral(id);
        }

        ui.add_space(20.0);
        if ui.button("Next: GPIO Pins ->").clicked() {
            self.page = Page::Pins;
        }
    }

    fn render_pins(&mut self, ui: &mut egui::Ui) {
        ui.heading("GPIO Pins Configuration");
        ui.label("Configure general purpose IO pins. Pins used by peripherals are not available here.");

        ui.group(|ui| {
            ui.label("Add New GPIO Pin");
            
            let all_pins = StmF401Pin::VARIANTS;
            let used_pins = self.config.all_uses_pins();

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

                    if let Err(e) = self.config.add_gpio_pin(pin_cfg) {
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
        for pin_config in self.config.gpio() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", pin_config));
                if ui.button("Remove").clicked() {
                    to_remove = Some(pin_config.pin.into());
                }
            });
        }
        if let Some(pin) = to_remove {
            self.config.remove_gpio_pin(&pin);
        }

        ui.add_space(20.0);
        if ui.button("Next: Run ->").clicked() {
            self.page = Page::Run;
        }
    }

    fn render_run(&mut self, ui: &mut egui::Ui) {
        ui.heading("Generate Project");

        ui.horizontal(|ui| {
            ui.label("Output Path:");
            ui.text_edit_singleline(&mut self.output_path);
        });

        if self.receiver.is_none() && !self.gen_finished && ui.button("Start Generation").clicked() {
            let path = PathBuf::from(self.output_path.clone());
            self.receiver = Some(start_generation(self.config.clone(), path));
            self.gen_status = "Starting...".to_string();
            self.gen_percent = 0.0;
            self.gen_error = None;
            self.gen_finished = false;
        }

        if let Some(rx) = &self.receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    WorkerMessage::Progress { percent, status } => {
                        self.gen_percent = percent as f32;
                        self.gen_status = status;
                    }
                    WorkerMessage::Done { output_dir } => {
                        self.gen_percent = 100.0;
                        self.gen_status = format!("Success! Saved to {:?}", output_dir);
                        self.gen_finished = true;
                        self.receiver = None;
                        break;
                    }
                    WorkerMessage::Error { message } => {
                        self.gen_status = "Generation failed".to_string();
                        self.gen_error = Some(message.to_string());
                        self.gen_finished = true;
                        self.receiver = None;
                        break;
                    }
                }
            }
        }

        if self.receiver.is_some() || self.gen_finished {
            ui.add_space(10.0);
            ui.label(format!("Status: {}", self.gen_status));
            ui.add(egui::ProgressBar::new(self.gen_percent / 100.0).show_percentage());

            if let Some(err) = &self.gen_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }
}

impl eframe::App for GeneratorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render_top_bar(ui);
        
        egui::ScrollArea::vertical().show(ui, |ui: &mut egui::Ui| {
            match self.page {
                Page::Start => self.render_start(ui),
                Page::Peripherals => self.render_peripherals(ui),
                Page::Pins => self.render_pins(ui),
                Page::Run => self.render_run(ui),
            }
        });

        if self.receiver.is_some() {
            ui.ctx().request_repaint();
        }
    }
}
