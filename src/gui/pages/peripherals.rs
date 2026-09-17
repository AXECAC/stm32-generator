use std::sync::{Arc, RwLock};

use adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw, gtk,
};
use strum::{IntoEnumIterator, VariantNames};

use crate::core::boards::PinType;
use crate::core::config::{Config, PeripheralId};
use crate::core::peripherals::ethernet::w5500::W5500Config;
use crate::core::peripherals::{Peripheral, PeripheralKind};
use crate::gui::components::forms::ComboField;
use crate::gui::components::forms::w5500::{W5500FormInput, W5500FormModel, W5500FormOutput};
use crate::gui::components::peripheral_row::{PeripheralRowModel, PeripheralRowOutput};

/// Состояние формы выбора типа периферии.
struct PeripheralForm {
    /// Выбранный тип периферии.
    kind: ComboField<PeripheralKind>,
}

impl PeripheralForm {
    /// Создаёт форму выбора типа периферии.
    fn new() -> Self {
        Self {
            kind: ComboField::new(
                PeripheralKind::iter().collect::<Vec<_>>(),
                PeripheralKind::VARIANTS,
            ),
        }
    }

    /// Возвращает выбранный тип периферии.
    fn selected_kind(&self) -> Option<PeripheralKind> {
        self.kind.selected()
    }
}

/// Состояние factory-списка уже настроенной периферии.
struct ConfiguredPeripheralsList {
    /// Кэш текущего списка периферии для защиты от лишнего пересоздания строк.
    cache: Vec<(PeripheralId, Peripheral)>,
    /// Factory-модель строк сконфигурированной периферии.
    factory: FactoryVecDeque<PeripheralRowModel>,
}

impl ConfiguredPeripheralsList {
    /// Создаёт состояние списка на базе factory-модели.
    fn new(factory: FactoryVecDeque<PeripheralRowModel>) -> Self {
        Self {
            cache: Vec::new(),
            factory,
        }
    }

    /// Синхронизирует factory-список сконфигурированной периферии.
    fn refresh(&mut self, peripherals: Vec<(PeripheralId, Peripheral)>) {
        if self.cache == peripherals {
            return;
        }

        self.cache = peripherals.clone();

        let mut guard = self.factory.guard();
        guard.clear();
        for peripheral in peripherals {
            guard.push_back(peripheral);
        }
    }
}

/// Модель страницы настройки периферии.
///
/// Страница хранит только верхнеуровневый выбор типа периферии, дочерние формы
/// конкретных устройств и factory-список уже настроенных устройств. Источником
/// истины остаётся общий [`Config`].
pub struct PeripheralsPageModel {
    /// Глобальная конфигурация приложения.
    pub(crate) config: Arc<RwLock<Config>>,
    /// Состояние выбора типа периферии.
    form: PeripheralForm,
    /// Дочерняя форма настройки W5500.
    w5500_form: Controller<W5500FormModel>,
    /// Список уже сконфигурированной периферии.
    configured: ConfiguredPeripheralsList,
}

/// Входящие сообщения страницы периферии.
#[derive(Debug)]
pub enum PeripheralsPageInput {
    /// Вкладка стала активной; нужно перечитать свежий [`Config`].
    UpdateConfig,
    /// Пользователь выбрал тип периферии по индексу.
    PeripheralKindSelected(usize),
    /// Дочерняя форма W5500 собрала валидный конфиг.
    AddW5500(W5500Config),
    /// Пользователь запросил удаление периферии.
    RemovePeripheral(PeripheralId),
}

#[relm4::component(pub)]
impl SimpleComponent for PeripheralsPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = PeripheralsPageInput;
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
                        set_title: "Добавить периферию",
                        set_description: Some("Выберите тип периферии и заполните параметры выбранного устройства."),

                        adw::ComboRow {
                            set_title: "Периферия",
                            set_model: Some(&model.form.kind.model),
                            #[watch]
                            set_selected: model.form.kind.selected_idx as u32,

                            connect_selected_notify[sender] => move |row| {
                                sender.input(PeripheralsPageInput::PeripheralKindSelected(row.selected() as usize));
                            }
                        }
                    },

                    #[local_ref]
                    w5500_form_widget -> gtk::Box {
                        #[watch]
                        set_visible: model.form.selected_kind() == Some(PeripheralKind::W5500),
                    },

                    #[local_ref]
                    configured_peripherals_group -> adw::PreferencesGroup {
                        set_title: "Сконфигурированная периферия",
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
        let w5500_form =
            W5500FormModel::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    W5500FormOutput::Submit(w5500) => PeripheralsPageInput::AddW5500(w5500),
                });

        let configured_peripherals = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .forward(sender.input_sender(), |output| match output {
                PeripheralRowOutput::Remove(id) => PeripheralsPageInput::RemovePeripheral(id),
            });

        let mut model = PeripheralsPageModel {
            config: init,
            form: PeripheralForm::new(),
            w5500_form,
            configured: ConfiguredPeripheralsList::new(configured_peripherals),
        };

        model.refresh_from_config();
        model.reset_w5500_indexes();

        let w5500_form_widget = model.w5500_form.widget();
        let configured_peripherals_group = model.configured.factory.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            PeripheralsPageInput::UpdateConfig => {
                self.send_w5500_input(W5500FormInput::ClearError);
                self.refresh_from_config();
            }
            PeripheralsPageInput::PeripheralKindSelected(idx) => {
                if self.form.kind.selected_idx == idx {
                    return;
                }
                self.form.kind.selected_idx = idx;
            }
            PeripheralsPageInput::AddW5500(w5500) => self.add_w5500(w5500),
            PeripheralsPageInput::RemovePeripheral(id) => self.remove_peripheral(id),
        }
    }
}

impl PeripheralsPageModel {
    /// Перечитывает глобальный [`Config`] и синхронизирует локальные списки страницы.
    fn refresh_from_config(&mut self) {
        let (available_spi_buses, available_pins, configured_peripherals) = {
            let config = self.config.read().unwrap();

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
                .collect::<Vec<_>>();

            (
                config.spi().iter().map(|spi| spi.bus).collect::<Vec<_>>(),
                available_pins,
                config.peripherals().to_vec(),
            )
        };

        self.form.kind.clamp_selected();
        self.send_w5500_input(W5500FormInput::UpdateOptions {
            spi_buses: available_spi_buses,
            pins: available_pins,
        });
        self.configured.refresh(configured_peripherals);
    }

    /// Добавляет [`W5500Config`] в глобальный [`Config`].
    fn add_w5500(&mut self, w5500: W5500Config) {
        let add_result = {
            let mut config = self.config.write().unwrap();
            config.add_peripheral(Peripheral::W5500(w5500))
        };

        match add_result {
            Ok(id) => {
                log::info!("Периферия W5500 #{} успешно добавлена", id.get());
            }
            Err(e) => {
                self.set_w5500_error(e.to_string());
                return;
            }
        }

        self.send_w5500_input(W5500FormInput::ClearError);
        self.refresh_from_config();
        self.reset_w5500_indexes();
    }

    /// Удаляет периферию из глобального [`Config`].
    fn remove_peripheral(&mut self, id: PeripheralId) {
        let removed = {
            let mut config = self.config.write().unwrap();
            config.remove_peripheral(id)
        };

        match removed {
            Some(peripheral) => {
                log::info!("Периферия {:?} #{} успешно удалена", peripheral, id.get());
            }
            None => {
                self.set_w5500_error(format!("Периферия #{} не найдена", id.get()));
                return;
            }
        }

        self.send_w5500_input(W5500FormInput::ClearError);
        self.refresh_from_config();
        self.reset_w5500_indexes();
    }

    /// Сбрасывает индексы формы W5500 в безопасные значения.
    fn reset_w5500_indexes(&self) {
        self.send_w5500_input(W5500FormInput::ResetIndexes);
    }

    /// Передаёт сообщение дочерней форме W5500.
    fn send_w5500_input(&self, input: W5500FormInput) {
        if let Err(e) = self.w5500_form.sender().send(input) {
            log::error!("Не удалось отправить сообщение в W5500FormModel: {:?}", e);
        }
    }

    /// Передаёт ошибку в форму W5500 и пишет её в лог.
    fn set_w5500_error(&self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки периферии: {}", message);
        self.send_w5500_input(W5500FormInput::SetError(message));
    }
}
