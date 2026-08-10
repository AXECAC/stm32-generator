use std::sync::{Arc, RwLock};

use adw::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw, gtk,
};

use crate::core::board::{Pin, PinType};
use crate::core::config::{Config, PinConfig};
use crate::gui::components::chip_canvas::{ChipCanvasInput, ChipCanvasModel, ChipCanvasOutput};
use crate::gui::components::forms::pin_mode::{
    PinModeFormConfig, PinModeFormInput, PinModeFormModel, PinModeFormOutput,
};

/// Модель страницы настройки пинов GPIO.
///
/// Страница отвечает за связь формы, canvas и глобального [`Config`]. Локальное
/// состояние формы режима пина хранится в [`PinModeFormModel`].
pub struct PinsPageModel {
    /// Глобальная конфигурация приложения.
    pub config: Arc<RwLock<Config>>,
    /// Список доступных пинов для выбранной платы.
    board_pins: Vec<Pin>,
    /// Пин, который сейчас выбран на canvas и будет применён формой.
    selected_pin: Option<Pin>,
    /// Форма настройки режима выбранного GPIO-пина.
    form: Controller<PinModeFormModel>,
    /// Компонент холста микроконтроллера.
    chip_canvas: Controller<ChipCanvasModel>,
}

/// Входящие события страницы настройки пинов.
#[derive(Debug)]
pub enum PinsPageInput {
    /// Вкладка стала активной; нужно перечитать свежий [`Config`].
    UpdateConfig,
    /// Пользователь выбрал пин на canvas.
    PinSelected(String),
    /// Форма собрала конфигурацию выбранного GPIO-пина.
    ApplyPinConfig(PinModeFormConfig),
}

#[relm4::component(pub)]
impl SimpleComponent for PinsPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = PinsPageInput;
    type Output = ();

    view! {
        gtk::Paned {
            set_orientation: gtk::Orientation::Horizontal,
            set_wide_handle: true,
            set_position: 350,
            set_hexpand: true,
            set_vexpand: true,

            #[wrap(Some)]
            set_start_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 16,
                set_spacing: 16,

                #[local_ref]
                form_widget -> gtk::Box {}
            },

            #[wrap(Some)]
            set_end_child = &gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    #[local_ref]
                    canvas_widget -> gtk::DrawingArea {}
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let board_pins = init.read().unwrap().board.build_pins();
        let config = init.read().unwrap();
        let pins_data = config.build_pins_with_aliases(&board_pins);
        let locked_pin_keys = config.not_gpio_configured_pins_keys(&board_pins);

        let chip_canvas = ChipCanvasModel::builder()
            .launch((config.board.chip_label().to_string(), pins_data))
            .forward(sender.input_sender(), |output| match output {
                ChipCanvasOutput::PinSelected(key) => PinsPageInput::PinSelected(key),
            });
        drop(config);

        if let Err(e) = chip_canvas
            .sender()
            .send(ChipCanvasInput::UpdateLockedPins(locked_pin_keys))
        {
            log::error!(
                "Не удалось отправить UpdateLockedPins в компонент ChipCanvas: {:?}",
                e
            );
        }

        let form =
            PinModeFormModel::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    PinModeFormOutput::Apply(config) => PinsPageInput::ApplyPinConfig(config),
                });

        let model = PinsPageModel {
            config: init,
            board_pins,
            selected_pin: None,
            form,
            chip_canvas,
        };

        let form_widget = model.form.widget();
        let canvas_widget = model.chip_canvas.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            PinsPageInput::UpdateConfig => self.handle_update_config(),
            PinsPageInput::PinSelected(key) => self.handle_pin_selected(&key),
            PinsPageInput::ApplyPinConfig(config) => self.apply_pin_config(config),
        }
    }
}

impl PinsPageModel {
    /// Перечитывает глобальный [`Config`] и обновляет canvas.
    fn handle_update_config(&mut self) {
        self.send_form_input(PinModeFormInput::ClearError);

        let config = self.config.read().unwrap();
        self.board_pins = config.board.build_pins();

        // Обновляем название чипа на холсте при смене платы
        if let Err(e) = self
            .chip_canvas
            .sender()
            .send(ChipCanvasInput::UpdateChipLabel(
                config.board.chip_label(),
            ))
        {
            log::error!(
                "Не удалось отправить UpdateChipLabel в ChipCanvas: {:?}",
                e
            );
        }

        self.update_canvas_pins(&config);
        self.update_locked_canvas_pins(&config);

        if let Some(selected_pin) = self.selected_pin.clone() {
            if self.is_pin_locked_by_non_gpio(&config, &selected_pin) {
                drop(config);
                self.clear_selection();
                return;
            }

            let configured = self.find_gpio_config(&config, &selected_pin);
            self.send_form_input(PinModeFormInput::SelectPin {
                pin: selected_pin,
                config: configured,
            });
        }
    }

    /// Обрабатывает выбор пина на canvas.
    fn handle_pin_selected(&mut self, key: &str) {
        self.send_form_input(PinModeFormInput::ClearError);

        let Some(pin) = self.board_pins.iter().find(|pin| pin.key == key).cloned() else {
            return;
        };

        let config = self.config.read().unwrap();
        if self.is_pin_locked_by_non_gpio(&config, &pin) {
            drop(config);
            self.clear_selection();
            return;
        }

        let configured = self.find_gpio_config(&config, &pin);

        self.selected_pin = Some(pin.clone());
        self.send_form_input(PinModeFormInput::SelectPin {
            pin,
            config: configured,
        });
    }

    /// Применяет конфигурацию формы к текущему выбранному GPIO-пину.
    fn apply_pin_config(&mut self, form_config: PinModeFormConfig) {
        let Some(pin) = &self.selected_pin else {
            return;
        };
        let PinType::Gpio(chosen_pin) = pin.pin_type else {
            return;
        };

        {
            let mut config = self.config.write().unwrap();
            config.remove_gpio_pin(&chosen_pin);

            if let Some(mode) = form_config.mode {
                let new_pin_config = PinConfig {
                    pin: mode,
                    label: form_config.alias,
                };

                if let Err(err) = config.add_gpio_pin(new_pin_config) {
                    self.set_form_error(err.to_string());
                    return;
                }
            }
        }

        self.send_form_input(PinModeFormInput::ClearError);
        {
            let config = self.config.read().unwrap();
            self.update_canvas_pins(&config);
            self.update_locked_canvas_pins(&config);
        }
        self.clear_selection();
    }

    /// Возвращает `true`, если пин занят SPI или периферией и не должен редактироваться на GPIO-странице.
    fn is_pin_locked_by_non_gpio(&self, config: &Config, pin: &Pin) -> bool {
        matches!(
            pin.pin_type,
            PinType::Gpio(chosen_pin) if config.not_gpio_configured_pins().contains(&chosen_pin)
        )
    }

    /// Возвращает текущую GPIO-конфигурацию выбранного пина.
    fn find_gpio_config(&self, config: &Config, pin: &Pin) -> Option<PinConfig> {
        let PinType::Gpio(chosen_pin) = pin.pin_type else {
            return None;
        };

        config
            .gpio()
            .iter()
            .find(|pin_config| pin_config.pin.pin() == chosen_pin)
            .cloned()
    }

    /// Отправляет в canvas актуальный список пинов и alias-ов.
    fn update_canvas_pins(&self, config: &Config) {
        let pins_data = config.build_pins_with_aliases(&self.board_pins);
        if let Err(e) = self
            .chip_canvas
            .sender()
            .send(ChipCanvasInput::UpdatePins(pins_data))
        {
            log::error!("Не удалось отправить UpdatePins в ChipCanvas: {:?}", e);
        }
    }

    /// Отправляет в canvas актуальный список некликабельных пинов.
    fn update_locked_canvas_pins(&self, config: &Config) {
        let locked_pin_keys = config.not_gpio_configured_pins_keys(&self.board_pins);
        if let Err(e) = self
            .chip_canvas
            .sender()
            .send(ChipCanvasInput::UpdateLockedPins(locked_pin_keys))
        {
            log::error!(
                "Не удалось отправить UpdateLockedPins в компонент ChipCanvas: {:?}",
                e
            );
        }
    }

    /// Очищает выбранный пин в форме, странице и canvas.
    fn clear_selection(&mut self) {
        self.selected_pin = None;
        self.send_form_input(PinModeFormInput::ClearSelection);

        if let Err(e) = self
            .chip_canvas
            .sender()
            .send(ChipCanvasInput::ClearSelection)
        {
            log::error!(
                "Не удалось отправить ClearSelection в компонент ChipCanvas: {:?}",
                e
            );
        }
    }

    /// Передаёт сообщение дочерней форме настройки GPIO-пина.
    fn send_form_input(&self, input: PinModeFormInput) {
        if let Err(e) = self.form.sender().send(input) {
            log::error!("Не удалось отправить сообщение в PinModeFormModel: {:?}", e);
        }
    }

    /// Передаёт ошибку в форму настройки GPIO-пина и пишет её в лог.
    fn set_form_error(&self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки GPIO-пина: {}", message);
        self.send_form_input(PinModeFormInput::SetError(message));
    }
}
