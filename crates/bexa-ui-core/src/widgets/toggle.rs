use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::{Signal, SetSignal};

pub struct Toggle {
    label: String,
    checked: Signal<bool>,
    set_checked: SetSignal<bool>,
    metrics: Metrics,
    track_width: f32,
    track_height: f32,
    gap: f32,
    padding: f32,
    // Colors
    track_off: [f32; 4],
    track_on: [f32; 4],
    knob_color: [f32; 4],
    border_color: [f32; 4],
    text_color: [u8; 3],
    // State
    hover: bool,
    focus: bool,
}

impl Toggle {
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
            track_width: 44.0,
            track_height: 22.0,
            gap: 10.0,
            padding: 4.0,
            track_off: [0.16, 0.28, 0.38, 1.0],
            track_on: [0.20, 0.65, 0.45, 1.0],
            knob_color: [0.95, 0.95, 0.97, 1.0],
            border_color: [0.40, 0.55, 0.70, 1.0],
            text_color: [230, 230, 230],
            hover: false,
            focus: false,
        }
    }

    pub fn with_sizes(mut self, width: f32, height: f32) -> Self {
        self.track_width = width;
        self.track_height = height;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_colors(
        mut self,
        track_off: [f32; 4],
        track_on: [f32; 4],
        knob: [f32; 4],
        border: [f32; 4],
        text: [u8; 3],
    ) -> Self {
        self.track_off = track_off;
        self.track_on = track_on;
        self.knob_color = knob;
        self.border_color = border;
        self.text_color = text;
        self
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }

    fn toggle(&self) {
        let current = self.checked.get();
        self.set_checked.set(!current);
    }
}

impl Widget for Toggle {
    fn style(&self) -> Style {
        let height = self.track_height.max(self.metrics.line_height) + self.padding * 2.0;
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
        let checked = self.checked.get();

        let track_x = layout.location.x + self.padding;
        let track_y = layout.location.y + (layout.size.height - self.track_height) / 2.0;
        let track_radius = self.track_height / 2.0;
        let track_color = if checked { self.track_on } else { self.track_off };

        let border_w = if self.focus { 2.0 } else if self.hover { 1.5 } else { 1.0 };
        let border_c = if self.focus {
            [0.3, 0.6, 0.9, 1.0]
        } else if self.hover {
            [
                (self.border_color[0] + 0.1).min(1.0),
                (self.border_color[1] + 0.1).min(1.0),
                (self.border_color[2] + 0.1).min(1.0),
                self.border_color[3],
            ]
        } else {
            self.border_color
        };

        ctx.renderer.fill_rect_styled(
            (track_x, track_y, self.track_width, self.track_height),
            track_color,
            track_radius,
            border_w,
            border_c,
        );

        let knob_d = (self.track_height - 4.0).max(2.0);
        let knob_x = if checked {
            track_x + self.track_width - knob_d - 2.0
        } else {
            track_x + 2.0
        };
        let knob_y = track_y + (self.track_height - knob_d) / 2.0;
        ctx.renderer.fill_rect_rounded(
            (knob_x, knob_y, knob_d, knob_d),
            self.knob_color,
            knob_d / 2.0,
        );

        if !self.label.is_empty() {
            let text_x = track_x + self.track_width + self.gap;
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
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;
        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                self.hover = self.hit_test(layout, position.x as f32, position.y as f32);
                false
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.hover {
                    self.toggle();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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
