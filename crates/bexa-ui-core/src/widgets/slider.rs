use glyphon::Metrics;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::{Signal, SetSignal};

pub struct Slider {
    value: Signal<f32>,
    set_value: SetSignal<f32>,
    min: f32,
    max: f32,
    step: f32,
    metrics: Metrics,
    track_height: f32,
    knob_radius: f32,
    padding: f32,
    // Colors
    track_bg: [f32; 4],
    track_fill: [f32; 4],
    knob_color: [f32; 4],
    border_color: [f32; 4],
    // State
    hover: bool,
    dragging: bool,
    focus: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl Slider {
    pub fn new(value: Signal<f32>, set_value: SetSignal<f32>, metrics: Metrics) -> Self {
        Self {
            value,
            set_value,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            metrics,
            track_height: 6.0,
            knob_radius: 8.0,
            padding: 8.0,
            track_bg: [0.16, 0.22, 0.30, 1.0],
            track_fill: [0.20, 0.65, 0.85, 1.0],
            knob_color: [0.92, 0.92, 0.95, 1.0],
            border_color: [0.35, 0.45, 0.60, 1.0],
            hover: false,
            dragging: false,
            focus: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max.max(min + 0.0001);
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step.max(0.0);
        self
    }

    pub fn with_sizes(mut self, track_height: f32, knob_radius: f32) -> Self {
        self.track_height = track_height;
        self.knob_radius = knob_radius;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_colors(
        mut self,
        track_bg: [f32; 4],
        track_fill: [f32; 4],
        knob: [f32; 4],
        border: [f32; 4],
    ) -> Self {
        self.track_bg = track_bg;
        self.track_fill = track_fill;
        self.knob_color = knob;
        self.border_color = border;
        self
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }

    fn track_bounds(&self, layout: &Layout) -> (f32, f32, f32) {
        let track_x = layout.location.x + self.padding;
        let track_w = (layout.size.width - self.padding * 2.0).max(1.0);
        let track_y = layout.location.y + (layout.size.height - self.track_height) / 2.0;
        (track_x, track_y, track_w)
    }

    fn clamp_value(&self, value: f32) -> f32 {
        let mut v = value.clamp(self.min, self.max);
        if self.step > 0.0 {
            let steps = ((v - self.min) / self.step).round();
            v = self.min + steps * self.step;
        }
        v.clamp(self.min, self.max)
    }

    fn set_value_from_x(&mut self, layout: &Layout, x: f32) {
        let (track_x, _track_y, track_w) = self.track_bounds(layout);
        let t = ((x - track_x) / track_w).clamp(0.0, 1.0);
        let value = self.min + t * (self.max - self.min);
        let value = self.clamp_value(value);
        if (value - self.value.get()).abs() > f32::EPSILON {
            self.set_value.set(value);
        }
    }

    fn adjust_value(&mut self, delta: f32) {
        let value = self.clamp_value(self.value.get() + delta);
        self.set_value.set(value);
    }
}

impl Widget for Slider {
    fn style(&self) -> Style {
        let height = self.knob_radius * 2.0 + self.padding * 2.0;
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(height.max(self.metrics.line_height + 4.0)),
            },
            flex_shrink: 0.0,
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let (track_x, track_y, track_w) = self.track_bounds(layout);
        let track_radius = self.track_height / 2.0;

        let border_w = if self.focus { 2.0 } else if self.hover { 1.5 } else { 1.0 };
        let border_c = if self.focus {
            [0.3, 0.6, 0.9, 1.0]
        } else if self.hover {
            [
                (self.border_color[0] + 0.08).min(1.0),
                (self.border_color[1] + 0.08).min(1.0),
                (self.border_color[2] + 0.08).min(1.0),
                self.border_color[3],
            ]
        } else {
            self.border_color
        };

        // Track background
        ctx.renderer.fill_rect_styled(
            (track_x, track_y, track_w, self.track_height),
            self.track_bg,
            track_radius,
            border_w,
            border_c,
        );

        // Filled portion
        let t = (self.value.get() - self.min) / (self.max - self.min);
        let fill_w = (track_w * t.clamp(0.0, 1.0)).max(0.0);
        ctx.renderer.fill_rect_rounded(
            (track_x, track_y, fill_w, self.track_height),
            self.track_fill,
            track_radius,
        );

        // Knob
        let knob_x = track_x + fill_w - self.knob_radius;
        let knob_y = track_y + (self.track_height / 2.0) - self.knob_radius;
        let knob_d = self.knob_radius * 2.0;
        ctx.renderer.fill_rect_rounded(
            (knob_x, knob_y, knob_d, knob_d),
            self.knob_color,
            self.knob_radius,
        );
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;
        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                self.last_mouse_x = px;
                self.last_mouse_y = py;
                self.hover = self.hit_test(layout, px, py);
                if self.dragging {
                    self.set_value_from_x(layout, px);
                }
                false
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.hover = self.hit_test(layout, self.last_mouse_x, self.last_mouse_y);
                if self.hover {
                    self.dragging = true;
                    self.set_value_from_x(layout, self.last_mouse_x);
                    true
                } else {
                    false
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                let was_dragging = self.dragging;
                self.dragging = false;
                was_dragging
            }
            _ => false,
        }
    }

    fn handle_key_event(&mut self, event: &KeyEvent, _modifiers: ModifiersState) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        let step = if self.step > 0.0 { self.step } else { 1.0 };
        match &event.logical_key {
            Key::Named(NamedKey::ArrowLeft) => {
                self.adjust_value(-step);
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.adjust_value(step);
                true
            }
            Key::Named(NamedKey::Home) => {
                self.set_value.set(self.min);
                true
            }
            Key::Named(NamedKey::End) => {
                self.set_value.set(self.max);
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
}
