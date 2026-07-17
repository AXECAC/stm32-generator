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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let text_size = 18.0;
        ui.columns(5, |cols| {
            cols[0].vertical_centered_justified(|ui| {
                if ui.selectable_label(self.page == Page::Start, egui::RichText::new("Начало").size(text_size)).clicked() {
                    self.page = Page::Start;
                }
            });
            cols[1].vertical_centered_justified(|ui| {
                if ui.selectable_label(self.page == Page::Pins, egui::RichText::new("1. Пины GPIO").size(text_size)).clicked() {
                    self.page = Page::Pins;
                }
            });
            cols[2].vertical_centered_justified(|ui| {
                if ui.selectable_label(self.page == Page::Spi, egui::RichText::new("2. Шины SPI").size(text_size)).clicked() {
                    self.page = Page::Spi;
                }
            });
            cols[3].vertical_centered_justified(|ui| {
                if ui.selectable_label(self.page == Page::Peripherals, egui::RichText::new("3. Периферия").size(text_size)).clicked() {
                    self.page = Page::Peripherals;
                }
            });
            cols[4].vertical_centered_justified(|ui| {
                if ui.selectable_label(self.page == Page::Run, egui::RichText::new("4. Генерация").size(text_size)).clicked() {
                    self.page = Page::Run;
                }
            });
        });
        ui.add_space(10.0);
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
