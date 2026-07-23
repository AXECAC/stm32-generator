use adw::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    adw, gtk,
};

use crate::core::board::TargetBoard;
use crate::core::config::Config;
use crate::core::gpio::TargetMcu;

use crate::gui::pages::peripherals::{
    PeripheralsPageInput, PeripheralsPageModel, PeripheralsPageOutput,
};
use crate::gui::pages::pins::{PinsPageInput, PinsPageModel, PinsPageOutput};
use crate::gui::pages::run::{RunPageInput, RunPageModel, RunPageOutput};
use crate::gui::pages::spi::{SpiPageInput, SpiPageModel, SpiPageOutput};
use crate::gui::pages::start::{StartPageInput, StartPageModel, StartPageOutput};

pub struct AppModel {
    config: Config,
    start_page: Controller<StartPageModel>,
    pins_page: Controller<PinsPageModel>,
    spi_page: Controller<SpiPageModel>,
    peripherals_page: Controller<PeripheralsPageModel>,
    run_page: Controller<RunPageModel>,
}

#[derive(Debug)]
pub enum AppInput {
    ConfigChanged(Config),
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
        let config = Config::new(TargetBoard::BlackPill(TargetMcu::StmF401));

        let start_page = StartPageModel::builder().launch(config.clone()).forward(
            sender.input_sender(),
            |msg| match msg {
                StartPageOutput::ConfigChanged(cfg) => AppInput::ConfigChanged(cfg),
            },
        );

        let pins_page =
            PinsPageModel::builder()
                .launch(config.clone())
                .forward(sender.input_sender(), |msg| match msg {
                    PinsPageOutput::ConfigChanged(cfg) => AppInput::ConfigChanged(cfg),
                });

        let spi_page =
            SpiPageModel::builder()
                .launch(config.clone())
                .forward(sender.input_sender(), |msg| match msg {
                    SpiPageOutput::ConfigChanged(cfg) => AppInput::ConfigChanged(cfg),
                });

        let peripherals_page = PeripheralsPageModel::builder()
            .launch(config.clone())
            .forward(sender.input_sender(), |msg| match msg {
                PeripheralsPageOutput::ConfigChanged(cfg) => AppInput::ConfigChanged(cfg),
            });

        let run_page =
            RunPageModel::builder()
                .launch(config.clone())
                .forward(sender.input_sender(), |msg| match msg {
                    RunPageOutput::ConfigChanged(cfg) => AppInput::ConfigChanged(cfg),
                });

        let model = AppModel {
            config,
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
            AppInput::ConfigChanged(cfg) => {
                self.config = cfg.clone();

                // Broadcast to all pages
                self.start_page.sender().send(StartPageInput::UpdateConfig(cfg.clone())).unwrap();
                self.pins_page.sender().send(PinsPageInput::UpdateConfig(cfg.clone())).unwrap();
                self.spi_page.sender().send(SpiPageInput::UpdateConfig(cfg.clone())).unwrap();
                self.peripherals_page.sender().send(PeripheralsPageInput::UpdateConfig(cfg.clone())).unwrap();
                self.run_page.sender().send(RunPageInput::UpdateConfig(cfg)).unwrap();
            }
        }
    }
}
