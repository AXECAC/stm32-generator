use crate::core::board::{Pin, PinType, TargetBoard};
use crate::core::config::{Config, PinConfig};
use crate::core::gpio::{ChosenPinWithMode, PinModeUiInfo};
use crate::gui::components::chip_canvas::{ChipCanvasInput, ChipCanvasModel, ChipCanvasOutput};
use crate::gui::components::property_row::{PropertyRowModel, PropertyRowOutput};
use adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw, gtk,
};

/// Модель страницы настройки пинов (GPIO).
///
/// Отвечает за хранение локального состояния конфигуратора:
/// текущего выбранного на холсте пина, его базового режима работы, текстового алиаса,
/// а также управляет динамически формируемым списком дополнительных свойств.
pub struct PinsPageModel {
    pub config: Config,
    pub selected_pin: Option<Pin>,
    pub current_alias: String,

    pub current_mode: Option<ChosenPinWithMode>,

    pub pin_type_model: gtk::StringList,

    pub alias_buffer: gtk::EntryBuffer,

    pub dynamic_properties: FactoryVecDeque<PropertyRowModel>,

    pub chip_canvas: Controller<ChipCanvasModel>,
}
