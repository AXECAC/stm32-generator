use crate::core::gpio::f4::{
    StmF4PinMode,
    f401::{StmF401Pin, StmF401SpiBus},
};
use strum::{IntoStaticStr, VariantNames};
use serde::Serialize;

pub mod f4;

/// Пин
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames, IntoStaticStr, Serialize)]
pub enum ChosenPin {
    StmF401(StmF401Pin),
}

impl From<ChosenPinWithMode> for ChosenPin {
    fn from(cur_pin: ChosenPinWithMode) -> Self {
        match cur_pin {
            ChosenPinWithMode::StmF401(pin, _) => Self::StmF401(pin),
        }
    }
}

/// Пин + режим
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames, IntoStaticStr, Serialize)]
pub enum ChosenPinWithMode {
    StmF401(StmF401Pin, StmF4PinMode),
}

/// Шина (номер шины)
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames, IntoStaticStr, Serialize)]
pub enum ChosenSpiBus {
    StmF401(StmF401SpiBus),
}
