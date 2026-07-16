use serde::Serialize;

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
