use crate::core::gpio::f4::{StmF4PinMode, f401::StmF401Pin};

pub mod f4;

/// Пин
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChosenPin {
    StmF401(StmF401Pin),
}

/// Пин + режим
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChosenPinWithMode {
    StmF401(StmF401Pin, StmF4PinMode),
}
