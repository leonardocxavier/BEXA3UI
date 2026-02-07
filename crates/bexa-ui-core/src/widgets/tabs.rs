use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::{Signal, SetSignal};

pub struct Tabs {
    labels: Vec<String>,
    active: Signal<usize>,
    set_active: SetSignal<usize>,
    metrics: Metrics,
    tab_height: f32,
    padding: f32,
    border_radius: f32,
    // Colors
    bg: [f32; 4],
    active_bg: [f32; 4],
    hover_bg: [f32; 4],
    border_color: [f32; 4],
    text_color: [u8; 3],
    active_text_color: [u8; 3],
    indicator_color: [f32; 4],
    // State
    hover: bool,
    hover_index: Option<usize>,
    focus: bool,
}

impl Tabs {
    pub fn new(
        labels: Vec<String>,
        active: Signal<usize>,
        set_active: SetSignal<usize>,
        metrics: Metrics,
    ) -> Self {
        Self {
            labels,
            active,
            set_active,
            metrics,
            tab_height: 40.0,
            padding: 16.0,
            border_radius: 0.0,
            bg: [0.12, 0.14, 0.18, 1.0],
            active_bg: [0.18, 0.20, 0.26, 1.0],
            hover_bg: [0.15, 0.17, 0.22, 1.0],
            border_color: [0.3, 0.3, 0.4, 1.0],
            text_color: [160, 160, 180],
            active_text_color: [230, 230, 250],
            indicator_color: [0.20, 0.65, 0.85, 1.0],
            hover: false,
            hover_index: None,
            focus: false,
        }
    }

    pub fn with_tab_height(mut self, height: f32) -> Self {
        self.tab_height = height;
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

    pub fn with_colors(
        mut self,
        bg: [f32; 4],
        active_bg: [f32; 4],
        border: [f32; 4],
        indicator: [f32; 4],
    ) -> Self {
        self.bg = bg;
        self.active_bg = active_bg;
        self.border_color = border;
        self.indicator_color = indicator;
        self
    }

    pub fn with_text_colors(mut self, normal: [u8; 3], active: [u8; 3]) -> Self {
        self.text_color = normal;
        self.active_text_color = active;
        self
    }

    fn tab_width(&self, total_width: f32) -> f32 {
        if self.labels.is_empty() {
            return 0.0;
        }
        total_width / self.labels.len() as f32
    }

    fn tab_at(&self, layout: &Layout, x: f32, y: f32) -> Option<usize> {
        let lx = layout.location.x;
        let ly = layout.location.y;
        if y < ly || y > ly + self.tab_height || x < lx || x > lx + layout.size.width {
            return None;
        }
        let tw = self.tab_width(layout.size.width);
        if tw <= 0.0 {
            return None;
        }
        let idx = ((x - lx) / tw) as usize;
        if idx < self.labels.len() {
            Some(idx)
        } else {
            None
        }
    }
}

impl Widget for Tabs {
    fn style(&self) -> Style {
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(self.tab_height),
            },
            flex_shrink: 0.0,
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = self.tab_height;
        let active_idx = self.active.get();
        let tw = self.tab_width(w);

        // Draw tab bar background
        ctx.renderer.fill_rect_styled(
            (x, y, w, h),
            self.bg,
            self.border_radius,
            0.0,
            [0.0; 4],
        );

        // Bottom border
        ctx.renderer.fill_rect_rounded(
            (x, y + h - 1.0, w, 1.0),
            self.border_color,
            0.0,
        );

        // Draw each tab
        for (i, label) in self.labels.iter().enumerate() {
            let tx = x + i as f32 * tw;
            let is_active = i == active_idx;
            let is_hover = self.hover_index == Some(i);

            // Tab background
            let tab_bg = if is_active {
                self.active_bg
            } else if is_hover {
                self.hover_bg
            } else {
                [0.0, 0.0, 0.0, 0.0] // transparent
            };

            if tab_bg[3] > 0.0 {
                ctx.renderer.fill_rect_rounded(
                    (tx, y, tw, h),
                    tab_bg,
                    0.0,
                );
            }

            // Tab text
            let tc = if is_active {
                self.active_text_color
            } else {
                self.text_color
            };
            let text_y = y + (h - self.metrics.line_height) / 2.0;
            ctx.renderer.draw_text(
                label,
                (tx + self.padding, text_y),
                tc,
                ((tw - self.padding * 2.0).max(0.0), self.metrics.line_height),
                self.metrics,
                Align::Center,
            );

            // Active indicator (bottom bar)
            if is_active {
                let indicator_h = 3.0;
                ctx.renderer.fill_rect_rounded(
                    (tx + 4.0, y + h - indicator_h, tw - 8.0, indicator_h),
                    self.indicator_color,
                    1.5,
                );
            }
        }

        // Focus ring
        if self.focus {
            ctx.renderer.fill_rect_styled(
                (x, y, w, h),
                [0.0, 0.0, 0.0, 0.0],
                self.border_radius,
                2.0,
                [0.3, 0.6, 0.9, 1.0],
            );
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;

        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                let over = px >= layout.location.x
                    && px <= layout.location.x + layout.size.width
                    && py >= layout.location.y
                    && py <= layout.location.y + self.tab_height;
                self.hover = over;
                self.hover_index = self.tab_at(layout, px, py);
                false
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(idx) = self.hover_index {
                    self.set_active.set(idx);
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
        let count = self.labels.len();
        if count == 0 {
            return false;
        }
        match &event.logical_key {
            Key::Named(NamedKey::ArrowRight) => {
                let next = (self.active.get() + 1) % count;
                self.set_active.set(next);
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                let current = self.active.get();
                let next = if current == 0 { count - 1 } else { current - 1 };
                self.set_active.set(next);
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
