use crate::core::peripherals::ethernet::w5500::W5500Config;

pub mod ethernet;

pub enum Peripheral {
    W5500(W5500Config),
}
