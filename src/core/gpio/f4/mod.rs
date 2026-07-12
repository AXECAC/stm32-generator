use strum::{EnumString, IntoStaticStr, VariantNames};
pub mod f401;

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames, IntoStaticStr)]
pub enum StmF4PinMode {
    #[strum(to_string = "Input")]
    Input(StmF4InputMode),
    #[strum(to_string = "Onput")]
    Output(StmF4OutputMode, StmF4OutputSpeed),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr)]
pub enum StmF4InputMode {
    Floating,

    #[strum(to_string = "Pull up")]
    PullUp,

    #[strum(to_string = "Pull down")]
    PullDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr)]
pub enum StmF4OutputSpeed {
    /// ~4 МГц. Для медленных сигналов.
    Low,

    /// ~25 МГц. Для общего применения.
    Medium,

    /// ~50 МГц. Для быстрого SPI, SDIO.
    High,

    /// ~100 МГц. Для LTDC, FMC, USB HS.
    /// На Black Pill используйте с осторожностью - плата не рассчитана на такие частоты.
    #[strum(to_string = "Very high")]
    VeryHigh,
}

/// Тип выхода GPIO на STM32F4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr)]
pub enum StmF4OutputMode {
    /// Активно управляет и высоким, и низким уровнем.
    /// Стандартный режим для большинства задач.
    #[strum(to_string = "Push pull")]
    PushPull,

    /// Активно управляет только низким уровнем.
    /// Высокий уровень - через внешний pull-up резистор.
    /// Используется для I2C (SDA, SCL).
    #[strum(to_string = "Open drain")]
    OpenDrain,
}
