use crate::core::gpio::ChosenPin;

pub mod board;
pub mod config;
pub mod errors;
pub mod generator;
pub mod gpio;
pub mod peripherals;
pub mod worker;

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
