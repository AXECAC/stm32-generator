use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use stm32_generator::core::board::TargetBoard;
use stm32_generator::core::config::{Config, SpiConfig, SpiMode};
use stm32_generator::core::gpio::f4::f401::{StmF401Pin, StmF401SpiBus};
use stm32_generator::core::gpio::{ChosenPin, ChosenSpiBus, TargetMcu};
use stm32_generator::core::peripherals::Peripheral;
use stm32_generator::core::peripherals::ethernet::MacAddr;
use stm32_generator::core::peripherals::ethernet::w5500::{NetworkConfig, SocketMode, W5500Config};
use stm32_generator::core::worker::{WorkerMessage, start_generation};

struct GeneratedProject {
    path: PathBuf,
}

impl GeneratedProject {
    fn read_to_string(&self, relative_path: &str) -> String {
        fs::read_to_string(self.path.join(relative_path))
            .unwrap_or_else(|e| panic!("failed to read generated `{relative_path}`: {e}"))
    }
}

impl Drop for GeneratedProject {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            eprintln!(
                "failed to remove generated test project `{}`: {e}",
                self.path.display()
            );
        }
    }
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "rendered output should contain `{needle}`"
    );
}

fn unique_output_dir(test_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after UNIX_EPOCH")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "stm32_generator_{test_name}_{}_{}",
        std::process::id(),
        timestamp
    ))
}

fn generate_project(config: Config, test_name: &str) -> GeneratedProject {
    let output_dir = unique_output_dir(test_name);
    let receiver = start_generation(config, output_dir.clone());

    loop {
        match receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("worker should send a message")
        {
            WorkerMessage::Progress { .. } => {}
            WorkerMessage::Done { output_dir: done } => {
                assert_eq!(done, output_dir);
                return GeneratedProject { path: output_dir };
            }
            WorkerMessage::Error { message } => {
                panic!("worker failed to generate project: {message}");
            }
        }
    }
}

fn full_config_with_first_w5500() -> Config {
    let mut config = Config::new(TargetBoard::BlackPill(TargetMcu::StmF401));

    add_first_w5500(&mut config);

    config
}

fn add_first_w5500(config: &mut Config) {
    config
        .add_spi_bus(SpiConfig {
            bus: { ChosenSpiBus::StmF401(StmF401SpiBus::SPI1) },
            frequency_mhz: 10,
            mode: SpiMode::Mode1,
            sck: { ChosenPin::StmF401(StmF401Pin::A5) },
            miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
            mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
        })
        .expect("SPI1 should be added");

    config
        .add_peripheral(Peripheral::W5500(W5500Config {
            spi_bus: { ChosenSpiBus::StmF401(StmF401SpiBus::SPI1) },
            cs: { ChosenPin::StmF401(StmF401Pin::A4) },
            rst: { ChosenPin::StmF401(StmF401Pin::A3) },
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
        }))
        .expect("first W5500 should be added");
}

fn add_second_w5500(config: &mut Config) {
    config
        .add_spi_bus(SpiConfig {
            bus: { ChosenSpiBus::StmF401(StmF401SpiBus::SPI2) },
            frequency_mhz: 10,
            mode: SpiMode::Mode1,
            sck: { ChosenPin::StmF401(StmF401Pin::B13) },
            miso: Some(ChosenPin::StmF401(StmF401Pin::B14)),
            mosi: Some(ChosenPin::StmF401(StmF401Pin::B15)),
        })
        .expect("SPI2 should be added");

    config
        .add_peripheral(Peripheral::W5500(W5500Config {
            spi_bus: { ChosenSpiBus::StmF401(StmF401SpiBus::SPI2) },
            cs: { ChosenPin::StmF401(StmF401Pin::B12) },
            rst: { ChosenPin::StmF401(StmF401Pin::B10) },
            network: NetworkConfig {
                mac: MacAddr([0x02, 0x00, 0x00, 11, 22, 34]),
                ip: Ipv4Addr::new(192, 168, 1, 51),
                subnet: Ipv4Addr::new(255, 255, 255, 0),
                gateway: Ipv4Addr::new(192, 168, 1, 1),
            },
            socket_mode: SocketMode::TcpServer {
                port: 8080,
                socket_num: 0,
            },
        }))
        .expect("second W5500 should be added");
}

#[test]
fn worker_generates_full_project_with_single_w5500_tcp_server() {
    let project = generate_project(full_config_with_first_w5500(), "single_w5500");
    let cargo_toml = project.read_to_string("Cargo.toml");
    let main_rs = project.read_to_string("src/main.rs");

    assert_contains(&cargo_toml, "stm32f4xx-hal");
    assert_contains(&cargo_toml, "features = [\"stm32f401\"]");
    assert_contains(&cargo_toml, "w5500-ll");
    assert_contains(&cargo_toml, "embedded-hal-bus");

    assert_contains(&main_rs, "let spi1 = Spi::new(");
    assert_contains(&main_rs, "let mut w5500_0 = W5500::new(spi_device_0);");
    assert_contains(&main_rs, "const TCP_SOCKET_0: Sn = Sn::Sn0;");
    assert_contains(&main_rs, "const SERVER_PORT_0: u16 = 8080;");
    assert_contains(&main_rs, "tcp_listen(TCP_SOCKET_0, SERVER_PORT_0)");
}

#[test]
fn worker_generates_demo_bridge_logic_for_two_w5500_tcp_servers() {
    let mut config = full_config_with_first_w5500();
    add_second_w5500(&mut config);

    let project = generate_project(config, "two_w5500_bridge");
    let main_rs = project.read_to_string("src/main.rs");

    assert_contains(&main_rs, "let spi1 = Spi::new(");
    assert_contains(&main_rs, "let spi2 = Spi::new(");
    assert_contains(&main_rs, "let mut w5500_0 = W5500::new(spi_device_0);");
    assert_contains(&main_rs, "let mut w5500_1 = W5500::new(spi_device_1);");
    assert_contains(&main_rs, "let status1 = w5500_0");
    assert_contains(&main_rs, "let status2 = w5500_1");
    assert_contains(&main_rs, "b\"User 1: \"");
    assert_contains(&main_rs, "b\"User 2: \"");
}
