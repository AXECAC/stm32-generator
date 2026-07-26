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

        model.refresh_from_config();
        model.reset_form_after_change();

        let configured_buses_group = model.configured_buses.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    /// Обрабатывает входящие сообщения страницы.
    ///
    /// Для событий выбора используется ранний выход, если состояние модели уже
    /// совпадает с состоянием виджета. Это снижает риск echo-циклов при
    /// реактивных обновлениях GTK.
    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SpiPageInput::UpdateConfig => {
                self.form_error = None;
                self.refresh_from_config();
            }
            SpiPageInput::BusSelected(idx) => {
                if self.form_bus_idx == idx {
                    return;
                }
                self.form_bus_idx = idx;
            }
            SpiPageInput::FrequencyChanged(frequency) => {
                self.form_frequency = frequency;
            }
            SpiPageInput::ModeSelected(idx) => {
                if self.form_mode_idx == idx {
                    return;
                }
                self.form_mode_idx = idx;
            }
            SpiPageInput::SckSelected(idx) => {
                if self.form_sck_idx == idx {
                    return;
                }
                self.form_sck_idx = idx;
            }
            SpiPageInput::UseMisoToggled(active) => {
                if self.form_use_miso == active {
                    return;
                }
                self.form_use_miso = active;
            }
            SpiPageInput::MisoSelected(idx) => {
                if self.form_miso_idx == idx {
                    return;
                }
                self.form_miso_idx = idx;
            }
            SpiPageInput::UseMosiToggled(active) => {
                if self.form_use_mosi == active {
                    return;
                }
                self.form_use_mosi = active;
            }
            SpiPageInput::MosiSelected(idx) => {
                if self.form_mosi_idx == idx {
                    return;
                }
                self.form_mosi_idx = idx;
            }
            SpiPageInput::AddBus => self.add_bus(),
            SpiPageInput::RemoveBus(bus) => self.remove_bus(bus),
        }
    }
}

impl SpiPageModel {
    /// Перечитывает глобальный [`Config`] и синхронизирует локальные списки страницы.
    ///
    /// Метод пересобирает:
    /// - список ещё не использованных SPI-шин;
    /// - список свободных GPIO-пинов;
    /// - список уже сконфигурированных SPI-шин.
    ///
    /// При обновлении GTK-моделей выставляется [`Self::refresh_guard`], чтобы
    /// `notify::selected`, сгенерированные `StringList::splice`, не попали обратно
    /// в [`Self::update`] как пользовательские события.
    fn refresh_from_config(&mut self) {
        let (available_buses, available_pins, configured_spis) = {
            let config = self.config.read().unwrap();

            let configured_buses: Vec<ChosenSpiBus> =
                config.spi().iter().map(|spi| spi.bus).collect();
            let available_buses = config
                .board
                .mcu()
                .all_spi_buses()
                .into_iter()
                .filter(|bus| !configured_buses.contains(bus))
                .collect();

            let used_pins = config.all_uses_pins();
            let available_pins = config
                .board
                .build_pins()
                .into_iter()
                .filter_map(|pin| match pin.pin_type {
                    PinType::Gpio(chosen_pin) if !used_pins.contains(&chosen_pin) => {
                        Some(chosen_pin)
                    }
                    _ => None,
                })
                .collect();

            (available_buses, available_pins, config.spi().to_vec())
        };

        self.available_buses = available_buses;
        self.available_pins = available_pins;

        self.refresh_guard.set(true);

        let bus_names = self
            .available_buses
            .iter()
            .map(|bus| bus.variant_name())
            .collect::<Vec<_>>();
        Self::splice_if_changed(&self.bus_model, &bus_names);

        let pin_names = self
            .available_pins
            .iter()
            .map(|pin| pin.variant_name())
            .collect::<Vec<_>>();
        Self::splice_if_changed(&self.sck_model, &pin_names);
        Self::splice_if_changed(&self.miso_model, &pin_names);
        Self::splice_if_changed(&self.mosi_model, &pin_names);

        self.clamp_form_indexes();
        self.refresh_configured_buses(configured_spis);

        self.refresh_guard.set(false);
    }

    /// Собирает [`SpiConfig`] из формы и добавляет его в глобальный [`Config`].
    ///
    /// Все ошибки валидации пишутся в UI через [`Self::set_form_error`] и в лог
    /// через `log::error!`. После успешного добавления форма сбрасывается, а
    /// локальные списки перечитываются из глобальной конфигурации.
    fn add_bus(&mut self) {
        let frequency_mhz = match self.form_frequency.trim().parse::<u32>() {
            Ok(frequency_mhz) if frequency_mhz > 0 => frequency_mhz,
            _ => {
                self.set_form_error("Частота SPI должна быть положительным числом");
                return;
            }
        };

        let Some(bus) = self.available_buses.get(self.form_bus_idx).copied() else {
            self.set_form_error("Нет доступных SPI-шин для добавления");
            return;
        };

        let Some(sck) = self.available_pins.get(self.form_sck_idx).copied() else {
            self.set_form_error("Выберите SCK из списка свободных пинов");
            return;
        };

        let mode = Self::mode_from_index(self.form_mode_idx);
        let miso = if self.form_use_miso {
            match self.available_pins.get(self.form_miso_idx).copied() {
                Some(pin) => Some(pin),
                None => {
                    self.set_form_error("Выберите MISO из списка свободных пинов");
                    return;
                }
            }
        } else {
            None
        };
        let mosi = if self.form_use_mosi {
            match self.available_pins.get(self.form_mosi_idx).copied() {
                Some(pin) => Some(pin),
                None => {
                    self.set_form_error("Выберите MOSI из списка свободных пинов");
                    return;
                }
            }
        } else {
            None
        };

        let spi = match SpiConfig::new(bus, frequency_mhz, mode, sck, miso, mosi) {
            Ok(spi) => spi,
            Err(e) => {
                self.set_form_error(e.to_string());
                return;
            }
        };

        let add_result = {
            let mut config = self.config.write().unwrap();
            config.add_spi_bus(spi)
        };

        if let Err(e) = add_result {
            self.set_form_error(e.to_string());
            return;
        }

        log::info!(
            "SPI-шина {} успешно добавлена: frequency_mhz={}, mode={:?}, sck={}, miso={:?}, mosi={:?}",
            bus.variant_name(),
            frequency_mhz,
            mode,
            sck.variant_name(),
            miso.map(|pin| pin.variant_name()),
            mosi.map(|pin| pin.variant_name()),
        );
        self.form_error = None;
        self.refresh_from_config();
        self.reset_form_after_change();
    }

    /// Удаляет SPI-шину из глобального [`Config`].
    ///
    /// Если шина используется периферией, удаление блокируется core-валидацией,
    /// ошибка отображается в форме и дополнительно пишется в лог.
    fn remove_bus(&mut self, bus: ChosenSpiBus) {
        let remove_result = {
            let mut config = self.config.write().unwrap();
            config.remove_spi(&bus)
        };

        match remove_result {
            Ok(Some(_)) => {}
            Ok(None) => {
                self.set_form_error(format!(
                    "SPI-шина {} не найдена в конфигурации",
                    bus.variant_name()
                ));
                return;
            }
            Err(ConfigError::SpiBusInUse(_)) => {
                self.set_form_error(
                    "Шина используется периферией. Сначала удалите связанную периферию.",
                );
                return;
            }
            Err(e) => {
                self.set_form_error(e.to_string());
                return;
            }
        }

        log::info!("SPI-шина {} успешно удалена", bus.variant_name());
        self.form_error = None;
        self.refresh_from_config();
        self.reset_form_after_change();
    }

    /// Синхронизирует factory-список сконфигурированных SPI-шин.
    ///
    /// Пересоздание строк выполняется только при реальном изменении списка,
    /// чтобы не провоцировать лишние GTK/Relm4 обновления.
    fn refresh_configured_buses(&mut self, spis: Vec<SpiConfig>) {
        if self.configured_spis_cache == spis {
            return;
        }

        self.configured_spis_cache = spis.clone();

        let mut guard = self.configured_buses.guard();
        guard.clear();
        for spi in spis {
            guard.push_back(spi);
        }
    }

    /// Сбрасывает форму добавления SPI в безопасные значения по умолчанию.
    ///
    /// При наличии нескольких свободных пинов SCK/MISO/MOSI получают разные
    /// стартовые индексы, чтобы обычное первое добавление не падало из-за
    /// совпадения пинов.
    fn reset_form_after_change(&mut self) {
        self.form_bus_idx = 0;
        self.form_mode_idx = 0;
        self.form_sck_idx = 0;
        self.form_miso_idx = Self::default_distinct_pin_index(1, self.available_pins.len());
        self.form_mosi_idx = Self::default_distinct_pin_index(2, self.available_pins.len());
        self.form_use_miso = true;
        self.form_use_mosi = true;
        self.form_frequency = "10".to_string();
        self.frequency_buffer.set_text("10");
    }

    /// Ограничивает индексы формы актуальными размерами списков.
    ///
    /// Это нужно после смены платы или после добавления/удаления элементов,
    /// когда ранее выбранный индекс может выйти за границы обновлённого списка.
    fn clamp_form_indexes(&mut self) {
        self.form_bus_idx = Self::clamp_index(self.form_bus_idx, self.available_buses.len());
        self.form_sck_idx = Self::clamp_index(self.form_sck_idx, self.available_pins.len());
        self.form_miso_idx = Self::clamp_index(self.form_miso_idx, self.available_pins.len());
        self.form_mosi_idx = Self::clamp_index(self.form_mosi_idx, self.available_pins.len());
    }

    /// Возвращает `idx`, если он входит в диапазон `0..len`, иначе последний валидный индекс.
    ///
    /// Для пустого списка возвращает `0`, потому что GTK `ComboRow` всё равно
    /// хранит выбранный индекс как число, а фактическая доступность контролируется
    /// через `set_sensitive`.
    fn clamp_index(idx: usize, len: usize) -> usize {
        if len == 0 { 0 } else { idx.min(len - 1) }
    }

    /// Возвращает предпочитаемый индекс пина, если он существует.
    ///
    /// Используется для стартового выбора разных SCK/MISO/MOSI без динамического
    /// вырезания выбранных пинов из соседних `ComboRow`.
    fn default_distinct_pin_index(preferred_idx: usize, len: usize) -> usize {
        if len > preferred_idx {
            preferred_idx
        } else {
            0
        }
    }

    /// Конвертирует индекс из `ComboRow` в [`SpiMode`].
    ///
    /// Использует `strum::FromRepr`, сгенерированный для core-enum. Невалидный
    /// индекс трактуется как значение [`SpiMode::default`].
    fn mode_from_index(idx: usize) -> SpiMode {
        SpiMode::from_repr(idx as u8).unwrap_or_default()
    }

    /// Обновляет [`gtk::StringList`] только при реальном изменении содержимого.
    ///
    /// `StringList::splice` сбрасывает выбранный элемент и генерирует
    /// `notify::selected`, поэтому вызов этого метода должен происходить под
    /// [`Self::refresh_guard`].
    fn splice_if_changed(model: &gtk::StringList, new_values: &[&str]) {
        let current_len = model.n_items();
        let mut changed = current_len as usize != new_values.len();

        if !changed {
            for i in 0..current_len {
                if let Some(item) = model.item(i)
                    && let Ok(string_obj) = item.downcast::<gtk::StringObject>()
                    && string_obj.string() != new_values[i as usize]
                {
                    changed = true;
                    break;
                }
            }
        }

        if changed {
            model.splice(0, current_len, new_values);
        }
    }

    /// Сохраняет сообщение ошибки для UI и пишет его в лог.
    fn set_form_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки SPI: {}", message);
        self.form_error = Some(message);
    }
}
