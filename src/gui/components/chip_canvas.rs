use gtk::cairo;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use crate::core::board::{Pin, PinType};
use crate::gui::colors;

macro_rules! set_color {
    ($ctx:expr, $color:expr) => {
        $ctx.set_source_rgb($color.0, $color.1, $color.2)
    };
}

/// Внутреннее состояние холста, хранящее информацию для отрисовки:
/// название чипа, список пинов с их статусом, и выбранный пин.
struct DrawingState {
    pub chip_label: String,
    pub pins: Vec<(Pin, Option<String>, bool)>,
    pub selected_pin_key: Option<String>,
}

/// Структура с параметрами компоновки (размерами и координатами) для всего чипа.
struct ChipLayout {
    pins_per_side: usize,
    pin_length: f64,
    pin_thickness: f64,
    chip_x: f64,
    chip_y: f64,
    chip_w: f64,
    chip_h: f64,
    cx: f64,
    cy: f64,
}

/// Структура с рассчитанными координатами и размерами прямоугольника для одного пина.
struct PinRect {
    side: usize,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Модель компонента холста микроконтроллера.
pub struct ChipCanvasModel {
    state: Rc<RefCell<DrawingState>>,
    area_tracker: Rc<RefCell<Option<gtk::DrawingArea>>>,
}

/// Обновление пинов, сброс выбора, клики мыши.
#[derive(Debug)]
pub enum ChipCanvasInput {
    UpdatePins(Vec<(Pin, Option<String>, bool)>),
    ClearSelection,
    HandleClick(f64, f64, f64, f64),
}

/// Уведомления о выборе пина.
#[derive(Debug)]
pub enum ChipCanvasOutput {
    PinSelected(String),
}

#[relm4::component(pub)]
impl SimpleComponent for ChipCanvasModel {
    type Init = (String, Vec<(Pin, Option<String>, bool)>);
    type Input = ChipCanvasInput;
    type Output = ChipCanvasOutput;

    view! {
        #[name = "drawing_area"]
        gtk::DrawingArea {
            set_hexpand: true,
            set_vexpand: true,

            add_controller = gtk::GestureClick {
                connect_pressed[sender, drawing_area] => move |_gesture, _n_press, x, y| {
                    let w = drawing_area.width() as f64;
                    let h = drawing_area.height() as f64;
                    sender.input(ChipCanvasInput::HandleClick(x, y, w, h));
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let state = Rc::new(RefCell::new(DrawingState {
            chip_label: init.0,
            pins: init.1,
            selected_pin_key: None,
        }));

        let model = ChipCanvasModel {
            state: state.clone(),
            area_tracker: Rc::new(RefCell::new(None)),
        };

        let widgets = view_output!();
        *model.area_tracker.borrow_mut() = Some(widgets.drawing_area.clone());

        // Первоначальный запрос размера
        let (total_size, _) = Self::get_layout(&state.borrow(), 0.0, 0.0);
        widgets
            .drawing_area
            .set_size_request(total_size as i32, total_size as i32);

        widgets
            .drawing_area
            .set_draw_func(move |_area, ctx, width, height| {
                if let Err(e) = Self::draw_canvas(&state.borrow(), ctx, width as f64, height as f64) {
                    eprintln!("Canvas draw error: {}", e);
                }
            });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        let mut redraw = false;
        let mut request_resize = false;

        match message {
            ChipCanvasInput::UpdatePins(pins) => {
                self.state.borrow_mut().pins = pins;
                redraw = true;
                request_resize = true;
            }
            ChipCanvasInput::ClearSelection => {
                self.state.borrow_mut().selected_pin_key = None;
                redraw = true;
            }
            ChipCanvasInput::HandleClick(x, y, w, h) => {
                let state = self.state.borrow();
                let (_, layout) = Self::get_layout(&state, w, h);

                let mut closest_pin = None;

                for (i, (pin, _, _)) in state.pins.iter().enumerate() {
                    if matches!(pin.pin_type, PinType::Power) {
                        continue;
                    }

                    let rect = Self::calculate_pin_rect(i, &layout);

                    // Проверяем попадание в прямоугольник
                    if x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h {
                        closest_pin = Some(pin.key.clone());
                        break;
                    }
                }

                drop(state);

                if let Some(key) = closest_pin {
                    self.state.borrow_mut().selected_pin_key = Some(key.clone());
                    sender
                        .output(ChipCanvasOutput::PinSelected(key))
                        .expect("Failed to emit PinSelected output message from ChipCanvasModel");
                    redraw = true;
                }
            }
        }

        if let Some(area) = &*self.area_tracker.borrow() {
            if request_resize {
                let (total_size, _) = Self::get_layout(&self.state.borrow(), 0.0, 0.0);
                area.set_size_request(total_size as i32, total_size as i32);
            }
            if redraw {
                area.queue_draw();
            }
        }
    }
}

impl ChipCanvasModel {
    /// Вычисляет и возвращает полные параметры компоновки чипа на основе состояния и размеров холста.
    fn get_layout(state: &DrawingState, w: f64, h: f64) -> (f64, ChipLayout) {
        let (total_size, pin_length, pin_thickness, pins_per_side) = Self::calculate_layout(state);
        let chip_w = pins_per_side as f64 * pin_thickness;
        let chip_h = pins_per_side as f64 * pin_thickness;

        let cx = w / 2.0;
        let cy = h / 2.0;
        let chip_x = cx - chip_w / 2.0;
        let chip_y = cy - chip_h / 2.0;

        (total_size, ChipLayout {
            pins_per_side, pin_length, pin_thickness, chip_x, chip_y, chip_w, chip_h, cx, cy
        })
    }

    /// Главная функция отрисовки всего холста. Очищает фон и вызывает остальные функции отрисовки.
    fn draw_canvas(state: &DrawingState, ctx: &cairo::Context, w: f64, h: f64) -> Result<(), cairo::Error> {
        let (_, layout) = Self::get_layout(state, w, h);

        // Background
        set_color!(ctx, colors::BG);
        ctx.paint()?;

        Self::draw_chip_body(state, ctx, &layout)?;

        for (i, (pin, alias, is_configured)) in state.pins.iter().enumerate() {
            let rect = Self::calculate_pin_rect(i, &layout);
            Self::draw_pin(state, ctx, pin, alias.as_deref(), *is_configured, &rect)?;
        }

        Ok(())
    }

    /// Рисует центральный квадрат (тело чипа) и его название.
    fn draw_chip_body(
        state: &DrawingState,
        ctx: &cairo::Context,
        layout: &ChipLayout,
    ) -> Result<(), cairo::Error> {
        // Тело чипа (фон)
        set_color!(ctx, colors::CHIP_BG);
        ctx.rectangle(layout.chip_x, layout.chip_y, layout.chip_w, layout.chip_h);
        ctx.fill()?;

        // Обводка чипа
        set_color!(ctx, colors::CHIP_BORDER);
        ctx.set_line_width(2.0);
        ctx.rectangle(layout.chip_x, layout.chip_y, layout.chip_w, layout.chip_h);
        ctx.stroke()?;

        // Текст по центру
        set_color!(ctx, colors::CHIP_TEXT);
        ctx.select_font_face(
            "Monospace",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Bold,
        );
        ctx.set_font_size(28.0);
        let extents = ctx.text_extents(&state.chip_label)?;
        ctx.move_to(layout.cx - extents.width() / 2.0, layout.cy + extents.height() / 2.0);
        ctx.show_text(&state.chip_label)?;

        Ok(())
    }

    /// Отрисовывает конкретную ножку (пин) со всеми цветами, обводками и текстом.
    fn draw_pin(
        state: &DrawingState,
        ctx: &cairo::Context,
        pin: &Pin,
        alias: Option<&str>,
        is_configured: bool,
        rect: &PinRect,
    ) -> Result<(), cairo::Error> {
        let is_selected = Some(&pin.key) == state.selected_pin_key.as_ref();
        let is_power = matches!(pin.pin_type, PinType::Power);

        // Цвет ножки
        if is_selected {
            set_color!(ctx, colors::PIN_SELECTED);
        } else if is_power {
            set_color!(ctx, colors::PIN_POWER);
        } else if is_configured {
            set_color!(ctx, colors::PIN_CONFIGURED);
        } else {
            set_color!(ctx, colors::PIN_DEFAULT);
        }

        ctx.rectangle(rect.x, rect.y, rect.w, rect.h);
        ctx.fill()?;

        // Обводка ножки
        if is_selected {
            set_color!(ctx, colors::BORDER_SELECTED);
            ctx.set_line_width(2.0);
        } else {
            set_color!(ctx, colors::BORDER_DEFAULT);
            ctx.set_line_width(1.0);
        }
        ctx.rectangle(rect.x, rect.y, rect.w, rect.h);
        ctx.stroke()?;

        Self::draw_pin_label(ctx, pin, is_selected, rect)?;

        if let Some(a) = alias {
            Self::draw_alias_label(ctx, a, rect)?;
        }

        Ok(())
    }

    /// Рисует текст внутри прямоугольника пина.
    fn draw_pin_label(
        ctx: &cairo::Context,
        pin: &Pin,
        is_selected: bool,
        rect: &PinRect,
    ) -> Result<(), cairo::Error> {
        ctx.save()?;

        if is_selected {
            set_color!(ctx, colors::TEXT_SELECTED);
            ctx.set_font_size(14.0);
            ctx.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        } else {
            set_color!(ctx, colors::TEXT_DEFAULT);
            ctx.set_font_size(12.0);
            ctx.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        }

        let text = pin.label.clone();
        let extents = ctx.text_extents(&text)?;

        let center_x = rect.x + rect.w / 2.0;
        let center_y = rect.y + rect.h / 2.0;

        match rect.side {
            0 | 2 => {
                ctx.move_to(
                    center_x - extents.width() / 2.0,
                    center_y + extents.height() / 2.0,
                );
                ctx.show_text(&text)?;
            }
            1 | 3 => {
                ctx.translate(center_x, center_y);
                ctx.rotate(-PI / 2.0);
                ctx.move_to(-extents.width() / 2.0, extents.height() / 2.0);
                ctx.show_text(&text)?;
            }
            _ => unreachable!(),
        }
        ctx.restore()?;
        Ok(())
    }

    /// Рисует пользовательский алиас (имя переменной) за пределами прямоугольника пина.
    fn draw_alias_label(
        ctx: &cairo::Context,
        alias: &str,
        rect: &PinRect,
    ) -> Result<(), cairo::Error> {
        ctx.save()?;
        set_color!(ctx, colors::TEXT_ALIAS);
        ctx.set_font_size(12.0);
        ctx.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        let a_ext = ctx.text_extents(alias)?;

        let center_x = rect.x + rect.w / 2.0;
        let center_y = rect.y + rect.h / 2.0;

        match rect.side {
            0 => {
                ctx.move_to(
                    rect.x - a_ext.width() - 8.0,
                    center_y + a_ext.height() / 2.0,
                );
                ctx.show_text(alias)?;
            }
            2 => {
                ctx.move_to(
                    rect.x + rect.w + 8.0,
                    center_y + a_ext.height() / 2.0,
                );
                ctx.show_text(alias)?;
            }
            1 => {
                ctx.translate(center_x, rect.y + rect.h + 8.0 + a_ext.width());
                ctx.rotate(-PI / 2.0);
                ctx.move_to(0.0, a_ext.height() / 2.0);
                ctx.show_text(alias)?;
            }
            3 => {
                ctx.translate(center_x, rect.y - 8.0);
                ctx.rotate(-PI / 2.0);
                ctx.move_to(0.0, a_ext.height() / 2.0);
                ctx.show_text(alias)?;
            }
            _ => unreachable!(),
        }
        ctx.restore()?;
        Ok(())
    }

    /// Рассчитывает координаты и размеры прямоугольника для ножки пина в зависимости от ее индекса.
    fn calculate_pin_rect(i: usize, layout: &ChipLayout) -> PinRect {
        let side = i / layout.pins_per_side;
        let idx = i % layout.pins_per_side;

        let (rect_x, rect_y, rect_w, rect_h) = match side {
            0 => (
                layout.chip_x - layout.pin_length,
                layout.chip_y + idx as f64 * layout.pin_thickness,
                layout.pin_length,
                layout.pin_thickness,
            ),
            1 => (
                layout.chip_x + idx as f64 * layout.pin_thickness,
                layout.chip_y + layout.chip_h,
                layout.pin_thickness,
                layout.pin_length,
            ),
            2 => (
                layout.chip_x + layout.chip_w,
                layout.chip_y + layout.chip_h - (idx as f64 + 1.0) * layout.pin_thickness,
                layout.pin_length,
                layout.pin_thickness,
            ),
            3 => (
                layout.chip_x + layout.chip_w - (idx as f64 + 1.0) * layout.pin_thickness,
                layout.chip_y - layout.pin_length,
                layout.pin_thickness,
                layout.pin_length,
            ),
            _ => unreachable!(),
        };

        PinRect {
            side,
            x: rect_x,
            y: rect_y,
            w: rect_w,
            h: rect_h,
        }
    }

    /// Оценивает общие размеры компоновки: общий требуемый размер виджета, длину/ширину пина и количество пинов на сторону.
    fn calculate_layout(state: &DrawingState) -> (f64, f64, f64, usize) {
        let mut max_len = 0;
        let mut max_alias_len = 0;
        for (pin, alias, _) in &state.pins {
            if pin.label.chars().count() > max_len {
                max_len = pin.label.chars().count();
            }
            if let Some(a) = alias
                && a.chars().count() > max_alias_len
            {
                max_alias_len = a.chars().count();
            }
        }

        // Примерная ширина шрифта 14px -> ~9px на символ. + отступы.
        let max_text_width = max_len as f64 * 8.5;
        let pin_length = max_text_width + 16.0; // длина ножки
        let pin_thickness = 26.0; // ширина ножки вдоль чипа

        let pins_per_side = state.pins.len().div_ceil(4);
        let chip_size = pins_per_side as f64 * pin_thickness;

        // Учитываем размер алиасов за пределами ножки при расчете общего размера
        let alias_margin = max_alias_len as f64 * 8.5 + 16.0;
        let total_size = chip_size + pin_length * 2.0 + alias_margin * 2.0 + 80.0; // + padding

        (total_size, pin_length, pin_thickness, pins_per_side)
    }
}
