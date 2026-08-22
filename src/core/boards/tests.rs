use super::{PinType, TargetBoard, TargetBoardId};
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

#[test]
fn blue_pill_exposes_sorted_gpio_subset() {
    let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103).unwrap();
    let gpio_keys = board
        .build_pins()
        .into_iter()
        .filter_map(|pin| match pin.pin_type {
            PinType::Gpio(_) => Some(pin.key),
            PinType::Power => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        gpio_keys,
        [
            "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11", "A12", "A15",
            "B0", "B1", "B3", "B4", "B5", "B6", "B7", "B8", "B9", "B10", "B11", "B12", "B13",
            "B14", "B15", "C13", "C14", "C15",
        ]
        .map(str::to_string)
        .to_vec()
    );

    assert_eq!(board.build_pins().len(), 39);
}
