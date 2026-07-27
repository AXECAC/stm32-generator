//! Компонент формы настройки SPI-шины.

use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};
use strum::VariantNames;

use crate::core::config::{SpiConfig, SpiMode};
use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::gui::components::forms::{ComboField, EntryField};
use crate::gui::utils::{default_distinct_pin_index, mode_from_index};

/// Модель компонента формы настройки SPI.
///
/// Компонент хранит локальное состояние формы, защищает программные обновления
/// списков от echo-событий GTK и отдаёт родителю уже собранный [`SpiConfig`].
pub(crate) struct SpiFormModel {
    /// Доступные SPI-шины выбранного MCU, которые ещё не добавлены в конфигурацию.
    bus: ComboField<ChosenSpiBus>,
    /// Выбранный режим SPI.
    mode: ComboField<SpiMode>,
    /// Частота SPI в МГц.
    frequency: EntryField,
    /// Свободные GPIO-пины для SCK.
    sck: ComboField<ChosenPin>,
    /// Флаг использования линии MISO.
    use_miso: bool,
    /// Свободные GPIO-пины для MISO.
    miso: ComboField<ChosenPin>,
    /// Флаг использования линии MOSI.
    use_mosi: bool,
    /// Свободные GPIO-пины для MOSI.
    mosi: ComboField<ChosenPin>,
    /// Сообщение об ошибке формы.
    error: Option<String>,
    /// Guard для программного обновления GTK-моделей.
    refresh_guard: Rc<Cell<bool>>,
}

/// Входящие сообщения компонента формы SPI.
#[derive(Debug)]
pub(crate) enum SpiFormInput {
    /// Обновить доступные SPI-шины и свободные GPIO-пины.
    UpdateOptions {
        /// SPI-шины, которые можно добавить.
        buses: Vec<ChosenSpiBus>,
        /// Свободные GPIO-пины для линий SPI.
        pins: Vec<ChosenPin>,
    },
    /// Пользователь выбрал SPI-шину по индексу.
    BusSelected(usize),
    /// Пользователь изменил текст частоты.
    FrequencyChanged(String),
    /// Пользователь выбрал режим SPI по индексу.
    ModeSelected(usize),
    /// Пользователь выбрал SCK по индексу.
    SckSelected(usize),
    /// Пользователь включил или выключил линию MISO.
    UseMisoToggled(bool),
    /// Пользователь выбрал MISO по индексу.
    MisoSelected(usize),
    /// Пользователь включил или выключил линию MOSI.
    UseMosiToggled(bool),
    /// Пользователь выбрал MOSI по индексу.
    MosiSelected(usize),
    /// Пользователь запросил сборку и отправку формы.
    Submit,
    /// Отобразить ошибку, полученную снаружи компонента.
    SetError(String),
    /// Очистить текущую ошибку.
    ClearError,
    /// Сбросить форму после изменения глобальной конфигурации.
    ResetAfterChange,
}

/// Исходящие сообщения компонента формы SPI.
#[derive(Debug)]
pub(crate) enum SpiFormOutput {
    /// Форма успешно собрана и готова к добавлению в глобальный [`crate::core::config::Config`].
    Submit(SpiConfig),
}

#[relm4::component(pub(crate))]
impl SimpleComponent for SpiFormModel {
    type Init = ();
    type Input = SpiFormInput;
    type Output = SpiFormOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,

            adw::PreferencesGroup {
                set_title: "Добавить шину SPI",
                set_description: Some("Настройте параметры шины и выберите свободные пины."),

                adw::ComboRow {
                    set_title: "Шина",
                    set_model: Some(&model.bus.model),
                    #[watch]
                    set_selected: model.bus.selected_idx as u32,
                    #[watch]
                    set_sensitive: !model.bus.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(SpiFormInput::BusSelected(row.selected() as usize));
                    }
                },

                adw::ComboRow {
                    set_title: "Режим",
                    set_subtitle: "CPOL / CPHA",
                    set_model: Some(&model.mode.model),
                    #[watch]
                    set_selected: model.mode.selected_idx as u32,

                    connect_selected_notify[sender] => move |row| {
                        sender.input(SpiFormInput::ModeSelected(row.selected() as usize));
                    }
                },

                adw::ActionRow {
                    set_title: "Частота (МГц)",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.frequency.buffer,
                        set_width_chars: 8,
                        set_max_width_chars: 8,
                        set_input_purpose: gtk::InputPurpose::Digits,
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(SpiFormInput::FrequencyChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(SpiFormInput::Submit);
                        }
                    }
                },

                adw::ComboRow {
                    set_title: "SCK",
                    set_model: Some(&model.sck.model),
                    #[watch]
                    set_selected: model.sck.selected_idx as u32,
                    #[watch]
                    set_sensitive: !model.sck.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(SpiFormInput::SckSelected(row.selected() as usize));
                    }
                },

                adw::ActionRow {
                    set_title: "Включить MISO",

                    add_suffix = &gtk::Switch {
                        #[watch]
                        set_active: model.use_miso,
                        set_valign: gtk::Align::Center,

                        connect_active_notify[sender] => move |switch| {
                            sender.input(SpiFormInput::UseMisoToggled(switch.is_active()));
                        }
                    }
                },

                adw::ComboRow {
                    set_title: "MISO",
                    set_model: Some(&model.miso.model),
                    #[watch]
                    set_selected: model.miso.selected_idx as u32,
                    #[watch]
                    set_visible: model.use_miso,
                    #[watch]
                    set_sensitive: !model.miso.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(SpiFormInput::MisoSelected(row.selected() as usize));
                    }
                },

                adw::ActionRow {
                    set_title: "Включить MOSI",

                    add_suffix = &gtk::Switch {
                        #[watch]
                        set_active: model.use_mosi,
                        set_valign: gtk::Align::Center,

                        connect_active_notify[sender] => move |switch| {
                            sender.input(SpiFormInput::UseMosiToggled(switch.is_active()));
                        }
                    }
                },

                adw::ComboRow {
                    set_title: "MOSI",
                    set_model: Some(&model.mosi.model),
                    #[watch]
                    set_selected: model.mosi.selected_idx as u32,
                    #[watch]
                    set_visible: model.use_mosi,
                    #[watch]
                    set_sensitive: !model.mosi.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(SpiFormInput::MosiSelected(row.selected() as usize));
                    }
                }
            },

            gtk::Label {
                #[watch]
                set_label: model.error.as_deref().unwrap_or(""),
                #[watch]
                set_visible: model.error.is_some(),
                add_css_class: "error",
                set_wrap: true,
                set_xalign: 0.0,
            },

            gtk::Button {
                set_label: "Добавить шину SPI",
                add_css_class: "suggested-action",
                set_halign: gtk::Align::Start,
                #[watch]
                set_sensitive: model.can_submit(),

                connect_clicked[sender] => move |_| {
                    sender.input(SpiFormInput::Submit);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = SpiFormModel::new();
        model.reset_after_change();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SpiFormInput::UpdateOptions { buses, pins } => self.update_options(buses, pins),
            SpiFormInput::BusSelected(idx) => {
                if self.bus.selected_idx == idx {
                    return;
                }
                self.bus.selected_idx = idx;
            }
            SpiFormInput::FrequencyChanged(frequency) => self.frequency.set_value(frequency),
            SpiFormInput::ModeSelected(idx) => {
                if self.mode.selected_idx == idx {
                    return;
                }
                self.mode.selected_idx = idx;
            }
            SpiFormInput::SckSelected(idx) => {
                if self.sck.selected_idx == idx {
                    return;
                }
                self.sck.selected_idx = idx;
            }
            SpiFormInput::UseMisoToggled(active) => {
                if self.use_miso == active {
                    return;
                }
                self.use_miso = active;
            }
            SpiFormInput::MisoSelected(idx) => {
                if self.miso.selected_idx == idx {
                    return;
                }
                self.miso.selected_idx = idx;
            }
            SpiFormInput::UseMosiToggled(active) => {
                if self.use_mosi == active {
                    return;
                }
                self.use_mosi = active;
            }
            SpiFormInput::MosiSelected(idx) => {
                if self.mosi.selected_idx == idx {
                    return;
                }
                self.mosi.selected_idx = idx;
            }
            SpiFormInput::Submit => self.submit(sender),
            SpiFormInput::SetError(message) => self.error = Some(message),
            SpiFormInput::ClearError => self.error = None,
            SpiFormInput::ResetAfterChange => self.reset_after_change(),
        }
    }
}

impl SpiFormModel {
    /// Создаёт форму SPI со значениями по умолчанию.
    fn new() -> Self {
        Self {
            bus: ComboField::empty(),
            mode: ComboField::new(Self::spi_modes(), SpiMode::VARIANTS),
            frequency: EntryField::new("10"),
            sck: ComboField::empty(),
            use_miso: true,
            miso: ComboField::empty(),
            use_mosi: true,
            mosi: ComboField::empty(),
            error: None,
            refresh_guard: Rc::new(Cell::new(false)),
        }
    }

    /// Возвращает варианты [`SpiMode`] в порядке, заданном core enum и `strum::VariantNames`.
    fn spi_modes() -> Vec<SpiMode> {
        (0..SpiMode::VARIANTS.len())
            .filter_map(|idx| SpiMode::from_repr(idx as u8))
            .collect()
    }

    /// Обновляет списки доступных SPI-шин и GPIO-пинов.
    fn update_options(&mut self, buses: Vec<ChosenSpiBus>, pins: Vec<ChosenPin>) {
        self.refresh_guard.set(true);

        let bus_names = buses
            .iter()
            .map(|bus| bus.variant_name())
            .collect::<Vec<_>>();
        self.bus.replace_items(buses, &bus_names);

        let pin_names = pins
            .iter()
            .map(|pin| pin.variant_name())
            .collect::<Vec<_>>();
        self.sck.replace_items(pins.clone(), &pin_names);
        self.miso.replace_items(pins.clone(), &pin_names);
        self.mosi.replace_items(pins, &pin_names);

        self.clamp_indexes();
        self.refresh_guard.set(false);
    }

    /// Возвращает, можно ли отправлять текущую форму SPI.
    fn can_submit(&self) -> bool {
        !self.bus.is_empty() && !self.sck.is_empty()
    }

    /// Обрабатывает отправку формы.
    fn submit(&mut self, sender: ComponentSender<Self>) {
        let spi = match self.build_config() {
            Ok(spi) => spi,
            Err(e) => {
                self.set_error(e);
                return;
            }
        };

        self.error = None;
        if let Err(e) = sender.output(SpiFormOutput::Submit(spi)) {
            log::error!("Не удалось отправить Submit из SpiFormModel: {:?}", e);
        }
    }

    /// Собирает [`SpiConfig`] из формы.
    fn build_config(&self) -> Result<SpiConfig, String> {
        let frequency_mhz = match self.frequency.value.trim().parse::<u32>() {
            Ok(frequency_mhz) if frequency_mhz > 0 => frequency_mhz,
            _ => return Err("Частота SPI должна быть положительным числом".to_string()),
        };

        let Some(bus) = self.bus.selected() else {
            return Err("Нет доступных SPI-шин для добавления".to_string());
        };

        let Some(sck) = self.sck.selected() else {
            return Err("Выберите SCK из списка свободных пинов".to_string());
        };

        let mode = mode_from_index(self.mode.selected_idx);
        let miso = if self.use_miso {
            match self.miso.selected() {
                Some(pin) => Some(pin),
                None => return Err("Выберите MISO из списка свободных пинов".to_string()),
            }
        } else {
            None
        };
        let mosi = if self.use_mosi {
            match self.mosi.selected() {
                Some(pin) => Some(pin),
                None => return Err("Выберите MOSI из списка свободных пинов".to_string()),
            }
        } else {
            None
        };

        SpiConfig::new(bus, frequency_mhz, mode, sck, miso, mosi).map_err(|e| e.to_string())
    }

    /// Сбрасывает форму добавления SPI в безопасные значения по умолчанию.
    fn reset_after_change(&mut self) {
        self.bus.reset_selected(0);
        self.mode.reset_selected(0);
        self.sck.reset_selected(0);
        self.miso
            .reset_selected(default_distinct_pin_index(1, self.sck.len()));
        self.mosi
            .reset_selected(default_distinct_pin_index(2, self.sck.len()));
        self.use_miso = true;
        self.use_mosi = true;
        self.frequency.set_text("10");
    }

    /// Ограничивает индексы формы актуальными размерами списков.
    fn clamp_indexes(&mut self) {
        self.bus.clamp_selected();
        self.mode.clamp_selected();
        self.sck.clamp_selected();
        self.miso.clamp_selected();
        self.mosi.clamp_selected();
    }

    /// Сохраняет локальную ошибку формы для UI и пишет её в лог.
    fn set_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки SPI: {}", message);
        self.error = Some(message);
    }
}
