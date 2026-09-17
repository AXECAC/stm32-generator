use thiserror::Error;

use std::net::Ipv4Addr;

use crate::core::boards::TargetBoardId;
use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::core::peripherals::ethernet::MacAddr;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TargetBoardError {
    #[error("MCU {mcu:?} не поддерживается платой {board:?}")]
    UnsupportedMcu {
        board: TargetBoardId,
        mcu: crate::core::gpio::TargetMcu,
    },
}

/// Ошибки создания конфигурации
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Дублирующийся SPI: {0:?}")]
    DuplicateSpiBus(ChosenSpiBus),

    #[error("SPI шина использованная в переферии не найдена: {0:?}")]
    SpiBusNotFound(ChosenSpiBus),

    #[error("SPI шина уже используется другой периферией: {0:?}")]
    SpiBusAlreadyUsedByPeripheral(ChosenSpiBus),

    #[error("Пин уже используется: {0:?}")]
    PinAlreadyInUse(ChosenPin),

    #[error("Недопустимая распиновка {bus:?}: SCK={sck:?}, MISO={miso:?}, MOSI={mosi:?}")]
    UnsupportedSpiMapping {
        bus: ChosenSpiBus,
        sck: ChosenPin,
        miso: Option<ChosenPin>,
        mosi: Option<ChosenPin>,
    },

    #[error("SPI mapping недоступен на выбранной плате: {0:?}")]
    SpiMappingUnavailableOnBoard(ChosenSpiBus),

    #[error("SPI шина используется периферией, удалите сначала её: {0:?}")]
    SpiBusInUse(ChosenSpiBus),

    #[error("Название пина уже используется: {0}")]
    LabelAlreadyInUse(String),

    #[error("MAC-адрес уже используется: {0}")]
    DuplicateMacAddress(MacAddr),

    #[error("IP-адрес уже используется: {0}")]
    DuplicateIpAddress(Ipv4Addr),

    #[error("TCP-порт уже используется: {0}")]
    DuplicateTcpPort(u16),

    #[error("Номер сокета уже используется: {0}")]
    DuplicateSocketNumber(u8),
}

/// Ошибки генерации проекта
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Ошибка шаблонизатора: {0}")]
    RenderError(#[from] minijinja::Error),

    #[error("Ошибка файловой системы: {0}")]
    IoError(#[from] std::io::Error),
}
