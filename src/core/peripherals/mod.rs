use crate::core::peripherals::ethernet::w5500::W5500Config;

pub mod ethernet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Peripheral {
    W5500(W5500Config),
}
