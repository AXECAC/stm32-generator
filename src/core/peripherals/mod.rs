use serde::Serialize;

pub mod ethernet;

use crate::core::{UsesPins, gpio::ChosenSpiBus};

macro_rules! define_peripherals {
    (
        $(
            $(#[$variant_meta:meta])*
            $variant:ident($config_type:ty)
        ),* $(,)?
    ) => {
        /// Тип периферии, доступный для добавления через UI.
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            strum::EnumIter,
            strum::VariantNames,
            strum::IntoStaticStr,
        )]
        pub enum PeripheralKind {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
        pub enum Peripheral {
            $(
                $(#[$variant_meta])*
                $variant($config_type),
            )*
        }

        impl Peripheral {
            pub fn spi_bus(&self) -> ChosenSpiBus {
                match self {
                    $(
                        Self::$variant(config) => config.spi_bus,
                    )*
                }
            }
        }

        impl UsesPins for Peripheral {
            fn uses_pins(&self) -> Vec<super::gpio::ChosenPin> {
                match self {
                    $(
                        Self::$variant(config) => config.uses_pins(),
                    )*
                }
            }
        }
    };
}

define_peripherals! {
    /// Ethernet-контроллер W5500 по SPI.
    W5500(crate::core::peripherals::ethernet::w5500::W5500Config),
}
