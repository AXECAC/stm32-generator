use thiserror::Error;

use crate::core::gpio::{ChosenPin, ChosenPinWithMode, ChosenSpiBus};

/// Ошибки создания конфигурации
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Дублирующийся SPI: {0:?}")]
    DuplicateSpiBus(ChosenSpiBus),

    #[error("Дублирующийся gpio пин: {0:?}")]
    DuplicateGPIOPin(ChosenPinWithMode),

    #[error("Конфликт в использовании уже занятого CS пина: {0:?}")]
    CsPinConflict(ChosenPin),
}
