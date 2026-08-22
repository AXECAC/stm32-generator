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

    assert_eq!(gpio_keys.len(), 32);
    assert_eq!(gpio_keys.first().map(String::as_str), Some("A0"));
    assert_eq!(gpio_keys.last().map(String::as_str), Some("C15"));
    assert!(!gpio_keys.iter().any(|pin| pin == "A13"));
    assert!(!gpio_keys.iter().any(|pin| pin == "A14"));
    assert!(!gpio_keys.iter().any(|pin| pin == "B2"));
}
