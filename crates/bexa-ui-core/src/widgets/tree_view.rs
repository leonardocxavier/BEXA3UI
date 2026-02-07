use glyphon::Metrics;
use glyphon::cosmic_text::Align;
use taffy::prelude::*;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::framework::{DrawContext, EventContext, Widget};
use crate::icons;

/// A node in the tree structure.
pub struct TreeNode {
    pub label: String,
    pub icon: Option<&'static str>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

impl TreeNode {
    pub fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children: vec![],
            expanded: false,
        }
    }

    pub fn branch(label: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children,
            expanded: true,
        }
    }

    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }
}

pub struct TreeView {
    roots: Vec<TreeNode>,
    metrics: Metrics,
    row_height: f32,
    indent: f32,
    padding: f32,
    // Colors
    bg: [f32; 4],
    text_color: [u8; 3],
    icon_color: [u8; 3],
    hover_bg: [f32; 4],
    selected_bg: [f32; 4],
    selected_text: [u8; 3],
    connector_color: [f32; 4],
    // State
    hover_flat_idx: Option<usize>,
    selected_flat_idx: Option<usize>,
    focus: bool,
}

impl TreeView {
    pub fn new(roots: Vec<TreeNode>, metrics: Metrics) -> Self {
        Self {
            roots,
            metrics,
            row_height: 28.0,
            indent: 20.0,
            padding: 6.0,
            bg: [0.0, 0.0, 0.0, 0.0],
            text_color: [210, 210, 220],
            icon_color: [140, 170, 220],
            hover_bg: [0.18, 0.22, 0.30, 1.0],
            selected_bg: [0.20, 0.45, 0.70, 1.0],
            selected_text: [255, 255, 255],
            connector_color: [0.3, 0.35, 0.4, 0.6],
            hover_flat_idx: None,
            selected_flat_idx: None,
            focus: false,
        }
    }

    pub fn with_row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    pub fn with_indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn with_colors(
        mut self,
        text: [u8; 3],
        icon: [u8; 3],
        hover_bg: [f32; 4],
        selected_bg: [f32; 4],
    ) -> Self {
        self.text_color = text;
        self.icon_color = icon;
        self.hover_bg = hover_bg;
        self.selected_bg = selected_bg;
        self
    }

    /// Count total visible (flattened) rows.
    fn visible_count(&self) -> usize {
        fn count_nodes(nodes: &[TreeNode]) -> usize {
            let mut n = 0;
            for node in nodes {
                n += 1;
                if node.is_branch() && node.expanded {
                    n += count_nodes(&node.children);
                }
            }
            n
        }
        count_nodes(&self.roots)
    }

    /// Iterate visible rows: (flat_index, depth, &TreeNode)
    fn walk_visible(&self, mut f: impl FnMut(usize, usize, &TreeNode)) {
        fn walk(nodes: &[TreeNode], depth: usize, idx: &mut usize, f: &mut dyn FnMut(usize, usize, &TreeNode)) {
            for node in nodes {
                f(*idx, depth, node);
                *idx += 1;
                if node.is_branch() && node.expanded {
                    walk(&node.children, depth + 1, idx, f);
                }
            }
        }
        let mut idx = 0;
        walk(&self.roots, 0, &mut idx, &mut f);
    }

    /// Get mutable TreeNode by flat index.
    fn node_at_mut(&mut self, flat_idx: usize) -> Option<&mut TreeNode> {
        fn find(nodes: &mut [TreeNode], target: usize, idx: &mut usize) -> Option<*mut TreeNode> {
            for node in nodes.iter_mut() {
                if *idx == target {
                    return Some(node as *mut TreeNode);
                }
                *idx += 1;
                if node.is_branch() && node.expanded {
                    if let Some(found) = find(&mut node.children, target, idx) {
                        return Some(found);
                    }
                }
            }
            None
        }
        let mut idx = 0;
        // SAFETY: We never hold multiple mutable references; the pointer
        // is immediately converted back to a mutable reference.
        find(&mut self.roots, flat_idx, &mut idx).map(|ptr| unsafe { &mut *ptr })
    }

    fn toggle(&mut self, flat_idx: usize) {
        if let Some(node) = self.node_at_mut(flat_idx) {
            if node.is_branch() {
                node.expanded = !node.expanded;
            }
        }
    }
}

impl Widget for TreeView {
    fn style(&self) -> Style {
        let total_h = self.visible_count() as f32 * self.row_height;
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Length(total_h.max(self.row_height)),
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

        // Background
        if self.bg[3] > 0.0 {
            ctx.renderer.fill_rect_rounded(
                (x, y, w, layout.size.height),
                self.bg,
                0.0,
            );
        }

        let icon_metrics = Metrics::new(
            self.metrics.font_size * 0.8,
            self.metrics.line_height,
        );

        self.walk_visible(|flat_idx, depth, node| {
            let ry = y + flat_idx as f32 * self.row_height;
            let indent_x = x + depth as f32 * self.indent;

            let is_selected = self.selected_flat_idx == Some(flat_idx);
            let is_hover = self.hover_flat_idx == Some(flat_idx);

            // Row highlight
            if is_selected {
                ctx.renderer.fill_rect_rounded(
                    (x, ry, w, self.row_height),
                    self.selected_bg,
                    0.0,
                );
            } else if is_hover {
                ctx.renderer.fill_rect_rounded(
                    (x, ry, w, self.row_height),
                    self.hover_bg,
                    0.0,
                );
            }

            // Indent connectors (vertical lines)
            for d in 0..depth {
                let cx = x + d as f32 * self.indent + self.indent * 0.5;
                ctx.renderer.fill_rect_rounded(
                    (cx, ry, 1.0, self.row_height),
                    self.connector_color,
                    0.0,
                );
            }

            // Expand/collapse chevron for branches
            let mut text_x = indent_x + self.padding;
            if node.is_branch() {
                let chevron = if node.expanded {
                    icons::CHEVRON_DOWN
                } else {
                    icons::CHEVRON_RIGHT
                };
                let cy = ry + (self.row_height - icon_metrics.line_height) / 2.0;
                ctx.renderer.draw_text_with_font(
                    chevron,
                    (indent_x, cy),
                    self.icon_color,
                    (self.indent, icon_metrics.line_height),
                    icon_metrics,
                    Align::Center,
                    icons::NERD_FONT_FAMILY,
                );
                text_x = indent_x + self.indent;
            }

            // Node icon
            if let Some(icon) = node.icon {
                let iy = ry + (self.row_height - icon_metrics.line_height) / 2.0;
                ctx.renderer.draw_text_with_font(
                    icon,
                    (text_x, iy),
                    self.icon_color,
                    (16.0, icon_metrics.line_height),
                    icon_metrics,
                    Align::Center,
                    icons::NERD_FONT_FAMILY,
                );
                text_x += 20.0;
            }

            // Label
            let tc = if is_selected {
                self.selected_text
            } else {
                self.text_color
            };
            let text_y = ry + (self.row_height - self.metrics.line_height) / 2.0;
            let remaining = (w - (text_x - x)).max(0.0);
            ctx.renderer.draw_text(
                &node.label,
                (text_x, text_y),
                tc,
                (remaining, self.metrics.line_height),
                self.metrics,
                Align::Left,
            );
        });

        // Focus ring
        if self.focus {
            let total_h = self.visible_count() as f32 * self.row_height;
            ctx.renderer.fill_rect_styled(
                (x, y, w, total_h.max(self.row_height)),
                [0.0, 0.0, 0.0, 0.0],
                0.0,
                2.0,
                [0.3, 0.6, 0.9, 1.0],
            );
        }
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
                    && py >= layout.location.y;

                let new_hover = if inside {
                    let rel_y = py - layout.location.y;
                    let idx = (rel_y / self.row_height) as usize;
                    let count = self.visible_count();
                    if idx < count { Some(idx) } else { None }
                } else {
                    None
                };

                if new_hover != self.hover_flat_idx {
                    self.hover_flat_idx = new_hover;
                    changed = true;
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(idx) = self.hover_flat_idx {
                    self.selected_flat_idx = Some(idx);
                    self.toggle(idx);
                    changed = true;
                }
            }
            _ => {}
        }

        changed
    }

    fn handle_key_event(&mut self, event: &KeyEvent, _modifiers: ModifiersState) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        let count = self.visible_count();
        if count == 0 {
            return false;
        }
        match &event.logical_key {
            Key::Named(NamedKey::ArrowDown) => {
                let current = self.selected_flat_idx.unwrap_or(0);
                let next = (current + 1).min(count - 1);
                self.selected_flat_idx = Some(next);
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                let current = self.selected_flat_idx.unwrap_or(0);
                let next = current.saturating_sub(1);
                self.selected_flat_idx = Some(next);
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                if let Some(idx) = self.selected_flat_idx {
                    if let Some(node) = self.node_at_mut(idx) {
                        if node.is_branch() && !node.expanded {
                            node.expanded = true;
                            return true;
                        }
                    }
                }
                false
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if let Some(idx) = self.selected_flat_idx {
                    if let Some(node) = self.node_at_mut(idx) {
                        if node.is_branch() && node.expanded {
                            node.expanded = false;
                            return true;
                        }
                    }
                }
                false
            }
            Key::Named(NamedKey::Space) | Key::Named(NamedKey::Enter) => {
                if let Some(idx) = self.selected_flat_idx {
                    self.toggle(idx);
                    true
                } else {
                    false
                }
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
