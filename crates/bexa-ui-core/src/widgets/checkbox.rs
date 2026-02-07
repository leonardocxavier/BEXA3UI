use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::icons;
use crate::signal::{Signal, SetSignal};

pub struct Checkbox {
    label: String,
    checked: Signal<bool>,
    set_checked: SetSignal<bool>,
    metrics: Metrics,
    box_size: f32,
    gap: f32,
    border_radius: f32,
    // Colors
    box_bg: [f32; 4],
    box_checked_bg: [f32; 4],
    box_border: [f32; 4],
    check_color: [u8; 3],
    text_color: [u8; 3],
    // State
    hover: bool,
    focus: bool,
}

impl Checkbox {
    pub fn new(
        label: impl Into<String>,
        checked: Signal<bool>,
        set_checked: SetSignal<bool>,
        metrics: Metrics,
    ) -> Self {
        Self {
            label: label.into(),
            checked,
            set_checked,
            metrics,
            box_size: 20.0,
            gap: 8.0,
            border_radius: 4.0,
            box_bg: [0.16, 0.28, 0.38, 1.0],
            box_checked_bg: [0.20, 0.65, 0.85, 1.0],
            box_border: [0.4, 0.55, 0.7, 1.0],
            check_color: [255, 255, 255],
            text_color: [230, 230, 230],
            hover: false,
            focus: false,
        }
    }

    pub fn with_box_size(mut self, size: f32) -> Self {
        self.box_size = size;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn with_colors(
        mut self,
        box_bg: [f32; 4],
        box_checked_bg: [f32; 4],
        box_border: [f32; 4],
        check_color: [u8; 3],
    ) -> Self {
        self.box_bg = box_bg;
        self.box_checked_bg = box_checked_bg;
        self.box_border = box_border;
        self.check_color = check_color;
        self
    }

    pub fn with_text_color(mut self, color: [u8; 3]) -> Self {
        self.text_color = color;
        self
    }

    fn toggle(&self) {
        let current = self.checked.get();
        self.set_checked.set(!current);
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }
}

impl Widget for Checkbox {
    fn style(&self) -> Style {
        let height = self.box_size.max(self.metrics.line_height) + 8.0;
        Style {
            size: Size {
                width: Dimension::Auto,
                height: Dimension::Length(height),
            },
            flex_shrink: 0.0,
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let is_checked = self.checked.get();

        // 1. Draw the box
        let box_x = layout.location.x + 4.0;
        let box_y = layout.location.y + (layout.size.height - self.box_size) / 2.0;
        let bg = if is_checked { self.box_checked_bg } else { self.box_bg };
        let border_w = if self.focus { 2.0 } else { 1.0 };
        let border_c = if self.focus {
            [0.3, 0.6, 0.9, 1.0]
        } else {
            self.box_border
        };
        ctx.renderer.fill_rect_styled(
            (box_x, box_y, self.box_size, self.box_size),
            bg,
            self.border_radius,
            border_w,
            border_c,
        );

        // 2. Draw checkmark icon if checked
        if is_checked {
            let icon_size = self.box_size * 0.7;
            let icon_metrics = Metrics::new(icon_size, icon_size);
            let icon_x = box_x + (self.box_size - icon_size) / 2.0;
            let icon_y = box_y + (self.box_size - icon_size) / 2.0;
            ctx.renderer.draw_text_with_font(
                icons::CHECK,
                (icon_x, icon_y),
                self.check_color,
                (icon_size, icon_size),
                icon_metrics,
                Align::Center,
                icons::NERD_FONT_FAMILY,
            );
        }

        // 3. Draw label text
        let text_x = box_x + self.box_size + self.gap;
        let text_y = layout.location.y + (layout.size.height - self.metrics.line_height) / 2.0;
        let text_w = (layout.size.width - (text_x - layout.location.x)).max(0.0);
        ctx.renderer.draw_text(
            &self.label,
            (text_x, text_y),
            self.text_color,
            (text_w, self.metrics.line_height),
            self.metrics,
            Align::Left,
        );
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;
        let mut changed = false;
        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let over = self.hit_test(layout, position.x as f32, position.y as f32);
                if over != self.hover {
                    self.hover = over;
                    changed = true;
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.hover {
                    self.toggle();
                    changed = true;
                }
            }
            _ => {}
        }
        changed
    }

    fn handle_key_event(&mut self, event: &KeyEvent, _modifiers: ModifiersState) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        match &event.logical_key {
            Key::Named(NamedKey::Space) | Key::Named(NamedKey::Enter) => {
                self.toggle();
                true
            }
            _ => false,
        }
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focus = focused;
    }

    fn activate(&mut self) {
        self.toggle();
    }
}
