use crate::core::errors::GeneratorError;
use crate::core::generator::context::TemplateContext;
use minijinja::Environment;
use std::collections::HashMap;

pub mod context;
pub mod templates;

/// Создает [`minijinja::Environment`] и регистрирует в нём все статические шаблоны.
///
/// Имена шаблонов сохраняют суффикс `.j2`, так как они используются для связи
/// модулей внутри самих шаблонов (через `{% include %}`).
fn build_environment<'a>() -> Result<Environment<'a>, GeneratorError> {
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

/// Финальный путь к файлу в проекте
type ProjectPath = String;

/// Готовый сгенерированный, с помощью [`minijinja`], исходный код
type Code = String;

/// Рендерит все файлы проекта, подставляя переданный `TemplateContext`.
///
/// # Errors
/// Функция вернет ошибку [`GeneratorError::RenderError`], если рендер окажется
/// неудачным (в .j2 файле не закрытая скобка, не правильный формат и тп)
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

    Ok(files)
}
