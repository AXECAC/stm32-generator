use eframe::egui::{self, Color32, Rect, Stroke, StrokeKind, Vec2};

use crate::core::board::{Pin, PinType};
use crate::core::gpio::ChosenPin;

#[derive(PartialEq)]
pub enum PinState {
    Available,
    Configured,
    UsedByPeripheral,
}

pub fn compute_pin_state(
    chosen: &ChosenPin,
    all_used: &[ChosenPin],
    gpio_configured: &[ChosenPin],
) -> PinState {
    if gpio_configured.contains(chosen) {
        PinState::Configured
    } else if all_used.contains(chosen) {
        PinState::UsedByPeripheral
    } else {
        PinState::Available
    }
}

fn pin_circle_color(state: &PinState, is_gpio: bool, label: &str) -> Color32 {
    if !is_gpio {
        if label.contains("GND") || label.contains("VSS") {
            return Color32::from_rgb(30, 30, 30);
        } else if label.contains("V") || label.contains("3V3") || label.contains("5V") {
            // VDD, 3V3, 5V, VBAT
            return Color32::from_rgb(200, 50, 50);
        }
        return Color32::DARK_GRAY;
    }
    match state {
        PinState::Configured => Color32::from_rgb(40, 190, 60),
        PinState::UsedByPeripheral => Color32::from_rgb(220, 140, 20),
        PinState::Available => Color32::from_rgb(140, 140, 155),
    }
}

const PIN_R: f32 = 6.0;
const PIN_SPACING: f32 = 18.0;
const LINE_LEN: f32 = 12.0;

pub struct ChipCanvas<'a> {
    chip_label: String,
    pins: Vec<Pin>,
    all_used_pins: &'a [ChosenPin],
    gpio_configured_pins: &'a [ChosenPin],
    selected_pin: Option<&'a str>,
}

impl<'a> ChipCanvas<'a> {
    pub fn new(chip_label: String, pins: Vec<Pin>) -> Self {
        Self {
            chip_label,
            pins,
            all_used_pins: &[],
            gpio_configured_pins: &[],
            selected_pin: None,
        }
    }

    pub fn with_used_pins(mut self, all: &'a [ChosenPin], gpio: &'a [ChosenPin]) -> Self {
        self.all_used_pins = all;
        self.gpio_configured_pins = gpio;
        self
    }

    pub fn with_selected(mut self, sel: Option<&'a str>) -> Self {
        self.selected_pin = sel;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<String> {
        let total_pins = self.pins.len();
        let pins_per_side = (total_pins as f32 / 4.0).ceil() as usize;

        let chip_size = (pins_per_side.max(1) as f32) * PIN_SPACING + 20.0;
        let canvas_size = chip_size + 140.0;

        let (resp, painter) =
            ui.allocate_painter(Vec2::new(canvas_size, canvas_size), egui::Sense::click());

        let center = resp.rect.center();

        let chip_rect = Rect::from_center_size(center, Vec2::new(chip_size, chip_size));
        painter.rect_filled(chip_rect, 8.0, Color32::from_rgb(40, 40, 45));
        painter.rect_stroke(
            chip_rect,
            8.0,
            Stroke::new(1.5, Color32::from_rgb(100, 100, 110)),
            StrokeKind::Outside,
        );

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            &self.chip_label,
            egui::FontId::monospace(14.0),
            Color32::from_rgb(200, 200, 200),
        );

        painter.circle_filled(
            chip_rect.min + Vec2::new(12.0, 12.0),
            3.0,
            Color32::from_rgb(150, 150, 150),
        );

        let hover_pos = resp.hover_pos();
        let click_pos = if resp.clicked() {
            resp.interact_pointer_pos()
        } else {
            None
        };
        let mut clicked_pin_key = None;

        for (i, pin) in self.pins.iter().enumerate() {
            let side = i / pins_per_side;
            let idx_on_side = i % pins_per_side;

            let start_offset = -((pins_per_side as f32 - 1.0) * PIN_SPACING) / 2.0;
            let offset = start_offset + (idx_on_side as f32 * PIN_SPACING);

            let (edge_pos, ext_pos, label_pos, align) = match side {
                0 => {
                    // Left
                    let p = center + Vec2::new(-chip_size / 2.0, offset);
                    (
                        p,
                        p + Vec2::new(-LINE_LEN, 0.0),
                        p + Vec2::new(-LINE_LEN - 8.0, 0.0),
                        egui::Align2::RIGHT_CENTER,
                    )
                }
                1 => {
                    // Bottom
                    let p = center + Vec2::new(offset, chip_size / 2.0);
                    (
                        p,
                        p + Vec2::new(0.0, LINE_LEN),
                        p + Vec2::new(0.0, LINE_LEN + 8.0),
                        egui::Align2::CENTER_TOP,
                    )
                }
                2 => {
                    // Right
                    let p = center + Vec2::new(chip_size / 2.0, -offset);
                    (
                        p,
                        p + Vec2::new(LINE_LEN, 0.0),
                        p + Vec2::new(LINE_LEN + 8.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                    )
                }
                _ => {
                    // Top (3)
                    let p = center + Vec2::new(-offset, -chip_size / 2.0);
                    (
                        p,
                        p + Vec2::new(0.0, -LINE_LEN),
                        p + Vec2::new(0.0, -LINE_LEN - 8.0),
                        egui::Align2::CENTER_BOTTOM,
                    )
                }
            };

            painter.line_segment(
                [edge_pos, ext_pos],
                Stroke::new(1.5, Color32::from_rgb(150, 150, 150)),
            );

            let is_gpio = matches!(pin.pin_type, PinType::Gpio(_));

            let state = match &pin.pin_type {
                PinType::Gpio(chosen) => {
                    compute_pin_state(chosen, self.all_used_pins, self.gpio_configured_pins)
                }
                PinType::Power => PinState::Available,
            };

            let color = pin_circle_color(&state, is_gpio, &pin.label);

            let is_selected = is_gpio && self.selected_pin == Some(&pin.key);
            let is_hovered = is_gpio
                && hover_pos
                    .map(|p| (p - ext_pos).length() < PIN_R + 4.0)
                    .unwrap_or(false);

            if is_selected {
                painter.circle_filled(
                    ext_pos,
                    PIN_R + 4.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 60),
                );
                painter.circle_stroke(ext_pos, PIN_R + 2.0, Stroke::new(2.0, Color32::WHITE));
            } else if is_hovered {
                painter.circle_filled(
                    ext_pos,
                    PIN_R + 3.0,
                    Color32::from_rgba_unmultiplied(255, 255, 0, 40),
                );
                painter.circle_stroke(ext_pos, PIN_R + 1.0, Stroke::new(1.5, Color32::YELLOW));
            }

            painter.circle_filled(ext_pos, PIN_R, color);
            painter.circle_stroke(ext_pos, PIN_R, Stroke::new(1.0, Color32::BLACK));

            painter.text(
                label_pos,
                align,
                &pin.label,
                egui::FontId::monospace(9.0),
                if is_gpio {
                    Color32::WHITE
                } else {
                    Color32::GRAY
                },
            );

            if is_gpio
                && let Some(cpos) = click_pos
                && (cpos - ext_pos).length() < PIN_R + 4.0
            {
                clicked_pin_key = Some(pin.key.clone());
            }
        }

        clicked_pin_key
    }
}
