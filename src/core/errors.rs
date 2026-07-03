use thiserror::Error;

use crate::core::gpio::{ChosenPin, ChosenSpiBus};

/// Ошибки создания конфигурации
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Дублирующийся SPI: {0:?}")]
    DuplicateSpiBus(ChosenSpiBus),

    #[error("SPI шина использованная в переферии не найдена: {0:?}")]
    SpiBusNotFound(ChosenSpiBus),

    #[error("Конфликт в использовании уже занятого пина: {0:?}")]
    PinConflict(ChosenPin),
}
