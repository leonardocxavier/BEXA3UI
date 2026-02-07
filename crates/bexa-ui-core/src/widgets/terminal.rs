use std::io::Write;
use std::sync::{Arc, Mutex};

use glyphon::cosmic_text::Align;
use glyphon::Metrics;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};

// ── Terminal cell ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TermCell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: Option<[u8; 3]>,
    pub bold: bool,
}

impl Default for TermCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: [204, 204, 204],
            bg: None,
            bold: false,
        }
    }
}

// ── Terminal grid (VTE performer) ────────────────────────────────────────

pub struct TermGrid {
    pub cells: Vec<Vec<TermCell>>,
    pub rows: usize,
    pub cols: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    // SGR state
    current_fg: [u8; 3],
    current_bg: Option<[u8; 3]>,
    current_bold: bool,
    // Scroll region
    scroll_top: usize,
    scroll_bottom: usize,
    // PTY writer for responding to DSR queries
    pty_writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
}

impl TermGrid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![vec![TermCell::default(); cols]; rows];
        Self {
            cells,
            rows,
            cols,
            cursor_row: 0,
            cursor_col: 0,
            current_fg: [204, 204, 204],
            current_bg: None,
            current_bold: false,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            pty_writer: None,
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;
        self.cells.resize(rows, vec![TermCell::default(); cols]);
        for row in &mut self.cells {
            row.resize(cols, TermCell::default());
        }
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.scroll_top = 0;
        self.scroll_bottom = rows.saturating_sub(1);
    }

    fn scroll_up(&mut self) {
        if self.scroll_top < self.scroll_bottom && self.scroll_bottom < self.rows {
            self.cells.remove(self.scroll_top);
            self.cells
                .insert(self.scroll_bottom, vec![TermCell::default(); self.cols]);
        }
    }

    fn newline(&mut self) {
        if self.cursor_row == self.scroll_bottom {
            self.scroll_up();
        } else if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        }
    }

    fn erase_in_display(&mut self, mode: u16) {
        match mode {
            0 => {
                // Erase from cursor to end of screen
                for col in self.cursor_col..self.cols {
                    self.cells[self.cursor_row][col] = TermCell::default();
                }
                for row in (self.cursor_row + 1)..self.rows {
                    for col in 0..self.cols {
                        self.cells[row][col] = TermCell::default();
                    }
                }
            }
            1 => {
                // Erase from start to cursor
                for row in 0..self.cursor_row {
                    for col in 0..self.cols {
                        self.cells[row][col] = TermCell::default();
                    }
                }
                for col in 0..=self.cursor_col.min(self.cols.saturating_sub(1)) {
                    self.cells[self.cursor_row][col] = TermCell::default();
                }
            }
            2 | 3 => {
                // Erase entire screen
                for row in &mut self.cells {
                    for cell in row.iter_mut() {
                        *cell = TermCell::default();
                    }
                }
            }
            _ => {}
        }
    }

    fn erase_in_line(&mut self, mode: u16) {
        let row = self.cursor_row;
        match mode {
            0 => {
                for col in self.cursor_col..self.cols {
                    self.cells[row][col] = TermCell::default();
                }
            }
            1 => {
                for col in 0..=self.cursor_col.min(self.cols.saturating_sub(1)) {
                    self.cells[row][col] = TermCell::default();
                }
            }
            2 => {
                for col in 0..self.cols {
                    self.cells[row][col] = TermCell::default();
                }
            }
            _ => {}
        }
    }

    fn apply_sgr(&mut self, params: &vte::Params) {
        let mut iter = params.iter();
        loop {
            let param = match iter.next() {
                Some(slice) => slice[0],
                None => break,
            };
            match param {
                0 => {
                    self.current_fg = [204, 204, 204];
                    self.current_bg = None;
                    self.current_bold = false;
                }
                1 => self.current_bold = true,
                22 => self.current_bold = false,
                // Standard foreground colors
                30 => self.current_fg = [0, 0, 0],
                31 => self.current_fg = [205, 49, 49],
                32 => self.current_fg = [13, 188, 121],
                33 => self.current_fg = [229, 229, 16],
                34 => self.current_fg = [36, 114, 200],
                35 => self.current_fg = [188, 63, 188],
                36 => self.current_fg = [17, 168, 205],
                37 => self.current_fg = [204, 204, 204],
                39 => self.current_fg = [204, 204, 204], // default fg
                // Standard background colors
                40 => self.current_bg = Some([0, 0, 0]),
                41 => self.current_bg = Some([205, 49, 49]),
                42 => self.current_bg = Some([13, 188, 121]),
                43 => self.current_bg = Some([229, 229, 16]),
                44 => self.current_bg = Some([36, 114, 200]),
                45 => self.current_bg = Some([188, 63, 188]),
                46 => self.current_bg = Some([17, 168, 205]),
                47 => self.current_bg = Some([204, 204, 204]),
                49 => self.current_bg = None, // default bg
                // Bright foreground
                90 => self.current_fg = [128, 128, 128],
                91 => self.current_fg = [255, 0, 0],
                92 => self.current_fg = [0, 255, 0],
                93 => self.current_fg = [255, 255, 0],
                94 => self.current_fg = [0, 0, 255],
                95 => self.current_fg = [255, 0, 255],
                96 => self.current_fg = [0, 255, 255],
                97 => self.current_fg = [255, 255, 255],
                // 256-color: 38;5;N
                38 => {
                    if let Some(five) = iter.next() {
                        if five[0] == 5 {
                            if let Some(n) = iter.next() {
                                self.current_fg = ansi_256_to_rgb(n[0]);
                            }
                        }
                    }
                }
                // 256-color bg: 48;5;N
                48 => {
                    if let Some(five) = iter.next() {
                        if five[0] == 5 {
                            if let Some(n) = iter.next() {
                                self.current_bg = Some(ansi_256_to_rgb(n[0]));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl vte::Perform for TermGrid {
    fn print(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.newline();
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.cells[self.cursor_row][self.cursor_col] = TermCell {
                ch: c,
                fg: self.current_fg,
                bg: self.current_bg,
                bold: self.current_bold,
            };
            self.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.cursor_col = 0,
            0x08 => {
                // Backspace
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            0x07 => {
                // Bell - ignore
            }
            0x09 => {
                // Tab - advance to next 8-column stop
                self.cursor_col = ((self.cursor_col / 8) + 1) * 8;
                if self.cursor_col >= self.cols {
                    self.cursor_col = self.cols.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        // Ignore DEC private mode sequences (CSI ? ... h/l) — just acknowledge silently
        if intermediates.contains(&b'?') {
            return;
        }
        let p = |idx: usize, default: u16| -> u16 {
            params
                .iter()
                .nth(idx)
                .map(|s| if s[0] == 0 { default } else { s[0] })
                .unwrap_or(default)
        };

        match action {
            'A' => {
                // Cursor up
                let n = p(0, 1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                // Cursor down
                let n = p(0, 1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1));
            }
            'C' => {
                // Cursor forward
                let n = p(0, 1) as usize;
                self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1));
            }
            'D' => {
                // Cursor backward
                let n = p(0, 1) as usize;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor position (1-based)
                let row = p(0, 1) as usize;
                let col = p(1, 1) as usize;
                self.cursor_row = row.saturating_sub(1).min(self.rows.saturating_sub(1));
                self.cursor_col = col.saturating_sub(1).min(self.cols.saturating_sub(1));
            }
            'J' => {
                let mode = p(0, 0);
                self.erase_in_display(mode);
            }
            'K' => {
                let mode = p(0, 0);
                self.erase_in_line(mode);
            }
            'm' => {
                // SGR
                if params.len() == 0 {
                    // Reset
                    self.current_fg = [204, 204, 204];
                    self.current_bg = None;
                    self.current_bold = false;
                } else {
                    self.apply_sgr(params);
                }
            }
            'r' => {
                // Set scroll region
                let top = p(0, 1) as usize;
                let bottom = p(1, self.rows as u16) as usize;
                self.scroll_top = top.saturating_sub(1).min(self.rows.saturating_sub(1));
                self.scroll_bottom = bottom.saturating_sub(1).min(self.rows.saturating_sub(1));
            }
            'L' => {
                // Insert lines
                let n = p(0, 1) as usize;
                for _ in 0..n {
                    if self.cursor_row <= self.scroll_bottom && self.scroll_bottom < self.rows {
                        if self.scroll_bottom < self.cells.len() {
                            self.cells.remove(self.scroll_bottom);
                        }
                        self.cells
                            .insert(self.cursor_row, vec![TermCell::default(); self.cols]);
                    }
                }
            }
            'M' => {
                // Delete lines
                let n = p(0, 1) as usize;
                for _ in 0..n {
                    if self.cursor_row < self.cells.len() {
                        self.cells.remove(self.cursor_row);
                        let insert_pos = self.scroll_bottom.min(self.cells.len());
                        self.cells
                            .insert(insert_pos, vec![TermCell::default(); self.cols]);
                    }
                }
            }
            'P' => {
                // Delete characters
                let n = p(0, 1) as usize;
                let row = self.cursor_row;
                for _ in 0..n {
                    if self.cursor_col < self.cells[row].len() {
                        self.cells[row].remove(self.cursor_col);
                        self.cells[row].push(TermCell::default());
                    }
                }
            }
            '@' => {
                // Insert characters
                let n = p(0, 1) as usize;
                let row = self.cursor_row;
                for _ in 0..n {
                    self.cells[row].insert(self.cursor_col, TermCell::default());
                    self.cells[row].truncate(self.cols);
                }
            }
            'n' => {
                // Device Status Report
                let mode = p(0, 0);
                if mode == 6 {
                    // CPR: respond with cursor position (1-based)
                    let response = format!("\x1b[{};{}R", self.cursor_row + 1, self.cursor_col + 1);
                    if let Some(ref writer) = self.pty_writer {
                        if let Ok(mut w) = writer.lock() {
                            let _ = w.write_all(response.as_bytes());
                            let _ = w.flush();
                        }
                    }
                }
            }
            'd' => {
                // Vertical line position absolute (1-based)
                let row = p(0, 1) as usize;
                self.cursor_row = row.saturating_sub(1).min(self.rows.saturating_sub(1));
            }
            'G' => {
                // Cursor horizontal absolute (1-based)
                let col = p(0, 1) as usize;
                self.cursor_col = col.saturating_sub(1).min(self.cols.saturating_sub(1));
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'M' => {
                // Reverse index (scroll down)
                if self.cursor_row == self.scroll_top {
                    // Insert line at top
                    if self.scroll_bottom < self.cells.len() {
                        self.cells.remove(self.scroll_bottom);
                    }
                    self.cells
                        .insert(self.scroll_top, vec![TermCell::default(); self.cols]);
                } else {
                    self.cursor_row = self.cursor_row.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
}

// ── 256-color lookup ─────────────────────────────────────────────────────

fn ansi_256_to_rgb(n: u16) -> [u8; 3] {
    if n < 16 {
        // Standard colors
        match n {
            0 => [0, 0, 0],
            1 => [205, 49, 49],
            2 => [13, 188, 121],
            3 => [229, 229, 16],
            4 => [36, 114, 200],
            5 => [188, 63, 188],
            6 => [17, 168, 205],
            7 => [204, 204, 204],
            8 => [128, 128, 128],
            9 => [255, 0, 0],
            10 => [0, 255, 0],
            11 => [255, 255, 0],
            12 => [0, 0, 255],
            13 => [255, 0, 255],
            14 => [0, 255, 255],
            15 => [255, 255, 255],
            _ => [204, 204, 204],
        }
    } else if n < 232 {
        // 216-color cube: 16..231
        let idx = (n - 16) as u8;
        let r = idx / 36;
        let g = (idx % 36) / 6;
        let b = idx % 6;
        let to_val = |v: u8| if v == 0 { 0u8 } else { 55 + 40 * v };
        [to_val(r), to_val(g), to_val(b)]
    } else {
        // Grayscale: 232..255
        let v = 8 + 10 * (n - 232) as u8;
        [v, v, v]
    }
}

// ── Terminal Widget ──────────────────────────────────────────────────────

pub struct Terminal {
    grid: Arc<Mutex<TermGrid>>,
    pty_writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    metrics: Metrics,
    focus: bool,
    font_family: String,
    bg_color: [f32; 4],
}

impl Terminal {
    pub fn new(metrics: Metrics) -> Self {
        // Estimate initial grid size (will be adjusted when we know layout dimensions)
        let initial_cols = 80;
        let initial_rows = 24;
        let grid = Arc::new(Mutex::new(TermGrid::new(initial_rows, initial_cols)));
        let grid_clone = grid.clone();

        let mut terminal = Self {
            grid,
            pty_writer: None,
            metrics,
            focus: false,
            font_family: "Consolas".to_string(),
            bg_color: [0.07, 0.07, 0.10, 1.0],
        };

        terminal.spawn_pty(initial_rows, initial_cols, grid_clone);
        terminal
    }

    pub fn with_font(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    pub fn with_background(mut self, color: [f32; 4]) -> Self {
        self.bg_color = color;
        self
    }

    fn spawn_pty(&mut self, rows: usize, cols: usize, grid: Arc<Mutex<TermGrid>>) {
        use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("open pty");

        // Spawn shell
        let cmd = CommandBuilder::new_default_prog();
        let mut child = pair.slave.spawn_command(cmd).expect("spawn shell");

        // Writer for sending input
        let writer = pair.master.take_writer().expect("take writer");
        let writer_arc: Arc<Mutex<Box<dyn Write + Send>>> = Arc::new(Mutex::new(Box::new(writer)));
        self.pty_writer = Some(writer_arc.clone());

        // Give the grid access to the writer so it can respond to DSR queries
        grid.lock().unwrap().pty_writer = Some(writer_arc.clone());

        // Reader thread: reads PTY output and feeds VTE parser
        let reader = pair.master.try_clone_reader().expect("clone reader");
        // Keep slave alive — dropping it on Windows (ConPTY) kills the PTY immediately
        let slave = pair.slave;
        std::thread::spawn(move || {
            use std::io::Read;
            let _slave = slave; // prevent drop until thread ends
            let mut reader = reader;
            let mut parser = vte::Parser::new();
            let mut buf = [0u8; 4096];

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut g = grid.lock().unwrap();
                        parser.advance(&mut *g, &buf[..n]);
                    }
                    Err(_) => break,
                }
            }

            // Child process exited
            let _ = child.wait();
        });
    }

    fn write_to_pty(&self, data: &[u8]) {
        if let Some(ref writer) = self.pty_writer {
            if let Ok(mut w) = writer.lock() {
                let _ = w.write_all(data);
                let _ = w.flush();
            }
        }
    }
}

impl Widget for Terminal {
    fn style(&self) -> Style {
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
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
        ctx.renderer.fill_rect_styled(
            (x, y, w, h),
            self.bg_color,
            0.0,
            0.0,
            [0.0; 4],
        );

        let grid = self.grid.lock().unwrap();
        let char_w = self.metrics.font_size * 0.6;
        let line_h = self.metrics.line_height;

        // Draw cells row by row — each character placed at its exact grid position
        for (row_idx, row) in grid.cells.iter().enumerate() {
            let cy = y + row_idx as f32 * line_h;
            if cy + line_h < y || cy > y + h {
                continue;
            }

            // First pass: draw background colored cells
            let mut col = 0;
            while col < row.len() {
                let cell = &row[col];
                if let Some(bg) = cell.bg {
                    let start_col = col;
                    while col < row.len() && row[col].bg == Some(bg) {
                        col += 1;
                    }
                    let cx = x + start_col as f32 * char_w;
                    let run_w = (col - start_col) as f32 * char_w;
                    ctx.renderer.fill_rect_styled(
                        (cx, cy, run_w, line_h),
                        [bg[0] as f32 / 255.0, bg[1] as f32 / 255.0, bg[2] as f32 / 255.0, 1.0],
                        0.0,
                        0.0,
                        [0.0; 4],
                    );
                } else {
                    col += 1;
                }
            }

            // Second pass: draw each non-space character individually at its grid position
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.ch == ' ' {
                    continue;
                }
                let cx = x + col_idx as f32 * char_w;
                let mut buf = [0u8; 4];
                let s = cell.ch.encode_utf8(&mut buf);
                ctx.renderer.draw_text_with_font(
                    s,
                    (cx, cy),
                    cell.fg,
                    (char_w * 2.0, line_h),
                    self.metrics,
                    Align::Left,
                    &self.font_family,
                );
            }
        }

        // Draw cursor
        if self.focus && grid.cursor_row < grid.rows && grid.cursor_col < grid.cols {
            let cursor_x = x + grid.cursor_col as f32 * char_w;
            let cursor_y = y + grid.cursor_row as f32 * line_h;
            ctx.renderer.fill_rect_styled(
                (cursor_x, cursor_y, char_w, line_h),
                [0.8, 0.8, 0.8, 0.6],
                0.0,
                0.0,
                [0.0; 4],
            );
        }
    }

    fn handle_key_event(&mut self, event: &KeyEvent, modifiers: ModifiersState) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }

        // Ctrl+key combinations
        if modifiers.control_key() {
            match &event.logical_key {
                Key::Character(c) => {
                    let ch = c.chars().next().unwrap_or('\0');
                    if ch >= 'a' && ch <= 'z' {
                        let ctrl_byte = (ch as u8) - b'a' + 1;
                        self.write_to_pty(&[ctrl_byte]);
                        return true;
                    }
                    if ch >= 'A' && ch <= 'Z' {
                        let ctrl_byte = (ch as u8) - b'A' + 1;
                        self.write_to_pty(&[ctrl_byte]);
                        return true;
                    }
                }
                _ => {}
            }
        }

        match &event.logical_key {
            Key::Named(NamedKey::Enter) => {
                self.write_to_pty(b"\r");
                true
            }
            Key::Named(NamedKey::Backspace) => {
                self.write_to_pty(b"\x7f");
                true
            }
            Key::Named(NamedKey::Tab) => {
                self.write_to_pty(b"\t");
                true
            }
            Key::Named(NamedKey::Escape) => {
                self.write_to_pty(b"\x1b");
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.write_to_pty(b"\x1b[A");
                true
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.write_to_pty(b"\x1b[B");
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.write_to_pty(b"\x1b[C");
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.write_to_pty(b"\x1b[D");
                true
            }
            Key::Named(NamedKey::Home) => {
                self.write_to_pty(b"\x1b[H");
                true
            }
            Key::Named(NamedKey::End) => {
                self.write_to_pty(b"\x1b[F");
                true
            }
            Key::Named(NamedKey::Delete) => {
                self.write_to_pty(b"\x1b[3~");
                true
            }
            Key::Named(NamedKey::PageUp) => {
                self.write_to_pty(b"\x1b[5~");
                true
            }
            Key::Named(NamedKey::PageDown) => {
                self.write_to_pty(b"\x1b[6~");
                true
            }
            Key::Character(c) => {
                self.write_to_pty(c.as_bytes());
                true
            }
            _ => false,
        }
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } = ctx.event
        {
            // Click anywhere on the terminal area to focus it
            return true;
        }
        false
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focus = focused;
    }
}
