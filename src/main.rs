mod core;
mod gui;

use gui::app::AppModel;
use relm4::RelmApp;

fn main() {
    env_logger::init();
    let app = RelmApp::new("com.github.stm32-generator");

    // Инициализируем libadwaita
    libadwaita::init().expect("Failed to initialize libadwaita");

    app.run::<AppModel>(());
}
