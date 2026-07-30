use std::sync::{Arc, RwLock};

use adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw, gtk,
};

use crate::core::board::PinType;
use crate::core::config::{Config, SpiConfig};
use crate::core::errors::ConfigError;
use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::gui::components::forms::spi::{SpiFormInput, SpiFormModel, SpiFormOutput};
use crate::gui::components::spi_bus_row::{SpiBusRowModel, SpiBusRowOutput};

/// Состояние factory-списка уже настроенных SPI-шин.
struct ConfiguredSpiBusesList {
    /// Кэш текущего списка SPI-шин для защиты от лишнего пересоздания factory-строк.
    cache: Vec<SpiConfig>,
    /// Factory-модель строк со сконфигурированными SPI-шинами.
    factory: FactoryVecDeque<SpiBusRowModel>,
}

impl ConfiguredSpiBusesList {
    /// Создаёт состояние списка на базе factory-модели.
    fn new(factory: FactoryVecDeque<SpiBusRowModel>) -> Self {
        Self {
            cache: Vec::new(),
            factory,
        }
    }

    /// Синхронизирует factory-список сконфигурированных SPI-шин.
    fn refresh(&mut self, spis: Vec<SpiConfig>) {
        if self.cache == spis {
            return;
        }

        self.cache = spis.clone();

        let mut guard = self.factory.guard();
        guard.clear();
        for spi in spis {
            guard.push_back(spi);
        }
    }
}

/// Модель страницы настройки SPI-шин.
///
/// Страница хранит глобальный [`Config`], дочернюю форму добавления SPI и
/// factory-список уже настроенных шин. Локальное состояние полей формы живёт в
/// [`SpiFormModel`].
pub struct SpiPageModel {
    /// Глобальная конфигурация приложения.
    pub(crate) config: Arc<RwLock<Config>>,
    /// Дочерняя форма добавления SPI-шины.
    form: Controller<SpiFormModel>,
    /// Список уже сконфигурированных SPI-шин.
    configured_buses: ConfiguredSpiBusesList,
}

/// Входящие сообщения страницы SPI.
#[derive(Debug)]
pub enum SpiPageInput {
    /// Вкладка стала активной; нужно перечитать свежий [`Config`].
    UpdateConfig,
    /// Дочерняя форма собрала валидный [`SpiConfig`].
    AddBus(SpiConfig),
    /// Пользователь запросил удаление сконфигурированной SPI-шины.
    RemoveBus(ChosenSpiBus),
}

#[relm4::component(pub)]
impl SimpleComponent for SpiPageModel {
    /// Данные, необходимые для инициализации страницы.
    type Init = Arc<RwLock<Config>>;
    /// Сообщения, которые страница принимает от дочерних компонентов.
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

                    #[local_ref]
                    form_widget -> gtk::Box {},

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
        let form = SpiFormModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                SpiFormOutput::Submit(spi) => SpiPageInput::AddBus(spi),
            });

        let configured_buses = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .forward(sender.input_sender(), |output| match output {
                SpiBusRowOutput::Remove(bus) => SpiPageInput::RemoveBus(bus),
            });

        let mut model = SpiPageModel {
            config: init,
            form,
            configured_buses: ConfiguredSpiBusesList::new(configured_buses),
        };

        model.refresh_from_config();
        model.reset_form_after_change();

        let form_widget = model.form.widget();
        let configured_buses_group = model.configured_buses.factory.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    /// Обрабатывает входящие сообщения страницы.
    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SpiPageInput::UpdateConfig => {
                self.send_form_input(SpiFormInput::ClearError);
                self.refresh_from_config();
            }
            SpiPageInput::AddBus(spi) => self.add_bus(spi),
            SpiPageInput::RemoveBus(bus) => self.remove_bus(bus),
        }
    }
}

impl SpiPageModel {
    /// Перечитывает глобальный [`Config`] и синхронизирует локальные списки страницы.
    fn refresh_from_config(&mut self) {
        let (available_buses, available_pins, configured_spis) = {
            let config = self.config.read().unwrap();

            let configured_buses = config
                .spi()
                .iter()
                .map(|spi| spi.bus)
                .collect::<Vec<ChosenSpiBus>>();
            let available_buses = config
                .board
                .mcu()
                .all_spi_buses()
                .into_iter()
                .filter(|bus| !configured_buses.contains(bus))
                .collect::<Vec<_>>();

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
                .collect::<Vec<ChosenPin>>();

            (available_buses, available_pins, config.spi().to_vec())
        };

        self.send_form_input(SpiFormInput::UpdateOptions {
            buses: available_buses,
            pins: available_pins,
        });
        self.configured_buses.refresh(configured_spis);
    }

    /// Добавляет [`SpiConfig`] в глобальный [`Config`].
    fn add_bus(&mut self, spi: SpiConfig) {
        let add_result = {
            let mut config = self.config.write().unwrap();
            config.add_spi_bus(spi.clone())
        };

        if let Err(e) = add_result {
            self.set_form_error(e.to_string());
            return;
        }

        log::info!(
            "SPI-шина {} успешно добавлена: frequency_mhz={}, mode={:?}, sck={}, miso={:?}, mosi={:?}",
            spi.bus.variant_name(),
            spi.frequency_mhz,
            spi.mode,
            spi.sck.variant_name(),
            spi.miso.map(|pin| pin.variant_name()),
            spi.mosi.map(|pin| pin.variant_name()),
        );
        self.send_form_input(SpiFormInput::ClearError);
        self.refresh_from_config();
        self.reset_form_after_change();
    }

    /// Удаляет SPI-шину из глобального [`Config`].
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
        self.send_form_input(SpiFormInput::ClearError);
        self.refresh_from_config();
        self.reset_form_after_change();
    }

    /// Сбрасывает форму добавления SPI в безопасные значения.
    fn reset_form_after_change(&self) {
        self.send_form_input(SpiFormInput::ResetAfterChange);
    }

    /// Передаёт сообщение дочерней форме SPI.
    fn send_form_input(&self, input: SpiFormInput) {
        if let Err(e) = self.form.sender().send(input) {
            log::error!("Не удалось отправить сообщение в SpiFormModel: {:?}", e);
        }
    }

    /// Передаёт ошибку в форму SPI и пишет её в лог.
    fn set_form_error(&self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки SPI: {}", message);
        self.send_form_input(SpiFormInput::SetError(message));
    }
}
