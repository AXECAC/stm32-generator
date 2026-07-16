// Точки входа
pub(crate) const CARGO_TOML: &str = include_str!("../../../assets/templates/Cargo.toml.j2");
pub(crate) const CARGO_CONFIG_TOML: &str = include_str!("../../../assets/templates/.cargo/config.toml.j2");
pub(crate) const MEMORY_X: &str = include_str!("../../../assets/templates/memory.x.j2");
pub(crate) const JUSTFILE: &str = include_str!("../../../assets/templates/justfile.j2");
pub(crate) const MAIN_RS: &str = include_str!("../../../assets/templates/main.rs.j2");
pub(crate) const GITIGNORE: &str = include_str!("../../../assets/templates/.gitignore.j2");

// Блоки MCU (STM32F4)
pub(crate) const MCU_STM32F4_IMPORTS: &str = include_str!("../../../assets/templates/blocks/mcu/stm32f4/imports.rs.j2");
pub(crate) const MCU_STM32F4_INIT: &str = include_str!("../../../assets/templates/blocks/mcu/stm32f4/init.rs.j2");
pub(crate) const MCU_STM32F4_GPIO: &str = include_str!("../../../assets/templates/blocks/mcu/stm32f4/gpio.rs.j2");

// Блоки Периферии (W5500)
pub(crate) const PERIPHERAL_W5500_IMPORTS: &str = include_str!("../../../assets/templates/blocks/peripherals/W5500/imports.rs.j2");
pub(crate) const PERIPHERAL_W5500_INIT: &str = include_str!("../../../assets/templates/blocks/peripherals/W5500/init.rs.j2");
pub(crate) const PERIPHERAL_W5500_LOGIC_SINGLE: &str = include_str!("../../../assets/templates/blocks/peripherals/W5500/logic_single.rs.j2");
pub(crate) const PERIPHERAL_W5500_LOGIC_BRIDGE: &str = include_str!("../../../assets/templates/blocks/peripherals/W5500/logic_bridge.rs.j2");
