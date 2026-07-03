use crate::core::gpio::ChosenPin;

pub mod config;
pub mod gpio;
pub mod peripherals;
pub mod errors;

/// Объект может выдать все, используемые им пины [`ChosenPin`]
///
/// Трейт предназначен для типов содержащих пины, чтобы удобно извлекать все
/// занятые этим объектом пины. Обычно используется для предотвращения
/// повторного использования одних и тех же пинов разными объектами.
///
/// TODO: придумать компактный пример для doc тестов
pub(crate) trait UsesPins {
    fn uses_pins(&self) -> Vec<ChosenPin>;
}
