use crate::core::board::PinType;
use crate::core::config::{Config, SpiConfig, SpiMode};
use crate::core::errors::ConfigError;
use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::gui::components::spi_bus_row::{SpiBusRowModel, SpiBusRowOutput};
use adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, adw, gtk};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use strum::VariantNames;

/// Модель страницы настройки SPI-шин.
///
/// Страница хранит только локальное состояние формы и GTK-модели списков.
/// Единым источником истины остаётся глобальный [`Config`], доступный через
/// [`Arc<RwLock<_>>`](Arc). Списки обновляются лениво из `Config` при входе
/// на вкладку и после успешного добавления или удаления SPI-шины.
pub struct SpiPageModel {
    /// Глобальная конфигурация приложения.
    pub(crate) config: Arc<RwLock<Config>>,

    /// SPI-шины выбранного MCU, которые ещё не добавлены в [`Config`].
    available_buses: Vec<ChosenSpiBus>,
    /// GTK-модель для выпадающего списка доступных SPI-шин.
    bus_model: gtk::StringList,
    /// Индекс выбранной шины в [`Self::available_buses`].
    form_bus_idx: usize,

    /// GTK-модель со всеми вариантами [`SpiMode`].
    mode_model: gtk::StringList,
    /// Индекс выбранного режима SPI.
    form_mode_idx: usize,

    /// GTK-буфер поля ввода частоты.
    frequency_buffer: gtk::EntryBuffer,
    /// Текущее текстовое значение частоты в МГц.
    form_frequency: String,

    /// Свободные GPIO-пины, которые ещё не используются конфигурацией.
    available_pins: Vec<ChosenPin>,
    /// GTK-модель для выбора SCK.
    sck_model: gtk::StringList,
    /// GTK-модель для выбора MISO.
    miso_model: gtk::StringList,
    /// GTK-модель для выбора MOSI.
    mosi_model: gtk::StringList,
    /// Индекс выбранного SCK в [`Self::available_pins`].
    form_sck_idx: usize,
    /// Флаг использования линии MISO.
    form_use_miso: bool,
    /// Индекс выбранного MISO в [`Self::available_pins`].
    form_miso_idx: usize,
    /// Флаг использования линии MOSI.
    form_use_mosi: bool,
    /// Индекс выбранного MOSI в [`Self::available_pins`].
    form_mosi_idx: usize,

    /// Сообщение об ошибке формы, отображаемое пользователю.
    form_error: Option<String>,
    /// Флаг программного обновления GTK-моделей.
    ///
    /// `ComboRow` отправляет `notify::selected` даже при программном изменении
    /// списка через `StringList::splice`. Этот guard позволяет игнорировать такие
    /// echo-сигналы и не запускать лишние циклы обновления Relm4.
    refresh_guard: Rc<Cell<bool>>,
    /// Кэш текущего списка SPI-шин для защиты от лишнего пересоздания factory-строк.
    configured_spis_cache: Vec<SpiConfig>,
    /// Factory-модель строк со сконфигурированными SPI-шинами.
    configured_buses: FactoryVecDeque<SpiBusRowModel>,
}

/// Входящие сообщения страницы SPI.
#[derive(Debug)]
pub enum SpiPageInput {
    /// Вкладка стала активной; нужно перечитать свежий [`Config`].
    UpdateConfig,
    /// Пользователь выбрал SPI-шину по индексу в списке доступных шин.
    BusSelected(usize),
    /// Пользователь изменил текст частоты.
    FrequencyChanged(String),
    /// Пользователь выбрал режим SPI по индексу.
    ModeSelected(usize),
    /// Пользователь выбрал SCK по индексу в списке свободных пинов.
    SckSelected(usize),
    /// Пользователь включил или выключил линию MISO.
    UseMisoToggled(bool),
    /// Пользователь выбрал MISO по индексу в списке свободных пинов.
    MisoSelected(usize),
    /// Пользователь включил или выключил линию MOSI.
    UseMosiToggled(bool),
    /// Пользователь выбрал MOSI по индексу в списке свободных пинов.
    MosiSelected(usize),
    /// Пользователь нажал кнопку добавления SPI-шины.
    AddBus,
    /// Пользователь запросил удаление сконфигурированной SPI-шины.
    RemoveBus(ChosenSpiBus),
}

#[relm4::component(pub)]
impl SimpleComponent for SpiPageModel {
    /// Данные, необходимые для инициализации страницы.
    type Init = Arc<RwLock<Config>>;
    /// Сообщения, которые страница принимает от своих GTK-виджетов и дочерних компонентов.
    type Input = SpiPageInput;
    /// Страница не отправляет события наружу: все изменения пишутся прямо в общий [`Config`].
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,

            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,
                    set_margin_all: 24,

                    adw::PreferencesGroup {
                        set_title: "Добавить шину SPI",
                        set_description: Some("Настройте параметры шины и выберите свободные пины."),

                        adw::ComboRow {
                            set_title: "Шина",
                            set_model: Some(&model.bus_model),
                            #[watch]
                            set_selected: model.form_bus_idx as u32,
                            #[watch]
                            set_sensitive: !model.available_buses.is_empty(),

                            connect_selected_notify[
                                sender,
                                refresh_guard = model.refresh_guard.clone()
                            ] => move |row| {
                                if refresh_guard.get() {
                                    return;
                                }
                                sender.input(SpiPageInput::BusSelected(row.selected() as usize));
                            }
                        },

                        adw::ComboRow {
                            set_title: "Режим",
                            set_subtitle: "CPOL / CPHA",
                            set_model: Some(&model.mode_model),
                            #[watch]
                            set_selected: model.form_mode_idx as u32,

                            connect_selected_notify[
                                sender,
                                refresh_guard = model.refresh_guard.clone()
                            ] => move |row| {
                                if refresh_guard.get() {
                                    return;
                                }
                                sender.input(SpiPageInput::ModeSelected(row.selected() as usize));
                            }
                        },

                        adw::ActionRow {
                            set_title: "Частота (МГц)",

                            add_suffix = &gtk::Entry {
                                set_buffer: &model.frequency_buffer,
                                set_width_chars: 8,
                                set_max_width_chars: 8,
                                set_input_purpose: gtk::InputPurpose::Digits,
                                set_valign: gtk::Align::Center,

                                connect_changed[sender] => move |entry| {
                                    sender.input(SpiPageInput::FrequencyChanged(entry.text().to_string()));
                                },

                                connect_activate[sender] => move |_| {
                                    sender.input(SpiPageInput::AddBus);
                                }
                            }
                        },

                        adw::ComboRow {
                            set_title: "SCK",
                            set_model: Some(&model.sck_model),
                            #[watch]
                            set_selected: model.form_sck_idx as u32,
                            #[watch]
                            set_sensitive: !model.available_pins.is_empty(),

                            connect_selected_notify[
                                sender,
                                refresh_guard = model.refresh_guard.clone()
                            ] => move |row| {
                                if refresh_guard.get() {
                                    return;
                                }
                                sender.input(SpiPageInput::SckSelected(row.selected() as usize));
                            }
                        },

                        adw::ActionRow {
                            set_title: "Включить MISO",

                            add_suffix = &gtk::Switch {
                                #[watch]
                                set_active: model.form_use_miso,
                                set_valign: gtk::Align::Center,

                                connect_active_notify[sender] => move |switch| {
                                    sender.input(SpiPageInput::UseMisoToggled(switch.is_active()));
                                }
                            }
                        },

                        adw::ComboRow {
                            set_title: "MISO",
                            set_model: Some(&model.miso_model),
                            #[watch]
                            set_selected: model.form_miso_idx as u32,
                            #[watch]
                            set_visible: model.form_use_miso,
                            #[watch]
                            set_sensitive: !model.available_pins.is_empty(),

                            connect_selected_notify[
                                sender,
                                refresh_guard = model.refresh_guard.clone()
                            ] => move |row| {
                                if refresh_guard.get() {
                                    return;
                                }
                                sender.input(SpiPageInput::MisoSelected(row.selected() as usize));
                            }
                        },

                        adw::ActionRow {
                            set_title: "Включить MOSI",

                            add_suffix = &gtk::Switch {
                                #[watch]
                                set_active: model.form_use_mosi,
                                set_valign: gtk::Align::Center,

                                connect_active_notify[sender] => move |switch| {
                                    sender.input(SpiPageInput::UseMosiToggled(switch.is_active()));
                                }
                            }
                        },

                        adw::ComboRow {
                            set_title: "MOSI",
                            set_model: Some(&model.mosi_model),
                            #[watch]
                            set_selected: model.form_mosi_idx as u32,
                            #[watch]
                            set_visible: model.form_use_mosi,
                            #[watch]
                            set_sensitive: !model.available_pins.is_empty(),

                            connect_selected_notify[
                                sender,
                                refresh_guard = model.refresh_guard.clone()
                            ] => move |row| {
                                if refresh_guard.get() {
                                    return;
                                }
                                sender.input(SpiPageInput::MosiSelected(row.selected() as usize));
                            }
                        }
                    },

                    gtk::Label {
                        #[watch]
                        set_label: model.form_error.as_deref().unwrap_or(""),
                        #[watch]
                        set_visible: model.form_error.is_some(),
                        add_css_class: "error",
                        set_wrap: true,
                        set_xalign: 0.0,
                    },

                    gtk::Button {
                        set_label: "Добавить шину SPI",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_sensitive: !model.available_buses.is_empty() && !model.available_pins.is_empty(),

                        connect_clicked[sender] => move |_| {
                            sender.input(SpiPageInput::AddBus);
                        }
                    },

                    #[local_ref]
                    configured_buses_group -> adw::PreferencesGroup {
                        set_title: "Сконфигурированные шины",
                        set_description: Some("Удалить можно только шины, которые не используются периферией."),
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let configured_buses = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .forward(sender.input_sender(), |output| match output {
                SpiBusRowOutput::Remove(bus) => SpiPageInput::RemoveBus(bus),
            });

        let mut model = SpiPageModel {
            config: init,
            available_buses: Vec::new(),
            bus_model: gtk::StringList::new(&[]),
            form_bus_idx: 0,
            mode_model: gtk::StringList::new(SpiMode::VARIANTS),
            form_mode_idx: 0,
            frequency_buffer: gtk::EntryBuffer::new(Some("10")),
            form_frequency: "10".to_string(),
            available_pins: Vec::new(),
            sck_model: gtk::StringList::new(&[]),
            miso_model: gtk::StringList::new(&[]),
            mosi_model: gtk::StringList::new(&[]),
            form_sck_idx: 0,
            form_use_miso: true,
            form_miso_idx: 0,
            form_use_mosi: true,
            form_mosi_idx: 0,
            form_error: None,
            refresh_guard: Rc::new(Cell::new(false)),
            configured_spis_cache: Vec::new(),
            configured_buses,
        };

        let configured_buses_group = model.configured_buses.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SpiPageInput::UpdateConfig => {
            }
            SpiPageInput::BusSelected(idx) => {
            }
            SpiPageInput::FrequencyChanged(frequency) => {
            }
            SpiPageInput::ModeSelected(idx) => {
            }
            SpiPageInput::SckSelected(idx) => {
            }
            SpiPageInput::UseMisoToggled(active) => {
            }
            SpiPageInput::MisoSelected(idx) => {
            }
            SpiPageInput::UseMosiToggled(active) => {
            }
            SpiPageInput::MosiSelected(idx) => {
            }
            SpiPageInput::AddBus => {},
            SpiPageInput::RemoveBus(bus) => {},
        }
    }
}
