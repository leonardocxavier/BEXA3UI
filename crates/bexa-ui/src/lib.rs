// Copyright (c) 2026 Leonardo C. Xavier
// SPDX-License-Identifier: GPL-3.0-or-later OR Commercial
// See LICENSE and LICENSE-COMMERCIAL for details.

pub use bexa_ui_core::*;
pub use bexa_ui_render::App;

pub mod prelude {
    pub use bexa_ui_core::{
        Align, Bar, BarChart, Button, Checkbox, Column, Container, Flex, Icon, Image, ImageFit, Label, Metrics,
        Modal, RadioButton, radio_group, Renderer, ScrollView, Select, Slider, Table, Tabs, TextInput, Toggle, Theme,
        Tooltip, TooltipPosition, TreeNode, TreeView, Widget, WidgetNode,
        Signal, SetSignal, create_signal, icons,
        WindowRequest, WindowRequests, create_window_requests,
        ui,
    };
    #[cfg(feature = "terminal")]
    pub use bexa_ui_core::Terminal;
    pub use bexa_ui_render::App;
}
