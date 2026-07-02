use crate::core::gpio::{ChosenBus, ChosenPin, ChosenPinWithMode};

/// Конфигурация для одного пина из gpio платы микроконтроллера
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinConfig {
    pub pin: ChosenPinWithMode,
    pub label: Option<String>,
}

/// Конфигурация периферии
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiConfig {
    pub bus: ChosenBus,
    pub frequency_mhz: u32,
    pub mode: SpiMode,
    pub sck: ChosenPin, // просто идентификатор пина
    pub miso: Option<ChosenPin>,
    pub mosi: Option<ChosenPin>,
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
