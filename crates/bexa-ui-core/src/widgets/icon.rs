use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;

use crate::framework::{DrawContext, Widget};
use crate::icons::NERD_FONT_FAMILY;

pub struct Icon {
    glyph: &'static str,
    metrics: Metrics,
    color: [u8; 3],
    padding: f32,
}

impl Icon {
    pub fn new(glyph: &'static str, size: f32, color: [u8; 3]) -> Self {
        Self {
            glyph,
            metrics: Metrics::new(size, size * 1.2),
            color,
            padding: 2.0,
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl Widget for Icon {
    fn style(&self) -> Style {
        let size = self.metrics.line_height + self.padding * 2.0;
        Style {
            size: Size {
                width: Dimension::Length(size),
                height: Dimension::Length(size),
            },
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let left = layout.location.x + self.padding;
        let top = layout.location.y + self.padding;
        let bounds = (
            layout.size.width - self.padding * 2.0,
            layout.size.height - self.padding * 2.0,
        );
        ctx.renderer.draw_text_with_font(
            self.glyph,
            (left, top),
            self.color,
            bounds,
            self.metrics,
            Align::Center,
            NERD_FONT_FAMILY,
        );
    }
}
