use crate::core::gpio::f4::{StmF4PinMode, f401::StmF401Pin};

pub mod f4;

pub enum ChosenPin {
    StmF401(StmF401Pin, StmF4PinMode),
}
