use crate::core::gpio::PinModeUiInfo;
use serde::Serialize;
use strum::{EnumString, FromRepr, IntoStaticStr, VariantNames};
pub mod f103;

/// Режим пина GPIO на STM32F1.
///
/// В отличие от STM32F4, серия F1 использует CRL/CRH регистры для конфигурации.
/// Каждый пин настраивается через 4 бита (CNF + MODE), а не через MODER/OTYPER/OSPEEDR,
/// как в F4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames, IntoStaticStr, Serialize)]
pub enum StmF1PinMode {
    #[strum(to_string = "Input")]
    Input(StmF1InputMode),
    #[strum(to_string = "Output")]
    Output(StmF1OutputMode, StmF1OutputSpeed),
}

impl Default for StmF1PinMode {
    fn default() -> Self {
        Self::Input(StmF1InputMode::PullUp)
    }
}

impl StmF1PinMode {
    pub fn template_vars(&self) -> (&'static str, bool, Option<&'static str>) {
        match self {
            Self::Input(input_mode) => (input_mode.method_name(), false, None),
            Self::Output(out_mode, out_speed) => {
                (out_mode.method_name(), true, Some(out_speed.into()))
            }
        }
    }
}

impl PinModeUiInfo for StmF1PinMode {
    fn mode_variants(&self) -> Vec<&'static str> {
        Self::VARIANTS.to_vec()
    }

    fn current_mode_index(&self) -> usize {
        match self {
            Self::Input(_) => 0,
            Self::Output(_, _) => 1,
        }
    }

    fn set_mode_index(&mut self, idx: usize) {
        match idx {
            0 => *self = Self::Input(StmF1InputMode::PullUp),
            1 => *self = Self::Output(StmF1OutputMode::PushPull, StmF1OutputSpeed::Max2MHz),
            _ => {}
        }
    }

    fn properties(&self) -> Vec<(&'static str, Vec<&'static str>, usize)> {
        match self {
            Self::Input(i) => vec![(
                "Режим входа",
                StmF1InputMode::VARIANTS.to_vec(),
                *i as usize,
            )],
            Self::Output(m, s) => vec![
                (
                    "Тип выхода",
                    StmF1OutputMode::VARIANTS.to_vec(),
                    *m as usize,
                ),
                ("Скорость", StmF1OutputSpeed::VARIANTS.to_vec(), *s as usize),
            ],
        }
    }

    fn set_property(&mut self, prop_idx: usize, variant_idx: usize) {
        match self {
            Self::Input(i) => {
                if prop_idx == 0
                    && let Some(new_i) = StmF1InputMode::from_repr(variant_idx as u32)
                {
                    *i = new_i;
                }
            }
            Self::Output(m, s) => {
                if prop_idx == 0 {
                    if let Some(new_m) = StmF1OutputMode::from_repr(variant_idx as u32) {
                        *m = new_m;
                    }
                } else if prop_idx == 1
                    && let Some(new_s) = StmF1OutputSpeed::from_repr(variant_idx as u32)
                {
                    *s = new_s;
                }
            }
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize, FromRepr,
)]
#[repr(u32)]
pub enum StmF1InputMode {
    Floating,

    #[strum(to_string = "Pull up")]
    PullUp,

    #[strum(to_string = "Pull down")]
    PullDown,
}

impl StmF1InputMode {
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::Floating => "into_floating_input",
            Self::PullUp => "into_pull_up_input",
            Self::PullDown => "into_pull_down_input",
        }
    }
}

/// Скорость выхода GPIO на STM32F1.
///
/// В отличие от F4, серия F1 имеет только три варианта скорости:
/// 2 МГц, 10 МГц, 50 МГц. Скорости определяются битами MODE[1:0]
/// в регистрах CRL/CRH.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize, FromRepr,
)]
#[repr(u32)]
pub enum StmF1OutputSpeed {
    /// ~2 МГц.
    #[strum(to_string = "2 MHz")]
    Max2MHz,

    /// ~10 МГц.
    #[strum(to_string = "10 MHz")]
    Max10MHz,

    /// ~50 МГц.
    #[strum(to_string = "50 MHz")]
    Max50MHz,
}

/// Тип выхода GPIO на STM32F1.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize, FromRepr,
)]
#[repr(u32)]
pub enum StmF1OutputMode {
    /// Активно управляет и высоким, и низким уровнем.
    #[strum(to_string = "Push pull")]
    PushPull,

    /// Активно управляет только низким уровнем.
    /// Высокий уровень - через внешний pull-up резистор.
    #[strum(to_string = "Open drain")]
    OpenDrain,
}

impl StmF1OutputMode {
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::PushPull => "into_push_pull_output",
            Self::OpenDrain => "into_open_drain_output",
        }
    }
}
