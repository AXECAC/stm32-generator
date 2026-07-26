use crate::core::config::Config;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};
use std::sync::{Arc, RwLock};

pub struct PeripheralsPageModel {
    pub(crate) config: Arc<RwLock<Config>>,
}

#[derive(Debug)]
pub enum PeripheralsPageInput {
    UpdateConfig,
}

#[relm4::component(pub)]
impl SimpleComponent for PeripheralsPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = PeripheralsPageInput;
    type Output = ();

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

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PeripheralsPageModel { config: init };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            PeripheralsPageInput::UpdateConfig => {}
        }
    }
}
