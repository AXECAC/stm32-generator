mod core;
mod gui;

use gui::app::AppModel;
use relm4::RelmApp;

fn main() {
    let app = RelmApp::new("com.github.stm32-generator");

    // Инициализируем libadwaita
    let _ = libadwaita::init();

    app.run::<AppModel>(());
}
