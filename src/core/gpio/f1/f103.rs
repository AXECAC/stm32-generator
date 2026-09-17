use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantNames};

use crate::core::gpio::{ChosenPin, ChosenSpiBus, SpiMapping};

/// Распиновка STM32F103C8T6
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF103Pin {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    A9,
    A10,
    A11,
    A12,
    A13,
    A14,
    A15,
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    B9,
    B10,
    B11,
    B12,
    B13,
    B14,
    B15,
    C13,
    C14,
    C15,
    D0,
    D1,
}

/// Доступные штатные SPI-шины STM32F103C8T6 без remap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF103SpiBus {
    SPI1,
    SPI2,
}

impl StmF103SpiBus {
    /// Возвращает полные аппаратно совместимые mapping для этой шины.
    pub fn spi_mappings(self) -> Vec<SpiMapping> {
        match self {
            Self::SPI1 => vec![SpiMapping {
                bus: ChosenSpiBus::StmF103(Self::SPI1),
                sck: ChosenPin::StmF103(StmF103Pin::A5),
                miso: ChosenPin::StmF103(StmF103Pin::A6),
                mosi: ChosenPin::StmF103(StmF103Pin::A7),
            }],
            Self::SPI2 => vec![SpiMapping {
                bus: ChosenSpiBus::StmF103(Self::SPI2),
                sck: ChosenPin::StmF103(StmF103Pin::B13),
                miso: ChosenPin::StmF103(StmF103Pin::B14),
                mosi: ChosenPin::StmF103(StmF103Pin::B15),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StmF103Pin, StmF103SpiBus};
    use crate::core::gpio::{ChosenPin, ChosenSpiBus, TargetMcu};

    #[test]
    fn f103_all_pins_matches_lqfp48_gpio_universe() {
        let actual = TargetMcu::StmF103
            .all_pins()
            .into_iter()
            .map(|pin| pin.variant_name().to_string())
            .collect::<Vec<_>>();
        let expected = [
            "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11", "A12", "A13",
            "A14", "A15", "B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9", "B10", "B11",
            "B12", "B13", "B14", "B15", "C13", "C14", "C15", "D0", "D1",
        ]
        .map(str::to_string)
        .to_vec();

        assert_eq!(actual, expected);
    }

    #[test]
    fn spi1_uses_default_pins() {
        let mapping = StmF103SpiBus::SPI1.spi_mappings();

        assert_eq!(mapping.len(), 1);
        assert_eq!(mapping[0].bus, ChosenSpiBus::StmF103(StmF103SpiBus::SPI1));
        assert_eq!(mapping[0].sck, ChosenPin::StmF103(StmF103Pin::A5));
        assert_eq!(mapping[0].miso, ChosenPin::StmF103(StmF103Pin::A6));
        assert_eq!(mapping[0].mosi, ChosenPin::StmF103(StmF103Pin::A7));
    }

    #[test]
    fn spi2_uses_default_pins() {
        let mapping = StmF103SpiBus::SPI2.spi_mappings();

        assert_eq!(mapping.len(), 1);
        assert_eq!(mapping[0].sck, ChosenPin::StmF103(StmF103Pin::B13));
        assert_eq!(mapping[0].miso, ChosenPin::StmF103(StmF103Pin::B14));
        assert_eq!(mapping[0].mosi, ChosenPin::StmF103(StmF103Pin::B15));
    }
}
