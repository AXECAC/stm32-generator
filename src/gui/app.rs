use adw::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    adw, gtk,
};
use std::sync::{Arc, RwLock};

use crate::core::board::{TargetBoard, TargetBoardId};
use crate::core::config::Config;
use crate::core::gpio::TargetMcu;

use crate::gui::pages::peripherals::{PeripheralsPageInput, PeripheralsPageModel};
use crate::gui::pages::pins::{PinsPageInput, PinsPageModel};
use crate::gui::pages::run::{RunPageInput, RunPageModel};
use crate::gui::pages::spi::{SpiPageInput, SpiPageModel};
use crate::gui::pages::start::{StartPageInput, StartPageModel};

pub struct AppModel {
    /// Удерживает Arc живым на всё время жизни приложения.
    /// Без этого поля счётчик ссылок мог бы упасть до нуля после init().
    _config: Arc<RwLock<Config>>,

    // Контроллеры для страниц
    start_page: Controller<StartPageModel>,
    pins_page: Controller<PinsPageModel>,
    spi_page: Controller<SpiPageModel>,
    peripherals_page: Controller<PeripheralsPageModel>,
    run_page: Controller<RunPageModel>,
}

#[derive(Debug)]
pub enum AppInput {
    TabSwitched(String),
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 1100,
            set_default_height: 800,

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                append = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::ViewSwitcherTitle {
                        set_stack: Some(&view_stack),
                        set_title: "STM32 Generator",
                    }
                },

                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[name = "view_stack"]
                    append = &adw::ViewStack {
                        set_hexpand: true,
                        set_vexpand: true,
                        connect_visible_child_name_notify[sender] => move |stack| {
                            if let Some(name) = stack.visible_child_name() {
                                sender.input(AppInput::TabSwitched(name.to_string()));
                            }
                        },

                        add_titled[Some("start"), "Начало"] = &gtk::Box {
                            #[local_ref]
                            start_widget -> adw::StatusPage {}
                        },

                        add_titled[Some("pins"), "Пины"] = &gtk::Box {
                            #[local_ref]
                            pins_widget -> gtk::Paned {}
                        },

                        add_titled[Some("spi"), "SPI"] = &gtk::Box {
                            #[local_ref]
                            spi_widget -> gtk::Box {}
                        },

                        add_titled[Some("peripherals"), "Периферия"] = &gtk::Box {
                            #[local_ref]
                            periph_widget -> gtk::Box {}
                        },

                        add_titled[Some("run"), "Генерация"] = &gtk::Box {
                            #[local_ref]
                            run_widget -> gtk::Box {}
                        },
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401)
            .expect("Black Pill / STM32F401 must be a supported platform");
        let config = Arc::new(RwLock::new(Config::new(board)));

        let start_page = StartPageModel::builder().launch(config.clone()).detach();
        let pins_page = PinsPageModel::builder().launch(config.clone()).detach();
        let spi_page = SpiPageModel::builder().launch(config.clone()).detach();
        let peripherals_page = PeripheralsPageModel::builder()
            .launch(config.clone())
            .detach();
        let run_page = RunPageModel::builder().launch(config.clone()).detach();

        let model = AppModel {
            _config: config,
            start_page,
            pins_page,
            spi_page,
            peripherals_page,
            run_page,
        };

        let start_widget = model.start_page.widget();
        let pins_widget = model.pins_page.widget();
        let spi_widget = model.spi_page.widget();
        let periph_widget = model.peripherals_page.widget();
        let run_widget = model.run_page.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::TabSwitched(name) => {
                let failed_message = "Не удалось отправить UpdateConfig в";
                match name.as_str() {
                    "start" => {
                        if let Err(e) = self.start_page.sender().send(StartPageInput::UpdateConfig)
                        {
                            log::error!("{} StartPageModel: {:?}", failed_message, e);
                        }
                    }
                    "pins" => {
                        if let Err(e) = self.pins_page.sender().send(PinsPageInput::UpdateConfig) {
                            log::error!("{} PinsPageModel: {:?}", failed_message, e);
                        }
                    }
                    "spi" => {
                        if let Err(e) = self.spi_page.sender().send(SpiPageInput::UpdateConfig) {
                            log::error!("{} SpiPageModel: {:?}", failed_message, e);
                        }
                    }
                    "peripherals" => {
                        if let Err(e) = self
                            .peripherals_page
                            .sender()
                            .send(PeripheralsPageInput::UpdateConfig)
                        {
                            log::error!("{} PeripheralsPageModel: {:?}", failed_message, e);
                        }
                    }
                    "run" => {
                        if let Err(e) = self.run_page.sender().send(RunPageInput::UpdateConfig) {
                            log::error!("{} RunPageModel: {:?}", failed_message, e);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
