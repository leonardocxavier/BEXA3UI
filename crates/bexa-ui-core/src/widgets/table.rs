use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::{Signal, SetSignal};

/// Column definition for a Table.
pub struct Column {
    pub header: String,
    /// Relative flex weight (e.g. 1.0, 2.0, 3.0).
    pub flex: f32,
}

impl Column {
    pub fn new(header: impl Into<String>, flex: f32) -> Self {
        Self {
            header: header.into(),
            flex,
        }
    }
}

pub struct Table {
    columns: Vec<Column>,
    rows: Signal<Vec<Vec<String>>>,
    selected_row: Signal<Option<usize>>,
    set_selected_row: SetSignal<Option<usize>>,
    metrics: Metrics,
    row_height: f32,
    header_height: f32,
    padding: f32,
    // Colors
    header_bg: [f32; 4],
    header_text: [u8; 3],
    row_bg: [f32; 4],
    row_alt_bg: [f32; 4],
    row_hover_bg: [f32; 4],
    row_selected_bg: [f32; 4],
    text_color: [u8; 3],
    selected_text: [u8; 3],
    border_color: [f32; 4],
    // State
    hover_row: Option<usize>,
    focus: bool,
    // layout cache
    max_visible: usize,
}

impl Table {
    pub fn new(
        columns: Vec<Column>,
        rows: Signal<Vec<Vec<String>>>,
        selected_row: Signal<Option<usize>>,
        set_selected_row: SetSignal<Option<usize>>,
        metrics: Metrics,
    ) -> Self {
        Self {
            columns,
            rows,
            selected_row,
            set_selected_row,
            metrics,
            row_height: 32.0,
            header_height: 36.0,
            padding: 8.0,
            header_bg: [0.14, 0.16, 0.22, 1.0],
            header_text: [180, 190, 220],
            row_bg: [0.10, 0.12, 0.16, 1.0],
            row_alt_bg: [0.12, 0.14, 0.19, 1.0],
            row_hover_bg: [0.18, 0.22, 0.30, 1.0],
            row_selected_bg: [0.20, 0.45, 0.70, 1.0],
            text_color: [210, 210, 220],
            selected_text: [255, 255, 255],
            border_color: [0.25, 0.28, 0.35, 1.0],
            hover_row: None,
            focus: false,
            max_visible: 100,
        }
    }

    pub fn with_row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    pub fn with_header_height(mut self, h: f32) -> Self {
        self.header_height = h;
        self
    }

    pub fn with_max_visible(mut self, n: usize) -> Self {
        self.max_visible = n;
        self
    }

    pub fn with_colors(
        mut self,
        header_bg: [f32; 4],
        row_bg: [f32; 4],
        row_alt: [f32; 4],
        selected_bg: [f32; 4],
        border: [f32; 4],
    ) -> Self {
        self.header_bg = header_bg;
        self.row_bg = row_bg;
        self.row_alt_bg = row_alt;
        self.row_selected_bg = selected_bg;
        self.border_color = border;
        self
    }

    pub fn with_text_colors(
        mut self,
        header: [u8; 3],
        row: [u8; 3],
        selected: [u8; 3],
    ) -> Self {
        self.header_text = header;
        self.text_color = row;
        self.selected_text = selected;
        self
    }

    fn total_flex(&self) -> f32 {
        self.columns.iter().map(|c| c.flex).sum::<f32>().max(1.0)
    }

    fn col_x_width(&self, total_w: f32) -> Vec<(f32, f32)> {
        let total_flex = self.total_flex();
        let mut result = Vec::with_capacity(self.columns.len());
        let mut cx = 0.0;
        for col in &self.columns {
            let w = (col.flex / total_flex) * total_w;
            result.push((cx, w));
            cx += w;
        }
        result
    }

    fn row_at(&self, layout: &Layout, y: f32) -> Option<usize> {
        let ly = layout.location.y;
        let data_y = ly + self.header_height;
        if y < data_y {
            return None;
        }
        let idx = ((y - data_y) / self.row_height) as usize;
        let count = self.rows.with(|r| r.len());
        if idx < count.min(self.max_visible) {
            Some(idx)
        } else {
            None
        }
    }
}

impl Widget for Table {
    fn style(&self) -> Style {
        let row_count = self.rows.with(|r| r.len().min(self.max_visible));
        let total_h = self.header_height + row_count as f32 * self.row_height;
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(total_h),
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
        let selected = self.selected_row.get();
        let col_info = self.col_x_width(w);

        // Header background
        ctx.renderer.fill_rect_rounded(
            (x, y, w, self.header_height),
            self.header_bg,
            0.0,
        );

        // Header text
        let header_metrics = Metrics::new(
            self.metrics.font_size * 0.85,
            self.metrics.line_height,
        );
        for (i, col) in self.columns.iter().enumerate() {
            let (cx, cw) = col_info[i];
            let text_y = y + (self.header_height - header_metrics.line_height) / 2.0;
            ctx.renderer.draw_text(
                &col.header.to_uppercase(),
                (x + cx + self.padding, text_y),
                self.header_text,
                ((cw - self.padding * 2.0).max(0.0), header_metrics.line_height),
                header_metrics,
                Align::Left,
            );
        }

        // Header bottom border
        ctx.renderer.fill_rect_rounded(
            (x, y + self.header_height - 1.0, w, 1.0),
            self.border_color,
            0.0,
        );

        // Data rows
        self.rows.with(|rows| {
            let count = rows.len().min(self.max_visible);
            for ri in 0..count {
                let ry = y + self.header_height + ri as f32 * self.row_height;
                let is_selected = selected == Some(ri);
                let is_hover = self.hover_row == Some(ri);

                // Row background
                let row_bg = if is_selected {
                    self.row_selected_bg
                } else if is_hover {
                    self.row_hover_bg
                } else if ri % 2 == 0 {
                    self.row_bg
                } else {
                    self.row_alt_bg
                };

                ctx.renderer.fill_rect_rounded(
                    (x, ry, w, self.row_height),
                    row_bg,
                    0.0,
                );

                // Cell text
                let tc = if is_selected {
                    self.selected_text
                } else {
                    self.text_color
                };

                let row = &rows[ri];
                for (ci, (cx, cw)) in col_info.iter().enumerate() {
                    let cell_text = row.get(ci).map(|s| s.as_str()).unwrap_or("");
                    let text_y = ry + (self.row_height - self.metrics.line_height) / 2.0;
                    ctx.renderer.draw_text(
                        cell_text,
                        (x + cx + self.padding, text_y),
                        tc,
                        ((cw - self.padding * 2.0).max(0.0), self.metrics.line_height),
                        self.metrics,
                        Align::Left,
                    );
                }

                // Row separator
                ctx.renderer.fill_rect_rounded(
                    (x, ry + self.row_height - 0.5, w, 0.5),
                    self.border_color,
                    0.0,
                );
            }
        });

        // Focus ring
        if self.focus {
            let total_h = self.header_height + self.rows.with(|r| r.len().min(self.max_visible)) as f32 * self.row_height;
            ctx.renderer.fill_rect_styled(
                (x, y, w, total_h),
                [0.0, 0.0, 0.0, 0.0],
                0.0,
                2.0,
                [0.3, 0.6, 0.9, 1.0],
            );
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;

        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                let inside = px >= layout.location.x
                    && px <= layout.location.x + layout.size.width
                    && py >= layout.location.y
                    && py <= layout.location.y + layout.size.height;

                let new_hover = if inside {
                    self.row_at(layout, py)
                } else {
                    None
                };

                self.hover_row = new_hover;
                false // don't consume — let siblings update hover too
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(idx) = self.hover_row {
                    let current = self.selected_row.get();
                    if current == Some(idx) {
                        self.set_selected_row.set(None);
                    } else {
                        self.set_selected_row.set(Some(idx));
                    }
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
        let count = self.rows.with(|r| r.len());
        if count == 0 {
            return false;
        }
        match &event.logical_key {
            Key::Named(NamedKey::ArrowDown) => {
                let current = self.selected_row.get().unwrap_or(0);
                let next = (current + 1).min(count - 1);
                self.set_selected_row.set(Some(next));
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                let current = self.selected_row.get().unwrap_or(0);
                let next = current.saturating_sub(1);
                self.set_selected_row.set(Some(next));
                true
            }
            Key::Named(NamedKey::Home) => {
                self.set_selected_row.set(Some(0));
                true
            }
            Key::Named(NamedKey::End) => {
                self.set_selected_row.set(Some(count - 1));
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
}
