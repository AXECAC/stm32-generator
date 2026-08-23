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

    // Блоки MCU (STM32F1)
    env.add_template(
        "blocks/mcu/stm32f1/imports.rs.j2",
        templates::MCU_STM32F1_IMPORTS,
    )?;
    env.add_template("blocks/mcu/stm32f1/init.rs.j2", templates::MCU_STM32F1_INIT)?;
    env.add_template("blocks/mcu/stm32f1/gpio.rs.j2", templates::MCU_STM32F1_GPIO)?;

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
    use super::render;
    use crate::core::boards::{TargetBoard, TargetBoardId};
    use crate::core::config::{Config, SpiConfig, SpiMode};
    use crate::core::generator::context::TemplateContext;
    use crate::core::gpio::ChosenPin;
    use crate::core::gpio::ChosenSpiBus;
    use crate::core::gpio::TargetMcu;
    use crate::core::gpio::f1::f103::{StmF103Pin, StmF103SpiBus};

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
