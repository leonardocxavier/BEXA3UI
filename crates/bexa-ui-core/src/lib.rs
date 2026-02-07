pub mod framework;
pub mod icons;
pub mod renderer;
pub mod signal;
pub mod theme;
pub mod tree;
pub mod widgets;

pub use framework::{DrawContext, EventContext, Widget};
pub use renderer::{ImageFit, QuadCommand, Renderer, TextCommand};
pub use signal::{Signal, SetSignal, IntoSignal, create_signal};
pub use theme::Theme;
pub use tree::{
    build_taffy, clear_active_widgets, collect_focus_paths, dispatch_event, dispatch_scroll,
    draw_widgets, handle_scrollbar_event, release_scrollbar_drag, scroll_root, sync_styles,
    try_start_scrollbar_drag, update_widget_measures, widget_mut_at_path, WidgetNode,
};
pub use widgets::{Bar, BarChart, Button, Checkbox, Column, Container, Flex, Icon, Image, Label, Modal, RadioButton, radio_group, ScrollView, Select, Slider, Table, Tabs, TextInput, Toggle, Tooltip, TooltipPosition, TreeNode, TreeView};

#[cfg(feature = "terminal")]
pub use widgets::Terminal;

// Re-export text types so downstream crates don't need glyphon directly
pub use glyphon::Metrics;
pub use glyphon::cosmic_text::Align;

use std::sync::{Arc, Mutex};

/// Request to open a new window from within a widget callback.
pub struct WindowRequest {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub root: WidgetNode,
    pub theme: Theme,
}

/// Shared handle for widgets to request new windows.
pub type WindowRequests = Arc<Mutex<Vec<WindowRequest>>>;

/// Create a new `WindowRequests` handle.
pub fn create_window_requests() -> WindowRequests {
    Arc::new(Mutex::new(Vec::new()))
}

/// Declarative UI macro for building widget trees.
///
/// # Syntax
/// ```ignore
/// ui! {
///     Container::new() => {
///         Flex::row(12.0) => { btn_a, btn_b },
///         label,
///     }
/// }
/// ```
///
/// - `widget => { children... }` — container with children
/// - `widget` — leaf node (no children), automatically wrapped
/// - Children can be bare widgets or nested `container => { ... }` expressions
/// - `ui!(widget)` still works for explicit leaf wrapping
#[macro_export]
macro_rules! ui {
    // Container with children: Widget => { child, child, ... }
    ( $widget:expr => { $( $child:tt )* } ) => {
        $crate::WidgetNode::new($widget, $crate::ui_items![ $( $child )* ])
    };

    // Leaf: Widget (no children)
    ( $widget:expr ) => {
        $crate::WidgetNode::new($widget, vec![])
    };
}

/// Internal macro to parse a comma-separated list of UI items.
/// Each item is either `widget => { children }` or a bare expression.
#[macro_export]
#[doc(hidden)]
macro_rules! ui_items {
    // Empty
    () => { vec![] };

    // Container item: expr => { ... }, rest...
    ( $widget:expr => { $( $inner:tt )* } , $( $rest:tt )* ) => {{
        let mut v = vec![ $crate::ui!( $widget => { $( $inner )* } ) ];
        v.extend( $crate::ui_items![ $( $rest )* ] );
        v
    }};

    // Container item: expr => { ... } (last, no trailing comma)
    ( $widget:expr => { $( $inner:tt )* } ) => {
        vec![ $crate::ui!( $widget => { $( $inner )* } ) ]
    };

    // Leaf item: expr, rest...
    ( $widget:expr , $( $rest:tt )* ) => {{
        let mut v = vec![ $crate::ui!( $widget ) ];
        v.extend( $crate::ui_items![ $( $rest )* ] );
        v
    }};

    // Leaf item: expr (last, no trailing comma)
    ( $widget:expr ) => {
        vec![ $crate::ui!( $widget ) ]
    };
}
