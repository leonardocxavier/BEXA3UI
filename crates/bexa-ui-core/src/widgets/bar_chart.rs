use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::signal::Signal;

/// A single bar entry.
pub struct Bar {
    pub label: String,
    pub value: f32,
    pub color: [f32; 4],
}

impl Bar {
    pub fn new(label: impl Into<String>, value: f32, color: [f32; 4]) -> Self {
        Self {
            label: label.into(),
            value,
            color,
        }
    }
}

pub struct BarChart {
    bars: Signal<Vec<Bar>>,
    metrics: Metrics,
    height: f32,
    bar_gap: f32,
    bar_radius: f32,
    padding: f32,
    // Colors
    bg: [f32; 4],
    axis_color: [f32; 4],
    label_color: [u8; 3],
    value_color: [u8; 3],
    hover_opacity: f32,
    // State
    hover_index: Option<usize>,
    max_value: Option<f32>,
    show_grid: bool,
    grid_lines: usize,
}

impl BarChart {
    pub fn new(bars: Signal<Vec<Bar>>, metrics: Metrics, height: f32) -> Self {
        Self {
            bars,
            metrics,
            height,
            bar_gap: 8.0,
            bar_radius: 4.0,
            padding: 8.0,
            bg: [0.0, 0.0, 0.0, 0.0],
            axis_color: [0.3, 0.35, 0.4, 1.0],
            label_color: [160, 170, 180],
            value_color: [220, 220, 230],
            hover_opacity: 0.3,
            hover_index: None,
            max_value: None,
            show_grid: true,
            grid_lines: 4,
        }
    }

    pub fn with_bar_gap(mut self, gap: f32) -> Self {
        self.bar_gap = gap;
        self
    }

    pub fn with_bar_radius(mut self, radius: f32) -> Self {
        self.bar_radius = radius;
        self
    }

    pub fn with_max_value(mut self, max: f32) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn with_grid(mut self, show: bool, lines: usize) -> Self {
        self.show_grid = show;
        self.grid_lines = lines;
        self
    }

    pub fn with_colors(
        mut self,
        bg: [f32; 4],
        axis: [f32; 4],
        label: [u8; 3],
        value: [u8; 3],
    ) -> Self {
        self.bg = bg;
        self.axis_color = axis;
        self.label_color = label;
        self.value_color = value;
        self
    }

    fn effective_max(&self) -> f32 {
        if let Some(m) = self.max_value {
            return m;
        }
        self.bars.with(|bars| {
            bars.iter().map(|b| b.value).fold(0.0_f32, f32::max).max(1.0)
        })
    }
}

impl Widget for BarChart {
    fn style(&self) -> Style {
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(self.height),
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
        if self.bg[3] > 0.0 {
            ctx.renderer.fill_rect_rounded(
                (x, y, w, h),
                self.bg,
                0.0,
            );
        }

        let label_area_h = self.metrics.line_height + 4.0;
        let value_area_h = self.metrics.line_height;
        let y_label_w: f32 = 44.0; // reserved width for Y-axis labels
        let chart_top = y + self.padding + value_area_h;
        let chart_bottom = y + h - self.padding - label_area_h;
        let chart_h = (chart_bottom - chart_top).max(10.0);
        let chart_left = x + self.padding + y_label_w;
        let chart_right = x + w - self.padding;
        let chart_w = (chart_right - chart_left).max(10.0);

        let max_val = self.effective_max();

        // Grid lines
        if self.show_grid && self.grid_lines > 0 {
            let small_metrics = Metrics::new(
                self.metrics.font_size * 0.7,
                self.metrics.line_height * 0.7,
            );
            for i in 0..=self.grid_lines {
                let frac = i as f32 / self.grid_lines as f32;
                let gy = chart_bottom - frac * chart_h;
                ctx.renderer.fill_rect_rounded(
                    (chart_left, gy, chart_w, 0.5),
                    [self.axis_color[0], self.axis_color[1], self.axis_color[2], 0.3],
                    0.0,
                );
                // Grid value label (drawn in the reserved Y-axis area)
                let val = frac * max_val;
                let val_str = if val >= 1000.0 {
                    format!("{:.0}k", val / 1000.0)
                } else if val == val.floor() {
                    format!("{:.0}", val)
                } else {
                    format!("{:.1}", val)
                };
                ctx.renderer.draw_text(
                    &val_str,
                    (x + self.padding, gy - small_metrics.line_height * 0.5),
                    self.label_color,
                    (y_label_w - 4.0, small_metrics.line_height),
                    small_metrics,
                    Align::Right,
                );
            }
        }

        // Bottom axis
        ctx.renderer.fill_rect_rounded(
            (chart_left, chart_bottom, chart_w, 1.0),
            self.axis_color,
            0.0,
        );

        // Bars
        self.bars.with(|bars| {
            let count = bars.len();
            if count == 0 {
                return;
            }

            let total_gaps = (count.saturating_sub(1)) as f32 * self.bar_gap;
            let bar_w = ((chart_w - total_gaps) / count as f32).max(4.0);

            for (i, bar) in bars.iter().enumerate() {
                let bx = chart_left + i as f32 * (bar_w + self.bar_gap);
                let bar_h = (bar.value / max_val * chart_h).min(chart_h);
                let by = chart_bottom - bar_h;

                let is_hover = self.hover_index == Some(i);

                // Bar rect
                let mut color = bar.color;
                if is_hover {
                    // Brighten on hover
                    color[0] = (color[0] + self.hover_opacity).min(1.0);
                    color[1] = (color[1] + self.hover_opacity).min(1.0);
                    color[2] = (color[2] + self.hover_opacity).min(1.0);
                }

                ctx.renderer.fill_rect_rounded(
                    (bx, by, bar_w, bar_h),
                    color,
                    self.bar_radius,
                );

                // Value on top of bar
                if is_hover || bar_h > value_area_h + 4.0 {
                    let val_str = if bar.value >= 1000.0 {
                        format!("{:.1}k", bar.value / 1000.0)
                    } else if bar.value == bar.value.floor() {
                        format!("{:.0}", bar.value)
                    } else {
                        format!("{:.1}", bar.value)
                    };
                    let val_metrics = Metrics::new(
                        self.metrics.font_size * 0.8,
                        self.metrics.line_height * 0.8,
                    );
                    ctx.renderer.draw_text(
                        &val_str,
                        (bx, by - val_metrics.line_height - 2.0),
                        self.value_color,
                        (bar_w, val_metrics.line_height),
                        val_metrics,
                        Align::Center,
                    );
                }

                // Label below axis
                let label_metrics = Metrics::new(
                    self.metrics.font_size * 0.75,
                    self.metrics.line_height * 0.75,
                );
                ctx.renderer.draw_text(
                    &bar.label,
                    (bx, chart_bottom + 4.0),
                    self.label_color,
                    (bar_w, label_metrics.line_height),
                    label_metrics,
                    Align::Center,
                );
            }
        });
    }

    fn handle_event(&mut self, ctx: &mut EventContext) -> bool {
        let layout = ctx.layout;
        let mut changed = false;

        match ctx.event {
            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                let inside = px >= layout.location.x
                    && px <= layout.location.x + layout.size.width
                    && py >= layout.location.y
                    && py <= layout.location.y + layout.size.height;

                let new_hover = if inside {
                    let label_area_h = self.metrics.line_height + 4.0;
                    let value_area_h = self.metrics.line_height;
                    let y_label_w: f32 = 44.0;
                    let chart_left = layout.location.x + self.padding + y_label_w;
                    let chart_w = (layout.size.width - self.padding * 2.0 - y_label_w).max(10.0);
                    let chart_bottom = layout.location.y + layout.size.height - self.padding - label_area_h;
                    let chart_top = layout.location.y + self.padding + value_area_h;

                    if py >= chart_top && py <= chart_bottom + label_area_h {
                        let count = self.bars.with(|b| b.len());
                        if count > 0 {
                            let total_gaps = (count.saturating_sub(1)) as f32 * self.bar_gap;
                            let bar_w = ((chart_w - total_gaps) / count as f32).max(4.0);
                            let rel_x = px - chart_left;
                            let idx = (rel_x / (bar_w + self.bar_gap)) as usize;
                            if idx < count { Some(idx) } else { None }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if new_hover != self.hover_index {
                    self.hover_index = new_hover;
                    changed = true;
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.hover_index.is_some() {
                    changed = true;
                }
            }
            _ => {}
        }

        changed
    }
}
