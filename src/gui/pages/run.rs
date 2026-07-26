use crate::core::config::Config;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};
use std::sync::{Arc, RwLock};

pub struct RunPageModel {
    pub(crate) config: Arc<RwLock<Config>>,
}

#[derive(Debug)]
pub enum RunPageInput {
    UpdateConfig,
}

#[relm4::component(pub)]
impl SimpleComponent for RunPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = RunPageInput;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 32,
            set_valign: gtk::Align::Center,

            gtk::Label {
                set_label: "Генерация кода",
                add_css_class: "title-1",
            },

            gtk::Button {
                set_label: "Сгенерировать",
                add_css_class: "suggested-action",
                add_css_class: "pill",
                set_halign: gtk::Align::Center,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RunPageModel { config: init };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            RunPageInput::UpdateConfig => {}
        }
    }
}
