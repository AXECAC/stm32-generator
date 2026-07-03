use crate::core::{UsesPins, peripherals::ethernet::w5500::W5500Config};

pub mod ethernet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Peripheral {
    W5500(W5500Config),
}

impl UsesPins for Peripheral {
    fn uses_pins(&self) -> Vec<super::gpio::ChosenPin> {
        match self {
            Self::W5500(w5500) => w5500.uses_pins(),
        }
    }
}
