use serde::Serialize;
use std::net::Ipv4Addr;

use crate::core::{
    UsesPins,
    errors::ConfigError,
    gpio::{ChosenPin, ChosenSpiBus},
    peripherals::ethernet::MacAddr,
};

/// Сетевые параметры
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetworkConfig {
    pub mac: MacAddr,
    pub ip: Ipv4Addr,
    pub subnet: Ipv4Addr,
    pub gateway: Ipv4Addr,
}

/// Режим работы сокета W5500.
///
/// Пока поддерживается только TCP-сервер.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SocketMode {
    /// Слушаем входящие TCP-подключения на указанном порту.
    TcpServer {
        port: u16,
        socket_num: u8, // 0..=7, W5500 поддерживает 8 независимых сокетов
    },
}

/// Конфигурация модуля W5500.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W5500Config {
    pub spi_bus: ChosenSpiBus,

    // Управляющие пины
    pub cs: ChosenPin,  // Chip Select
    pub rst: ChosenPin, // Hardware Reset

    pub network: NetworkConfig,

    /// Режим работы сокета
    pub socket_mode: SocketMode,
}

impl W5500Config {
    pub fn new(
        spi_bus: ChosenSpiBus,
        cs: ChosenPin,
        rst: ChosenPin,
        network: NetworkConfig,
        socket_mode: SocketMode,
    ) -> Result<Self, ConfigError> {
        if cs == rst {
            return Err(ConfigError::PinAlreadyInUse(rst));
        }

        Ok(Self {
            spi_bus,
            cs,
            rst,
            network,
            socket_mode,
        })
    }
}

impl UsesPins for W5500Config {
    fn uses_pins(&self) -> Vec<ChosenPin> {
        vec![self.cs, self.rst]
    }
}
