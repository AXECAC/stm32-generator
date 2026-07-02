use std::net::Ipv4Addr;

use crate::core::peripherals::ethernet::MacAddr;

/// Сетевые параметры
pub struct NetworkConfig {
    pub mac_addr: MacAddr,
    pub ip: Ipv4Addr,
    pub subnet: Ipv4Addr,
    pub gateway: Ipv4Addr,
}
