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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TargetBoard {
    BlackPill(TargetMcu),
    BluePill(TargetMcu),
}

impl TargetBoard {
    pub fn mcu(&self) -> TargetMcu {
        match self {
            Self::BlackPill(mcu) => *mcu,
            Self::BluePill(mcu) => *mcu,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::BlackPill(mcu) => format!("Black Pill ({:?})", mcu),
            Self::BluePill(mcu) => format!("Blue Pill ({:?})", mcu),
        }
    }

    pub fn chip_label(&self) -> String {
        match self.mcu() {
            TargetMcu::StmF401 => "STM32F401".to_string(),
            TargetMcu::StmF103 => "STM32F103".to_string(),
        }
    }

    pub fn build_pins(&self) -> Vec<Pin> {
        match self {
            Self::BlackPill(mcu) => {
                let mut pins = Vec::new();

                // Начальные пины питания
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "VBAT".into(),
                    key: "VBAT".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "3V3".into(),
                    key: "3V3".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "GND".into(),
                    key: "GND".into(),
                });

                // Пины МК
                for pin in mcu.all_pins() {
                    let variant_name = pin.variant_name();
                    pins.push(Pin {
                        pin_type: PinType::Gpio(pin),
                        label: format!("P{}", variant_name),
                        key: variant_name.to_string(),
                    });
                }

                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "5V".into(),
                    key: "5V".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "GND".into(),
                    key: "GND".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "3V3".into(),
                    key: "3V3".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "GND".into(),
                    key: "GND".into(),
                });

                pins
            }
            Self::BluePill(mcu) => {
                let mut pins = Vec::new();

                // Пины питания (верхняя рейка)
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "VBAT".into(),
                    key: "VBAT".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "3V3".into(),
                    key: "3V3".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "GND".into(),
                    key: "GND".into(),
                });

                // Пины МК
                for pin in mcu.all_pins() {
                    let variant_name = pin.variant_name();
                    pins.push(Pin {
                        pin_type: PinType::Gpio(pin),
                        label: format!("P{}", variant_name),
                        key: variant_name.to_string(),
                    });
                }

                // Пины питания (нижняя рейка)
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "5V".into(),
                    key: "5V".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "GND".into(),
                    key: "GND".into(),
                });
                pins.push(Pin {
                    pin_type: PinType::Power,
                    label: "3V3".into(),
                    key: "3V3".into(),
                });

                pins
            }
        }
    }
}
