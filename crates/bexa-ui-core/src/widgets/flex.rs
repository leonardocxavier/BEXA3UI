use taffy::prelude::*;

use crate::framework::Widget;

pub struct Flex {
    style: Style,
}

impl Flex {
    pub fn row(gap: f32) -> Self {
        Self {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                align_items: Some(AlignItems::Stretch),
                gap: Size {
                    width: LengthPercentage::Length(gap),
                    height: LengthPercentage::Length(0.0),
                },
                ..Default::default()
            },
        }
    }

    pub fn column(gap: f32, padding: f32) -> Self {
        Self {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                align_items: Some(AlignItems::Stretch),
                justify_content: Some(JustifyContent::SpaceBetween),
                gap: Size {
                    width: LengthPercentage::Length(0.0),
                    height: LengthPercentage::Length(gap),
                },
                padding: Rect {
                    left: LengthPercentage::Length(padding),
                    right: LengthPercentage::Length(padding),
                    top: LengthPercentage::Length(padding),
                    bottom: LengthPercentage::Length(padding),
                },
                ..Default::default()
            },
        }
    }
}

impl Widget for Flex {
    fn style(&self) -> Style {
        self.style.clone()
    }
}
