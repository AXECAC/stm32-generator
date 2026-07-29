use serde::Serialize;

use crate::core::gpio::ChosenPinWithMode;

/// Конфигурация одного пользовательского GPIO-пина микроконтроллера.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PinConfig {
    pub pin: ChosenPinWithMode,
    pub label: Option<String>,
}

impl PinConfig {
    /// Возвращает пользовательский alias или имя, построенное из номера пина.
    pub fn label(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| format!("p{}", self.pin.pin().variant_name().to_lowercase()))
    }
}
