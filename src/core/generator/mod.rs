use crate::core::errors::GeneratorError;
use crate::core::generator::context::TemplateContext;
use minijinja::Environment;
use std::collections::HashMap;

pub(crate) mod context;
pub(crate) mod templates;
pub(crate) mod writer;

/// Создает [`minijinja::Environment`] и регистрирует в нём все статические шаблоны.
///
/// Имена шаблонов сохраняют суффикс `.j2`, так как они используются для связи
/// модулей внутри самих шаблонов (через `{% include %}`).
///
/// # Errors
/// Функция вернет ошибку [`GeneratorError::RenderError`], если синтаксис j2
/// файлов - не верный
fn build_environment<'a>() -> Result<Environment<'a>, GeneratorError> {
    let mut env = Environment::new();

    // Точки входа
    env.add_template("Cargo.toml.j2", templates::CARGO_TOML)?;
    env.add_template(".cargo/config.toml.j2", templates::CARGO_CONFIG_TOML)?;
    env.add_template("memory.x.j2", templates::MEMORY_X)?;
    env.add_template("justfile.j2", templates::JUSTFILE)?;
    env.add_template("main.rs.j2", templates::MAIN_RS)?;
    env.add_template(".gitignore.j2", templates::GITIGNORE)?;
    env.add_template("build.rs.j2", templates::BUILD_RS)?;

    // Блоки MCU
    env.add_template(
        "blocks/mcu/stm32f4/imports.rs.j2",
        templates::MCU_STM32F4_IMPORTS,
    )?;
    env.add_template("blocks/mcu/stm32f4/init.rs.j2", templates::MCU_STM32F4_INIT)?;
    env.add_template("blocks/mcu/stm32f4/gpio.rs.j2", templates::MCU_STM32F4_GPIO)?;
    env.add_template(
        "blocks/mcu/stm32f4/w5500_pins.rs.j2",
        templates::MCU_STM32F4_W5500_PINS,
    )?;

    // Блоки MCU (STM32F1)
    env.add_template(
        "blocks/mcu/stm32f1/imports.rs.j2",
        templates::MCU_STM32F1_IMPORTS,
    )?;
    env.add_template("blocks/mcu/stm32f1/init.rs.j2", templates::MCU_STM32F1_INIT)?;
    env.add_template("blocks/mcu/stm32f1/gpio.rs.j2", templates::MCU_STM32F1_GPIO)?;
    env.add_template(
        "blocks/mcu/stm32f1/w5500_pins.rs.j2",
        templates::MCU_STM32F1_W5500_PINS,
    )?;

    // Блоки Периферии (W5500)
    env.add_template(
        "blocks/peripherals/W5500/imports.rs.j2",
        templates::PERIPHERAL_W5500_IMPORTS,
    )?;
    env.add_template(
        "blocks/peripherals/W5500/init.rs.j2",
        templates::PERIPHERAL_W5500_INIT,
    )?;
    env.add_template(
        "blocks/peripherals/W5500/logic_single.rs.j2",
        templates::PERIPHERAL_W5500_LOGIC_SINGLE,
    )?;
    env.add_template(
        "blocks/peripherals/W5500/logic_bridge.rs.j2",
        templates::PERIPHERAL_W5500_LOGIC_BRIDGE,
    )?;

    Ok(env)
}

/// Финальный путь к файлу в проекте
pub type ProjectPath = String;

/// Готовый сгенерированный, с помощью [`minijinja`], исходный код
pub type Code = String;

/// Рендерит все файлы проекта, подставляя переданный `TemplateContext`.
///
/// # Errors
/// Функция вернет ошибку [`GeneratorError::RenderError`], если нужные файлы
/// не будут найдены
fn render_templates(
    env: &Environment,
    context: &TemplateContext,
) -> Result<HashMap<ProjectPath, Code>, GeneratorError> {
    let mut files = HashMap::new();

    let cargo_toml = env.get_template("Cargo.toml.j2")?.render(context)?;
    files.insert("Cargo.toml".to_string(), cargo_toml);

    let cargo_config = env.get_template(".cargo/config.toml.j2")?.render(context)?;
    files.insert(".cargo/config.toml".to_string(), cargo_config);

    let memory_x = env.get_template("memory.x.j2")?.render(context)?;
    files.insert("memory.x".to_string(), memory_x);

    let justfile = env.get_template("justfile.j2")?.render(context)?;
    files.insert("justfile".to_string(), justfile);

    let main_rs = env.get_template("main.rs.j2")?.render(context)?;
    files.insert("src/main.rs".to_string(), main_rs);

    let gitignore = env.get_template(".gitignore.j2")?.render(context)?;
    files.insert(".gitignore".to_string(), gitignore);

    let build_rs = env.get_template("build.rs.j2")?.render(context)?;
    files.insert("build.rs".to_string(), build_rs);

    Ok(files)
}

/// Рендер проект из контекста шаблона в [`HashMap<ProjectPath, Code>`]
///
/// Создает окружение, рендерит шаблоны и возвращает словарь готовых файлов.
pub fn render(context: &TemplateContext) -> Result<HashMap<ProjectPath, Code>, GeneratorError> {
    let env = build_environment()?;
    render_templates(&env, context)
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::render;
    use crate::core::boards::{TargetBoard, TargetBoardId};
    use crate::core::config::{Config, SpiConfig, SpiMode};
    use crate::core::generator::context::TemplateContext;
    use crate::core::gpio::ChosenPin;
    use crate::core::gpio::ChosenSpiBus;
    use crate::core::gpio::TargetMcu;
    use crate::core::gpio::f1::f103::{StmF103Pin, StmF103SpiBus};
    use crate::core::gpio::f4::f401::{StmF401Pin, StmF401SpiBus};
    use crate::core::peripherals::Peripheral;
    use crate::core::peripherals::ethernet::MacAddr;
    use crate::core::peripherals::ethernet::w5500::{NetworkConfig, SocketMode, W5500Config};

    #[test]
    fn render_selects_stm32f1_blocks_from_mcu_family() {
        let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103).unwrap();
        let config = Config::new(board);
        let context = TemplateContext::from_config(&config, "blue_pill".to_string()).unwrap();

        let files = render(&context).expect("F1 templates should render");
        let main_rs = files
            .get("src/main.rs")
            .expect("main.rs should be rendered");

        assert!(main_rs.contains("stm32f1xx_hal"));
        assert!(main_rs.contains("rcc.cfgr.freeze(&mut flash.acr)"));
        assert!(!main_rs.contains("stm32f4xx_hal"));

        let cargo_toml = files
            .get("Cargo.toml")
            .expect("Cargo.toml should be rendered");
        assert!(
            cargo_toml
                .contains("stm32f1xx-hal = { version = \"0.11.0\", features = [\"stm32f103\"]}")
        );

        let cargo_config = files
            .get(".cargo/config.toml")
            .expect("Cargo config should be rendered");
        assert!(cargo_config.contains("[target.thumbv7m-none-eabi]"));
        assert!(cargo_config.contains("probe-rs run --chip STM32F103C8T6"));

        let memory_x = files.get("memory.x").expect("memory.x should be rendered");
        assert!(memory_x.contains("MCU STM32F103C8T6 (stm32f1)"));
        assert!(memory_x.contains("FLASH : ORIGIN = 0x08000000, LENGTH = 64K"));
        assert!(memory_x.contains("RAM : ORIGIN = 0x20000000, LENGTH = 20K"));

        let justfile = files.get("justfile").expect("justfile should be rendered");
        assert!(justfile.contains("dfu-util -a 0 -s 0x08000000:leave"));
    }

    #[test]
    fn render_emits_f1_spi_pin_configuration() {
        let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103).unwrap();
        let mut config = Config::new(board);
        config
            .add_spi_bus(SpiConfig {
                bus: ChosenSpiBus::StmF103(StmF103SpiBus::SPI1),
                frequency_mhz: 2,
                mode: SpiMode::Mode0,
                sck: ChosenPin::StmF103(StmF103Pin::A5),
                miso: Some(ChosenPin::StmF103(StmF103Pin::A6)),
                mosi: Some(ChosenPin::StmF103(StmF103Pin::A7)),
            })
            .unwrap();

        let context = TemplateContext::from_config(&config, "blue_pill_spi".to_string()).unwrap();
        let files = render(&context).expect("F1 SPI templates should render");
        let main_rs = files
            .get("src/main.rs")
            .expect("main.rs should be rendered");

        assert!(main_rs.contains("gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl)"));
        assert!(main_rs.contains("gpioa.pa6.into_floating_input(&mut gpioa.crl)"));
        assert!(main_rs.contains("gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl)"));
        assert!(main_rs.contains("dp.SPI1"));
        assert!(main_rs.contains("(Some(sck_spi1), Some(miso_spi1), Some(mosi_spi1))"));
    }

    #[test]
    fn render_emits_f1_specific_w5500_control_pin_setup() {
        let board = TargetBoard::try_new(TargetBoardId::BluePill, TargetMcu::StmF103).unwrap();
        let mut config = Config::new(board);
        config
            .add_spi_bus(SpiConfig {
                bus: ChosenSpiBus::StmF103(StmF103SpiBus::SPI1),
                frequency_mhz: 2,
                mode: SpiMode::Mode0,
                sck: ChosenPin::StmF103(StmF103Pin::A5),
                miso: Some(ChosenPin::StmF103(StmF103Pin::A6)),
                mosi: Some(ChosenPin::StmF103(StmF103Pin::A7)),
            })
            .unwrap();

        config
            .add_peripheral(Peripheral::W5500(W5500Config {
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
            }))
            .unwrap();

        let context = TemplateContext::from_config(&config, "blue_pill_w5500".to_string()).unwrap();
        let files = render(&context).expect("F1 W5500 templates should render");
        let main_rs = files
            .get("src/main.rs")
            .expect("main.rs should be rendered");

        assert!(main_rs.contains("gpioa.pa3.into_push_pull_output(&mut gpioa.crl)"));
        assert!(main_rs.contains("gpioa.pa4.into_push_pull_output(&mut gpioa.crl)"));
        assert!(main_rs.contains("w5500_cs_0.set_high();"));
        assert!(!main_rs.contains("w5500_rst_0 = gpioa.pa3.into_push_pull_output();"));
        assert_generic_tcp_server(&main_rs);
    }

    #[test]
    fn render_keeps_generic_w5500_tcp_path_for_f4() {
        let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
        let mut config = Config::new(board);
        config
            .add_spi_bus(SpiConfig {
                bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
                frequency_mhz: 10,
                mode: SpiMode::Mode0,
                sck: ChosenPin::StmF401(StmF401Pin::A5),
                miso: Some(ChosenPin::StmF401(StmF401Pin::A6)),
                mosi: Some(ChosenPin::StmF401(StmF401Pin::A7)),
            })
            .unwrap();

        config
            .add_peripheral(Peripheral::W5500(W5500Config {
                spi_bus: ChosenSpiBus::StmF401(StmF401SpiBus::SPI1),
                cs: ChosenPin::StmF401(StmF401Pin::A4),
                rst: ChosenPin::StmF401(StmF401Pin::A3),
                network: NetworkConfig {
                    mac: MacAddr([0x02, 0x00, 0x00, 11, 22, 34]),
                    ip: Ipv4Addr::new(192, 168, 1, 51),
                    subnet: Ipv4Addr::new(255, 255, 255, 0),
                    gateway: Ipv4Addr::new(192, 168, 1, 1),
                },
                socket_mode: SocketMode::TcpServer {
                    port: 8081,
                    socket_num: 1,
                },
            }))
            .unwrap();

        let context =
            TemplateContext::from_config(&config, "black_pill_w5500".to_string()).unwrap();
        let files = render(&context).expect("F4 W5500 templates should render");
        let main_rs = files
            .get("src/main.rs")
            .expect("main.rs should be rendered");

        assert!(main_rs.contains("gpioa.pa3.into_push_pull_output();"));
        assert!(main_rs.contains("gpioa.pa4.into_push_pull_output();"));
        assert!(!main_rs.contains("into_push_pull_output(&mut gpioa.crl)"));
        assert_generic_tcp_server(&main_rs);
    }

    fn assert_generic_tcp_server(main_rs: &str) {
        assert!(main_rs.contains("tcp_listen"));
        assert!(main_rs.contains("SocketStatus::Established | SocketStatus::CloseWait"));
        assert!(main_rs.contains("tcp_read"));
        assert!(main_rs.contains("SocketStatus::TimeWait"));
        assert!(main_rs.contains("Application hook"));
        assert!(!main_rs.contains("LED ON"));
        assert!(!main_rs.contains("LED OFF"));
        assert!(!main_rs.contains("my_pin"));
    }

    #[test]
    fn render_keeps_stm32f4_blocks_for_black_pill() {
        let board = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401).unwrap();
        let config = Config::new(board);
        let context = TemplateContext::from_config(&config, "black_pill".to_string()).unwrap();

        let files = render(&context).expect("F4 templates should render");
        let main_rs = files
            .get("src/main.rs")
            .expect("main.rs should be rendered");

        assert!(main_rs.contains("stm32f4xx_hal"));
        assert!(main_rs.contains("let mut rcc = dp.RCC.constrain();"));
        assert!(!main_rs.contains("stm32f1xx_hal"));
    }
}
