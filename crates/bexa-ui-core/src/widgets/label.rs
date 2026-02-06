use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;

use crate::framework::{DrawContext, Widget};
use crate::signal::{Signal, IntoSignal};

pub struct Label {
    text: Signal<String>,
    metrics: Metrics,
    color: [u8; 3],
    align: Align,
    padding: f32,
    font_family: Option<String>,
}

impl Label {
    pub fn new(text: impl IntoSignal<String>, metrics: Metrics, color: [u8; 3]) -> Self {
        Self {
            text: text.into_signal(),
            metrics,
            color,
            align: Align::Left,
            padding: 4.0,
            font_family: None,
        }
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_font_family(mut self, family: &str) -> Self {
        self.font_family = Some(family.to_string());
        self
    }
}

impl Widget for Label {
    fn style(&self) -> Style {
        let height = self.metrics.line_height + self.padding * 2.0;
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(height),
            },
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let left = layout.location.x + self.padding;
        let top = layout.location.y + self.padding;
        let bounds = (layout.size.width - self.padding * 2.0, layout.size.height - self.padding * 2.0);
        self.text.with(|text| {
            if let Some(ref family) = self.font_family {
                ctx.renderer.draw_text_with_font(
                    text,
                    (left, top),
                    self.color,
                    bounds,
                    self.metrics,
                    self.align,
                    family,
                );
            } else {
                ctx.renderer.draw_text(
                    text,
                    (left, top),
                    self.color,
                    bounds,
                    self.metrics,
                    self.align,
                );
            }
        });
    }
}
