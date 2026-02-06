pub mod framework;
pub mod icons;
pub mod renderer;
pub mod signal;
pub mod theme;
pub mod tree;
pub mod widgets;

pub use framework::{DrawContext, EventContext, Widget};
pub use renderer::{QuadCommand, Renderer, TextCommand};
pub use signal::{Signal, SetSignal, IntoSignal, create_signal};
pub use theme::Theme;
pub use tree::{
    build_taffy, clear_active_widgets, collect_focus_paths, dispatch_event, dispatch_scroll,
    draw_widgets, scroll_root, sync_styles, update_widget_measures, widget_mut_at_path, WidgetNode,
};
pub use widgets::{Button, Container, Flex, Icon, Label, TextInput};

// Re-export text types so downstream crates don't need glyphon directly
pub use glyphon::Metrics;
pub use glyphon::cosmic_text::Align;
