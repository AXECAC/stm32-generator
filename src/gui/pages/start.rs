use eframe::egui;
use crate::gui::pages::Page;

pub struct StartState {}

impl Default for StartState {
    fn default() -> Self {
        Self {}
    }
}

impl StartState {
    pub fn render(&mut self, ui: &mut egui::Ui, page: &mut Page) {
        ui.heading("STM32 Code Generator (Black-Pill)");
        ui.label("Welcome to the STM32 generator. Follow the steps in the top bar to configure and generate your project.");
        
        ui.add_space(20.0);
        if ui.button("Begin Configuration ->").clicked() {
            *page = Page::Peripherals;
        }
    }
}
