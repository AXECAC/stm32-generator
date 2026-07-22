use crate::core::gpio::{ChosenPin, TargetMcu};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PinType {
    Gpio(ChosenPin),
    Power,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pin {
    pub pin_type: PinType,
    pub label: String,
    pub key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TargetBoard {
    BlackPill(TargetMcu),
}
