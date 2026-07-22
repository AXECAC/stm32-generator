use crate::core::board::{Pin, PinType, TargetBoard};
use crate::core::config::{Config, PinConfig};
use crate::core::gpio::f4::{StmF4InputMode, StmF4OutputMode, StmF4OutputSpeed, StmF4PinMode};
use crate::core::gpio::{ChosenPin, ChosenPinWithMode};
use crate::gui::components::chip_canvas::{ChipCanvasInput, ChipCanvasModel, ChipCanvasOutput};
use adw::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw, gtk,
};
use strum::VariantNames;

