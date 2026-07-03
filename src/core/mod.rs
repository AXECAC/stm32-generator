use crate::core::gpio::ChosenPin;

pub mod config;
pub mod gpio;
pub mod peripherals;
pub mod errors;

trait UsesPins {
    fn uses_pins(&self) -> Vec<ChosenPin>;
}
