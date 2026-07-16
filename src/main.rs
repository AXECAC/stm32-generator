mod core;
mod gui;

use gui::app::GeneratorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "STM32 Generator (Black Pill)",
        options,
        Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))),
    )
}
