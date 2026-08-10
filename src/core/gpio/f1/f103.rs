use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantNames};

/// Распиновка под STM32F103C8T6 (Blue Pill)
///
/// Blue Pill имеет 48-pin LQFP корпус. Доступные для пользователя пины
/// выведены на две штыревые рейки по бокам платы.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF103Pin {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    A9,
    A10,
    A11,
    A12,
    A15,

    B0,
    B1,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    B9,
    B10,
    B11,
    B12,
    B13,
    B14,
    B15,

    C13,
    C14,
    C15,
}

/// Доступные SPI для STM32F103
///
/// STM32F103C8T6 имеет два аппаратных SPI:
/// - SPI1 (APB2, до 36 МГц)
/// - SPI2 (APB1, до 18 МГц)
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF103SpiBus {
    SPI1,
    SPI2,
}
