use strum::{EnumString, IntoStaticStr, VariantNames};
use serde::Serialize;
pub mod f401;

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF4PinMode {
    #[strum(to_string = "Input")]
    Input(StmF4InputMode),
    #[strum(to_string = "Onput")]
    Output(StmF4OutputMode, StmF4OutputSpeed),
}

impl StmF4PinMode {
    pub fn template_vars(&self) -> (&'static str, bool, Option<&'static str>) {
        match self {
            Self::Input(input_mode) => (input_mode.method_name(), false, None),
            Self::Output(out_mode, out_speed) => (out_mode.method_name(), true, Some(out_speed.speed_name())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF4InputMode {
    Floating,

    #[strum(to_string = "Pull up")]
    PullUp,

    #[strum(to_string = "Pull down")]
    PullDown,
}

impl StmF4InputMode {
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::Floating => "into_floating_input",
            Self::PullUp => "into_pull_up_input",
            Self::PullDown => "into_pull_down_input",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
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

impl StmF4OutputSpeed {
    pub fn speed_name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::VeryHigh => "VeryHigh",
        }
    }
}

/// Тип выхода GPIO на STM32F4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize)]
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

impl StmF4OutputMode {
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::PushPull => "into_push_pull_output",
            Self::OpenDrain => "into_open_drain_output",
        }
    }
}
