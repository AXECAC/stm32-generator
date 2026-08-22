mod black_pill;
mod blue_pill;

#[cfg(test)]
mod tests;

use crate::core::errors::TargetBoardError;
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

impl Pin {
    pub fn gpio(pin: ChosenPin) -> Self {
        let variant_name = pin.variant_name();

        Self {
            pin_type: PinType::Gpio(pin),
            label: format!("P{variant_name}"),
            key: variant_name.to_string(),
        }
    }

    pub fn power(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            pin_type: PinType::Power,
            label: label.clone(),
            key: label,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TargetBoardId {
    BlackPill,
    BluePill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TargetBoard {
    id: TargetBoardId,
    mcu: TargetMcu,
}

impl TargetBoard {
    pub fn try_new(id: TargetBoardId, mcu: TargetMcu) -> Result<Self, TargetBoardError> {
        if !Self::supported_mcus(id).contains(&mcu) {
            return Err(TargetBoardError::UnsupportedMcu { board: id, mcu });
        }

        Ok(Self { id, mcu })
    }

    pub fn supported_mcus(id: TargetBoardId) -> &'static [TargetMcu] {
        match id {
            TargetBoardId::BlackPill => &[TargetMcu::StmF401],
            TargetBoardId::BluePill => &[TargetMcu::StmF103],
        }
    }

    pub fn id(&self) -> TargetBoardId {
        self.id
    }

    pub fn mcu(&self) -> TargetMcu {
        self.mcu
    }

    pub fn name(&self) -> String {
        format!("{} ({:?})", self.id.label(), self.mcu)
    }

    pub fn chip_label(&self) -> String {
        match self.mcu() {
            TargetMcu::StmF103 => "STM32F103".to_string(),
            TargetMcu::StmF401 => "STM32F401".to_string(),
        }
    }

    pub fn build_pins(&self) -> Vec<Pin> {
        match self.id {
            TargetBoardId::BlackPill => black_pill::build_pins(self.mcu),
            TargetBoardId::BluePill => blue_pill::build_pins(self.mcu),
        }
    }
}

impl TargetBoardId {
    pub fn label(self) -> &'static str {
        match self {
            Self::BlackPill => "Black Pill",
            Self::BluePill => "Blue Pill",
        }
    }
}
