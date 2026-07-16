use crate::core::errors::GeneratorError;
use minijinja::Environment;
use std::collections::HashMap;

pub mod context;
pub mod templates;

fn build_environment() -> Result<Environment<'static>, GeneratorError> {
    let mut env = Environment::new();

    // Точки входа
    env.add_template("Cargo.toml.j2", templates::CARGO_TOML)?;
    env.add_template(".cargo/config.toml.j2", templates::CARGO_CONFIG_TOML)?;
    env.add_template("memory.x.j2", templates::MEMORY_X)?;
    env.add_template("justfile.j2", templates::JUSTFILE)?;
    env.add_template("main.rs.j2", templates::MAIN_RS)?;

    // Блоки MCU
    env.add_template(
        "blocks/mcu/stm32f4/imports.rs.j2",
        templates::MCU_STM32F4_IMPORTS,
    )?;
    env.add_template("blocks/mcu/stm32f4/init.rs.j2", templates::MCU_STM32F4_INIT)?;
    env.add_template("blocks/mcu/stm32f4/gpio.rs.j2", templates::MCU_STM32F4_GPIO)?;

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
