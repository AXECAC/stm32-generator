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

#[derive(Serialize, Default)]
pub struct SocketModeCtx {
    pub tcp_server: Option<TcpServerCtx>,
}

#[derive(Serialize)]
pub struct TcpServerCtx {
    pub port: u16,
    pub socket_num: u8,
}
