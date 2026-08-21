use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantNames};

use crate::core::gpio::{ChosenPin, ChosenSpiBus, SpiMapping};

/// Распиновка под STM32F401 (в частности под black pill)
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF401Pin {
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
    B12,
    B13,
    B14,
    B15,

    C13,
    C14,
    C15,

    E2,
    E5,
    E6,
    E12,
    E13,
    E14,

    H0,
    H1,
}

/// Доступные SPI для STM32F401
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF401SpiBus {
    SPI1,
    SPI2,
    SPI3,
    SPI4,
}

impl StmF401SpiBus {
    /// Возвращает все совместимые полные SPI-mapping для STM32F401.
    ///
    /// HAL F4 задаёт допустимые пины независимо для SCK, MISO и MOSI.
    /// Поэтому здесь строятся все комбинации из этих capability-списков,
    /// исключая варианты с повторным использованием одного GPIO.
    pub fn spi_mappings(self) -> Vec<SpiMapping> {
        match self {
            Self::SPI1 => make_mappings(
                ChosenSpiBus::StmF401(Self::SPI1),
                &[StmF401Pin::A5, StmF401Pin::B3],
                &[StmF401Pin::A6, StmF401Pin::B4],
                &[StmF401Pin::A7, StmF401Pin::B5],
            ),
            Self::SPI2 => make_mappings(
                ChosenSpiBus::StmF401(Self::SPI2),
                &[StmF401Pin::B10, StmF401Pin::B13],
                &[StmF401Pin::B14],
                &[StmF401Pin::B15],
            ),
            Self::SPI3 => make_mappings(
                ChosenSpiBus::StmF401(Self::SPI3),
                &[StmF401Pin::B3],
                &[StmF401Pin::B4],
                &[StmF401Pin::B5],
            ),
            Self::SPI4 => make_mappings(
                ChosenSpiBus::StmF401(Self::SPI4),
                &[StmF401Pin::E2, StmF401Pin::E12],
                &[StmF401Pin::E5, StmF401Pin::E13],
                &[StmF401Pin::E6, StmF401Pin::E14],
            ),
        }
    }
}

impl StmF401Pin {
    /// GPIO, выведенные в текущем описании платы Black Pill.
    ///
    /// Остальные пины остаются частью MCU capability и могут быть добавлены
    /// отдельной платой без изменения модели STM32F401.
    pub fn black_pill_pins() -> &'static [Self] {
        use StmF401Pin::*;

        &[
            A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, B0, B1, B2, B3,
            B4, B5, B6, B7, B8, B9, B10, B12, B13, B14, B15, C13, C14, C15, H0, H1,
        ]
    }
}

fn make_mappings(
    bus: ChosenSpiBus,
    sck_pins: &[StmF401Pin],
    miso_pins: &[StmF401Pin],
    mosi_pins: &[StmF401Pin],
) -> Vec<SpiMapping> {
    sck_pins
        .iter()
        .flat_map(|sck| {
            miso_pins.iter().flat_map(move |miso| {
                mosi_pins.iter().filter_map(move |mosi| {
                    let sck = ChosenPin::StmF401(*sck);
                    let miso = ChosenPin::StmF401(*miso);
                    let mosi = ChosenPin::StmF401(*mosi);

                    (sck != miso && sck != mosi && miso != mosi).then_some(SpiMapping {
                        bus,
                        sck,
                        miso,
                        mosi,
                    })
                })
            })
        })
        .collect()
}
