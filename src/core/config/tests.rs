use std::net::Ipv4Addr;

use super::*;
use crate::core::boards::{TargetBoard, TargetBoardId};
use crate::core::gpio::TargetMcu;
use crate::core::gpio::f1::StmF1PinMode;
use crate::core::gpio::f1::f103::{StmF103Pin, StmF103SpiBus};
use crate::core::gpio::f4::f401::{StmF401Pin, StmF401SpiBus};
use crate::core::peripherals::ethernet::MacAddr;
use crate::core::peripherals::ethernet::w5500::{NetworkConfig, SocketMode, W5500Config};

// TODO: Порефакторить
fn config_with_spi1() -> Config {
    let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
    let mut config = Config::new(board);

    config
        .add_spi_bus(SpiConfig {
            bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            frequency_mhz: 10,
            mode: SpiMode::Mode1,
            sck: ChosenPin::StmF401(StmF401Pin::A5),
            miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
            mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
        })
        .expect("SPI1 should be added");

    config
}

fn config_with_spi1_and_spi2() -> Config {
    let mut config = config_with_spi1();

    config
        .add_spi_bus(SpiConfig {
            bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
            frequency_mhz: 10,
            mode: SpiMode::Mode1,
            sck: ChosenPin::StmF401(StmF401Pin::B13),
            miso: Some(ChosenPin::StmF401(StmF401Pin::B14)),
            mosi: Some(ChosenPin::StmF401(StmF401Pin::B15)),
        })
        .expect("SPI2 should be added");

    config
}

fn blue_pill_config() -> Config {
    let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103).unwrap();
    Config::new(board)
}

fn f103_spi1_config() -> SpiConfig {
    SpiConfig {
        bus: ChosenSpiBus::StmF103(StmF103SpiBus::SPI1),
        frequency_mhz: 2,
        mode: SpiMode::Mode0,
        sck: ChosenPin::StmF103(StmF103Pin::A5),
        miso: Some(ChosenPin::StmF103(StmF103Pin::A6)),
        mosi: Some(ChosenPin::StmF103(StmF103Pin::A7)),
    }
}

#[test]
fn blue_pill_exposes_both_default_f103_spi_buses() {
    let config = blue_pill_config();
    let available_buses = config.available_spi_buses();

    assert_eq!(available_buses.len(), 2);
    assert!(available_buses.contains(&ChosenSpiBus::StmF103(StmF103SpiBus::SPI1)));
    assert!(available_buses.contains(&ChosenSpiBus::StmF103(StmF103SpiBus::SPI2)));
}

#[test]
fn blue_pill_accepts_default_spi1_mapping() {
    let mut config = blue_pill_config();

    config
        .add_spi_bus(f103_spi1_config())
        .expect("default F103 SPI1 mapping should be available on Blue Pill");

    assert_eq!(config.spi().len(), 1);
    assert!(
        !config
            .available_spi_buses()
            .contains(&ChosenSpiBus::StmF103(StmF103SpiBus::SPI1))
    );
}

#[test]
fn f103_spi1_remap_is_rejected_until_remap_is_supported() {
    let mut config = blue_pill_config();
    let spi = SpiConfig {
        bus: ChosenSpiBus::StmF103(StmF103SpiBus::SPI1),
        frequency_mhz: 2,
        mode: SpiMode::Mode0,
        sck: ChosenPin::StmF103(StmF103Pin::B3),
        miso: Some(ChosenPin::StmF103(StmF103Pin::B4)),
        mosi: Some(ChosenPin::StmF103(StmF103Pin::B5)),
    };

    assert!(matches!(
        config.add_spi_bus(spi),
        Err(ConfigError::UnsupportedSpiMapping {
            bus: ChosenSpiBus::StmF103(StmF103SpiBus::SPI1),
            ..
        })
    ));
}

#[test]
fn gpio_usage_blocks_conflicting_f103_spi_bus() {
    let mut config = blue_pill_config();
    config
        .add_gpio_pin(PinConfig {
            pin: crate::core::gpio::ChosenPinWithMode::StmF103(
                StmF103Pin::A5,
                StmF1PinMode::default(),
            ),
            label: Some("spi_conflict".to_string()),
        })
        .expect("GPIO should be added before SPI configuration");

    let available_buses = config.available_spi_buses();

    assert!(!available_buses.contains(&ChosenSpiBus::StmF103(StmF103SpiBus::SPI1)));
    assert!(available_buses.contains(&ChosenSpiBus::StmF103(StmF103SpiBus::SPI2)));
}

#[test]
fn blue_pill_accepts_w5500_on_f103_spi1() {
    let mut config = blue_pill_config();
    config
        .add_spi_bus(f103_spi1_config())
        .expect("default F103 SPI1 mapping should be available on Blue Pill");

    let w5500 = W5500Config {
        spi_bus: ChosenSpiBus::StmF103(StmF103SpiBus::SPI1),
        cs: ChosenPin::StmF103(StmF103Pin::A4),
        rst: ChosenPin::StmF103(StmF103Pin::A3),
        network: NetworkConfig {
            mac: MacAddr([0x02, 0x00, 0x00, 11, 22, 33]),
            ip: Ipv4Addr::new(192, 168, 1, 50),
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: Ipv4Addr::new(192, 168, 1, 1),
        },
        socket_mode: SocketMode::TcpServer {
            port: 8080,
            socket_num: 0,
        },
    };

    assert!(config.add_peripheral(Peripheral::W5500(w5500)).is_ok());
}

#[test]
fn add_spi_rejects_invalid_mapping_from_direct_struct_construction() {
    let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
    let mut config = Config::new(board);
    let spi = SpiConfig {
        bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
        frequency_mhz: 10,
        mode: SpiMode::Mode0,
        sck: ChosenPin::StmF401(StmF401Pin::B13),
        miso: Some(ChosenPin::StmF401(StmF401Pin::B14)),
        mosi: Some(ChosenPin::StmF401(StmF401Pin::B15)),
    };

    assert!(matches!(
        config.add_spi_bus(spi),
        Err(ConfigError::UnsupportedSpiMapping {
            bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
            ..
        })
    ));
}

#[test]
fn black_pill_does_not_expose_f401_spi4_without_board_pins() {
    let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
    let config = Config::new(board);
    let available_buses = config.available_spi_buses();

    assert!(available_buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI1)));
    assert!(available_buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI2)));
    assert!(available_buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI3)));
    assert!(!available_buses.contains(&ChosenSpiBus::StmF401(StmF401SpiBus::SPI4)));
}

#[test]
fn black_pill_rejects_f401_spi4_mapping_at_config_level() {
    let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
    let mut config = Config::new(board);
    let spi = SpiConfig {
        bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI4),
        frequency_mhz: 10,
        mode: SpiMode::Mode0,
        sck: ChosenPin::StmF401(StmF401Pin::E2),
        miso: Some(ChosenPin::StmF401(StmF401Pin::E5)),
        mosi: Some(ChosenPin::StmF401(StmF401Pin::E6)),
    };

    assert_eq!(
        config.add_spi_bus(spi),
        Err(ConfigError::SpiMappingUnavailableOnBoard(
            ChosenSpiBus::StmF401(StmF401SpiBus::SPI4)
        ))
    );
}

fn w5500_config(spi_bus: ChosenSpiBus, cs: StmF401Pin, rst: StmF401Pin) -> W5500Config {
    W5500Config {
        spi_bus,
        cs: ChosenPin::StmF401(cs),
        rst: ChosenPin::StmF401(rst),
        network: NetworkConfig {
            mac: MacAddr([0x02, 0x00, 0x00, 11, 22, 33]),
            ip: Ipv4Addr::new(192, 168, 1, 50),
            subnet: Ipv4Addr::new(255, 255, 255, 0),
            gateway: Ipv4Addr::new(192, 168, 1, 1),
        },
        socket_mode: SocketMode::TcpServer {
            port: 8080,
            socket_num: 0,
        },
    }
}

#[test]
fn add_peripheral_rejects_second_peripheral_on_same_spi_bus() {
    let mut config = config_with_spi1();

    config
        .add_peripheral({
            let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
            let cs = StmF401Pin::A4;
            let rst = StmF401Pin::A3;
            Peripheral::W5500(w5500_config(spi_bus, cs, rst))
        })
        .expect("first peripheral on SPI1 should be added");

    let result = config.add_peripheral({
        let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
        let cs = StmF401Pin::B0;
        let rst = StmF401Pin::B1;
        Peripheral::W5500(w5500_config(spi_bus, cs, rst))
    });

    assert!(matches!(
        result,
        Err(ConfigError::SpiBusAlreadyUsedByPeripheral(bus))
            if bus == ChosenSpiBus::StmF401(StmF401SpiBus::SPI1)
    ));
}

#[test]
fn add_peripheral_rejects_duplicate_mac_address() {
    let mut config = config_with_spi1_and_spi2();

    config
        .add_peripheral({
            let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
            let cs = StmF401Pin::A4;
            let rst = StmF401Pin::A3;
            Peripheral::W5500(w5500_config(spi_bus, cs, rst))
        })
        .expect("first peripheral should be added");

    let peripheral = w5500_config(
        ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        StmF401Pin::B12,
        StmF401Pin::B10,
    );

    let result = config.add_peripheral(Peripheral::W5500(peripheral));

    assert_eq!(
        result,
        Err(ConfigError::DuplicateMacAddress(MacAddr([
            0x02, 0x00, 0x00, 11, 22, 33,
        ])))
    );
}

#[test]
fn add_peripheral_rejects_duplicate_ip_address() {
    let mut config = config_with_spi1_and_spi2();

    config
        .add_peripheral({
            let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
            let cs = StmF401Pin::A4;
            let rst = StmF401Pin::A3;
            Peripheral::W5500(w5500_config(spi_bus, cs, rst))
        })
        .expect("first peripheral should be added");

    let mut peripheral = w5500_config(
        ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        StmF401Pin::B12,
        StmF401Pin::B10,
    );
    peripheral.network.mac = MacAddr([0x02, 0x00, 0x00, 11, 22, 34]);

    let result = config.add_peripheral(Peripheral::W5500(peripheral));

    assert_eq!(
        result,
        Err(ConfigError::DuplicateIpAddress(Ipv4Addr::new(
            192, 168, 1, 50
        )))
    );
}

#[test]
fn add_peripheral_rejects_duplicate_tcp_port() {
    let mut config = config_with_spi1_and_spi2();

    config
        .add_peripheral({
            let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
            let cs = StmF401Pin::A4;
            let rst = StmF401Pin::A3;
            Peripheral::W5500(w5500_config(spi_bus, cs, rst))
        })
        .expect("first peripheral should be added");

    let mut peripheral = w5500_config(
        ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        StmF401Pin::B12,
        StmF401Pin::B10,
    );
    peripheral.network.mac = MacAddr([0x02, 0x00, 0x00, 11, 22, 34]);
    peripheral.network.ip = Ipv4Addr::new(192, 168, 1, 51);
    peripheral.socket_mode = SocketMode::TcpServer {
        port: 8080,
        socket_num: 1,
    };

    let result = config.add_peripheral(Peripheral::W5500(peripheral));

    assert_eq!(result, Err(ConfigError::DuplicateTcpPort(8080)));
}

#[test]
fn add_peripheral_rejects_duplicate_socket_number() {
    let mut config = config_with_spi1_and_spi2();

    config
        .add_peripheral({
            let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
            let cs = StmF401Pin::A4;
            let rst = StmF401Pin::A3;
            Peripheral::W5500(w5500_config(spi_bus, cs, rst))
        })
        .expect("first peripheral should be added");

    let mut peripheral = w5500_config(
        ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        StmF401Pin::B12,
        StmF401Pin::B10,
    );
    peripheral.network.mac = MacAddr([0x02, 0x00, 0x00, 11, 22, 34]);
    peripheral.network.ip = Ipv4Addr::new(192, 168, 1, 51);
    peripheral.socket_mode = SocketMode::TcpServer {
        port: 8081,
        socket_num: 0,
    };

    let result = config.add_peripheral(Peripheral::W5500(peripheral));

    assert_eq!(result, Err(ConfigError::DuplicateSocketNumber(0)));
}

#[test]
fn add_peripheral_accepts_distinct_network_and_socket_config() {
    let mut config = config_with_spi1_and_spi2();

    config
        .add_peripheral({
            let spi_bus = ChosenSpiBus::StmF401(StmF401SpiBus::SPI1);
            let cs = StmF401Pin::A4;
            let rst = StmF401Pin::A3;
            Peripheral::W5500(w5500_config(spi_bus, cs, rst))
        })
        .expect("first peripheral should be added");

    let mut peripheral = w5500_config(
        ChosenSpiBus::StmF401(StmF401SpiBus::SPI2),
        StmF401Pin::B12,
        StmF401Pin::B10,
    );
    peripheral.network.mac = MacAddr([0x02, 0x00, 0x00, 11, 22, 34]);
    peripheral.network.ip = Ipv4Addr::new(192, 168, 1, 51);
    peripheral.socket_mode = SocketMode::TcpServer {
        port: 8081,
        socket_num: 1,
    };

    let result = config.add_peripheral(Peripheral::W5500(peripheral));

    assert!(result.is_ok());
}
