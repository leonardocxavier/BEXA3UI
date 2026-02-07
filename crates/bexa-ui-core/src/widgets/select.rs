use std::cell::Cell;

use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::icons;
use crate::signal::{Signal, SetSignal};

pub struct Select {
    options: Vec<String>,
    selected: Signal<usize>,
    set_selected: SetSignal<usize>,
    metrics: Metrics,
    padding: f32,
    border_radius: f32,
    item_height: Cell<f32>,
    // Colors
    bg: [f32; 4],
    border: [f32; 4],
    text_color: [u8; 3],
    dropdown_bg: [f32; 4],
    dropdown_border: [f32; 4],
    hover_bg: [f32; 4],
    hover_text: [u8; 3],
    // State
    open: bool,
    hover: bool,
    hover_index: Option<usize>,
    focus: bool,
    // Cached absolute position for overlay drawing (set in draw, used in handle_event)
    abs_x: Cell<f32>,
    abs_y: Cell<f32>,
    abs_w: Cell<f32>,
    abs_h: Cell<f32>,
}

impl Select {
    pub fn new(
        options: Vec<String>,
        selected: Signal<usize>,
        set_selected: SetSignal<usize>,
        metrics: Metrics,
    ) -> Self {
        Self {
            options,
            selected,
            set_selected,
            metrics,
            padding: 8.0,
            border_radius: 6.0,
            item_height: Cell::new(0.0),
            bg: [0.16, 0.28, 0.38, 1.0],
            border: [0.4, 0.55, 0.7, 1.0],
            text_color: [230, 230, 230],
            dropdown_bg: [0.14, 0.24, 0.34, 1.0],
            dropdown_border: [0.4, 0.55, 0.7, 1.0],
            hover_bg: [0.20, 0.65, 0.85, 1.0],
            hover_text: [255, 255, 255],
            open: false,
            hover: false,
            hover_index: None,
            focus: false,
            abs_x: Cell::new(0.0),
            abs_y: Cell::new(0.0),
            abs_w: Cell::new(0.0),
            abs_h: Cell::new(0.0),
        }
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
        border: [f32; 4],
        text_color: [u8; 3],
    ) -> Self {
        self.bg = bg;
        self.border = border;
        self.text_color = text_color;
        self
    }

    pub fn with_dropdown_colors(
        mut self,
        dropdown_bg: [f32; 4],
        dropdown_border: [f32; 4],
        hover_bg: [f32; 4],
        hover_text: [u8; 3],
    ) -> Self {
        self.dropdown_bg = dropdown_bg;
        self.dropdown_border = dropdown_border;
        self.hover_bg = hover_bg;
        self.hover_text = hover_text;
        self
    }

    fn selected_text(&self) -> &str {
        let idx = self.selected.get();
        self.options.get(idx).map(|s| s.as_str()).unwrap_or("")
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }

    fn dropdown_item_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.open {
            return None;
        }
        let item_h = self.item_height.get();
        let dropdown_x = self.abs_x.get();
        let dropdown_y = self.abs_y.get() + self.abs_h.get();
        let dropdown_w = self.abs_w.get();

        if x < dropdown_x || x > dropdown_x + dropdown_w {
            return None;
        }

        let rel_y = y - dropdown_y;
        if rel_y < 0.0 {
            return None;
        }

        let idx = (rel_y / item_h) as usize;
        if idx < self.options.len() {
            Some(idx)
        } else {
            None
        }
    }
}

impl Widget for Select {
    fn style(&self) -> Style {
        let height = self.metrics.line_height + self.padding * 2.0;
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(height),
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
        let h = layout.size.height;

        // Cache absolute position for event handling
        self.abs_x.set(x);
        self.abs_y.set(y);
        self.abs_w.set(w);
        self.abs_h.set(h);
        self.item_height.set(self.metrics.line_height + self.padding);

        // Draw the select box
        let border_w = if self.focus { 2.0 } else if self.hover { 1.5 } else { 1.0 };
        let border_c = if self.focus {
            [0.3, 0.6, 0.9, 1.0]
        } else if self.hover {
            [
                (self.border[0] + 0.1).min(1.0),
                (self.border[1] + 0.1).min(1.0),
                (self.border[2] + 0.1).min(1.0),
                self.border[3],
            ]
        } else {
            self.border
        };
        let select_bg = if self.hover && !self.open {
            [
                (self.bg[0] + 0.04).min(1.0),
                (self.bg[1] + 0.04).min(1.0),
                (self.bg[2] + 0.04).min(1.0),
                self.bg[3],
            ]
        } else {
            self.bg
        };
        ctx.renderer.fill_rect_styled(
            (x, y, w, h),
            select_bg,
            self.border_radius,
            border_w,
            border_c,
        );

        // Draw selected text
        let text_x = x + self.padding;
        let text_y = y + (h - self.metrics.line_height) / 2.0;
        let chevron_space = 24.0;
        let text_w = (w - self.padding * 2.0 - chevron_space).max(0.0);
        ctx.renderer.draw_text(
            self.selected_text(),
            (text_x, text_y),
            self.text_color,
            (text_w, self.metrics.line_height),
            self.metrics,
            Align::Left,
        );

        // Draw chevron icon
        let icon_metrics = Metrics::new(self.metrics.font_size * 0.8, self.metrics.line_height);
        let icon_x = x + w - self.padding - 16.0;
        let icon_y = text_y;
        let chevron = if self.open { icons::CHEVRON_UP } else { icons::CHEVRON_DOWN };
        ctx.renderer.draw_text_with_font(
            chevron,
            (icon_x, icon_y),
            self.text_color,
            (16.0, self.metrics.line_height),
            icon_metrics,
            Align::Center,
            icons::NERD_FONT_FAMILY,
        );

        // Draw dropdown overlay when open
        if self.open {
            let item_h = self.item_height.get();
            let dropdown_h = item_h * self.options.len() as f32 + self.padding;
            let dropdown_y = y + h;

            // Dropdown background
            ctx.renderer.overlay_fill_rect_styled(
                (x, dropdown_y, w, dropdown_h),
                self.dropdown_bg,
                self.border_radius,
                1.0,
                self.dropdown_border,
            );

            // Dropdown items
            for (i, option) in self.options.iter().enumerate() {
                let iy = dropdown_y + self.padding * 0.5 + i as f32 * item_h;
                let is_hover = self.hover_index == Some(i);
                let is_selected = self.selected.get() == i;

                // Hover highlight
                if is_hover {
                    ctx.renderer.overlay_fill_rect_styled(
                        (x + 2.0, iy, w - 4.0, item_h),
                        self.hover_bg,
                        4.0,
                        0.0,
                        [0.0; 4],
                    );
                }

                let tc = if is_hover {
                    self.hover_text
                } else if is_selected {
                    [180, 220, 255]
                } else {
                    self.text_color
                };

                ctx.renderer.overlay_draw_text(
                    option,
                    (x + self.padding, iy + (item_h - self.metrics.line_height) / 2.0),
                    tc,
                    (w - self.padding * 2.0, self.metrics.line_height),
                    self.metrics,
                    Align::Left,
                );
            }
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;

        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                let over = self.hit_test(layout, px, py);
                self.hover = over;

                // Track hover over dropdown items
                if self.open {
                    self.hover_index = self.dropdown_item_at(px, py);
                }
                false // don't consume — let siblings update hover too
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.open {
                    // Check if clicking on dropdown item
                    if let Some(idx) = self.hover_index {
                        self.set_selected.set(idx);
                        self.open = false;
                        self.hover_index = None;
                    } else {
                        // Clicked outside or on the select box → close
                        self.open = false;
                    }
                    true
                } else if self.hover {
                    self.open = true;
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
                if self.open {
                    // Select hovered item or close
                    if let Some(idx) = self.hover_index {
                        self.set_selected.set(idx);
                    }
                    self.open = false;
                } else {
                    self.open = true;
                }
                true
            }
            Key::Named(NamedKey::Escape) => {
                if self.open {
                    self.open = false;
                    true
                } else {
                    false
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                if self.open {
                    let current = self.hover_index.unwrap_or(self.selected.get());
                    let next = (current + 1).min(self.options.len().saturating_sub(1));
                    self.hover_index = Some(next);
                } else {
                    // Move selection down
                    let current = self.selected.get();
                    let next = (current + 1).min(self.options.len().saturating_sub(1));
                    self.set_selected.set(next);
                }
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                if self.open {
                    let current = self.hover_index.unwrap_or(self.selected.get());
                    let next = current.saturating_sub(1);
                    self.hover_index = Some(next);
                } else {
                    let current = self.selected.get();
                    let next = current.saturating_sub(1);
                    self.set_selected.set(next);
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
        if !focused {
            self.open = false;
        }
    }

    fn activate(&mut self) {
        self.open = !self.open;
    }
}
