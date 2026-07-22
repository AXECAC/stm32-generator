use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use gtk::prelude::*;
use crate::core::config::Config;

pub struct PeripheralsPageModel {
    pub config: Config,
}

#[derive(Debug)]
pub enum PeripheralsPageInput {
    UpdateConfig(Config),
}

#[derive(Debug)]
pub enum PeripheralsPageOutput {
    ConfigChanged(Config),
}

#[relm4::component(pub)]
impl SimpleComponent for PeripheralsPageModel {
    type Init = Config;
    type Input = PeripheralsPageInput;
    type Output = PeripheralsPageOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 32,
            set_valign: gtk::Align::Center,

            gtk::Label {
                set_label: "Периферия (W5500)",
                add_css_class: "title-1",
            },
            gtk::Label {
                set_label: "Здесь будут настройки IP, MAC, Портов.",
            }
        }
    }

    fn init(init: Self::Init, root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = PeripheralsPageModel { config: init };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            PeripheralsPageInput::UpdateConfig(cfg) => self.config = cfg,
        }
    }
}
