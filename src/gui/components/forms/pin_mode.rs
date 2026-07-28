//! Компонент формы настройки GPIO-пина.

use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};

use crate::core::board::{Pin, PinType};
use crate::core::config::PinConfig;
use crate::core::gpio::{ChosenPinWithMode, PinModeUiInfo};
use crate::gui::components::forms::{ComboField, EntryField};
use crate::gui::components::property_row::{PropertyRowModel, PropertyRowOutput};

/// Доменное значение выбранного режима GPIO-пина в форме.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PinMode {
    /// Пин не будет сконфигурирован как GPIO.
    NotConfigured,
    /// Пин будет сконфигурирован в выбранном режиме.
    Configured(ChosenPinWithMode),
}

impl PinMode {
    /// Возвращает сконфигурированный режим пина, если он выбран.
    fn configured(self) -> Option<ChosenPinWithMode> {
        match self {
            Self::NotConfigured => None,
            Self::Configured(mode) => Some(mode),
        }
    }

    /// Возвращает дополнительные свойства выбранного режима.
    fn properties(self) -> Vec<(&'static str, Vec<&'static str>, usize)> {
        match self {
            Self::NotConfigured => Vec::new(),
            Self::Configured(mode) => mode.properties(),
        }
    }

    /// Обновляет дополнительное свойство выбранного режима.
    fn set_property(&mut self, prop_idx: usize, variant_idx: usize) {
        if let Self::Configured(mode) = self {
            mode.set_property(prop_idx, variant_idx);
        }
    }
}

/// Данные, которые форма отдаёт странице для применения к глобальному [`crate::core::config::Config`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PinModeFormConfig {
    /// Выбранный режим GPIO или `None`, если пин нужно оставить не сконфигурированным.
    pub(crate) mode: Option<ChosenPinWithMode>,
    /// Пользовательский alias пина.
    pub(crate) alias: Option<String>,
}

/// Модель компонента формы настройки GPIO-пина.
pub(crate) struct PinModeFormModel {
    /// Пин, выбранный пользователем на canvas.
    selected_pin: Option<Pin>,
    /// Список режимов пина.
    mode: ComboField<PinMode>,
    /// Alias выбранного пина.
    alias: EntryField,
    /// Опциональное сообщение об ошибке.
    error: Option<String>,
    /// Guard для программного обновления GTK-моделей.
    refresh_guard: Rc<Cell<bool>>,
    /// Динамический список дополнительных свойств выбранного режима.
    properties: FactoryVecDeque<PropertyRowModel>,
}

/// Входящие сообщения формы настройки GPIO-пина.
#[derive(Debug)]
pub(crate) enum PinModeFormInput {
    /// Выбран новый пин и его существующая GPIO-конфигурация.
    SelectPin {
        /// Пин с платы.
        pin: Pin,
        /// Текущая GPIO-конфигурация этого пина, если она есть.
        config: Option<PinConfig>,
    },
    /// Очистить выбранный пин и скрыть форму.
    ClearSelection,
    /// Пользователь изменил alias.
    AliasChanged(String),
    /// Пользователь выбрал режим пина.
    ModeSelected(usize),
    /// Пользователь изменил дополнительное свойство режима.
    PropertyChanged(usize, usize),
    /// Пользователь запросил применение формы.
    Apply,
    /// Отобразить ошибку, полученную снаружи компонента.
    SetError(String),
    /// Очистить текущую ошибку.
    ClearError,
}

/// Исходящие сообщения формы настройки GPIO-пина.
#[derive(Debug)]
pub(crate) enum PinModeFormOutput {
    /// Форма собрана и готова к применению к выбранному пину.
    Apply(PinModeFormConfig),
}

#[relm4::component(pub(crate))]
impl SimpleComponent for PinModeFormModel {
    type Init = ();
    type Input = PinModeFormInput;
    type Output = PinModeFormOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,
            #[watch]
            set_visible: model.selected_pin.is_some(),

            gtk::Label {
                #[watch]
                set_text: model.error.as_deref().unwrap_or(""),
                #[watch]
                set_visible: model.error.is_some(),
                add_css_class: "error",
                set_wrap: true,
            },

            adw::PreferencesGroup {
                set_title: "Настройка пина",
                set_width_request: 340,

                adw::ActionRow {
                    set_title: "Выбранный пин",
                    #[watch]
                    set_subtitle: model.selected_pin.as_ref().map(|p| p.label.as_str()).unwrap_or(""),
                },

                adw::ComboRow {
                    set_title: "Режим пина",
                    set_model: Some(&model.mode.model),
                    #[watch]
                    set_selected: model.mode.selected_idx as u32,

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(PinModeFormInput::ModeSelected(row.selected() as usize));
                    }
                },

                adw::ActionRow {
                    set_title: "Имя переменной",
                    #[watch]
                    set_visible: model.selected_mode().is_some(),

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.alias.buffer,
                        set_max_length: 25,
                        set_placeholder_text: Some("pin_name"),
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(PinModeFormInput::AliasChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(PinModeFormInput::Apply);
                        }
                    }
                }
            },

            #[local_ref]
            properties_group_widget -> adw::PreferencesGroup {
                set_title: "Дополнительные свойства",
                #[watch]
                set_visible: model.selected_mode().is_some(),
            },

            gtk::Button {
                set_label: "Подтвердить",
                set_margin_top: 16,
                add_css_class: "suggested-action",

                connect_clicked[sender] => move |_| {
                    sender.input(PinModeFormInput::Apply);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let properties = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .forward(sender.input_sender(), |output| match output {
                PropertyRowOutput::SelectionChanged(prop_idx, variant_idx) => {
                    PinModeFormInput::PropertyChanged(prop_idx, variant_idx)
                }
            });

        let mut model = PinModeFormModel {
            selected_pin: None,
            mode: ComboField::empty(),
            alias: EntryField::new(""),
            error: None,
            refresh_guard: Rc::new(Cell::new(false)),
            properties,
        };

        model.refresh_properties();

        let properties_group_widget = model.properties.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PinModeFormInput::SelectPin { pin, config } => self.select_pin(pin, config),
            PinModeFormInput::ClearSelection => self.clear_selection(),
            PinModeFormInput::AliasChanged(alias) => self.alias.set_value(alias),
            PinModeFormInput::ModeSelected(idx) => self.select_mode(idx),
            PinModeFormInput::PropertyChanged(prop_idx, variant_idx) => {
                self.set_property(prop_idx, variant_idx);
            }
            PinModeFormInput::Apply => self.apply(sender),
            PinModeFormInput::SetError(message) => self.error = Some(message),
            PinModeFormInput::ClearError => self.error = None,
        }
    }
}

impl PinModeFormModel {
    /// Выбирает новый пин и синхронизирует форму с его текущей GPIO-конфигурацией.
    fn select_pin(&mut self, pin: Pin, config: Option<PinConfig>) {
        self.error = None;
        self.selected_pin = Some(pin.clone());
        self.rebuild_mode_options(&pin, config.as_ref());

        if let Some(config) = config {
            self.alias.set_text(config.label.as_deref().unwrap_or(""));
        } else {
            self.alias.set_text("");
        }

        self.refresh_properties();
    }

    /// Очищает выбранный пин и скрывает форму.
    fn clear_selection(&mut self) {
        self.selected_pin = None;
        self.refresh_guard.set(true);
        self.mode.replace_items(Vec::new(), &[]);
        self.refresh_guard.set(false);
        self.alias.set_text("");
        self.error = None;
        self.refresh_properties();
    }

    /// Пересобирает варианты режима для выбранного GPIO-пина.
    fn rebuild_mode_options(&mut self, pin: &Pin, config: Option<&PinConfig>) {
        let PinType::Gpio(chosen_pin) = pin.pin_type else {
            self.refresh_guard.set(true);
            self.mode.replace_items(Vec::new(), &[]);
            self.refresh_guard.set(false);
            return;
        };

        let default_mode = chosen_pin.default_mode();
        let mut items = vec![PinMode::NotConfigured];
        let mut labels = vec!["Not Configured"];
        let variants = default_mode.mode_variants();

        for (idx, label) in variants.into_iter().enumerate() {
            let mut mode = default_mode;
            mode.set_mode_index(idx);
            items.push(PinMode::Configured(mode));
            labels.push(label);
        }

        let selected_idx = config
            .map(|config| config.pin.current_mode_index() + 1)
            .unwrap_or(0);
        if let Some(config) = config
            && let Some(item) = items.get_mut(selected_idx)
        {
            *item = PinMode::Configured(config.pin);
        }

        self.refresh_guard.set(true);
        self.mode.replace_items(items, &labels);
        self.mode.reset_selected(selected_idx);
        self.refresh_guard.set(false);
    }

    /// Выбирает режим по индексу из `ComboRow`.
    fn select_mode(&mut self, idx: usize) {
        if self.mode.selected_idx == idx {
            return;
        }
        self.mode.selected_idx = idx;
        self.refresh_properties();
    }

    /// Возвращает текущий сконфигурированный режим пина.
    fn selected_mode(&self) -> Option<ChosenPinWithMode> {
        self.mode.selected().and_then(PinMode::configured)
    }

    /// Обновляет дополнительное свойство выбранного режима.
    fn set_property(&mut self, prop_idx: usize, variant_idx: usize) {
        if let Some(mode) = self.mode.items.get_mut(self.mode.selected_idx) {
            mode.set_property(prop_idx, variant_idx);
            self.refresh_properties();
        }
    }

    /// Синхронизирует factory-список дополнительных свойств.
    fn refresh_properties(&mut self) {
        let mut guard = self.properties.guard();
        guard.clear();

        if let Some(mode) = self.mode.selected() {
            for (idx, (title, variants, selected)) in mode.properties().into_iter().enumerate() {
                guard.push_back((
                    idx,
                    title.to_string(),
                    variants
                        .into_iter()
                        .map(|value| value.to_string())
                        .collect(),
                    selected,
                ));
            }
        }
    }

    /// Отправляет текущие значения формы родителю.
    fn apply(&mut self, sender: ComponentSender<Self>) {
        let alias = self.alias.value.trim();
        let config = PinModeFormConfig {
            mode: self.selected_mode(),
            alias: if alias.is_empty() {
                None
            } else {
                Some(alias.to_string())
            },
        };

        self.error = None;
        if let Err(e) = sender.output(PinModeFormOutput::Apply(config)) {
            log::error!("Не удалось отправить Apply из PinModeFormModel: {:?}", e);
        }
    }
}
