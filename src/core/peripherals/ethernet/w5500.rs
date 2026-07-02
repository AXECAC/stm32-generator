use std::net::Ipv4Addr;

use crate::core::{gpio::ChosenPin, peripherals::ethernet::MacAddr};

/// Сетевые параметры
pub struct NetworkConfig {
    pub mac_addr: MacAddr,
    pub ip: Ipv4Addr,
    pub subnet: Ipv4Addr,
    pub gateway: Ipv4Addr,
}

/// Режим работы сокета W5500.
///
/// Пока поддерживается только TCP-сервер.
pub enum SocketMode {
    /// Слушаем входящие TCP-подключения на указанном порту.
    TcpServer {
        port: u16,
        socket_num: u8, // 0..=7, W5500 поддерживает 8 независимых сокетов
    },
}

/// Конфигурация модуля W5500.
pub struct W5500Config {
    // SPI пины
    pub sck: ChosenPin,
    pub miso: ChosenPin,
    pub mosi: ChosenPin,

    // Управляющие пины
    pub cs: ChosenPin,  // Chip Select
    pub rst: ChosenPin, // Hardware Reset

    /// Настройки шины
    pub spi_frequency_mhz: u32,

    pub network: NetworkConfig,

    /// Режим работы сокета
    pub socket_mode: SocketMode,
}
