pub mod f401;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmF4PinMode {
    InputMode(StmF4InputMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmF4InputMode {
    Analog,

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
