#[derive(Copy, Clone, Debug)]
pub struct RgbColor(pub f64, pub f64, pub f64);

impl RgbColor {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        debug_assert!(r >= 0.0 && r <= 1.0);
        debug_assert!(g >= 0.0 && g <= 1.0);
        debug_assert!(b >= 0.0 && b <= 1.0);
        Self(r, g, b)
    }
}

pub const BG: RgbColor = RgbColor::new(0.12, 0.12, 0.12);
pub const CHIP_BG: RgbColor = RgbColor::new(0.2, 0.2, 0.2);
pub const CHIP_BORDER: RgbColor = RgbColor::new(0.4, 0.4, 0.4);
pub const CHIP_TEXT: RgbColor = RgbColor::new(0.9, 0.9, 0.9);

pub const PIN_SELECTED: RgbColor = RgbColor::new(1.0, 0.8, 0.2);
pub const PIN_POWER: RgbColor = RgbColor::new(0.7, 0.3, 0.3);
pub const PIN_CONFIGURED: RgbColor = RgbColor::new(0.3, 0.7, 0.3);
pub const PIN_DEFAULT: RgbColor = RgbColor::new(0.3, 0.5, 0.7);

pub const BORDER_SELECTED: RgbColor = RgbColor::new(1.0, 1.0, 1.0);
pub const BORDER_DEFAULT: RgbColor = RgbColor::new(0.1, 0.1, 0.1);

pub const TEXT_SELECTED: RgbColor = RgbColor::new(0.1, 0.1, 0.1);
pub const TEXT_DEFAULT: RgbColor = RgbColor::new(0.9, 0.9, 0.9);
pub const TEXT_ALIAS: RgbColor = RgbColor::new(0.4, 0.8, 0.4);
