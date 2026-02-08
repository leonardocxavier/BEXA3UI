// Copyright (c) 2026 Leonardo C. Xavier
// SPDX-License-Identifier: GPL-3.0-or-later OR Commercial
// See LICENSE and LICENSE-COMMERCIAL for details.

pub mod framework;
pub mod icons;
pub mod reactive;
pub mod renderer;
pub mod signal;
pub mod theme;
pub mod tree;
pub mod widgets;

pub use framework::{DrawContext, EventContext, Widget};
pub use reactive::{create_effect, signal_changed};
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
///         if (show_extra) {
///             extra_widget,
///         },
///         if (condition) {
///             TrueWidget => { child }
///         } else {
///             FalseWidget => { other_child }
///         }
///     }
/// }
/// ```
///
/// - `widget => { children... }` — container with children
/// - `widget` — leaf node (no children), automatically wrapped
/// - `if (cond) { items... }` — conditional rendering (optional)
/// - `if (cond) { items... } else { items... }` — conditional with alternative
/// - Children can be bare widgets or nested `container => { ... }` expressions
/// - `ui!(widget)` still works for explicit leaf wrapping
/// - **Note:** Conditions must be wrapped in parentheses `if (expr) { ... }`
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
/// Each item is either `widget => { children }`, a bare expression, or a conditional (if/else).
#[macro_export]
#[doc(hidden)]
macro_rules! ui_items {
    // Empty
    () => { vec![] };

    // Conditional: if (expr) { items... } else { items... }, rest...
    ( if ( $cond:expr ) { $( $if_items:tt )* } else { $( $else_items:tt )* } , $( $rest:tt )* ) => {{
        let mut v = if $cond {
            $crate::ui_items![ $( $if_items )* ]
        } else {
            $crate::ui_items![ $( $else_items )* ]
        };
        v.extend( $crate::ui_items![ $( $rest )* ] );
        v
    }};

    // Conditional: if (expr) { items... } else { items... } (last, no trailing comma)
    ( if ( $cond:expr ) { $( $if_items:tt )* } else { $( $else_items:tt )* } ) => {
        if $cond {
            $crate::ui_items![ $( $if_items )* ]
        } else {
            $crate::ui_items![ $( $else_items )* ]
        }
    };

    // Conditional: if (expr) { items... }, rest... (no else branch)
    ( if ( $cond:expr ) { $( $if_items:tt )* } , $( $rest:tt )* ) => {{
        let mut v = if $cond {
            $crate::ui_items![ $( $if_items )* ]
        } else {
            vec![]
        };
        v.extend( $crate::ui_items![ $( $rest )* ] );
        v
    }};

    // Conditional: if (expr) { items... } (last, no trailing comma, no else branch)
    ( if ( $cond:expr ) { $( $if_items:tt )* } ) => {
        if $cond {
            $crate::ui_items![ $( $if_items )* ]
        } else {
            vec![]
        }
    };

    // For loop: for item in (collection) { items... }, rest...
    ( for $item:pat in ( $collection:expr ) { $( $loop_items:tt )* } , $( $rest:tt )* ) => {{
        let mut v = Vec::new();
        for $item in $collection {
            v.extend($crate::ui_items![ $( $loop_items )* ]);
        }
        v.extend($crate::ui_items![ $( $rest )* ]);
        v
    }};

    // For loop: for item in (collection) { items... } (last, no trailing comma)
    ( for $item:pat in ( $collection:expr ) { $( $loop_items:tt )* } ) => {{
        let mut v = Vec::new();
        for $item in $collection {
            v.extend($crate::ui_items![ $( $loop_items )* ]);
        }
        v
    }};

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
