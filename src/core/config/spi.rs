use serde::Serialize;

use crate::core::{
    UsesPins,
    errors::ConfigError,
    gpio::{ChosenPin, ChosenSpiBus},
};

/// Конфигурация одной SPI-шины.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpiConfig {
    pub bus: ChosenSpiBus,
    pub frequency_mhz: u32,
    pub mode: SpiMode,
    pub sck: ChosenPin,
    pub miso: Option<ChosenPin>,
    pub mosi: Option<ChosenPin>,
}

impl SpiConfig {
    pub fn new(
        bus: ChosenSpiBus,
        frequency_mhz: u32,
        mode: SpiMode,
        sck: ChosenPin,
        miso: Option<ChosenPin>,
        mosi: Option<ChosenPin>,
    ) -> Result<Self, ConfigError> {
        if let Some(miso) = miso
            && sck == miso
        {
            return Err(ConfigError::PinAlreadyInUse(miso));
        }

        if let Some(mosi) = mosi
            && (sck == mosi || miso == Some(mosi))
        {
            return Err(ConfigError::PinAlreadyInUse(mosi));
        }

        Ok(Self {
            bus,
            frequency_mhz,
            mode,
            sck,
            miso,
            mosi,
        })
    }
}

impl UsesPins for SpiConfig {
    fn uses_pins(&self) -> Vec<ChosenPin> {
        let mut pins = vec![self.sck];

        if let Some(miso) = self.miso {
            pins.push(miso);
        }
        if let Some(mosi) = self.mosi {
            pins.push(mosi);
        }

        pins
    }
}

/// Режим SPI, определяемый сочетанием CPOL и CPHA.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    strum::FromRepr,
    strum::VariantNames,
    strum::IntoStaticStr,
    Serialize,
)]
#[repr(u8)]
pub enum SpiMode {
    /// Polarity: IdleLow (CPOL=0), Phase: CaptureOnFirstTransition (CPHA=0).
    #[default]
    Mode0,
    /// Polarity: IdleLow (CPOL=0), Phase: CaptureOnSecondTransition (CPHA=1).
    Mode1,
    /// Polarity: IdleHigh (CPOL=1), Phase: CaptureOnFirstTransition (CPHA=0).
    Mode2,
    /// Polarity: IdleHigh (CPOL=1), Phase: CaptureOnSecondTransition (CPHA=1).
    Mode3,
}

impl SpiMode {
    pub fn template_vars(&self) -> (&'static str, &'static str) {
        match self {
            Self::Mode0 => ("IdleLow", "CaptureOnFirstTransition"),
            Self::Mode1 => ("IdleLow", "CaptureOnSecondTransition"),
            Self::Mode2 => ("IdleHigh", "CaptureOnFirstTransition"),
            Self::Mode3 => ("IdleHigh", "CaptureOnSecondTransition"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gpio::TargetMcu;
    use crate::core::gpio::f4::f401::{StmF401Pin, StmF401SpiBus};

    /// Проверяет корректное создание SPI-конфигурации с тремя различными пинами.
    #[test]
    fn new_accepts_distinct_pins() {
        let result = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            ChosenPin::StmF401(StmF401Pin::A5),
            Some(ChosenPin::StmF401(StmF401Pin::A6)),
            Some(ChosenPin::StmF401(StmF401Pin::A7)),
        );

        assert_eq!(
            result,
            Ok(SpiConfig {
                bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
                frequency_mhz: 10,
                mode: SpiMode::Mode0,
                sck: ChosenPin::StmF401(StmF401Pin::A5),
                miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
                mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
            })
        );
    }

    #[test]
    fn new_accepts_optional_lines_from_a_valid_mapping() {
        let result = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            ChosenPin::StmF401(StmF401Pin::A5),
            Some(ChosenPin::StmF401(StmF401Pin::A6)),
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn new_accepts_distinct_pins_before_board_validation() {
        let result = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            ChosenPin::StmF401(StmF401Pin::B13),
            Some(ChosenPin::StmF401(StmF401Pin::B14)),
            Some(ChosenPin::StmF401(StmF401Pin::B15)),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn f401_spi_mapping_catalog_contains_only_distinct_pins() {
        for bus in [
            StmF401SpiBus::SPI1,
            StmF401SpiBus::SPI2,
            StmF401SpiBus::SPI3,
            StmF401SpiBus::SPI4,
        ] {
            for mapping in bus.spi_mappings() {
                assert_ne!(mapping.sck, mapping.miso);
                assert_ne!(mapping.sck, mapping.mosi);
                assert_ne!(mapping.miso, mapping.mosi);
            }
        }
    }

    #[test]
    fn f401_exposes_all_mcu_spi_buses() {
        let buses = TargetMcu::StmF401.all_spi_buses();

        assert_eq!(buses.len(), 4);
        assert!(buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI1)));
        assert!(buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI2)));
        assert!(buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI3)));
        assert!(buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI4)));
    }

    /// Проверяет, что MISO не может использовать тот же пин, что и SCK.
    #[test]
    fn new_rejects_miso_equal_to_sck() {
        let duplicated_pin = ChosenPin::StmF401(StmF401Pin::A5);

        let result = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            duplicated_pin,
            Some(duplicated_pin),
            None,
        );

        assert_eq!(result, Err(ConfigError::PinAlreadyInUse(duplicated_pin)));
    }

    /// Проверяет, что MOSI не может использовать тот же пин, что и SCK.
    #[test]
    fn new_rejects_mosi_equal_to_sck() {
        let duplicated_pin = {
            let pin = StmF401Pin::A5;
            ChosenPin::StmF401(pin)
        };

        let result = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            duplicated_pin,
            None,
            Some(duplicated_pin),
        );

        assert_eq!(result, Err(ConfigError::PinAlreadyInUse(duplicated_pin)));
    }

    /// Проверяет, что MOSI и MISO не могут использовать один и тот же пин.
    #[test]
    fn new_rejects_mosi_equal_to_miso() {
        let duplicated_pin = {
            let pin = StmF401Pin::A6;
            ChosenPin::StmF401(pin)
        };

        let result = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            ChosenPin::StmF401(StmF401Pin::A5),
            Some(duplicated_pin),
            Some(duplicated_pin),
        );

        assert_eq!(result, Err(ConfigError::PinAlreadyInUse(duplicated_pin)));
    }

    /// Проверяет корректность возвращаемых пинов используемых SPI
    #[test]
    fn uses_pins_returns_all_configured_pins_in_bus_order() {
        let config = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            ChosenPin::StmF401(StmF401Pin::A5),
            Some(ChosenPin::StmF401(StmF401Pin::A6)),
            Some(ChosenPin::StmF401(StmF401Pin::A7)),
        )
        .expect("distinct SPI pins should produce a valid config");

        assert_eq!(
            config.uses_pins(),
            vec![
                ChosenPin::StmF401(StmF401Pin::A5),
                ChosenPin::StmF401(StmF401Pin::A6),
                ChosenPin::StmF401(StmF401Pin::A7),
            ]
        );
    }

    /// Проверяет, что отсутствующие опциональные пины не попадают в результат.
    #[test]
    fn uses_pins_omits_unconfigured_optional_pins() {
        let config = SpiConfig::new(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            10,
            SpiMode::Mode0,
            ChosenPin::StmF401(StmF401Pin::A5),
            None,
            None,
        )
        .expect("SCK-only SPI config should be valid");

        assert_eq!(config.uses_pins(), vec![ChosenPin::StmF401(StmF401Pin::A5)]);
    }
}
