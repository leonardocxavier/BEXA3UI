use std::cell::Cell;
use std::time::Instant;

use arboard::Clipboard;
use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::SetSignal;

pub struct TextInput {
    text: String,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    on_change: Option<SetSignal<String>>,
    metrics: Metrics,
    text_color: [u8; 3],
    placeholder_color: [u8; 3],
    placeholder: String,
    background: [f32; 4],
    focus_border_color: [f32; 4],
    border_radius: f32,
    padding: f32,
    focused: bool,
    last_input_time: Instant,
    /// Cached pixel width of text before cursor, updated by render layer
    pub(crate) cursor_pixel_x: f32,
    /// Cached pixel positions for selection highlight
    selection_lo_px: f32,
    selection_hi_px: f32,
    /// Pixel x-positions of each character edge (0..=char_count), for click-to-position
    char_edges: Vec<f32>,
    /// Whether mouse is currently dragging a selection
    mouse_dragging: bool,
    /// Last known cursor position (window coords)
    last_mouse_x: f32,
    last_mouse_y: f32,
    /// Index of the text command emitted during draw (for measure feedback)
    text_cmd_index: Cell<Option<usize>>,
}

impl TextInput {
    pub fn new(on_change: SetSignal<String>) -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            selection: None,
            on_change: Some(on_change),
            metrics: Metrics::new(16.0, 22.0),
            text_color: [230, 230, 230],
            placeholder_color: [120, 120, 140],
            placeholder: String::new(),
            background: [0.12, 0.16, 0.22, 1.0],
            focus_border_color: [0.3, 0.6, 0.9, 1.0],
            border_radius: 6.0,
            padding: 10.0,
            focused: false,
            last_input_time: Instant::now(),
            cursor_pixel_x: 0.0,
            selection_lo_px: 0.0,
            selection_hi_px: 0.0,
            char_edges: Vec::new(),
            mouse_dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            text_cmd_index: Cell::new(None),
        }
    }

    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn with_text_color(mut self, color: [u8; 3]) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_background(mut self, color: [f32; 4]) -> Self {
        self.background = color;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_initial_value(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor_pos = self.text.len();
        self
    }

    fn notify_change(&self) {
        if let Some(ref sig) = self.on_change {
            sig.set(self.text.clone());
        }
    }

    fn insert_text(&mut self, s: &str) {
        self.delete_selection();
        let byte_pos = self.cursor_byte_pos();
        self.text.insert_str(byte_pos, s);
        self.cursor_pos += s.chars().count();
        self.last_input_time = Instant::now();
        self.notify_change();
    }

    fn delete_back(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let byte_pos = self.cursor_byte_pos();
            self.text.remove(byte_pos);
            self.last_input_time = Instant::now();
            self.notify_change();
        }
    }

    fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }
        let char_count = self.text.chars().count();
        if self.cursor_pos < char_count {
            let byte_pos = self.cursor_byte_pos();
            self.text.remove(byte_pos);
            self.last_input_time = Instant::now();
            self.notify_change();
        }
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection.take() {
            let (lo, hi) = if start < end { (start, end) } else { (end, start) };
            let lo_byte = self.char_to_byte(lo);
            let hi_byte = self.char_to_byte(hi);
            self.text.replace_range(lo_byte..hi_byte, "");
            self.cursor_pos = lo;
            self.last_input_time = Instant::now();
            self.notify_change();
            true
        } else {
            false
        }
    }

    fn move_cursor(&mut self, delta: i32, shift: bool) {
        let char_count = self.text.chars().count();
        let old_pos = self.cursor_pos;

        if delta < 0 {
            self.cursor_pos = self.cursor_pos.saturating_sub((-delta) as usize);
        } else {
            self.cursor_pos = (self.cursor_pos + delta as usize).min(char_count);
        }

        if shift {
            match self.selection {
                None => self.selection = Some((old_pos, self.cursor_pos)),
                Some((anchor, _)) => self.selection = Some((anchor, self.cursor_pos)),
            }
        } else {
            self.selection = None;
        }

        self.last_input_time = Instant::now();
    }

    fn select_all(&mut self) {
        let char_count = self.text.chars().count();
        self.selection = Some((0, char_count));
        self.cursor_pos = char_count;
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection?;
        let (lo, hi) = if start < end { (start, end) } else { (end, start) };
        let lo_byte = self.char_to_byte(lo);
        let hi_byte = self.char_to_byte(hi);
        Some(self.text[lo_byte..hi_byte].to_string())
    }

    fn copy_selection(&self) {
        if let Some(text) = self.selected_text() {
            if let Ok(mut cb) = Clipboard::new() {
                let _ = cb.set_text(text);
            }
        }
    }

    fn cut_selection(&mut self) {
        self.copy_selection();
        self.delete_selection();
    }

    fn paste(&mut self) {
        if let Ok(mut cb) = Clipboard::new() {
            if let Ok(text) = cb.get_text() {
                self.insert_text(&text);
            }
        }
    }

    fn cursor_byte_pos(&self) -> usize {
        self.char_to_byte(self.cursor_pos)
    }

    fn char_to_byte(&self, char_pos: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn cursor_visible(&self) -> bool {
        let elapsed = self.last_input_time.elapsed().as_millis();
        (elapsed % 1060) < 530
    }

    /// Returns the text substring before the cursor position.
    pub fn text_before_cursor(&self) -> &str {
        let byte_pos = self.cursor_byte_pos();
        &self.text[..byte_pos]
    }

    /// Returns the full text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns cursor position in chars.
    pub fn cursor(&self) -> usize {
        self.cursor_pos
    }

    /// Returns whether the widget has focus.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Given an absolute x pixel position, find the closest char position using glyph edges.
    fn char_pos_from_x(&self, layout: &Layout, x: f32) -> usize {
        let text_x = layout.location.x + self.padding;
        let rel_x = x - text_x;
        if self.char_edges.is_empty() {
            return 0;
        }
        // Find the edge closest to rel_x
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (i, &edge) in self.char_edges.iter().enumerate() {
            let dist = (edge - rel_x).abs();
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        best
    }

    fn hit_test(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && x <= layout.location.x + layout.size.width
            && y >= layout.location.y
            && y <= layout.location.y + layout.size.height
    }
}

impl Widget for TextInput {
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

        // Background
        let border_w = if self.focused { 1.5 } else { 0.0 };
        let border_c = if self.focused {
            self.focus_border_color
        } else {
            [0.0; 4]
        };
        ctx.renderer.fill_rect_styled(
            (x, y, w, h),
            self.background,
            self.border_radius,
            border_w,
            border_c,
        );

        let text_x = x + self.padding;
        let text_y = y + self.padding;
        let text_w = (w - self.padding * 2.0).max(0.0);
        let text_h = (h - self.padding * 2.0).max(0.0);

        // Selection highlight
        if let Some((start, end)) = self.selection {
            let (lo, hi) = if start < end { (start, end) } else { (end, start) };
            if lo != hi {
                let sel_x0 = text_x + self.selection_lo_px;
                let sel_x1 = text_x + self.selection_hi_px;
                let sel_w = (sel_x1 - sel_x0).max(0.0);
                ctx.renderer.fill_rect_rounded(
                    (sel_x0, text_y, sel_w, text_h),
                    [0.2, 0.4, 0.7, 0.5],
                    2.0,
                );
            }
        }

        // Text or placeholder
        if self.text.is_empty() && !self.placeholder.is_empty() {
            self.text_cmd_index.set(None);
            ctx.renderer.draw_text(
                &self.placeholder,
                (text_x, text_y),
                self.placeholder_color,
                (text_w, text_h),
                self.metrics,
                Align::Left,
            );
        } else {
            // Measure all char edges [0, 1, 2, ..., char_count] for mouse positioning
            let char_count = self.text.chars().count();
            let measure: Vec<usize> = (0..=char_count).collect();
            let idx = ctx.renderer.draw_text_measured(
                &self.text,
                (text_x, text_y),
                self.text_color,
                (text_w, text_h),
                self.metrics,
                Align::Left,
                measure,
            );
            self.text_cmd_index.set(Some(idx));
        }

        // Cursor (caret) — positioned using real pixel width from render layer
        if self.focused && self.cursor_visible() {
            let cursor_x = text_x + self.cursor_pixel_x;
            let cursor_h = self.metrics.font_size;
            let cursor_y = text_y + (text_h - cursor_h) * 0.5;
            ctx.renderer.fill_rect_rounded(
                (cursor_x, cursor_y, 1.5, cursor_h),
                [0.4, 0.7, 1.0, 1.0],
                0.0,
            );
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;
        match ctx.event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.hit_test(layout, self.last_mouse_x, self.last_mouse_y) {
                    let pos = self.char_pos_from_x(layout, self.last_mouse_x);
                    self.cursor_pos = pos;
                    self.selection = None;
                    self.mouse_dragging = true;
                    self.last_input_time = Instant::now();
                    true
                } else {
                    false
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_dragging = false;
                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.last_mouse_x = position.x as f32;
                self.last_mouse_y = position.y as f32;
                if self.mouse_dragging && self.focused {
                    let pos = self.char_pos_from_x(layout, position.x as f32);
                    if pos != self.cursor_pos {
                        let anchor = match self.selection {
                            Some((anchor, _)) => anchor,
                            None => self.cursor_pos,
                        };
                        self.cursor_pos = pos;
                        if anchor != pos {
                            self.selection = Some((anchor, pos));
                        } else {
                            self.selection = None;
                        }
                        self.last_input_time = Instant::now();
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn handle_key_event(&mut self, event: &KeyEvent, modifiers: ModifiersState) -> bool {
        let ctrl = modifiers.control_key();
        let shift = modifiers.shift_key();

        if ctrl {
            match &event.logical_key {
                Key::Character(c) if c.as_str() == "a" => {
                    self.select_all();
                    return true;
                }
                Key::Character(c) if c.as_str() == "c" => {
                    self.copy_selection();
                    return true;
                }
                Key::Character(c) if c.as_str() == "v" => {
                    self.paste();
                    return true;
                }
                Key::Character(c) if c.as_str() == "x" => {
                    self.cut_selection();
                    return true;
                }
                _ => {}
            }
        }

        match &event.logical_key {
            Key::Character(c) => {
                if !ctrl {
                    self.insert_text(c.as_str());
                    return true;
                }
                false
            }
            Key::Named(NamedKey::Backspace) => {
                self.delete_back();
                true
            }
            Key::Named(NamedKey::Delete) => {
                self.delete_forward();
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.move_cursor(-1, shift);
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.move_cursor(1, shift);
                true
            }
            Key::Named(NamedKey::Home) => {
                let old = self.cursor_pos;
                self.cursor_pos = 0;
                if shift {
                    match self.selection {
                        None => self.selection = Some((old, 0)),
                        Some((anchor, _)) => self.selection = Some((anchor, 0)),
                    }
                } else {
                    self.selection = None;
                }
                self.last_input_time = Instant::now();
                true
            }
            Key::Named(NamedKey::End) => {
                let old = self.cursor_pos;
                let end = self.text.chars().count();
                self.cursor_pos = end;
                if shift {
                    match self.selection {
                        None => self.selection = Some((old, end)),
                        Some((anchor, _)) => self.selection = Some((anchor, end)),
                    }
                } else {
                    self.selection = None;
                }
                self.last_input_time = Instant::now();
                true
            }
            Key::Named(NamedKey::Tab) => false,
            Key::Named(NamedKey::Enter) => false,
            _ => false,
        }
    }

    fn update_measures(&mut self, measures: &[Vec<f32>]) {
        if let Some(idx) = self.text_cmd_index.get() {
            if let Some(edges) = measures.get(idx) {
                self.char_edges = edges.clone();
                // cursor_pixel_x = edge at cursor_pos
                if let Some(&w) = edges.get(self.cursor_pos) {
                    self.cursor_pixel_x = w;
                }
                // selection highlight positions
                if let Some((start, end)) = self.selection {
                    let (lo, hi) = if start < end { (start, end) } else { (end, start) };
                    self.selection_lo_px = edges.get(lo).copied().unwrap_or(0.0);
                    self.selection_hi_px = edges.get(hi).copied().unwrap_or(0.0);
                }
            }
        }
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        if focused {
            self.last_input_time = Instant::now();
        }
    }
}
