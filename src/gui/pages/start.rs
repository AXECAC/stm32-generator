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
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading(egui::RichText::new("STM32 Code Generator (Black-Pill)").size(24.0).strong());
            ui.add_space(10.0);
            ui.label("Welcome to the STM32 generator. Follow the steps in the top bar to configure and generate your project.");
            
            ui.add_space(20.0);
            if ui.button(egui::RichText::new("Begin Configuration ->").size(16.0)).clicked() {
                *page = Page::Pins;
            }
        });
    }
}
