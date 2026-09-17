use crate::core::config::Config;
use crate::core::errors::GeneratorError;
use crate::core::gpio::ChosenPin;
use crate::core::peripherals::Peripheral;
use crate::core::peripherals::ethernet::w5500::{SocketMode, W5500Config};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct TemplateContext {
    pub project_name: String,
    pub mcu_family: String,
    pub hal_version: String,
    pub hal_feature: String,
    pub target: String,
    pub chip: String,
    pub flash_origin: String,
    pub flash_length: String,
    pub ram_origin: String,
    pub ram_length: String,
    pub used_ports: Vec<String>,
    pub gpio_pins: Vec<GpioPinCtx>,
    pub spis: Vec<SpiCtx>,
    pub features: GeneratorFeaturesCtx,
    pub peripherals: Vec<PeripheralCtx>,
}

#[derive(Serialize)]
pub struct GeneratorFeaturesCtx {
    pub uses_w5500: bool,
    pub uses_w5500_tcp: bool,
    pub uses_embedded_hal_bus: bool,
}

#[derive(Serialize)]
pub struct PeripheralCtx {
    pub id: u64,
    pub kind: &'static str,
    pub w5500: Option<W5500Ctx>,
}

struct PeripheralsBuildCtx {
    features: GeneratorFeaturesCtx,
    peripherals: Vec<PeripheralCtx>,
}

#[derive(Serialize)]
pub struct GpioPinCtx {
    pub label: String,
    pub port: String,
    pub pin_num: String,
    pub cr_reg: String,
    pub method: String,
    pub is_output: bool,
    pub speed: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct PinCtx {
    pub port: String,
    pub pin_num: String,
    pub cr_reg: String,
    pub hal_pin_type: String,
}

impl PinCtx {
    pub fn new(pin: &ChosenPin) -> Self {
        Self::from_str(pin.variant_name())
    }

    fn from_str(s: &'static str) -> Self {
        let port = s[..1].to_lowercase();
        let pin_num = s[1..].to_string();
        let pin_number = pin_num
            .parse::<u8>()
            .expect("pin variant must contain a number");

        Self {
            cr_reg: if pin_number <= 7 {
                "crl".to_string()
            } else {
                "crh".to_string()
            },
            hal_pin_type: format!("gpio::gpio{}::P{}{}", port, port.to_uppercase(), pin_num),
            port,
            pin_num,
        }
    }
}

#[derive(Serialize)]
pub struct SpiCtx {
    pub bus_name: String,
    pub pac_bus: String,
    pub sck: PinCtx,
    pub miso: Option<PinCtx>,
    pub mosi: Option<PinCtx>,
    pub polarity: String,
    pub phase: String,
    pub frequency_mhz: u32,
    pub pins_tuple: String,
    pub f1_pins_tuple: String,
}

#[derive(Clone, Serialize)]
pub struct W5500Ctx {
    pub id: u64,
    pub spi_bus: String,
    pub cs: PinCtx,
    pub rst: PinCtx,
    pub mac: [u8; 6],
    pub ip: [u8; 4],
    pub subnet: [u8; 4],
    pub gateway: [u8; 4],
    pub socket_mode: SocketModeCtx,
}

#[derive(Clone, Serialize, Default)]
pub struct SocketModeCtx {
    pub tcp_server: Option<TcpServerCtx>,
}

#[derive(Clone, Serialize)]
pub struct TcpServerCtx {
    pub port: u16,
    pub socket_num: u8,
}

impl TemplateContext {
    /// Подготавливает контекст для всех GPIO пинов, конвертируя
    /// сложные Enum'ы (режимы и скорости) в строковые эквиваленты.
    fn build_gpio_ctx(config: &Config) -> Vec<GpioPinCtx> {
        let mut gpio_pins = Vec::new();
        for p in config.gpio() {
            let pin_ctx = PinCtx::new(&p.pin.pin());
            let (method, is_output, speed) = p.pin.template_vars();

            gpio_pins.push(GpioPinCtx {
                label: p.label(),
                port: pin_ctx.port,
                pin_num: pin_ctx.pin_num,
                cr_reg: pin_ctx.cr_reg,
                method: method.to_string(),
                is_output,
                speed: speed.map(|s| s.to_string()),
            });
        }
        gpio_pins
    }

    /// Собирает контекст для SPI шин, включая формирование строкового
    /// кортежа пинов (с подстановкой `None` для отсутствующих MISO/MOSI).
    fn build_spi_ctx(config: &Config) -> Vec<SpiCtx> {
        let mut spis = Vec::new();
        for spi in config.spi() {
            let s = spi.bus.variant_name();
            let bus_name = s.to_lowercase();
            let pac_bus = s.to_string();

            let (polarity, phase) = spi.mode.template_vars();

            let sck = PinCtx::new(&spi.sck);
            let miso = spi.miso.as_ref().map(PinCtx::new);
            let mosi = spi.mosi.as_ref().map(PinCtx::new);

            let mapping = spi
                .bus
                .spi_mappings()
                .into_iter()
                .find(|mapping| mapping.sck == spi.sck);
            let default_miso = mapping.as_ref().map(|mapping| PinCtx::new(&mapping.miso));
            let default_mosi = mapping.as_ref().map(|mapping| PinCtx::new(&mapping.mosi));

            // Собираем кортеж пинов
            let miso_str = if miso.is_some() {
                format!("Some(miso_{})", bus_name)
            } else {
                "None".to_string()
            };
            let mosi_str = if mosi.is_some() {
                format!("Some(mosi_{})", bus_name)
            } else {
                "None".to_string()
            };
            let pins_tuple = format!("Some(sck_{}), {}, {}", bus_name, miso_str, mosi_str);

            let f1_miso_str = if miso.is_some() {
                format!("Some(miso_{})", bus_name)
            } else {
                format!(
                    "None::<stm32f1xx_hal::{}>",
                    default_miso
                        .as_ref()
                        .expect("SPI mapping must provide MISO")
                        .hal_pin_type
                )
            };
            let f1_mosi_str = if mosi.is_some() {
                format!("Some(mosi_{})", bus_name)
            } else {
                format!(
                    "None::<stm32f1xx_hal::{}>",
                    default_mosi
                        .as_ref()
                        .expect("SPI mapping must provide MOSI")
                        .hal_pin_type
                )
            };
            let f1_pins_tuple = format!("Some(sck_{}), {}, {}", bus_name, f1_miso_str, f1_mosi_str);

            spis.push(SpiCtx {
                bus_name,
                pac_bus,
                sck,
                miso,
                mosi,
                polarity: polarity.to_string(),
                phase: phase.to_string(),
                frequency_mhz: spi.frequency_mhz,
                pins_tuple,
                f1_pins_tuple,
            });
        }
        spis
    }

    /// Подготавливает контекст для всех периферийных устройств.
    ///
    /// Каждая периферия попадает в общий [`PeripheralCtx`], а агрегированные
    /// флаги возможностей попадают в [`GeneratorFeaturesCtx`].
    fn build_peripherals_ctx(config: &Config) -> PeripheralsBuildCtx {
        let mut build_ctx = PeripheralsBuildCtx {
            features: GeneratorFeaturesCtx {
                uses_w5500: false,
                uses_w5500_tcp: false,
                uses_embedded_hal_bus: false,
            },
            peripherals: Vec::new(),
        };

        for (id, peripheral) in config.peripherals() {
            match peripheral {
                Peripheral::W5500(w5500) => {
                    let (w5500_ctx, has_tcp) = Self::build_w5500_ctx(id.get(), w5500);

                    build_ctx.features.uses_w5500 = true;
                    build_ctx.features.uses_embedded_hal_bus = true;
                    if has_tcp {
                        build_ctx.features.uses_w5500_tcp = true;
                    }

                    build_ctx.peripherals.push(PeripheralCtx {
                        id: id.get(),
                        kind: "w5500",
                        w5500: Some(w5500_ctx),
                    });
                }
            }
        }

        build_ctx
    }

    /// Подготавливает контекст для одного модуля W5500.
    ///
    /// Возвращает контекст устройства и флаг наличия TCP-сервера, который
    /// используется feature-флагами генератора и legacy W5500-шаблонами.
    fn build_w5500_ctx(id: u64, w5500: &W5500Config) -> (W5500Ctx, bool) {
        let spi_bus = w5500.spi_bus.variant_name().to_lowercase();

        let (socket_mode_ctx, has_tcp) = match w5500.socket_mode {
            SocketMode::TcpServer { port, socket_num } => (
                SocketModeCtx {
                    tcp_server: Some(TcpServerCtx { port, socket_num }),
                },
                true,
            ),
        };

        (
            W5500Ctx {
                id,
                spi_bus,
                cs: PinCtx::new(&w5500.cs),
                rst: PinCtx::new(&w5500.rst),
                mac: w5500.network.mac.0,
                ip: w5500.network.ip.octets(),
                subnet: w5500.network.subnet.octets(),
                gateway: w5500.network.gateway.octets(),
                socket_mode: socket_mode_ctx,
            },
            has_tcp,
        )
    }

    /// Строит контекст для Jinja из сырого конфига
    pub fn from_config(config: &Config, project_name: String) -> Result<Self, GeneratorError> {
        let mcu = config.board.mcu();
        let mut used_ports_set = HashSet::new();
        for pin in config.all_uses_pins() {
            used_ports_set.insert(PinCtx::new(&pin).port);
        }
        let mut used_ports: Vec<String> = used_ports_set.into_iter().collect();
        used_ports.sort();

        let gpio_pins = Self::build_gpio_ctx(config);
        let spis = Self::build_spi_ctx(config);
        let peripherals_ctx = Self::build_peripherals_ctx(config);

        Ok(Self {
            project_name,
            mcu_family: mcu.mcu_family().to_string(),
            hal_version: mcu.hal_version().to_string(),
            hal_feature: mcu.hal_feature().to_string(),
            target: mcu.target().to_string(),
            chip: mcu.chip().to_string(),
            flash_origin: mcu.flash_origin().to_string(),
            flash_length: mcu.flash_length().to_string(),
            ram_origin: mcu.ram_origin().to_string(),
            ram_length: mcu.ram_length().to_string(),
            used_ports,
            gpio_pins,
            spis,
            features: peripherals_ctx.features,
            peripherals: peripherals_ctx.peripherals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TemplateContext;
    use crate::core::boards::{TargetBoard, TargetBoardId};
    use crate::core::config::Config;
    use crate::core::gpio::TargetMcu;

    #[test]
    fn empty_f103_config_uses_selected_board_metadata() {
        let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103).unwrap();
        let config = Config::new(board);

        let context = TemplateContext::from_config(&config, "blue_pill".to_string())
            .expect("board metadata should be enough to build an empty context");

        assert_eq!(context.mcu_family, "stm32f1");
        assert_eq!(context.hal_version, "0.11.0");
        assert_eq!(context.hal_feature, "stm32f103");
        assert_eq!(context.target, "thumbv7m-none-eabi");
        assert_eq!(context.chip, "STM32F103C8T6");
        assert_eq!(context.flash_origin, "0x08000000");
        assert_eq!(context.flash_length, "64K");
        assert_eq!(context.ram_origin, "0x20000000");
        assert_eq!(context.ram_length, "20K");
        assert!(context.used_ports.is_empty());
    }

    #[test]
    fn empty_f401_config_uses_selected_board_metadata() {
        let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
        let config = Config::new(board);

        let context = TemplateContext::from_config(&config, "black_pill".to_string())
            .expect("board metadata should be enough to build an empty context");

        assert_eq!(context.mcu_family, "stm32f4");
        assert_eq!(context.hal_version, "0.23.0");
        assert_eq!(context.hal_feature, "stm32f401");
        assert_eq!(context.target, "thumbv7em-none-eabi");
        assert_eq!(context.chip, "STM32F401CCU6");
        assert_eq!(context.flash_length, "256K");
        assert_eq!(context.ram_length, "64K");
    }
}
