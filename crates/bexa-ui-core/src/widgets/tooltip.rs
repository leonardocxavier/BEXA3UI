use std::cell::Cell;

use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::WindowEvent;

use crate::framework::{DrawContext, EventContext, Widget};

/// Position of the tooltip relative to its trigger area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TooltipPosition {
    Top,
    Bottom,
}

pub struct Tooltip {
    text: String,
    position: TooltipPosition,
    metrics: Metrics,
    padding: f32,
    bg: [f32; 4],
    border: [f32; 4],
    text_color: [u8; 3],
    border_radius: f32,
    // State
    hover: bool,
    // Cached absolute position (set during draw)
    abs_x: Cell<f32>,
    abs_y: Cell<f32>,
    abs_w: Cell<f32>,
    abs_h: Cell<f32>,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            position: TooltipPosition::Top,
            metrics: Metrics::new(12.0, 16.0),
            padding: 6.0,
            bg: [0.12, 0.14, 0.20, 0.95],
            border: [0.35, 0.40, 0.55, 1.0],
            text_color: [220, 220, 230],
            border_radius: 4.0,
            hover: false,
            abs_x: Cell::new(0.0),
            abs_y: Cell::new(0.0),
            abs_w: Cell::new(0.0),
            abs_h: Cell::new(0.0),
        }
    }

    pub fn with_position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn with_colors(mut self, bg: [f32; 4], border: [f32; 4], text_color: [u8; 3]) -> Self {
        self.bg = bg;
        self.border = border;
        self.text_color = text_color;
        self
    }
}

impl Widget for Tooltip {
    fn style(&self) -> Style {
        // Tooltip acts as a transparent wrapper — takes full width, auto height
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Auto,
            },
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;

        // Cache position for event handling
        self.abs_x.set(x);
        self.abs_y.set(y);
        self.abs_w.set(w);
        self.abs_h.set(h);

        // Draw tooltip overlay when hovering
        if self.hover && !self.text.is_empty() {
            let tip_h = self.metrics.line_height + self.padding * 2.0;
            // Estimate text width: ~7px per char at 12px font, clamped
            let estimated_w = (self.text.len() as f32 * self.metrics.font_size * 0.6)
                .max(60.0)
                .min(300.0);
            let tip_w = estimated_w + self.padding * 2.0;

            // Center horizontally over the trigger
            let tip_x = x + (w - tip_w) / 2.0;
            let tip_y = match self.position {
                TooltipPosition::Top => y - tip_h - 4.0,
                TooltipPosition::Bottom => y + h + 4.0,
            };

            // Background
            ctx.renderer.overlay_fill_rect_styled(
                (tip_x, tip_y, tip_w, tip_h),
                self.bg,
                self.border_radius,
                1.0,
                self.border,
            );

            // Text
            ctx.renderer.overlay_draw_text(
                &self.text,
                (tip_x + self.padding, tip_y + self.padding),
                self.text_color,
                (estimated_w, self.metrics.line_height),
                self.metrics,
                Align::Center,
            );
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                let x = self.abs_x.get();
                let y = self.abs_y.get();
                let w = self.abs_w.get();
                let h = self.abs_h.get();

                let inside = px >= x && px <= x + w && py >= y && py <= y + h;
                self.hover = inside;
            }
            _ => {}
        }
        false
    }
}
