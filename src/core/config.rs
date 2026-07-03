use crate::core::{
    UsesPins,
    errors::ConfigError,
    gpio::{ChosenPin, ChosenPinWithMode, ChosenSpiBus},
    peripherals::Peripheral,
};

type ConfigResult<T> = Result<T, ConfigError>;

/// Вся конфигурация платы и её переферии
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    gpio_pins: Vec<PinConfig>,
    spi_buses: Vec<SpiConfig>,
    peripherals: Vec<Peripheral>,
}

impl Config {
    /// Создает новый пустой [`Config`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gpio(&self) -> &[PinConfig] {
        &self.gpio_pins
    }

    pub fn spi(&self) -> &[SpiConfig] {
        &self.spi_buses
    }

    pub fn peripherals(&self) -> &[Peripheral] {
        &self.peripherals
    }

    /// Возвращает список всех использованных пинов с gpio, spi и переферии
    fn all_uses_pins(&self) -> Vec<ChosenPin> {
        let mut pins = Vec::new();

        for pin in &self.gpio_pins {
            pins.push(pin.pin.into());
        }

        for pin in &self.spi_buses {
            pins.extend(pin.uses_pins());
        }

        for pin in &self.peripherals {
            pins.extend(pin.uses_pins());
        }

        pins
    }

    /// Проверка повторного использования [`ChosenPin`]
    /// Возвращает ошибку, если один из пинов уже используется
    fn check_conflicts_pins(&self, new_pins: &[ChosenPin]) -> ConfigResult<()> {
        let uses = self.all_uses_pins();
        for pin in new_pins {
            if uses.contains(pin) {
                return Err(ConfigError::PinAlreadyInUse(*pin));
            }
        }
        Ok(())
    }

    pub fn add_gpio_pin(&mut self, gpio_pin: PinConfig) -> ConfigResult<()> {
        self.check_conflicts_pins(&[gpio_pin.pin.into()])?;
        self.gpio_pins.push(gpio_pin);
        Ok(())
    }

    pub fn add_spi_bus(&mut self, spi: SpiConfig) -> ConfigResult<()> {
        if self.spi_buses.iter().any(|s| s.bus == spi.bus) {
            return Err(ConfigError::DuplicateSpiBus(spi.bus));
        }

        self.check_conflicts_pins(&spi.uses_pins())?;
        self.spi_buses.push(spi);
        Ok(())
    }

    pub fn add_peripheral(&mut self, peripheral: Peripheral) -> ConfigResult<()> {
        if !self
            .spi_buses
            .iter()
            .any(|spi| spi.bus == peripheral.spi_bus())
        {
            return Err(ConfigError::SpiBusNotFound(peripheral.spi_bus()));
        }

        self.check_conflicts_pins(&peripheral.uses_pins())?;
        self.peripherals.push(peripheral);
        Ok(())
    }
}

/// Конфигурация для одного пина из gpio платы микроконтроллера
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinConfig {
    pub pin: ChosenPinWithMode,
    pub label: Option<String>,
}

/// Конфигурация периферии
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiConfig {
    pub bus: ChosenSpiBus,
    pub frequency_mhz: u32,
    pub mode: SpiMode,
    pub sck: ChosenPin,
    pub miso: Option<ChosenPin>,
    pub mosi: Option<ChosenPin>,
}

impl UsesPins for SpiConfig {
    fn uses_pins(&self) -> Vec<ChosenPin> {
        let mut pins = vec![self.sck];

        if let Some(miso) = self.miso {
            pins.push(miso);
        }
        if let Some(mosi) = self.mosi {
            pins.push(mosi);
        }

        pins
    }
}

/// Конфигурация шины SPI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpiMode {
    /// Polarity: IdleLow (CPOL=0)
    /// Phase CaptureOnFirstTransition (CPHA=0)
    Mode0,
    /// Polarity: IdleLow (CPOL=0)
    /// Phase: CaptureOnSecondTransition (CPHA=1)
    Mode1,
    /// Polarity: IdleHigh (CPOL=1)
    /// Phase: CaptureOnFirstTransition (CPHA=0)
    Mode2,
    /// Polarity: IdleHigh (CPOL=1)
    /// Phase: CaptureOnSecondTransition (CPHA=1)
    Mode3,
}
