use crate::core::config::Config;
use crate::core::errors::GeneratorError;
use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::core::peripherals::Peripheral;
use crate::core::peripherals::ethernet::w5500::SocketMode;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct TemplateContext {
    pub project_name: String,
    pub mcu_family: String,
    pub hal_version: String,
    pub hal_feature: String,
    pub used_ports: Vec<String>,
    pub gpio_pins: Vec<GpioPinCtx>,
    pub spis: Vec<SpiCtx>,
    pub has_w5500_tcp: bool,
    pub w5500_peripherals: Vec<W5500Ctx>,
}

#[derive(Serialize)]
pub struct GpioPinCtx {
    pub label: String,
    pub port: String,
    pub pin_num: String,
    pub method: String,
    pub is_output: bool,
    pub speed: Option<String>,
}

#[derive(Serialize)]
pub struct PinCtx {
    pub port: String,
    pub pin_num: String,
}

impl PinCtx {
    pub fn new(pin: &ChosenPin) -> Self {
        match pin {
            ChosenPin::StmF401(p) => Self::from_str(p.into()),
        }
    }

    fn from_str(s: &'static str) -> Self {
        Self {
            port: s[..1].to_lowercase(),
            pin_num: s[1..].to_string(),
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
}

#[derive(Serialize)]
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

#[derive(Serialize, Default)]
pub struct SocketModeCtx {
    pub tcp_server: Option<TcpServerCtx>,
}

#[derive(Serialize)]
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
            let (bus_name, pac_bus) = match &spi.bus {
                ChosenSpiBus::StmF401(b) => {
                    let s: &'static str = b.into(); // "SPI1", "SPI2"
                    (s.to_lowercase(), s.to_string())
                }
            };

            let (polarity, phase) = spi.mode.template_vars();

            let sck = PinCtx::new(&spi.sck);
            let miso = spi.miso.as_ref().map(PinCtx::new);
            let mosi = spi.mosi.as_ref().map(PinCtx::new);

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
            });
        }
        spis
    }

    /// Подготавливает контекст для всех модулей W5500.
    /// Возвращает вектор контекстов периферии и флаг наличия TCP сервера.
    fn build_w5500_ctx(config: &Config) -> (Vec<W5500Ctx>, bool) {
        let mut w5500_peripherals = Vec::new();
        let mut has_w5500_tcp = false;

        for (id, peripheral) in config.peripherals() {
            match peripheral {
                Peripheral::W5500(w) => {
                    let spi_bus = match &w.spi_bus {
                        ChosenSpiBus::StmF401(b) => {
                            let s: &'static str = b.into();
                            s.to_lowercase()
                        }
                    };

                    let mut socket_mode_ctx = SocketModeCtx::default();
                    match w.socket_mode {
                        SocketMode::TcpServer { port, socket_num } => {
                            has_w5500_tcp = true;
                            socket_mode_ctx.tcp_server = Some(TcpServerCtx { port, socket_num });
                        }
                    }

                    w5500_peripherals.push(W5500Ctx {
                        id: id.get(),
                        spi_bus,
                        cs: PinCtx::new(&w.cs),
                        rst: PinCtx::new(&w.rst),
                        mac: w.network.mac_addr.0,
                        ip: w.network.ip.octets(),
                        subnet: w.network.subnet.octets(),
                        gateway: w.network.gateway.octets(),
                        socket_mode: socket_mode_ctx,
                    });
                }
            }
        }
        (w5500_peripherals, has_w5500_tcp)
    }

    /// Строит контекст для Jinja из сырого конфига
    pub fn from_config(config: &Config, project_name: String) -> Result<Self, GeneratorError> {
        let mut used_ports_set = HashSet::new();
        for pin in config.all_uses_pins() {
            used_ports_set.insert(PinCtx::new(&pin).port);
        }
        let mut used_ports: Vec<String> = used_ports_set.into_iter().collect();
        used_ports.sort();

        let first_pin = config.all_uses_pins().first().copied();

        let mcu_family = first_pin
            .map(|p| p.mcu_family())
            .ok_or(GeneratorError::EmptyConfig)?
            .to_string();

        let hal_version = first_pin
            .map(|p| p.hal_version())
            .unwrap_or("0.21.0")
            .to_string();

        let hal_feature = first_pin
            .map(|p| p.hal_feature())
            .unwrap_or("stm32f401")
            .to_string();

        let gpio_pins = Self::build_gpio_ctx(config);
        let spis = Self::build_spi_ctx(config);
        let (w5500_peripherals, has_w5500_tcp) = Self::build_w5500_ctx(config);

        Ok(Self {
            project_name,
            mcu_family,
            hal_version,
            hal_feature,
            used_ports,
            gpio_pins,
            spis,
            has_w5500_tcp,
            w5500_peripherals,
        })
    }
}
