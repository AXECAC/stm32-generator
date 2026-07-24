use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use gtk::prelude::*;
use crate::core::config::Config;
use std::sync::{Arc, RwLock};

pub struct SpiPageModel {
    pub(crate) config: Arc<RwLock<Config>>,
}

#[derive(Debug)]
pub enum SpiPageInput {
    UpdateConfig,
}

#[relm4::component(pub)]
impl SimpleComponent for SpiPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = SpiPageInput;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 32,
            set_valign: gtk::Align::Center,

            gtk::Label {
                set_label: "Настройка SPI",
                add_css_class: "title-1",
            },
            gtk::Label {
                set_label: "Здесь будет выбор SPI шин и CS пинов.",
            }
        }
    }

    fn init(init: Self::Init, root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = SpiPageModel { config: init };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SpiPageInput::UpdateConfig => {},
        }
    }
}
