use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::{Signal, SetSignal};
use crate::WidgetNode;

pub struct RadioButton {
    label: String,
    index: usize,
    selected: Signal<usize>,
    set_selected: SetSignal<usize>,
    metrics: Metrics,
    circle_size: f32,
    gap: f32,
    // Colors
    circle_bg: [f32; 4],
    circle_border: [f32; 4],
    dot_color: [f32; 4],
    text_color: [u8; 3],
    // State
    hover: bool,
    focus: bool,
}

impl RadioButton {
    pub fn new(
        label: impl Into<String>,
        index: usize,
        selected: Signal<usize>,
        set_selected: SetSignal<usize>,
        metrics: Metrics,
    ) -> Self {
        Self {
            label: label.into(),
            index,
            selected,
            set_selected,
            metrics,
            circle_size: 20.0,
            gap: 8.0,
            circle_bg: [0.16, 0.28, 0.38, 1.0],
            circle_border: [0.4, 0.55, 0.7, 1.0],
            dot_color: [0.20, 0.65, 0.85, 1.0],
            text_color: [230, 230, 230],
            hover: false,
            focus: false,
        }
    }

    pub fn with_circle_size(mut self, size: f32) -> Self {
        self.circle_size = size;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_colors(
        mut self,
        circle_bg: [f32; 4],
        circle_border: [f32; 4],
        dot_color: [f32; 4],
    ) -> Self {
        self.circle_bg = circle_bg;
        self.circle_border = circle_border;
        self.dot_color = dot_color;
        self
    }

    pub fn with_text_color(mut self, color: [u8; 3]) -> Self {
        self.text_color = color;
        self
    }

    fn select(&self) {
        self.set_selected.set(self.index);
    }

    fn is_selected(&self) -> bool {
        self.selected.get() == self.index
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }
}

impl Widget for RadioButton {
    fn style(&self) -> Style {
        let height = self.circle_size.max(self.metrics.line_height) + 8.0;
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
        let selected = self.is_selected();

        // 1. Draw outer circle (border_radius = half size makes it a circle)
        let cx = layout.location.x + 4.0;
        let cy = layout.location.y + (layout.size.height - self.circle_size) / 2.0;
        let radius = self.circle_size / 2.0;
        let border_w = if self.focus { 2.0 } else if self.hover { 1.5 } else { 1.0 };
        let border_c = if self.focus {
            [0.3, 0.6, 0.9, 1.0]
        } else if self.hover {
            [
                (self.circle_border[0] + 0.1).min(1.0),
                (self.circle_border[1] + 0.1).min(1.0),
                (self.circle_border[2] + 0.1).min(1.0),
                self.circle_border[3],
            ]
        } else {
            self.circle_border
        };
        let circle_bg = if self.hover && !selected {
            [
                (self.circle_bg[0] + 0.06).min(1.0),
                (self.circle_bg[1] + 0.06).min(1.0),
                (self.circle_bg[2] + 0.06).min(1.0),
                self.circle_bg[3],
            ]
        } else {
            self.circle_bg
        };
        ctx.renderer.fill_rect_styled(
            (cx, cy, self.circle_size, self.circle_size),
            circle_bg,
            radius,
            border_w,
            border_c,
        );

        // 2. Draw inner filled dot if selected
        if selected {
            let dot_size = self.circle_size * 0.5;
            let dot_x = cx + (self.circle_size - dot_size) / 2.0;
            let dot_y = cy + (self.circle_size - dot_size) / 2.0;
            ctx.renderer.fill_rect_rounded(
                (dot_x, dot_y, dot_size, dot_size),
                self.dot_color,
                dot_size / 2.0,
            );
        }

        // 3. Draw label text
        let text_x = cx + self.circle_size + self.gap;
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
        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let over = self.hit_test(layout, position.x as f32, position.y as f32);
                self.hover = over;
                false // don't consume — let siblings update hover too
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.hover && !self.is_selected() {
                    self.select();
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
                if !self.is_selected() {
                    self.select();
                }
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
        self.select();
    }
}

/// Convenience: create a column of radio buttons sharing the same signal.
pub fn radio_group(
    options: &[&str],
    selected: Signal<usize>,
    set_selected: SetSignal<usize>,
    metrics: Metrics,
) -> Vec<WidgetNode> {
    options
        .iter()
        .enumerate()
        .map(|(i, label)| {
            WidgetNode::new(
                RadioButton::new(*label, i, selected.clone(), set_selected.clone(), metrics),
                vec![],
            )
        })
        .collect()
}
