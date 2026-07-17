use std::path::PathBuf;

use crate::core::{
    config::Config,
    worker::{WorkerMessage, start_generation},
};

mod core;
mod gui;

fn build_test_config() -> Config {
    use crate::core::config::{PinConfig, SpiConfig, SpiMode};
    use crate::core::gpio::{ChosenPinWithMode, ChosenPin, ChosenSpiBus};
    use crate::core::gpio::f4::{StmF4PinMode, StmF4OutputMode, StmF4OutputSpeed};
    use crate::core::gpio::f4::f401::{StmF401Pin, StmF401SpiBus};
    use crate::core::peripherals::Peripheral;
    use crate::core::peripherals::ethernet::w5500::{W5500Config, NetworkConfig, SocketMode};
    use crate::core::peripherals::ethernet::MacAddr;
    use std::net::Ipv4Addr;

    let mut config = Config::new();

    // Светодиод на PC13
    config.add_gpio_pin(PinConfig {
        pin: ChosenPinWithMode::StmF401(
            StmF401Pin::C13,
            StmF4PinMode::Output(StmF4OutputMode::PushPull, StmF4OutputSpeed::Medium)
        ),
        label: Some("led".to_string()),
    }).unwrap();

    // Шина SPI1 (PA5, PA6, PA7)
    config.add_spi_bus(SpiConfig {
        bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
        frequency_mhz: 10,
        mode: SpiMode::Mode1, // Mode1 дает CaptureOnSecondTransition
        sck: ChosenPin::StmF401(StmF401Pin::A5),
        miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
        mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
    }).unwrap();

    // W5500
    config.add_peripheral(Peripheral::W5500(W5500Config {
        spi_bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
        cs: ChosenPin::StmF401(StmF401Pin::A4),
        rst: ChosenPin::StmF401(StmF401Pin::A3),
        network: NetworkConfig {
            mac_addr: MacAddr([0x02, 0x00, 0x00, 0x11, 0x22, 0x33]),
            ip: Ipv4Addr::new(192, 168, 1, 50),
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: Ipv4Addr::new(192, 168, 1, 1),
        },
        socket_mode: SocketMode::TcpServer {
            port: 8080,
            socket_num: 0,
        },
    })).unwrap();

    config
}

fn build_bridge_test_config() -> Config {
    use crate::core::config::{SpiConfig, SpiMode};
    use crate::core::gpio::{ChosenPin, ChosenSpiBus};
    use crate::core::gpio::f4::f401::{StmF401Pin, StmF401SpiBus};
    use crate::core::peripherals::Peripheral;
    use crate::core::peripherals::ethernet::w5500::{W5500Config, NetworkConfig, SocketMode};
    use crate::core::peripherals::ethernet::MacAddr;
    use std::net::Ipv4Addr;

    let mut config = Config::new();

    // Шина SPI1 для первого W5500 (PA5, PA6, PA7)
    config.add_spi_bus(SpiConfig {
        bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
        frequency_mhz: 10,
        mode: SpiMode::Mode1,
        sck: ChosenPin::StmF401(StmF401Pin::A5),
        miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
        mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
    }).unwrap();

    // Первый W5500 (User 1)
    config.add_peripheral(Peripheral::W5500(W5500Config {
        spi_bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
        cs: ChosenPin::StmF401(StmF401Pin::A4),
        rst: ChosenPin::StmF401(StmF401Pin::A3),
        network: NetworkConfig {
            mac_addr: MacAddr([0x02, 0x00, 0x00, 11, 22, 33]),
            ip: Ipv4Addr::new(192, 168, 1, 50),
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: Ipv4Addr::new(192, 168, 1, 1),
        },
        socket_mode: SocketMode::TcpServer { port: 8080, socket_num: 0 },
    })).unwrap();

    // Шина SPI2 для второго W5500 (PB13, PB14, PB15)
    config.add_spi_bus(SpiConfig {
        bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        frequency_mhz: 10,
        mode: SpiMode::Mode1,
        sck: ChosenPin::StmF401(StmF401Pin::B13),
        miso: Some(ChosenPin::StmF401(StmF401Pin::B14)),
        mosi: Some(ChosenPin::StmF401(StmF401Pin::B15)),
    }).unwrap();

    // Второй W5500 (User 2)
    config.add_peripheral(Peripheral::W5500(W5500Config {
        spi_bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        cs: ChosenPin::StmF401(StmF401Pin::B12),
        rst: ChosenPin::StmF401(StmF401Pin::B10),
        network: NetworkConfig {
            mac_addr: MacAddr([0x02, 0x00, 0x00, 11, 22, 34]), // Другой MAC
            ip: Ipv4Addr::new(192, 168, 1, 51), // Другой IP
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: Ipv4Addr::new(192, 168, 1, 1),
        },
        socket_mode: SocketMode::TcpServer { port: 8080, socket_num: 0 },
    })).unwrap();

    config
}

fn main() {
    // let test_config = build_test_config(); // Одиночный тест
    let test_config = build_bridge_test_config(); // Тест мессенджера-моста
    let mut path = PathBuf::new();
    path.push("/home/aragami3070/test/stm32-gen-test/");
    let reciver = start_generation(test_config, path);
    'main: loop {
        while let Ok(msg) = reciver.try_recv() {
            match msg {
                WorkerMessage::Progress { percent, status } => {
                    println!("Статус: {status}, процент выполненного: {percent}");
                }
                WorkerMessage::Done { output_dir } => {
                    println!("Успех! Сохранено в {:?}", output_dir);
                    break 'main;
                }
                WorkerMessage::Error { message } => {
                    println!("Ошибка: {}", message);
                    break 'main;
                }
            }
        }
    }
}
