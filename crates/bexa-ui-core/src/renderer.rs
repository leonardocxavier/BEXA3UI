use glyphon::Metrics;
use glyphon::cosmic_text::Align;

/// Clip rectangle (x, y, width, height) in pixel coords.
pub type ClipRect = (f32, f32, f32, f32);

#[derive(Clone, Copy)]
pub struct QuadCommand {
    pub rect: (f32, f32, f32, f32),
    pub color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub clip: Option<ClipRect>,
}

pub struct TextCommand {
    pub text: String,
    pub pos: (f32, f32),
    pub color: [u8; 3],
    pub bounds: (f32, f32),
    pub metrics: Metrics,
    pub align: Align,
    pub clip: Option<ClipRect>,
    pub font_family: Option<String>,
    /// Char indices to measure pixel widths at.
    /// Results stored in Renderer::text_measures at the same command index.
    pub measure_chars: Vec<usize>,
}

pub struct Renderer {
    pub quad_commands: Vec<QuadCommand>,
    pub text_commands: Vec<TextCommand>,
    /// Pixel widths measured by the render layer (indexed by TextCommand index).
    /// Each entry corresponds to `measure_chars` of the same TextCommand.
    pub text_measures: Vec<Vec<f32>>,
    clip_stack: Vec<ClipRect>,
    /// Overlay commands drawn on top of everything (for dropdowns, tooltips, etc.)
    pub overlay_quad_commands: Vec<QuadCommand>,
    pub overlay_text_commands: Vec<TextCommand>,
    viewport_size: (f32, f32),
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            quad_commands: Vec::new(),
            text_commands: Vec::new(),
            text_measures: Vec::new(),
            clip_stack: Vec::new(),
            overlay_quad_commands: Vec::new(),
            overlay_text_commands: Vec::new(),
            viewport_size: (0.0, 0.0),
        }
    }

    pub fn clear(&mut self) {
        self.quad_commands.clear();
        self.text_commands.clear();
        self.text_measures.clear();
        self.clip_stack.clear();
        self.overlay_quad_commands.clear();
        self.overlay_text_commands.clear();
    }

    pub fn set_viewport_size(&mut self, size: (f32, f32)) {
        self.viewport_size = size;
    }

    pub fn viewport_size(&self) -> (f32, f32) {
        self.viewport_size
    }

    /// Push a quad command to the overlay layer (drawn on top of everything).
    pub fn overlay_fill_rect_styled(
        &mut self,
        rect: (f32, f32, f32, f32),
        color: [f32; 4],
        border_radius: f32,
        border_width: f32,
        border_color: [f32; 4],
    ) {
        self.overlay_quad_commands.push(QuadCommand {
            rect,
            color,
            border_radius,
            border_width,
            border_color,
            clip: None,
        });
    }

    /// Push a text command to the overlay layer (drawn on top of everything).
    pub fn overlay_draw_text(
        &mut self,
        text: &str,
        pos: (f32, f32),
        color: [u8; 3],
        bounds: (f32, f32),
        metrics: Metrics,
        align: Align,
    ) {
        self.overlay_text_commands.push(TextCommand {
            text: text.to_string(),
            pos,
            color,
            bounds,
            metrics,
            align,
            clip: None,
            font_family: None,
            measure_chars: vec![],
        });
    }

    /// Push a text command with font to the overlay layer.
    pub fn overlay_draw_text_with_font(
        &mut self,
        text: &str,
        pos: (f32, f32),
        color: [u8; 3],
        bounds: (f32, f32),
        metrics: Metrics,
        align: Align,
        font_family: &str,
    ) {
        self.overlay_text_commands.push(TextCommand {
            text: text.to_string(),
            pos,
            color,
            bounds,
            metrics,
            align,
            clip: None,
            font_family: Some(font_family.to_string()),
            measure_chars: vec![],
        });
    }

    pub fn push_clip(&mut self, clip: ClipRect) {
        self.clip_stack.push(clip);
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn current_clip(&self) -> Option<ClipRect> {
        self.clip_stack.last().copied()
    }

    pub fn fill_rect(&mut self, rect: (f32, f32, f32, f32), color: [f32; 3]) {
        self.quad_commands.push(QuadCommand {
            rect,
            color: [color[0], color[1], color[2], 1.0],
            border_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
            clip: self.current_clip(),
        });
    }

    pub fn fill_rect_rounded(
        &mut self,
        rect: (f32, f32, f32, f32),
        color: [f32; 4],
        border_radius: f32,
    ) {
        self.quad_commands.push(QuadCommand {
            rect,
            color,
            border_radius,
            border_width: 0.0,
            border_color: [0.0; 4],
            clip: self.current_clip(),
        });
    }

    pub fn fill_rect_styled(
        &mut self,
        rect: (f32, f32, f32, f32),
        color: [f32; 4],
        border_radius: f32,
        border_width: f32,
        border_color: [f32; 4],
    ) {
        self.quad_commands.push(QuadCommand {
            rect,
            color,
            border_radius,
            border_width,
            border_color,
            clip: self.current_clip(),
        });
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        pos: (f32, f32),
        color: [u8; 3],
        bounds: (f32, f32),
        metrics: Metrics,
        align: Align,
    ) {
        self.text_commands.push(TextCommand {
            text: text.to_string(),
            pos,
            color,
            bounds,
            metrics,
            align,
            clip: self.current_clip(),
            font_family: None,
            measure_chars: vec![],
        });
    }

    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        pos: (f32, f32),
        color: [u8; 3],
        bounds: (f32, f32),
        metrics: Metrics,
        align: Align,
        font_family: &str,
    ) {
        self.text_commands.push(TextCommand {
            text: text.to_string(),
            pos,
            color,
            bounds,
            metrics,
            align,
            clip: self.current_clip(),
            font_family: Some(font_family.to_string()),
            measure_chars: vec![],
        });
    }

    pub fn draw_text_measured(
        &mut self,
        text: &str,
        pos: (f32, f32),
        color: [u8; 3],
        bounds: (f32, f32),
        metrics: Metrics,
        align: Align,
        measure_chars: Vec<usize>,
    ) -> usize {
        let idx = self.text_commands.len();
        self.text_commands.push(TextCommand {
            text: text.to_string(),
            pos,
            color,
            bounds,
            metrics,
            align,
            clip: self.current_clip(),
            font_family: None,
            measure_chars,
        });
        idx
    }
}
