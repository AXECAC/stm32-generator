use crate::core::gpio::{ChosenPin, ChosenPinWithMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinConfig {
    pub pin: ChosenPinWithMode,
    pub label: Option<String>,
}

// Периферия
pub struct SpiConfig {
    pub enabled: bool,
    pub frequency_mhz: u32,
    pub mode: SpiMode,
    pub sck: ChosenPin, // просто идентификатор пина
    pub miso: Option<ChosenPin>,
    pub mosi: Option<ChosenPin>,
    pub nss: Option<ChosenPin>,
}

pub enum SpiMode {
    Mode0, // CPOL=0, CPHA=0
    Mode1, // CPOL=0, CPHA=1
    Mode2, // CPOL=1, CPHA=0
    Mode3, // CPOL=1, CPHA=1
}
