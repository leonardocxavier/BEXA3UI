use std::cell::Cell;

use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::icons;
use crate::signal::{Signal, SetSignal};

pub struct Modal {
    open: Signal<bool>,
    set_open: SetSignal<bool>,
    title: String,
    body_lines: Vec<String>,
    width: f32,
    metrics: Metrics,
    title_metrics: Metrics,
    // Colors
    backdrop_color: [f32; 4],
    bg: [f32; 4],
    border: [f32; 4],
    title_color: [u8; 3],
    text_color: [u8; 3],
    close_color: [u8; 3],
    border_radius: f32,
    close_on_backdrop: bool,
    // Cached viewport for rendering
    viewport_w: Cell<f32>,
    viewport_h: Cell<f32>,
}

impl Modal {
    pub fn new(open: Signal<bool>, set_open: SetSignal<bool>) -> Self {
        Self {
            open,
            set_open,
            title: String::new(),
            body_lines: vec![],
            width: 400.0,
            metrics: Metrics::new(14.0, 20.0),
            title_metrics: Metrics::new(18.0, 26.0),
            backdrop_color: [0.0, 0.0, 0.0, 0.6],
            bg: [0.12, 0.14, 0.20, 1.0],
            border: [0.30, 0.35, 0.50, 1.0],
            title_color: [230, 235, 245],
            text_color: [190, 195, 210],
            close_color: [160, 165, 180],
            border_radius: 10.0,
            close_on_backdrop: true,
            viewport_w: Cell::new(800.0),
            viewport_h: Cell::new(600.0),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_body(mut self, lines: Vec<String>) -> Self {
        self.body_lines = lines;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_metrics(mut self, metrics: Metrics, title_metrics: Metrics) -> Self {
        self.metrics = metrics;
        self.title_metrics = title_metrics;
        self
    }

    pub fn with_colors(mut self, bg: [f32; 4], border: [f32; 4], title: [u8; 3], text: [u8; 3]) -> Self {
        self.bg = bg;
        self.border = border;
        self.title_color = title;
        self.text_color = text;
        self
    }

    pub fn with_close_on_backdrop(mut self, close: bool) -> Self {
        self.close_on_backdrop = close;
        self
    }
}

impl Widget for Modal {
    fn style(&self) -> Style {
        // Modal takes no layout space — it renders entirely in the overlay layer
        Style {
            size: Size {
                width: Dimension::Length(0.0),
                height: Dimension::Length(0.0),
            },
            ..Default::default()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        // Cache the root layout dimensions as viewport estimate
        // The layout.location gives us the widget's own position, but we need
        // to draw over the entire viewport. We use a large enough backdrop.
        let layout = ctx.layout;
        // Prefer renderer viewport size when available
        let (vw, vh) = ctx.renderer.viewport_size();
        let est_w = if vw > 0.0 { vw } else { layout.location.x + layout.size.width };
        let est_h = if vh > 0.0 { vh } else { layout.location.y + layout.size.height };
        self.viewport_w.set(est_w);
        self.viewport_h.set(est_h);

        if !self.open.get() {
            return;
        }

        // Use large backdrop to cover entire window
        let vw = self.viewport_w.get().max(800.0).max(4000.0);
        let vh = self.viewport_h.get().max(600.0).max(4000.0);

        // Backdrop
        ctx.renderer.overlay_fill_rect_styled(
            (0.0, 0.0, vw, vh),
            self.backdrop_color,
            0.0,
            0.0,
            [0.0; 4],
        );

        // Calculate modal dimensions
        let modal_w = self.width;
        let title_h = if self.title.is_empty() { 0.0 } else { self.title_metrics.line_height + 16.0 };
        let body_h = self.body_lines.len() as f32 * (self.metrics.line_height + 4.0);
        let padding = 20.0;
        let close_btn_h = 20.0;
        let modal_h = title_h + body_h + padding * 2.0 + close_btn_h;

        // Center in viewport (use cached values, with fallback)
        let est_vw = self.viewport_w.get().max(800.0);
        let est_vh = self.viewport_h.get().max(600.0);
        let mx = (est_vw - modal_w) / 2.0;
        let my = (est_vh - modal_h) / 2.0;

        // Modal background
        ctx.renderer.overlay_fill_rect_styled(
            (mx, my, modal_w, modal_h),
            self.bg,
            self.border_radius,
            1.5,
            self.border,
        );

        // Close X button (top-right)
        let close_x = mx + modal_w - padding - 12.0;
        let close_y = my + padding * 0.5;
        let close_metrics = Metrics::new(14.0, 20.0);
        ctx.renderer.overlay_draw_text_with_font(
            icons::CLOSE,
            (close_x, close_y),
            self.close_color,
            (16.0, 20.0),
            close_metrics,
            Align::Center,
            icons::NERD_FONT_FAMILY,
        );

        // Title
        let mut cy = my + padding;
        if !self.title.is_empty() {
            ctx.renderer.overlay_draw_text(
                &self.title,
                (mx + padding, cy),
                self.title_color,
                (modal_w - padding * 2.0, self.title_metrics.line_height),
                self.title_metrics,
                Align::Left,
            );
            cy += self.title_metrics.line_height + 16.0;

            // Separator line
            ctx.renderer.overlay_fill_rect_styled(
                (mx + padding, cy - 8.0, modal_w - padding * 2.0, 1.0),
                [self.border[0], self.border[1], self.border[2], 0.5],
                0.0,
                0.0,
                [0.0; 4],
            );
        }

        // Body lines
        for line in &self.body_lines {
            ctx.renderer.overlay_draw_text(
                line,
                (mx + padding, cy),
                self.text_color,
                (modal_w - padding * 2.0, self.metrics.line_height),
                self.metrics,
                Align::Left,
            );
            cy += self.metrics.line_height + 4.0;
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        if !self.open.get() {
            return false;
        }

        match ctx.event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.close_on_backdrop {
                    // Close on any click when modal is open
                    // (A more precise check would test if click is outside the modal rect,
                    //  but since the modal is overlay-rendered, any click effectively interacts with it)
                    self.set_open.set(false);
                    return true;
                }
            }
            _ => {}
        }

        // Consume all events when modal is open to prevent interaction with widgets below
        true
    }

    fn handle_key_event(&mut self, event: &KeyEvent, _modifiers: ModifiersState) -> bool {
        if !self.open.get() {
            return false;
        }

        if event.state == ElementState::Pressed {
            if let Key::Named(NamedKey::Escape) = &event.logical_key {
                self.set_open.set(false);
                return true;
            }
        }

        // Consume all key events when modal is open
        true
    }

    fn is_focusable(&self) -> bool {
        true
    }
}
