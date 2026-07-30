use serde::Serialize;

use crate::core::{
    UsesPins,
    board::{Pin, PinType, TargetBoard},
    errors::ConfigError,
    gpio::{ChosenPin, ChosenSpiBus},
    peripherals::Peripheral,
    peripherals::ethernet::w5500::SocketMode,
};

mod gpio;
mod spi;

pub use gpio::PinConfig;
pub use spi::{SpiConfig, SpiMode};

type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PeripheralId(u64);

impl PeripheralId {
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// Вся конфигурация платы и её периферии.
#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub board: TargetBoard,
    next_periph_id: u64,
    gpio_pins: Vec<PinConfig>,
    spi_buses: Vec<SpiConfig>,
    peripherals: Vec<(PeripheralId, Peripheral)>,
}

impl Config {
    /// Создает новый пустой [`Config`].
    pub fn new(board: TargetBoard) -> Self {
        Self {
            board,
            next_periph_id: 0,
            gpio_pins: Vec::new(),
            spi_buses: Vec::new(),
            peripherals: Vec::new(),
        }
    }

    /// Возвращает слайс GPIO-пинов из [`Config`].
    pub fn gpio(&self) -> &[PinConfig] {
        &self.gpio_pins
    }

    /// Возвращает слайс SPI-шин из [`Config`].
    pub fn spi(&self) -> &[SpiConfig] {
        &self.spi_buses
    }

    /// Возвращает слайс периферийных устройств из [`Config`].
    pub fn peripherals(&self) -> &[(PeripheralId, Peripheral)] {
        &self.peripherals
    }

    /// Возвращает список всех пинов, используемых GPIO, SPI и периферией.
    pub fn all_uses_pins(&self) -> Vec<ChosenPin> {
        let mut pins = Vec::new();

        for pin in &self.gpio_pins {
            pins.push(pin.pin.into());
        }

        for spi in &self.spi_buses {
            pins.extend(spi.uses_pins());
        }

        for (_, peripheral) in &self.peripherals {
            pins.extend(peripheral.uses_pins());
        }

        pins
    }

    /// Возвращает список пинов платы вместе с их alias-ами и статусом настройки.
    ///
    /// GPIO-пины получают пользовательский alias из [`PinConfig`]. Пины,
    /// занятые SPI или периферией, также считаются настроенными, чтобы GUI мог
    /// отобразить их занятыми и заблокировать редактирование вне страницы, где
    /// они были созданы.
    pub(crate) fn build_pins_with_aliases(
        &self,
        board_pins: &[Pin],
    ) -> Vec<(Pin, Option<String>, bool)> {
        let configured_gpio = self.gpio();
        let peripheral_pins = self.peripheral_pins();

        board_pins
            .iter()
            .map(|pin| {
                let mut alias = None;
                let mut is_configured = false;

                if let PinType::Gpio(chosen_pin) = pin.pin_type
                    && let Some(cfg) = configured_gpio.iter().find(|p| p.pin.pin() == chosen_pin)
                {
                    is_configured = true;
                    alias = cfg.label.clone();
                } else if let PinType::Gpio(chosen_pin) = pin.pin_type {
                    if let Some(spi_bus_name) = self.spi_pin_bus_name(chosen_pin) {
                        is_configured = true;
                        alias = Some(spi_bus_name);
                    } else if peripheral_pins.contains(&chosen_pin) {
                        is_configured = true;
                        alias = Some("Peripheral".to_string());
                    }
                }

                (pin.clone(), alias, is_configured)
            })
            .collect()
    }

    /// Возвращает имя SPI-шины, которой занят пин.
    pub(crate) fn spi_pin_bus_name(&self, pin: ChosenPin) -> Option<String> {
        self.spi()
            .iter()
            .find(|spi| spi.uses_pins().contains(&pin))
            .map(|spi| spi.bus.variant_name().to_string())
    }

    /// Возвращает пины, занятые SPI-шинами.
    pub(crate) fn spi_pins(&self) -> Vec<ChosenPin> {
        self.spi().iter().flat_map(|spi| spi.uses_pins()).collect()
    }

    /// Возвращает пины, занятые периферийными устройствами.
    pub(crate) fn peripheral_pins(&self) -> Vec<ChosenPin> {
        self.peripherals()
            .iter()
            .flat_map(|(_, peripheral)| peripheral.uses_pins())
            .collect()
    }

    /// Возвращает пины, которые настроены не на странице GPIO.
    ///
    /// Такие пины должны отображаться занятыми на холсте, но не должны
    /// редактироваться через страницу `pins`.
    pub(crate) fn not_gpio_configured_pins(&self) -> Vec<ChosenPin> {
        let mut pins = self.spi_pins();
        pins.extend(self.peripheral_pins());
        pins
    }

    /// Возвращает ключи пинов, которые должны быть некликабельными на холсте GPIO.
    pub(crate) fn not_gpio_configured_pins_keys(&self, board_pins: &[Pin]) -> Vec<String> {
        let external_pins = self.not_gpio_configured_pins();
        board_pins
            .iter()
            .filter_map(|pin| match pin.pin_type {
                PinType::Gpio(chosen_pin) if external_pins.contains(&chosen_pin) => {
                    Some(pin.key.clone())
                }
                _ => None,
            })
            .collect()
    }

    /// Проверяет, что новые пины ещё не используются текущей конфигурацией.
    fn check_conflicts_pins(&self, new_pins: &[ChosenPin]) -> ConfigResult<()> {
        let used_pins = self.all_uses_pins();
        for pin in new_pins {
            if used_pins.contains(pin) {
                return Err(ConfigError::PinAlreadyInUse(*pin));
            }
        }
        Ok(())
    }

    /// Проверяет повторное использование параметров W5500.
    fn check_conflicts_w5500(&self, new_peripheral: &Peripheral) -> ConfigResult<()> {
        for (_, configured_peripheral) in self.peripherals() {
            match (configured_peripheral, new_peripheral) {
                (Peripheral::W5500(configured), Peripheral::W5500(new)) => {
                    if configured.network.mac == new.network.mac {
                        return Err(ConfigError::DuplicateMacAddress(new.network.mac));
                    }

                    if configured.network.ip == new.network.ip {
                        return Err(ConfigError::DuplicateIpAddress(new.network.ip));
                    }

                    match (&configured.socket_mode, &new.socket_mode) {
                        (
                            SocketMode::TcpServer {
                                port: configured_port,
                                socket_num: configured_socket_num,
                            },
                            SocketMode::TcpServer { port, socket_num },
                        ) => {
                            if configured_port == port {
                                return Err(ConfigError::DuplicateTcpPort(*port));
                            }

                            if configured_socket_num == socket_num {
                                return Err(ConfigError::DuplicateSocketNumber(*socket_num));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn add_gpio_pin(&mut self, gpio_pin: PinConfig) -> ConfigResult<()> {
        self.check_conflicts_pins(&[gpio_pin.pin.into()])?;

        let new_label = gpio_pin.label();
        if self.gpio_pins.iter().any(|pin| pin.label() == new_label) {
            return Err(ConfigError::LabelAlreadyInUse(new_label));
        }

        self.gpio_pins.push(gpio_pin);
        Ok(())
    }

    pub fn add_spi_bus(&mut self, spi: SpiConfig) -> ConfigResult<()> {
        if self.spi_buses.iter().any(|current| current.bus == spi.bus) {
            return Err(ConfigError::DuplicateSpiBus(spi.bus));
        }

        self.check_conflicts_pins(&spi.uses_pins())?;
        self.spi_buses.push(spi);
        Ok(())
    }

    pub fn add_peripheral(&mut self, peripheral: Peripheral) -> ConfigResult<PeripheralId> {
        let spi_bus = peripheral.spi_bus();

        if !self.spi_buses.iter().any(|spi| spi.bus == spi_bus) {
            return Err(ConfigError::SpiBusNotFound(spi_bus));
        }

        if self
            .peripherals
            .iter()
            .any(|(_, configured)| configured.spi_bus() == spi_bus)
        {
            return Err(ConfigError::SpiBusAlreadyUsedByPeripheral(spi_bus));
        }

        peripheral.validate()?;
        self.check_conflicts_w5500(&peripheral)?;
        self.check_conflicts_pins(&peripheral.uses_pins())?;

        let peripheral_id = PeripheralId(self.next_periph_id);
        self.peripherals.push((peripheral_id, peripheral));
        self.next_periph_id += 1;
        Ok(peripheral_id)
    }

    /// Удаляет GPIO-пин по его идентификатору.
    pub fn remove_gpio_pin(&mut self, pin: &ChosenPin) -> Option<PinConfig> {
        let pos = self
            .gpio_pins
            .iter()
            .position(|current| *pin == current.pin.into())?;
        Some(self.gpio_pins.remove(pos))
    }

    /// Удаляет SPI-шину, если на неё не завязана периферия.
    pub fn remove_spi(&mut self, bus: &ChosenSpiBus) -> Result<Option<SpiConfig>, ConfigError> {
        if self
            .peripherals
            .iter()
            .any(|(_, peripheral)| peripheral.spi_bus() == *bus)
        {
            return Err(ConfigError::SpiBusInUse(*bus));
        }

        let pos = self.spi_buses.iter().position(|spi| &spi.bus == bus);
        Ok(pos.map(|index| self.spi_buses.remove(index)))
    }

    /// Удаляет периферию по [`PeripheralId`].
    pub fn remove_peripheral(&mut self, id: PeripheralId) -> Option<Peripheral> {
        let pos = self
            .peripherals
            .iter()
            .position(|(current_id, _)| *current_id == id)?;
        Some(self.peripherals.remove(pos).1)
    }
}
