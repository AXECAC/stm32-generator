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
            ui.heading(egui::RichText::new("Генератор кода STM32 (Black-Pill)").size(24.0).strong());
            ui.add_space(10.0);
            ui.label("Добро пожаловать в генератор STM32. Следуйте шагам на верхней панели, чтобы настроить и сгенерировать ваш проект.");
            
            ui.add_space(20.0);
            if ui.button(egui::RichText::new("Начать настройку ->").size(16.0)).clicked() {
                *page = Page::Pins;
            }
        });
    }
}
