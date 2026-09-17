use super::Pin;
use crate::core::gpio::f4::f401::StmF401Pin;
use crate::core::gpio::{ChosenPin, TargetMcu};

pub(super) fn build_pins(mcu: TargetMcu) -> Vec<Pin> {
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
