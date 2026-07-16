use crate::core::config::Config;
use crate::core::errors::GeneratorError;
use crate::core::gpio::{ChosenPin, ChosenSpiBus};
use crate::core::peripherals::Peripheral;
use crate::core::peripherals::ethernet::w5500::SocketMode;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct TemplateContext {
    pub mcu_family: String,
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
}
