//! Компонент формы настройки SPI-шины.

use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};
use strum::VariantNames;

use crate::core::config::{SpiConfig, SpiMode};
use crate::core::gpio::{ChosenSpiBus, SpiMapping};
use crate::gui::components::forms::{ComboField, EntryField};
use crate::gui::utils::mode_from_index;

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
    /// Все mapping, полученные от выбранного MCU.
    all_mappings: Vec<SpiMapping>,
    /// Доступные mapping с учётом выбранной шины и занятых пинов.
    mapping: ComboField<SpiMapping>,
    /// Флаг использования линии MISO.
    use_miso: bool,
    /// Флаг использования линии MOSI.
    use_mosi: bool,
    /// Сообщение об ошибке формы.
    error: Option<String>,
    /// Guard для программного обновления GTK-моделей.
    refresh_guard: Rc<Cell<bool>>,
}

/// Входящие сообщения компонента формы SPI.
#[derive(Debug)]
pub(crate) enum SpiFormInput {
    /// Обновить доступные SPI-шины и совместимые mapping.
    UpdateOptions {
        /// SPI-шины, которые можно добавить.
        buses: Vec<ChosenSpiBus>,
        /// Полные аппаратные mapping доступных шин.
        mappings: Vec<SpiMapping>,
    },
    /// Пользователь выбрал SPI-шину по индексу.
    BusSelected(usize),
    /// Пользователь изменил текст частоты.
    FrequencyChanged(String),
    /// Пользователь выбрал режим SPI по индексу.
    ModeSelected(usize),
    /// Пользователь выбрал готовый SPI mapping.
    MappingSelected(usize),
    /// Пользователь включил или выключил линию MISO.
    UseMisoToggled(bool),
    /// Пользователь включил или выключил линию MOSI.
    UseMosiToggled(bool),
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
                set_description: Some("Настройте параметры шины и выберите совместимую распиновку."),

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
                    set_title: "Распиновка",
                    set_subtitle: "Порядок: SCK / MISO / MOSI",
                    set_model: Some(&model.mapping.model),
                    #[watch]
                    set_selected: model.mapping.selected_idx as u32,
                    #[watch]
                    set_sensitive: !model.mapping.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(SpiFormInput::MappingSelected(row.selected() as usize));
                    }
                },

                adw::ActionRow {
                    set_title: "SCK",
                    #[watch]
                    set_subtitle: model.selected_mapping_sck(),
                },

                adw::ActionRow {
                    set_title: "MISO",
                    #[watch]
                    set_subtitle: model.selected_mapping_miso(),
                },

                adw::ActionRow {
                    set_title: "MOSI",
                    #[watch]
                    set_subtitle: model.selected_mapping_mosi(),
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
            SpiFormInput::UpdateOptions { buses, mappings } => self.update_options(buses, mappings),
            SpiFormInput::BusSelected(idx) => {
                if self.bus.selected_idx == idx {
                    return;
                }
                self.bus.selected_idx = idx;
                self.refresh_mapping_options();
            }
            SpiFormInput::FrequencyChanged(frequency) => self.frequency.set_value(frequency),
            SpiFormInput::ModeSelected(idx) => {
                if self.mode.selected_idx == idx {
                    return;
                }
                self.mode.selected_idx = idx;
            }
            SpiFormInput::MappingSelected(idx) => {
                if self.mapping.selected_idx == idx {
                    return;
                }
                self.mapping.selected_idx = idx;
            }
            SpiFormInput::UseMisoToggled(active) => {
                if self.use_miso == active {
                    return;
                }
                self.use_miso = active;
                self.refresh_mapping_options();
            }
            SpiFormInput::UseMosiToggled(active) => {
                if self.use_mosi == active {
                    return;
                }
                self.use_mosi = active;
                self.refresh_mapping_options();
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
            all_mappings: Vec::new(),
            mapping: ComboField::empty(),
            use_miso: true,
            use_mosi: true,
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

    /// Обновляет списки доступных SPI-шин и совместимых mapping.
    fn update_options(&mut self, buses: Vec<ChosenSpiBus>, mappings: Vec<SpiMapping>) {
        self.refresh_guard.set(true);

        let bus_names = buses
            .iter()
            .map(|bus| bus.variant_name())
            .collect::<Vec<_>>();
        self.bus.replace_items(buses, &bus_names);

        self.all_mappings = mappings;
        self.rebuild_mapping_options();
        self.refresh_guard.set(false);
    }

    /// Возвращает, можно ли отправлять текущую форму SPI.
    fn can_submit(&self) -> bool {
        !self.bus.is_empty() && !self.mapping.is_empty()
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

        let Some(mapping) = self.mapping.selected() else {
            return Err("Выберите совместимую распиновку SPI".to_string());
        };

        let mode = mode_from_index(self.mode.selected_idx);
        let miso = if self.use_miso {
            Some(mapping.miso)
        } else {
            None
        };
        let mosi = if self.use_mosi {
            Some(mapping.mosi)
        } else {
            None
        };

        SpiConfig::new(bus, frequency_mhz, mode, mapping.sck, miso, mosi).map_err(|e| e.to_string())
    }

    /// Сбрасывает форму добавления SPI в безопасные значения по умолчанию.
    fn reset_after_change(&mut self) {
        self.refresh_guard.set(true);
        self.bus.reset_selected(0);
        self.mode.reset_selected(0);
        self.use_miso = true;
        self.use_mosi = true;
        self.frequency.set_text("10");
        self.rebuild_mapping_options();
        self.refresh_guard.set(false);
    }

    /// Перестраивает список mapping после выбора шины или optional-линий.
    fn refresh_mapping_options(&mut self) {
        self.refresh_guard.set(true);
        self.rebuild_mapping_options();
        self.refresh_guard.set(false);
    }

    /// Перестраивает mapping и сбрасывает выбранный индекс в одной защищённой операции.
    fn rebuild_mapping_options(&mut self) {
        let selected_bus = self.bus.selected();
        let mut mappings = Vec::new();
        let mut seen = Vec::new();

        for mapping in self.all_mappings.iter().copied() {
            if Some(mapping.bus) != selected_bus {
                continue;
            }

            let key = (
                mapping.sck,
                self.use_miso.then_some(mapping.miso),
                self.use_mosi.then_some(mapping.mosi),
            );
            if !seen.contains(&key) {
                seen.push(key);
                mappings.push(mapping);
            }
        }

        let labels = mappings
            .iter()
            .map(|mapping| {
                format!(
                    "{} / {} / {}",
                    mapping.sck.variant_name(),
                    mapping.miso.variant_name(),
                    mapping.mosi.variant_name(),
                )
            })
            .collect::<Vec<_>>();
        self.mapping.replace_owned_items(mappings, &labels);
        self.mapping.reset_selected(0);
    }

    /// Возвращает отображаемый SCK выбранного mapping.
    fn selected_mapping_sck(&self) -> &'static str {
        self.mapping
            .selected()
            .map(|mapping| mapping.sck.variant_name())
            .unwrap_or("—")
    }

    /// Возвращает отображаемый MISO выбранного mapping.
    fn selected_mapping_miso(&self) -> &'static str {
        if !self.use_miso {
            return "отключён";
        }

        self.mapping
            .selected()
            .map(|mapping| mapping.miso.variant_name())
            .unwrap_or("—")
    }

    /// Возвращает отображаемый MOSI выбранного mapping.
    fn selected_mapping_mosi(&self) -> &'static str {
        if !self.use_mosi {
            return "отключён";
        }

        self.mapping
            .selected()
            .map(|mapping| mapping.mosi.variant_name())
            .unwrap_or("—")
    }

    /// Сохраняет локальную ошибку формы для UI и пишет её в лог.
    fn set_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки SPI: {}", message);
        self.error = Some(message);
    }
}
