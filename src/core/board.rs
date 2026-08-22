use crate::core::errors::TargetBoardError;
use crate::core::gpio::f1::f103::StmF103Pin;
use crate::core::gpio::f4::f401::StmF401Pin;
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
            TargetBoardId::BlackPill => build_black_pill_pins(self.mcu),
            TargetBoardId::BluePill => build_blue_pill_pins(self.mcu),
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

fn build_black_pill_pins(mcu: TargetMcu) -> Vec<Pin> {
    let mut pins = vec![Pin::power("VBAT"), Pin::power("3V3"), Pin::power("GND")];
    let board_pins = StmF401Pin::black_pill_pins()
        .iter()
        .copied()
        .map(ChosenPin::StmF401)
        .collect::<Vec<_>>();

    pins.extend(
        mcu.all_pins()
            .into_iter()
            .filter(|pin| board_pins.contains(pin))
            .map(Pin::gpio),
    );

    pins.extend([
        Pin::power("5V"),
        Pin::power("GND"),
        Pin::power("3V3"),
        Pin::power("GND"),
    ]);

    pins
}

fn build_blue_pill_pins(mcu: TargetMcu) -> Vec<Pin> {
    use StmF103Pin::*;

    // RESET не является GPIO и пока не представлен отдельным PinType.
    let mcu_pins = mcu.all_pins();
    let board_pins = [
        A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A15, B0, B1, B3, B4, B5, B6, B7, B8,
        B9, B10, B11, B12, B13, B14, B15, C13, C14, C15,
    ]
    .into_iter()
    .map(ChosenPin::StmF103)
    .collect::<Vec<_>>();

    let mut pins = vec![Pin::power("VBAT"), Pin::power("3V3"), Pin::power("GND")];
    pins.extend(
        mcu_pins
            .into_iter()
            .filter(|pin| board_pins.contains(pin))
            .map(Pin::gpio),
    );
    pins.extend([
        Pin::power("5V"),
        Pin::power("GND"),
        Pin::power("3V3"),
        Pin::power("GND"),
    ]);

    pins
}

#[cfg(test)]
mod tests {
    use super::{TargetBoard, TargetBoardId};
    use crate::core::errors::TargetBoardError;
    use crate::core::gpio::TargetMcu;

    #[test]
    fn board_pair_is_created_for_supported_mcu() {
        let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103)
            .expect("Blue Pill should support STM32F103");

        assert_eq!(board.id(), TargetBoardId::BluePill);
        assert_eq!(board.mcu(), TargetMcu::StmF103);
    }

    #[test]
    fn board_pair_rejects_unsupported_mcu() {
        let error = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF103)
            .expect_err("Black Pill must reject STM32F103 for now");

        assert_eq!(
            error,
            TargetBoardError::UnsupportedMcu {
                board: TargetBoardId::BlackPill,
                mcu: TargetMcu::StmF103,
            }
        );
    }
}
