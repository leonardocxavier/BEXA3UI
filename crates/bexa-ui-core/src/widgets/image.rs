use taffy::prelude::*;

use crate::framework::{DrawContext, Widget};
use crate::renderer::ImageFit;

pub struct Image {
    path: String,
    style: Style,
    tint: [f32; 4],
    border_radius: f32,
    background: Option<[f32; 4]>,
    fit: ImageFit,
}

impl Image {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            style: Style {
                size: Size {
                    width: Dimension::Length(120.0),
                    height: Dimension::Length(120.0),
                },
                flex_shrink: 0.0,
                ..Default::default()
            },
            tint: [1.0, 1.0, 1.0, 1.0],
            border_radius: 0.0,
            background: None,
            fit: ImageFit::Fill,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.style.size = Size {
            width: Dimension::Length(width),
            height: Dimension::Length(height),
        };
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.style.size.width = Dimension::Length(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.style.size.height = Dimension::Length(height);
        self
    }

    pub fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }

    pub fn with_fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn with_background(mut self, color: [f32; 4]) -> Self {
        self.background = Some(color);
        self
    }
}

impl Widget for Image {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let layout = ctx.layout;
        let rect = (
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        );

        if let Some(color) = self.background {
            ctx.renderer.fill_rect_styled(rect, color, self.border_radius, 0.0, [0.0; 4]);
        }

        ctx.renderer.draw_image(&self.path, rect, self.tint, self.fit);
    }
}
