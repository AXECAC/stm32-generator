use eframe::egui;

use crate::core::config::Config;
use crate::gui::pages::{
    Page,
    start::StartState,
    peripherals::PeripheralsState,
    pins::PinsState,
    spi::SpiState,
    run::RunState,
};

pub struct GeneratorApp {
    page: Page,
    config: Config,
    output_path: String,
    
    start_state: StartState,
    peripherals_state: PeripheralsState,
    pins_state: PinsState,
    spi_state: SpiState,
    run_state: RunState,
}

impl Default for GeneratorApp {
    fn default() -> Self {
        Self {
            page: Page::Start,
            config: Config::new(),
            output_path: "./output/".to_string(),
            
            start_state: StartState::default(),
            peripherals_state: PeripheralsState::default(),
            pins_state: PinsState::default(),
            spi_state: SpiState::default(),
            run_state: RunState::default(),
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
            ui.selectable_value(&mut self.page, Page::Pins, "1. GPIO Pins");
            ui.selectable_value(&mut self.page, Page::Spi, "2. SPI Buses");
            ui.selectable_value(&mut self.page, Page::Peripherals, "3. Peripherals");
            ui.selectable_value(&mut self.page, Page::Run, "4. Run");
        });
        ui.separator();
    }
}

impl eframe::App for GeneratorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render_top_bar(ui);
        
        egui::ScrollArea::vertical().show(ui, |ui: &mut egui::Ui| {
            match self.page {
                Page::Start => {
                    self.start_state.render(ui, &mut self.page);
                }
                Page::Pins => {
                    self.pins_state.render(ui, &mut self.config, &mut self.page);
                }
                Page::Spi => {
                    self.spi_state.render(ui, &mut self.config, &mut self.page);
                }
                Page::Peripherals => {
                    self.peripherals_state.render(ui, &mut self.config, &mut self.page);
                }
                Page::Run => {
                    self.run_state.render(ui, &self.config, &mut self.output_path);
                }
            }
        });

        // We only need to request repaint if the run state is actively polling
        if self.run_state.receiver.is_some() {
            ui.ctx().request_repaint();
        }
    }
}
