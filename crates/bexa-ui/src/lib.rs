pub use bexa_ui_core::*;
pub use bexa_ui_render::App;

pub mod prelude {
    pub use bexa_ui_core::{
        Align, Bar, BarChart, Button, Checkbox, Column, Container, Flex, Icon, Label, Metrics,
        Modal, RadioButton, radio_group, Renderer, Select, Table, Tabs, TextInput, Theme,
        Tooltip, TooltipPosition, TreeNode, TreeView, Widget, WidgetNode,
        Signal, SetSignal, create_signal, icons,
        WindowRequest, WindowRequests, create_window_requests,
        ui,
    };
    #[cfg(feature = "terminal")]
    pub use bexa_ui_core::Terminal;
    pub use bexa_ui_render::App;
}
