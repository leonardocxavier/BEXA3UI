use bexa_ui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let metrics = Metrics::new(14.0, 20.0);

    // State for conditional rendering
    let show_panel_a = Rc::new(RefCell::new(true));
    let show_extra_info = Rc::new(RefCell::new(false));

    // Toggle buttons
    let show_a_clone = show_panel_a.clone();
    let mut toggle_a = Button::new("Toggle Panel A", metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
        .with_padding(12.0)
        .with_border_radius(6.0);

    toggle_a.set_on_click(move || {
        let mut val = show_a_clone.borrow_mut();
        *val = !*val;
    });

    let show_info_clone = show_extra_info.clone();
    let mut toggle_info = Button::new("Toggle Extra Info", metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
        .with_padding(12.0)
        .with_border_radius(6.0);

    toggle_info.set_on_click(move || {
        let mut val = show_info_clone.borrow_mut();
        *val = !*val;
    });

    // Read state for UI building
    let should_show_a = *show_panel_a.borrow();
    let should_show_info = *show_extra_info.borrow();

    // Build UI with conditional rendering
    let root = ui! {
        Container::new()
            .with_background(theme.background)
            .with_padding(32.0) => {

            Flex::column(16.0, 0.0) => {
                // Title
                Label::new(
                    Rc::new(RefCell::new("Conditional Rendering Demo".to_string())),
                    Metrics::new(18.0, 24.0),
                    theme.text_primary,
                ).with_padding(8.0),

                // Toggle buttons
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::row(12.0) => {
                        toggle_a,
                        toggle_info,
                    }
                },

                // Conditional content area
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::column(12.0, 0.0) => {
                        // Conditional: if/else with widgets
                        if (should_show_a) {
                            Container::new()
                                .with_background([0.2, 0.4, 0.3])
                                .with_padding(12.0)
                                .with_border_radius(6.0) => {
                                Label::new(
                                    Rc::new(RefCell::new("✓ Panel A is visible".to_string())),
                                    metrics,
                                    [200, 255, 200],
                                ).with_padding(8.0)
                            }
                        } else {
                            Container::new()
                                .with_background([0.4, 0.2, 0.2])
                                .with_padding(12.0)
                                .with_border_radius(6.0) => {
                                Label::new(
                                    Rc::new(RefCell::new("✗ Panel A is hidden".to_string())),
                                    metrics,
                                    [255, 200, 200],
                                ).with_padding(8.0)
                            }
                        },

                        // Conditional: if without else
                        if (should_show_info) {
                            Container::new()
                                .with_background([0.2, 0.3, 0.4])
                                .with_padding(12.0)
                                .with_border_radius(6.0) => {
                                Flex::column(8.0, 0.0) => {
                                    Label::new(
                                        Rc::new(RefCell::new("ℹ Extra Information Panel".to_string())),
                                        Metrics::new(16.0, 22.0),
                                        [200, 220, 255],
                                    ).with_padding(4.0),

                                    Label::new(
                                        Rc::new(RefCell::new("This panel only shows when enabled.".to_string())),
                                        metrics,
                                        [180, 200, 220],
                                    ).with_padding(4.0),

                                    Label::new(
                                        Rc::new(RefCell::new("Click 'Toggle Extra Info' to hide it.".to_string())),
                                        metrics,
                                        [180, 200, 220],
                                    ).with_padding(4.0),
                                }
                            }
                        },

                        // Always visible
                        Label::new(
                            Rc::new(RefCell::new("This label is always visible".to_string())),
                            metrics,
                            theme.text_secondary,
                        ).with_padding(8.0),
                    }
                },

                // Status
                Label::new(
                    Rc::new(RefCell::new(format!(
                        "Panel A: {} | Extra Info: {}",
                        if (should_show_a) { "ON" } else { "OFF" },
                        if (should_show_info) { "ON" } else { "OFF" }
                    ))),
                    metrics,
                    theme.text_secondary,
                ).with_padding(8.0),
            }
        }
    };

    App::new(root)
        .theme(theme)
        .title("BEXAUI - Conditional Rendering Example")
        .run();
}
