use taffy::geometry::Point;
use taffy::prelude::*;
use winit::event::WindowEvent;

use crate::framework::{DrawContext, EventContext, Widget};
use crate::renderer::Renderer;

pub struct WidgetNode {
    pub(crate) widget: Box<dyn Widget>,
    pub(crate) children: Vec<WidgetNode>,
    pub(crate) node: Option<NodeId>,
    pub(crate) scroll_y: f32,
}

impl WidgetNode {
    pub fn new(widget: impl Widget + 'static, children: Vec<WidgetNode>) -> Self {
        Self {
            widget: Box::new(widget),
            children,
            node: None,
            scroll_y: 0.0,
        }
    }
}

pub fn build_taffy(node: &mut WidgetNode, taffy: &mut TaffyTree) -> NodeId {
    let child_nodes = node
        .children
        .iter_mut()
        .map(|child| build_taffy(child, taffy))
        .collect::<Vec<_>>();

    let style = node.widget.style();
    let node_id = if child_nodes.is_empty() {
        taffy.new_leaf(style).expect("create leaf")
    } else {
        taffy
            .new_with_children(style, &child_nodes)
            .expect("create node")
    };

    node.node = Some(node_id);
    node_id
}

pub fn sync_styles(node: &mut WidgetNode, taffy: &mut TaffyTree, width: f32, height: f32, is_root: bool) {
    let Some(node_id) = node.node else {
        return;
    };

    let mut style = node.widget.style();
    if is_root {
        style.size = Size {
            width: Dimension::Length(width),
            height: Dimension::Length(height),
        };
    }

    taffy.set_style(node_id, style).expect("set style");

    for child in &mut node.children {
        sync_styles(child, taffy, width, height, false);
    }
}

pub fn collect_focus_paths(node: &WidgetNode, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    if node.widget.is_focusable() {
        out.push(path.clone());
    }

    for (index, child) in node.children.iter().enumerate() {
        path.push(index);
        collect_focus_paths(child, path, out);
        path.pop();
    }
}

pub fn widget_mut_at_path<'a>(node: &'a mut WidgetNode, path: &[usize]) -> Option<&'a mut dyn Widget> {
    if path.is_empty() {
        return Some(node.widget.as_mut());
    }

    let idx = path[0];
    if idx >= node.children.len() {
        return None;
    }

    widget_mut_at_path(&mut node.children[idx], &path[1..])
}

pub fn draw_widgets(node: &WidgetNode, taffy: &TaffyTree, renderer: &mut Renderer) {
    draw_widgets_offset(node, taffy, renderer, 0.0, 0.0);
}

fn draw_widgets_offset(node: &WidgetNode, taffy: &TaffyTree, renderer: &mut Renderer, parent_x: f32, parent_y: f32) {
    let Some(node_id) = node.node else {
        return;
    };

    let layout = taffy.layout(node_id).expect("layout");
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;

    let mut absolute_layout = *layout;
    absolute_layout.location = Point { x: abs_x, y: abs_y };

    let mut ctx = DrawContext {
        renderer,
        layout: &absolute_layout,
    };
    node.widget.draw(&mut ctx);

    let is_scroll = node.widget.is_scrollable();
    if is_scroll {
        renderer.push_clip((abs_x, abs_y, layout.size.width, layout.size.height));
    }

    let child_y = abs_y - node.scroll_y;
    for child in &node.children {
        draw_widgets_offset(child, taffy, renderer, abs_x, child_y);
    }

    if is_scroll {
        renderer.pop_clip();
    }
}

pub fn dispatch_event(
    node: &mut WidgetNode,
    taffy: &TaffyTree,
    event: &WindowEvent,
    path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
    dispatch_event_offset(node, taffy, event, path, 0.0, 0.0)
}

fn dispatch_event_offset(
    node: &mut WidgetNode,
    taffy: &TaffyTree,
    event: &WindowEvent,
    path: &mut Vec<usize>,
    parent_x: f32,
    parent_y: f32,
) -> Option<Vec<usize>> {
    let Some(node_id) = node.node else {
        return None;
    };
    let layout = taffy.layout(node_id).expect("layout");
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;

    let child_y = abs_y - node.scroll_y;
    for (index, child) in node.children.iter_mut().enumerate() {
        path.push(index);
        if let Some(found) = dispatch_event_offset(child, taffy, event, path, abs_x, child_y) {
            return Some(found);
        }
        path.pop();
    }

    let mut absolute_layout = *layout;
    absolute_layout.location = Point { x: abs_x, y: abs_y };

    let mut ctx = EventContext {
        event,
        layout: &absolute_layout,
    };
    if node.widget.handle_event(&mut ctx) {
        return Some(path.clone());
    }

    None
}

/// Dispatches a scroll event to the deepest scrollable node under cursor,
/// falling back to the root node.
pub fn dispatch_scroll(node: &mut WidgetNode, delta_y: f32, cursor_x: f32, cursor_y: f32, taffy: &TaffyTree) {
    if !dispatch_scroll_offset(node, delta_y, cursor_x, cursor_y, taffy, 0.0, 0.0) {
        // Fallback: scroll root
        scroll_node(node, delta_y, taffy);
    }
}

fn dispatch_scroll_offset(
    node: &mut WidgetNode,
    delta_y: f32,
    cx: f32,
    cy: f32,
    taffy: &TaffyTree,
    parent_x: f32,
    parent_y: f32,
) -> bool {
    let Some(node_id) = node.node else { return false; };
    let layout = taffy.layout(node_id).expect("layout");
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;

    // Check if cursor is inside this node
    let inside = cx >= abs_x
        && cx <= abs_x + layout.size.width
        && cy >= abs_y
        && cy <= abs_y + layout.size.height;

    if !inside {
        return false;
    }

    // Try children first (deepest scrollable wins)
    let child_y = abs_y - node.scroll_y;
    for child in &mut node.children {
        if dispatch_scroll_offset(child, delta_y, cx, cy, taffy, abs_x, child_y) {
            return true;
        }
    }

    // If this node is scrollable, consume the scroll
    if node.widget.is_scrollable() {
        scroll_node(node, delta_y, taffy);
        return true;
    }

    false
}

fn scroll_node(node: &mut WidgetNode, delta_y: f32, taffy: &TaffyTree) {
    let Some(node_id) = node.node else { return; };
    let layout = taffy.layout(node_id).expect("layout");
    let container_h = layout.size.height;

    // Content height = max bottom edge of all children
    let mut content_h: f32 = 0.0;
    for child in &node.children {
        if let Some(child_id) = child.node {
            let cl = taffy.layout(child_id).expect("child layout");
            let bottom = cl.location.y + cl.size.height;
            content_h = content_h.max(bottom);
        }
    }

    let max_scroll = (content_h - container_h).max(0.0);
    node.scroll_y = (node.scroll_y - delta_y).clamp(0.0, max_scroll);
}

/// Scrolls the root node (backward compat).
pub fn scroll_root(node: &mut WidgetNode, delta_y: f32, viewport_h: f32, taffy: &TaffyTree) {
    let _ = viewport_h;
    scroll_node(node, delta_y, taffy);
}

pub fn update_widget_measures(node: &mut WidgetNode, measures: &[Vec<f32>]) {
    node.widget.update_measures(measures);
    for child in &mut node.children {
        update_widget_measures(child, measures);
    }
}

pub fn clear_active_widgets(node: &mut WidgetNode) {
    node.widget.clear_active();
    for child in &mut node.children {
        clear_active_widgets(child);
    }
}
