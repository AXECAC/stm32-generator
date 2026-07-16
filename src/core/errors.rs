use thiserror::Error;

use crate::core::gpio::{ChosenPin, ChosenSpiBus};

/// Ошибки создания конфигурации
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Дублирующийся SPI: {0:?}")]
    DuplicateSpiBus(ChosenSpiBus),

    #[error("SPI шина использованная в переферии не найдена: {0:?}")]
    SpiBusNotFound(ChosenSpiBus),

    #[error("Пин уже используется: {0:?}")]
    PinAlreadyInUse(ChosenPin),

    #[error("SPI шина используется периферией, удалите сначала её: {0:?}")]
    SpiBusInUse(ChosenSpiBus),

    #[error("Название пина уже используется: {0}")]
    LabelAlreadyInUse(String),
}

/// Ошибки генерации проекта
#[derive(Debug, Error, Clone)]
pub enum GeneratorError {
    #[error("Невозможно определить семейство микроконтроллера: конфигурация пуста")]
    EmptyConfig,
}
