use super::Pin;
use crate::core::gpio::f1::f103::StmF103Pin;
use crate::core::gpio::{ChosenPin, TargetMcu};

pub(super) fn build_pins(mcu: TargetMcu) -> Vec<Pin> {
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
