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

    pub fn add_gpio_pin(&mut self, gpio_pin: PinConfig) -> ConfigResult<()> {
        if self.gpio_pins.contains(&gpio_pin) {
            return Err(ConfigError::DuplicateGPIOPin(gpio_pin.pin));
        }

        self.gpio_pins.push(gpio_pin);
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
