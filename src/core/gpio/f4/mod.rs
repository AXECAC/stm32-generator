pub mod f401;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmF4PinMode {
    Input(StmF4InputMode),
    Output(StmF4OutputMode, StmF4OutputSpeed),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmF4InputMode {
    Floating,

    PullUp,

    PullDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmF4OutputSpeed {
    /// ~4 МГц. Для медленных сигналов.
    Low,

    /// ~25 МГц. Для общего применения.
    Medium,

    /// ~50 МГц. Для быстрого SPI, SDIO.
    High,

    /// ~100 МГц. Для LTDC, FMC, USB HS.
    /// На Black Pill используйте с осторожностью - плата не рассчитана на такие частоты.
    VeryHigh,
}

/// Тип выхода GPIO на STM32F4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmF4OutputMode {
    /// Активно управляет и высоким, и низким уровнем.
    /// Стандартный режим для большинства задач.
    PushPull,

    /// Активно управляет только низким уровнем.
    /// Высокий уровень - через внешний pull-up резистор.
    /// Используется для I2C (SDA, SCL).
    OpenDrain,
}
