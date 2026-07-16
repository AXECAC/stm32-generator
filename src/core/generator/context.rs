use serde::Serialize;

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
#[derive(Serialize, Default)]
pub struct SocketModeCtx {
    pub tcp_server: Option<TcpServerCtx>,
}

#[derive(Serialize)]
pub struct TcpServerCtx {
    pub port: u16,
    pub socket_num: u8,
}
