use taffy::prelude::*;
use taffy::Overflow;

use crate::framework::{DrawContext, Widget};

pub struct Container {
    style: Style,
    background: Option<[f32; 4]>,
    border_radius: f32,
    border_width: f32,
    border_color: [f32; 4],
    scrollable: bool,
}

impl Container {
    pub fn new() -> Self {
        Self {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                ..Default::default()
            },
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
            scrollable: false,
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.style.padding = Rect {
            left: LengthPercentage::Length(padding),
            right: LengthPercentage::Length(padding),
            top: LengthPercentage::Length(padding),
            bottom: LengthPercentage::Length(padding),
        };
        self
    }

    pub fn with_background(mut self, color: [f32; 3]) -> Self {
        self.background = Some([color[0], color[1], color[2], 1.0]);
        self
    }

    pub fn with_background_alpha(mut self, color: [f32; 4]) -> Self {
        self.background = Some(color);
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn with_border(mut self, width: f32, color: [f32; 4]) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.style.size.height = Dimension::Length(height);
        self
    }

    pub fn with_max_height(mut self, height: f32) -> Self {
        self.style.max_size.height = Dimension::Length(height);
        self
    }

    pub fn with_scroll(mut self) -> Self {
        self.scrollable = true;
        self.style.overflow.y = Overflow::Hidden;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.style.gap = Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        };
        self
    }
}

impl Widget for Container {
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn is_scrollable(&self) -> bool {
        self.scrollable
    }

    fn draw(&self, ctx: &mut DrawContext) {
        if let Some(color) = self.background {
            let layout = ctx.layout;
            ctx.renderer.fill_rect_styled(
                (
                    layout.location.x,
                    layout.location.y,
                    layout.size.width,
                    layout.size.height,
                ),
                color,
                self.border_radius,
                self.border_width,
                self.border_color,
            );
        }
    }
}
