use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use gtk::prelude::*;
use crate::core::config::Config;

pub struct StartPageModel {
    pub config: Config,
}

#[derive(Debug)]
pub enum StartPageInput {
    UpdateConfig(Config),
}

#[derive(Debug)]
pub enum StartPageOutput {
    ConfigChanged(Config),
}

#[relm4::component(pub)]
impl SimpleComponent for StartPageModel {
    type Init = Config;
    type Input = StartPageInput;
    type Output = StartPageOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 32,
            set_valign: gtk::Align::Center,

            gtk::Label {
                set_label: "Платформа",
                add_css_class: "title-1",
            },
            gtk::Label {
                set_label: "Здесь будет выбор платы и микроконтроллера.",
            }
        }
    }

    fn init(init: Self::Init, root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = StartPageModel { config: init };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            StartPageInput::UpdateConfig(cfg) => self.config = cfg,
        }
    }
}
