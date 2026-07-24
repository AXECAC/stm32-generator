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
use std::sync::{Arc, RwLock};

/// Модель страницы настройки пинов (GPIO).
///
/// Отвечает за хранение локального состояния конфигуратора:
/// текущего выбранного на холсте пина, его базового режима работы, текстового алиаса,
/// а также управляет динамически формируемым списком дополнительных свойств.
pub struct PinsPageModel {
    pub config: Arc<RwLock<Config>>,
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
    UpdateConfig,
    PinSelected(String),

    AliasChanged(String),
    ApplyPinConfig,
    PinTypeChanged(usize),
    PropertyChanged(usize, usize),
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

                #[name = "error_label"]
                gtk::Label {
                    #[watch]
                    set_text: model.error_message.as_deref().unwrap_or(""),
                    #[watch]
                    set_visible: model.error_message.is_some(),
                    add_css_class: "error",
                    set_wrap: true,
                },

                #[name = "settings_group"]
                adw::PreferencesGroup {
                    set_title: "Настройка пина",
                    set_width_request: 340,
                    #[watch]
                    set_visible: model.selected_pin.is_some(),

                    #[name = "selected_pin_row"]
                    adw::ActionRow {
                        set_title: "Выбранный пин",
                        #[watch]
                        set_subtitle: model.selected_pin.as_ref().map(|p| p.label.as_str()).unwrap_or(""),
                    },

                    #[name = "pin_type_row"]
                    adw::ComboRow {
                        set_title: "Режим пина",
                        #[watch]
                        set_selected: match &model.current_mode {
                            None => 0,
                            Some(m) => (m.current_mode_index() + 1) as u32,
                        },
                        set_model: Some(&model.pin_type_model),
                        connect_selected_notify[sender] => move |row| {
                            let idx = row.selected() as usize;
                            sender.input(PinsPageInput::PinTypeChanged(idx));
                        }
                    },

                    #[name = "pin_alias_row"]
                    adw::ActionRow {
                        set_title: "Имя переменной",
                        #[watch]
                        set_visible: model.current_mode.is_some(),
                        add_suffix = &gtk::Entry {
                            set_buffer: &model.alias_buffer,
                            set_max_length: 25,
                            set_placeholder_text: Some("pin_name"),
                            set_valign: gtk::Align::Center,
                            connect_changed[sender] => move |entry| {
                                sender.input(PinsPageInput::AliasChanged(entry.text().to_string()));
                            },
                            connect_activate[sender] => move |_| {
                                sender.input(PinsPageInput::ApplyPinConfig);
                            }
                        }
                    },
                },

                #[local_ref]
                dynamic_group_widget -> adw::PreferencesGroup {
                    set_title: "Дополнительные свойства",
                    #[watch]
                    set_visible: model.current_mode.is_some(),
                },

                gtk::Button {
                    set_label: "Подтвердить",
                    set_margin_top: 16,
                    add_css_class: "suggested-action",
                    #[watch]
                    set_visible: model.selected_pin.is_some(),
                    connect_clicked[sender] => move |_| {
                        sender.input(PinsPageInput::ApplyPinConfig);
                    }
                }
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
        let pins_data = Self::build_pins_with_aliases(&board_pins, &init.read().unwrap());
        let chip_canvas = ChipCanvasModel::builder()
            .launch((init.read().unwrap().board.chip_label().to_string(), pins_data))
            .forward(sender.input_sender(), |output| match output {
                ChipCanvasOutput::PinSelected(key) => PinsPageInput::PinSelected(key),
            });

        let dynamic_properties = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .forward(sender.input_sender(), |output| match output {
                PropertyRowOutput::SelectionChanged(p, v) => PinsPageInput::PropertyChanged(p, v),
            });

        let mut model = PinsPageModel {
            config: init,
            board_pins,
            selected_pin: None,
            current_alias: String::new(),
            current_mode: None,
            pin_type_model: gtk::StringList::new(&[]),
            alias_buffer: gtk::EntryBuffer::new(None::<&str>),
            error_message: None,
            dynamic_properties,
            chip_canvas,
        };

        model.update_dynamic_properties();

        let dynamic_group_widget = model.dynamic_properties.widget();
        let canvas_widget = model.chip_canvas.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PinsPageInput::UpdateConfig => {
                self.error_message = None;
                let config = self.config.read().unwrap();
                self.board_pins = config.board.build_pins();
                let pins_data = Self::build_pins_with_aliases(&self.board_pins, &config);
                if let Err(e) = self
                    .chip_canvas
                    .sender()
                    .send(ChipCanvasInput::UpdatePins(pins_data))
                {
                    log::error!(
                        "Не удалось отправить UpdatePins в компонент ChipCanvas: {:?}",
                        e
                    );
                }
            }
            PinsPageInput::PinSelected(key) => {
                self.error_message = None;
                if let Some(pin) = self.board_pins.iter().find(|p| p.key == key) {
                    self.selected_pin = Some(pin.clone());

                    self.current_mode = None;
                    self.current_alias.clear();
                    self.alias_buffer.set_text("");

                    if let PinType::Gpio(chosen_pin) = pin.pin_type {
                        let mut v = vec!["Not Configured"];
                        v.extend(chosen_pin.default_mode().mode_variants());
                        self.pin_type_model
                            .splice(0, self.pin_type_model.n_items(), v.as_slice());

                        if let Some(pin_cfg) = self
                            .config
                            .read()
                            .unwrap()
                            .gpio()
                            .iter()
                            .find(|p| p.pin.pin() == chosen_pin)
                        {
                            if let Some(label) = &pin_cfg.label {
                                self.current_alias = label.clone();
                                self.alias_buffer.set_text(label);
                            }

                            self.current_mode = Some(pin_cfg.pin);
                        }
                    }
                    self.update_dynamic_properties();
                }
            }
            PinsPageInput::AliasChanged(alias) => {
                self.current_alias = alias;
            }
            PinsPageInput::ApplyPinConfig => {
                self.rebuild_and_emit_config(&sender);
            }
            PinsPageInput::PinTypeChanged(idx) => {
                if idx == 0 {
                    self.current_mode = None;
                } else {
                    if let Some(pin) = &self.selected_pin
                        && let PinType::Gpio(chosen_pin) = pin.pin_type
                    {
                        let mut mode = self
                            .current_mode
                            .unwrap_or_else(|| chosen_pin.default_mode());
                        mode.set_mode_index(idx - 1);
                        self.current_mode = Some(mode);
                    }
                }
                self.update_dynamic_properties();
            }
            PinsPageInput::PropertyChanged(prop_idx, variant_idx) => {
                if let Some(ref mut mode) = self.current_mode {
                    mode.set_property(prop_idx, variant_idx);
                }
            }
        }
    }
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
    fn rebuild_and_emit_config(&mut self, _sender: &ComponentSender<Self>) {
        if let Some(pin) = &self.selected_pin
            && let PinType::Gpio(chosen_pin) = pin.pin_type
        {
            // Сначала удаляем старый конфиг для этого пина
            {
                let mut config = self.config.write().unwrap();
                config.remove_gpio_pin(&chosen_pin);

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
                    if let Err(err) = config.add_gpio_pin(new_pin_config) {
                        self.error_message = Some(err.to_string());
                        return;
                    }
                }
            }

            self.error_message = None;

            // Локально обновляем холст чипа (не ждём, пока таб переключится обратно)
            {
                let config = self.config.read().unwrap();
                let pins_data = Self::build_pins_with_aliases(&self.board_pins, &config);
                if let Err(e) = self.chip_canvas.sender().send(ChipCanvasInput::UpdatePins(pins_data)) {
                    log::error!("Не удалось отправить UpdatePins в ChipCanvas: {:?}", e);
                }
            }

            // Очищаем выбор, чтобы панель скрылась, а пин вернул свой обычный цвет (зеленый если настроен)
            self.selected_pin = None;
            self.current_mode = None;
            self.current_alias.clear();
            self.alias_buffer.set_text("");
            self.update_dynamic_properties();

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
    }
}
