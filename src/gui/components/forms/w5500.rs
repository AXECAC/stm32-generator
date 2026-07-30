//! Компонент формы настройки W5500.

use std::cell::Cell;
use std::net::Ipv4Addr;
use std::rc::Rc;

use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};

use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::core::peripherals::ethernet::MacAddr;
use crate::core::peripherals::ethernet::w5500::{NetworkConfig, SocketMode, W5500Config};
use crate::gui::components::forms::{ComboField, EntryField};
use crate::gui::utils::default_distinct_pin_index;

/// Модель компонента формы настройки W5500.
///
/// Компонент хранит локальные поля формы, сам парсит пользовательский ввод и
/// отдаёт родителю уже собранный [`W5500Config`]. Проверки, завязанные на
/// глобальный [`crate::core::config::Config`], остаются на стороне родителя.
pub(crate) struct W5500FormModel {
    /// Выбранная SPI-шина.
    spi_bus: ComboField<ChosenSpiBus>,
    /// Выбранный CS-пин.
    cs_pin: ComboField<ChosenPin>,
    /// Выбранный RST-пин.
    rst_pin: ComboField<ChosenPin>,
    /// MAC-адрес.
    mac: EntryField,
    /// IP-адрес.
    ip: EntryField,
    /// Subnet mask.
    subnet: EntryField,
    /// Gateway.
    gateway: EntryField,
    /// TCP-порт.
    port: EntryField,
    /// Номер сокета.
    socket_num: EntryField,
    /// Сообщение об ошибке формы.
    error: Option<String>,
    /// Guard для программного обновления GTK-моделей.
    refresh_guard: Rc<Cell<bool>>,
}

/// Входящие сообщения компонента формы W5500.
#[derive(Debug)]
pub(crate) enum W5500FormInput {
    /// Обновить доступные SPI-шины и свободные GPIO-пины.
    UpdateOptions {
        /// Настроенные SPI-шины, доступные для подключения W5500.
        spi_buses: Vec<ChosenSpiBus>,
        /// Свободные GPIO-пины для управляющих линий W5500.
        pins: Vec<ChosenPin>,
    },
    /// Пользователь выбрал SPI-шину по индексу.
    SpiBusSelected(usize),
    /// Пользователь выбрал CS по индексу.
    CsPinSelected(usize),
    /// Пользователь выбрал RST по индексу.
    RstPinSelected(usize),
    /// Пользователь изменил MAC-адрес.
    MacChanged(String),
    /// Пользователь изменил IP-адрес.
    IpChanged(String),
    /// Пользователь изменил subnet mask.
    SubnetChanged(String),
    /// Пользователь изменил gateway.
    GatewayChanged(String),
    /// Пользователь изменил TCP-порт.
    PortChanged(String),
    /// Пользователь изменил номер сокета.
    SocketNumChanged(String),
    /// Пользователь запросил сборку и отправку формы.
    Submit,
    /// Отобразить ошибку, полученную снаружи компонента.
    SetError(String),
    /// Очистить текущую ошибку.
    ClearError,
    /// Сбросить выбранные индексы в безопасные значения.
    ResetIndexes,
}

/// Исходящие сообщения компонента формы W5500.
#[derive(Debug)]
pub(crate) enum W5500FormOutput {
    /// Форма успешно собрана и готова к добавлению в глобальный [`crate::core::config::Config`].
    Submit(W5500Config),
}

#[relm4::component(pub(crate))]
impl SimpleComponent for W5500FormModel {
    type Init = ();
    type Input = W5500FormInput;
    type Output = W5500FormOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,

            adw::PreferencesGroup {
                set_title: "Конфигурация W5500",
                set_description: Some("W5500 использует уже настроенную SPI-шину и два свободных GPIO-пина для CS/RST."),

                adw::ComboRow {
                    set_title: "SPI-шина",
                    set_model: Some(&model.spi_bus.model),
                    #[watch]
                    set_selected: model.spi_bus.selected_idx as u32,
                    #[watch]
                    set_sensitive: !model.spi_bus.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(W5500FormInput::SpiBusSelected(row.selected() as usize));
                    }
                },

                adw::ComboRow {
                    set_title: "CS",
                    set_model: Some(&model.cs_pin.model),
                    #[watch]
                    set_selected: model.cs_pin.selected_idx as u32,
                    #[watch]
                    set_sensitive: !model.cs_pin.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(W5500FormInput::CsPinSelected(row.selected() as usize));
                    }
                },

                adw::ComboRow {
                    set_title: "RST",
                    set_model: Some(&model.rst_pin.model),
                    #[watch]
                    set_selected: model.rst_pin.selected_idx as u32,
                    #[watch]
                    set_sensitive: !model.rst_pin.is_empty(),

                    connect_selected_notify[
                        sender,
                        refresh_guard = model.refresh_guard.clone()
                    ] => move |row| {
                        if refresh_guard.get() {
                            return;
                        }
                        sender.input(W5500FormInput::RstPinSelected(row.selected() as usize));
                    }
                },

                adw::ActionRow {
                    set_title: "MAC",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.mac.buffer,
                        set_width_chars: 18,
                        set_max_width_chars: 18,
                        set_placeholder_text: Some("02:00:00:0B:16:21"),
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(W5500FormInput::MacChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(W5500FormInput::Submit);
                        }
                    }
                },

                adw::ActionRow {
                    set_title: "IP",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.ip.buffer,
                        set_width_chars: 15,
                        set_max_width_chars: 15,
                        set_input_purpose: gtk::InputPurpose::Digits,
                        set_placeholder_text: Some("192.168.1.50"),
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(W5500FormInput::IpChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(W5500FormInput::Submit);
                        }
                    }
                },

                adw::ActionRow {
                    set_title: "Subnet",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.subnet.buffer,
                        set_width_chars: 15,
                        set_max_width_chars: 15,
                        set_input_purpose: gtk::InputPurpose::Digits,
                        set_placeholder_text: Some("255.255.255.0"),
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(W5500FormInput::SubnetChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(W5500FormInput::Submit);
                        }
                    }
                },

                adw::ActionRow {
                    set_title: "Gateway",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.gateway.buffer,
                        set_width_chars: 15,
                        set_max_width_chars: 15,
                        set_input_purpose: gtk::InputPurpose::Digits,
                        set_placeholder_text: Some("192.168.1.1"),
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(W5500FormInput::GatewayChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(W5500FormInput::Submit);
                        }
                    }
                },

                adw::ActionRow {
                    set_title: "TCP-порт",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.port.buffer,
                        set_width_chars: 8,
                        set_max_width_chars: 8,
                        set_input_purpose: gtk::InputPurpose::Digits,
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(W5500FormInput::PortChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(W5500FormInput::Submit);
                        }
                    }
                },

                adw::ActionRow {
                    set_title: "Socket",
                    set_subtitle: "W5500 поддерживает 0..=7",

                    add_suffix = &gtk::Entry {
                        set_buffer: &model.socket_num.buffer,
                        set_width_chars: 3,
                        set_max_width_chars: 3,
                        set_input_purpose: gtk::InputPurpose::Digits,
                        set_valign: gtk::Align::Center,

                        connect_changed[sender] => move |entry| {
                            sender.input(W5500FormInput::SocketNumChanged(entry.text().to_string()));
                        },

                        connect_activate[sender] => move |_| {
                            sender.input(W5500FormInput::Submit);
                        }
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
                set_label: "Добавить периферию",
                add_css_class: "suggested-action",
                set_halign: gtk::Align::Start,
                #[watch]
                set_sensitive: model.can_submit(),

                connect_clicked[sender] => move |_| {
                    sender.input(W5500FormInput::Submit);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = W5500FormModel::new();
        model.reset_indexes();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            W5500FormInput::UpdateOptions { spi_buses, pins } => {
                self.update_options(spi_buses, pins);
            }
            W5500FormInput::SpiBusSelected(idx) => {
                if self.spi_bus.selected_idx == idx {
                    return;
                }
                self.spi_bus.selected_idx = idx;
            }
            W5500FormInput::CsPinSelected(idx) => {
                if self.cs_pin.selected_idx == idx {
                    return;
                }
                self.cs_pin.selected_idx = idx;
            }
            W5500FormInput::RstPinSelected(idx) => {
                if self.rst_pin.selected_idx == idx {
                    return;
                }
                self.rst_pin.selected_idx = idx;
            }
            W5500FormInput::MacChanged(mac) => self.mac.set_value(mac),
            W5500FormInput::IpChanged(ip) => self.ip.set_value(ip),
            W5500FormInput::SubnetChanged(subnet) => self.subnet.set_value(subnet),
            W5500FormInput::GatewayChanged(gateway) => self.gateway.set_value(gateway),
            W5500FormInput::PortChanged(port) => self.port.set_value(port),
            W5500FormInput::SocketNumChanged(socket_num) => {
                self.socket_num.set_value(socket_num);
            }
            W5500FormInput::Submit => self.submit(sender),
            W5500FormInput::SetError(message) => self.error = Some(message),
            W5500FormInput::ClearError => self.error = None,
            W5500FormInput::ResetIndexes => self.reset_indexes(),
        }
    }
}

impl W5500FormModel {
    /// Создаёт форму W5500 со значениями по умолчанию.
    fn new() -> Self {
        Self {
            spi_bus: ComboField::empty(),
            cs_pin: ComboField::empty(),
            rst_pin: ComboField::empty(),
            mac: EntryField::new("02:00:00:0B:16:21"),
            ip: EntryField::new("192.168.1.50"),
            subnet: EntryField::new("255.255.255.0"),
            gateway: EntryField::new("192.168.1.1"),
            port: EntryField::new("8080"),
            socket_num: EntryField::new("0"),
            error: None,
            refresh_guard: Rc::new(Cell::new(false)),
        }
    }

    /// Обновляет списки доступных SPI-шин и GPIO-пинов.
    fn update_options(&mut self, spi_buses: Vec<ChosenSpiBus>, pins: Vec<ChosenPin>) {
        self.refresh_guard.set(true);

        let spi_bus_names = spi_buses
            .iter()
            .map(|bus| bus.variant_name())
            .collect::<Vec<_>>();
        self.spi_bus.replace_items(spi_buses, &spi_bus_names);

        let pin_names = pins
            .iter()
            .map(|pin| pin.variant_name())
            .collect::<Vec<_>>();
        self.cs_pin.replace_items(pins.clone(), &pin_names);
        self.rst_pin.replace_items(pins, &pin_names);

        self.clamp_indexes();
        self.refresh_guard.set(false);
    }

    /// Возвращает, можно ли отправлять текущую форму W5500.
    fn can_submit(&self) -> bool {
        !self.spi_bus.is_empty() && self.cs_pin.len() >= 2
    }

    /// Обрабатывает отправку формы.
    fn submit(&mut self, sender: ComponentSender<Self>) {
        let w5500 = match self.build_config() {
            Ok(w5500) => w5500,
            Err(e) => {
                self.set_error(e);
                return;
            }
        };

        self.error = None;
        if let Err(e) = sender.output(W5500FormOutput::Submit(w5500)) {
            log::error!("Не удалось отправить Submit из W5500FormModel: {:?}", e);
        }
    }

    /// Сбрасывает индексы формы в безопасные значения.
    fn reset_indexes(&mut self) {
        self.spi_bus.reset_selected(0);
        self.cs_pin.reset_selected(0);
        self.rst_pin
            .reset_selected(default_distinct_pin_index(1, self.cs_pin.len()));
    }

    /// Ограничивает индексы формы актуальными размерами списков.
    fn clamp_indexes(&mut self) {
        self.spi_bus.clamp_selected();
        self.cs_pin.clamp_selected();
        self.rst_pin.clamp_selected();
    }

    /// Собирает [`W5500Config`] из формы.
    fn build_config(&self) -> Result<W5500Config, String> {
        let Some(spi_bus) = self.spi_bus.selected() else {
            return Err("Сначала настройте SPI-шину для W5500".to_string());
        };

        let Some(cs) = self.cs_pin.selected() else {
            return Err("Выберите CS из списка свободных пинов".to_string());
        };

        let Some(rst) = self.rst_pin.selected() else {
            return Err("Выберите RST из списка свободных пинов".to_string());
        };

        let mac = Self::parse_mac(&self.mac.value)?;
        let ip = Self::parse_ipv4("IP", &self.ip.value)?;
        let subnet = Self::parse_ipv4("Subnet", &self.subnet.value)?;
        let gateway = Self::parse_ipv4("Gateway", &self.gateway.value)?;
        let port = Self::parse_port(&self.port.value)?;
        let socket_num = Self::parse_socket_num(&self.socket_num.value)?;

        W5500Config::new(
            spi_bus,
            cs,
            rst,
            NetworkConfig {
                mac,
                ip,
                subnet,
                gateway,
            },
            SocketMode::TcpServer { port, socket_num },
        )
        .map_err(|e| e.to_string())
    }

    /// Парсит MAC-адрес в формате `AA:BB:CC:DD:EE:FF` или `AA-BB-CC-DD-EE-FF`.
    fn parse_mac(value: &str) -> Result<MacAddr, String> {
        let parts = value.trim().split([':', '-']).collect::<Vec<_>>();
        if parts.len() != 6 {
            return Err("MAC должен состоять из 6 hex-байтов".to_string());
        }

        let mut bytes = [0_u8; 6];
        for (idx, part) in parts.iter().enumerate() {
            if part.is_empty() || part.len() > 2 {
                return Err("MAC должен состоять из 6 hex-байтов".to_string());
            }

            bytes[idx] = u8::from_str_radix(part, 16)
                .map_err(|_| "MAC должен состоять из hex-байтов".to_string())?;
        }

        Ok(MacAddr(bytes))
    }

    /// Парсит IPv4-адрес формы.
    fn parse_ipv4(field_name: &str, value: &str) -> Result<Ipv4Addr, String> {
        value
            .trim()
            .parse()
            .map_err(|_| format!("{} должен быть IPv4-адресом", field_name))
    }

    /// Парсит TCP-порт.
    fn parse_port(value: &str) -> Result<u16, String> {
        match value.trim().parse::<u16>() {
            Ok(port) if port > 0 => Ok(port),
            _ => Err("TCP-порт должен быть числом 1..=65535".to_string()),
        }
    }

    /// Парсит номер сокета W5500.
    fn parse_socket_num(value: &str) -> Result<u8, String> {
        match value.trim().parse::<u8>() {
            Ok(socket_num) if socket_num <= 7 => Ok(socket_num),
            _ => Err("Socket должен быть числом 0..=7".to_string()),
        }
    }

    /// Сохраняет локальную ошибку формы для UI и пишет её в лог.
    fn set_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        log::error!("Ошибка настройки W5500: {}", message);
        self.error = Some(message);
    }
}
