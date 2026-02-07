#[derive(Clone, Copy)]
pub struct Theme {
    pub background: [f32; 3],
    pub panel: [f32; 3],
    pub button: [f32; 3],
    pub button_hover: [f32; 3],
    pub button_active: [f32; 3],
    pub button_focus: [f32; 3],
    pub text_primary: [u8; 3],
    pub text_secondary: [u8; 3],
    // Checkbox
    pub checkbox_bg: [f32; 4],
    pub checkbox_checked_bg: [f32; 4],
    pub checkbox_border: [f32; 4],
    pub checkbox_check: [u8; 3],
    // Radio
    pub radio_bg: [f32; 4],
    pub radio_border: [f32; 4],
    pub radio_dot: [f32; 4],
}

impl Theme {
    pub fn ocean() -> Self {
        Self {
            background: [0.12, 0.20, 0.30],
            panel: [0.16, 0.28, 0.38],
            button: [0.20, 0.65, 0.85],
            button_hover: [0.35, 0.75, 0.92],
            button_active: [0.15, 0.55, 0.75],
            button_focus: [0.18, 0.60, 0.82],
            text_primary: [230, 230, 230],
            text_secondary: [200, 200, 200],
            checkbox_bg: [0.16, 0.28, 0.38, 1.0],
            checkbox_checked_bg: [0.20, 0.65, 0.85, 1.0],
            checkbox_border: [0.4, 0.55, 0.7, 1.0],
            checkbox_check: [255, 255, 255],
            radio_bg: [0.16, 0.28, 0.38, 1.0],
            radio_border: [0.4, 0.55, 0.7, 1.0],
            radio_dot: [0.20, 0.65, 0.85, 1.0],
        }
    }
}
