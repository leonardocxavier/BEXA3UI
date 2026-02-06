use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::framework::{DrawContext, EventContext, Widget};

pub struct Button {
    label: String,
    metrics: Metrics,
    padding: f32,
    border_radius: f32,
    bg_color: [f32; 3],
    hover_color: [f32; 3],
    active_color: [f32; 3],
    focus_color: [f32; 3],
    text_color: [u8; 3],
    hover_text_color: [u8; 3],
    active_text_color: [u8; 3],
    hover: bool,
    active: bool,
    focus: bool,
    on_click: Option<Box<dyn FnMut()>>,
}

impl Button {
    pub fn new(label: impl Into<String>, metrics: Metrics) -> Self {
        Self {
            label: label.into(),
            metrics,
            padding: 16.0,
            border_radius: 0.0,
            bg_color: [0.20, 0.65, 0.85],
            hover_color: [0.35, 0.75, 0.92],
            active_color: [0.15, 0.55, 0.75],
            focus_color: [0.18, 0.60, 0.82],
            text_color: [25, 25, 25],
            hover_text_color: [15, 15, 15],
            active_text_color: [250, 250, 250],
            hover: false,
            active: false,
            focus: false,
            on_click: None,
        }
    }

    pub fn with_colors(
        mut self,
        bg: [f32; 3],
        hover: [f32; 3],
        active: [f32; 3],
        focus: [f32; 3],
    ) -> Self {
        self.bg_color = bg;
        self.hover_color = hover;
        self.active_color = active;
        self.focus_color = focus;
        self
    }

    pub fn with_text_colors(mut self, normal: [u8; 3], hover: [u8; 3], active: [u8; 3]) -> Self {
        self.text_color = normal;
        self.hover_text_color = hover;
        self.active_text_color = active;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn set_on_click(&mut self, handler: impl FnMut() + 'static) {
        self.on_click = Some(Box::new(handler));
    }

    fn current_color(&self) -> [f32; 3] {
        if self.active {
            self.active_color
        } else if self.hover {
            self.hover_color
        } else if self.focus {
            self.focus_color
        } else {
            self.bg_color
        }
    }

    fn current_text_color(&self) -> [u8; 3] {
        if self.active {
            self.active_text_color
        } else if self.hover {
            self.hover_text_color
        } else {
            self.text_color
        }
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }

    fn click(&mut self) {
        if let Some(handler) = self.on_click.as_mut() {
            handler();
        }
    }
}

impl Widget for Button {
    fn style(&self) -> Style {
        Style {
            flex_grow: 1.0,
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let color = self.current_color();
        ctx.renderer.fill_rect_rounded(
            (
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            ),
            [color[0], color[1], color[2], 1.0],
            self.border_radius,
        );

        let inner_width = (layout.size.width - self.padding * 2.0).max(0.0);
        let inner_height = (layout.size.height - self.padding * 2.0).max(0.0);
        let baseline_nudge = 2.0;
        let vertical_offset = ((inner_height - self.metrics.font_size).max(0.0)) * 0.5 + baseline_nudge;
        let text_left = layout.location.x + self.padding;
        let text_top = layout.location.y + self.padding + vertical_offset;

        ctx.renderer.draw_text(
            &self.label,
            (text_left, text_top),
            self.current_text_color(),
            (inner_width, inner_height),
            self.metrics,
            Align::Center,
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
                    self.active = !self.active;
                    self.click();
                    changed = true;
                }
            }
            _ => {}
        }

        changed
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focus = focused;
    }

    fn activate(&mut self) {
        self.active = !self.active;
        self.click();
    }

    fn clear_active(&mut self) {
        self.active = false;
    }
}
