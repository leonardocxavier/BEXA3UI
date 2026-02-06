use taffy::prelude::*;
use winit::event::WindowEvent;
use winit::event::KeyEvent;
use winit::keyboard::ModifiersState;

pub struct DrawContext<'a> {
    pub renderer: &'a mut crate::Renderer,
    pub layout: &'a Layout,
}

pub struct EventContext<'a> {
    pub event: &'a WindowEvent,
    pub layout: &'a Layout,
}

pub trait Widget {
    fn style(&self) -> Style {
        Style::default()
    }

    fn draw(&self, _ctx: &mut DrawContext) {}

    fn handle_event(&mut self, _ctx: &mut EventContext) -> bool {
        false
    }

    /// Called when this widget has focus and a key is pressed.
    /// Returns true if the event was consumed.
    fn handle_key_event(&mut self, _event: &KeyEvent, _modifiers: ModifiersState) -> bool {
        false
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn set_focus(&mut self, _focused: bool) {}

    fn activate(&mut self) {}

    fn clear_active(&mut self) {}

    fn is_scrollable(&self) -> bool {
        false
    }

    /// Called after text rendering to feed back measured pixel widths.
    fn update_measures(&mut self, _measures: &[Vec<f32>]) {}
}
