use serde::Serialize;

pub mod f4;

macro_rules! define_mcus {
    (
        $(
            $variant:ident {
                pin_type: $pin_type:ty,
                mode_type: $mode_type:ty,
                spi_bus_type: $spi_bus_type:ty,
                family: $family:expr,
                hal_version: $hal_version:expr,
                feature: $feature:expr $(,)?
            }
        ),* $(,)?
    ) => {
        /// Идентификатор микроконтроллера
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
        pub enum TargetMcu {
            $( $variant ),*
        }

        impl TargetMcu {
            /// Получить все пины для данного микроконтроллера
            pub fn all_pins(&self) -> Vec<ChosenPin> {
                match self {
                    $(
                        Self::$variant => {
                            <$pin_type as strum::VariantNames>::VARIANTS.iter().map(|v| {
                                ChosenPin::$variant(<$pin_type as std::str::FromStr>::from_str(v).unwrap())
                            }).collect()
                        }
                    ),*
                }
            }
        }

        /// Пин
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::IntoStaticStr, Serialize)]
        pub enum ChosenPin {
            $( $variant($pin_type) ),*
        }

        impl ChosenPin {
            pub fn mcu_family(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $family ),*
                }
            }

            pub fn hal_version(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $hal_version ),*
                }
            }

            pub fn hal_feature(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $feature ),*
                }
            }

            pub fn variant_name(&self) -> &'static str {
                match self {
                    $( Self::$variant(p) => p.into() ),*
                }
            }
        }

        impl From<ChosenPinWithMode> for ChosenPin {
            fn from(cur_pin: ChosenPinWithMode) -> Self {
                match cur_pin {
                    $( ChosenPinWithMode::$variant(pin, _) => Self::$variant(pin) ),*
                }
            }
        }

        /// Пин + режим
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::IntoStaticStr, Serialize)]
        pub enum ChosenPinWithMode {
            $( $variant($pin_type, $mode_type) ),*
        }

        impl ChosenPinWithMode {
            pub fn pin(&self) -> ChosenPin {
                match self {
                    $( Self::$variant(pin, _) => ChosenPin::$variant(*pin) ),*
                }
            }

            pub fn template_vars(&self) -> (&'static str, bool, Option<&'static str>) {
                match self {
                    $( Self::$variant(_, mode) => mode.template_vars() ),*
                }
            }
        }

        /// Шина SPI
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::IntoStaticStr, Serialize)]
        pub enum ChosenSpiBus {
            $( $variant($spi_bus_type) ),*
        }
    };
}

define_mcus! {
    StmF401 {
        pin_type: crate::core::gpio::f4::f401::StmF401Pin,
        mode_type: crate::core::gpio::f4::StmF4PinMode,
        spi_bus_type: crate::core::gpio::f4::f401::StmF401SpiBus,
        family: "stm32f4",
        hal_version: "0.23.0",
        feature: "stm32f401",
    },
}
