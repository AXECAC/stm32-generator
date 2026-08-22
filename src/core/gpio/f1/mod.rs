use crate::core::gpio::PinModeUiInfo;
use serde::Serialize;
use strum::{EnumString, FromRepr, IntoStaticStr, VariantNames};

pub mod f103;

/// Режимы GPIO для STM32F1.
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
            Self::Output(output_mode, output_speed) => (
                output_mode.method_name(),
                true,
                Some((*output_speed).into()),
            ),
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
            1 => *self = Self::Output(StmF1OutputMode::PushPull, StmF1OutputSpeed::Mhz2),
            _ => {}
        }
    }

    fn properties(&self) -> Vec<(&'static str, Vec<&'static str>, usize)> {
        match self {
            Self::Input(input_mode) => vec![(
                "Режим входа",
                StmF1InputMode::VARIANTS.to_vec(),
                *input_mode as usize,
            )],
            Self::Output(output_mode, output_speed) => vec![
                (
                    "Тип выхода",
                    StmF1OutputMode::VARIANTS.to_vec(),
                    *output_mode as usize,
                ),
                (
                    "Скорость",
                    StmF1OutputSpeed::VARIANTS.to_vec(),
                    *output_speed as usize,
                ),
            ],
        }
    }

    fn set_property(&mut self, prop_idx: usize, variant_idx: usize) {
        match self {
            Self::Input(input_mode) => {
                if prop_idx == 0
                    && let Some(new_mode) = StmF1InputMode::from_repr(variant_idx as u32)
                {
                    *input_mode = new_mode;
                }
            }
            Self::Output(output_mode, output_speed) => {
                if prop_idx == 0 {
                    if let Some(new_mode) = StmF1OutputMode::from_repr(variant_idx as u32) {
                        *output_mode = new_mode;
                    }
                } else if prop_idx == 1
                    && let Some(new_speed) = StmF1OutputSpeed::from_repr(variant_idx as u32)
                {
                    *output_speed = new_speed;
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize, FromRepr,
)]
#[repr(u32)]
pub enum StmF1OutputSpeed {
    #[strum(to_string = "2 MHz")]
    Mhz2,

    #[strum(to_string = "10 MHz")]
    Mhz10,

    #[strum(to_string = "50 MHz")]
    Mhz50,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, VariantNames, IntoStaticStr, Serialize, FromRepr,
)]
#[repr(u32)]
pub enum StmF1OutputMode {
    #[strum(to_string = "Push pull")]
    PushPull,

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

#[cfg(test)]
mod tests {
    use super::{StmF1InputMode, StmF1OutputMode, StmF1OutputSpeed, StmF1PinMode};
    use crate::core::gpio::PinModeUiInfo;

    #[test]
    fn default_mode_is_pull_up_input() {
        let mode = StmF1PinMode::default();

        assert_eq!(mode.current_mode_index(), 0);
        assert_eq!(mode.mode_variants(), vec!["Input", "Output"]);
        assert_eq!(mode, StmF1PinMode::Input(StmF1InputMode::PullUp));
    }

    #[test]
    fn output_mode_exposes_type_and_f1_speed_properties() {
        let mut mode = StmF1PinMode::default();
        mode.set_mode_index(1);

        assert_eq!(
            mode,
            StmF1PinMode::Output(StmF1OutputMode::PushPull, StmF1OutputSpeed::Mhz2)
        );
        assert_eq!(mode.properties()[0].0, "Тип выхода");
        assert_eq!(mode.properties()[1].0, "Скорость");

        mode.set_property(0, 1);
        mode.set_property(1, 2);

        assert_eq!(
            mode,
            StmF1PinMode::Output(StmF1OutputMode::OpenDrain, StmF1OutputSpeed::Mhz50)
        );
    }
}
