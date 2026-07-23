use crate::core::board::{Pin, PinType};
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
    board_pins: Vec<Pin>,
    selected_pin: Option<Pin>,
    current_alias: String,

    current_mode: Option<ChosenPinWithMode>,

    pin_type_model: gtk::StringList,

    alias_buffer: gtk::EntryBuffer,

    error_message: Option<String>,

    dynamic_properties: FactoryVecDeque<PropertyRowModel>,

    chip_canvas: Controller<ChipCanvasModel>,
}

/// Входящие события страницы настройки пинов.
///
/// Обрабатывает действия пользователя и информацию о [`Config`] с других страниц.
#[derive(Debug)]
pub enum PinsPageInput {
    UpdateConfig(Config),
    PinSelected(String),

    AliasChanged(String),
    ApplyPinConfig,
    PinTypeChanged(usize),
    PropertyChanged(usize, usize),
}

/// Исходящие события страницы настройки пинов.
///
/// Используется для уведомления родительского окна о том,
/// что конфигурация микроконтроллера была успешно обновлена.
#[derive(Debug)]
pub enum PinsPageOutput {
    ConfigChanged(Config),
}

impl PinsPageModel {
    fn update_dynamic_properties(&mut self) {
        let mut guard = self.dynamic_properties.guard();
        guard.clear();

        if let Some(mode) = &self.current_mode {
            let props = mode.properties();
            for (i, (title, variants, selected)) in props.into_iter().enumerate() {
                guard.push_back((
                    i,
                    title.to_string(),
                    variants.into_iter().map(|s| s.to_string()).collect(),
                    selected,
                ));
            }
        }
    }

    /// Вспомогательная функция для сборки списка пинов вместе с их статусом
    /// настройки и алиасами.
    /// Возвращает вектор кортежей, содержащих:
    /// - сам пин,
    /// - алиас пина (если есть)
    /// - настроен ли пин
    fn build_pins_with_aliases(
        board_pins: &[Pin],
        config: &Config,
    ) -> Vec<(Pin, Option<String>, bool)> {
        let mut result = Vec::new();
        let configured_gpio = config.gpio();
        for pin in board_pins {
            let mut alias = None;
            let mut is_configured = false;
            if let PinType::Gpio(chosen_pin) = pin.pin_type
                && let Some(cfg) = configured_gpio.iter().find(|p| p.pin.pin() == chosen_pin)
            {
                is_configured = true;
                alias = cfg.label.clone();
            }
            result.push((pin.clone(), alias, is_configured));
        }
        result
    }

    /// Пересобирает конфигурацию для текущего выбранного пина на основе
    /// состояния UI, обновляет глобальную конфигурацию и уведомляет
    /// родительский компонент.
    fn rebuild_and_emit_config(&mut self, sender: &ComponentSender<Self>) {
        if let Some(pin) = &self.selected_pin
            && let PinType::Gpio(chosen_pin) = pin.pin_type
        {
            // Сначала удаляем старый конфиг для этого пина
            self.config.remove_gpio_pin(&chosen_pin);

            // Если выбран какой-то режим (не 0 "Не настроен"), собираем и добавляем новый конфиг
            if let Some(mode) = self.current_mode {
                let new_pin_config = PinConfig {
                    pin: mode,
                    label: if self.current_alias.is_empty() {
                        None
                    } else {
                        Some(self.current_alias.clone())
                    },
                };
                if let Err(err) = self.config.add_gpio_pin(new_pin_config) {
                    self.error_message = Some(err.to_string());
                    return;
                }
            }

            self.error_message = None;

            sender
                .output(PinsPageOutput::ConfigChanged(self.config.clone()))
                .expect("Failed to emit ConfigChanged output message from PinsPageModel");

            // Очищаем выбор, чтобы панель скрылась, а пин вернул свой обычный цвет (зеленый если настроен)
            self.selected_pin = None;
            self.chip_canvas
                .sender()
                .send(ChipCanvasInput::ClearSelection)
                .expect("Failed to send ClearSelection message to ChipCanvas component");
        }
    }
}
