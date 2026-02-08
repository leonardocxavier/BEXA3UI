use bexa_ui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let metrics = Metrics::new(14.0, 20.0);
    let theme = Theme::ocean();

    // Sample data - list of server names
    let servers = vec![
        ("Server 1", "192.168.1.10"),
        ("Server 2", "192.168.1.20"),
        ("Server 3", "192.168.1.30"),
    ];

    // Build UI with for loop
    let root = ui! {
        Container::new()
            .with_background(theme.background)
            .with_padding(32.0) => {

            Flex::column(16.0, 0.0) => {
                // Title
                Label::new(
                    Rc::new(RefCell::new("For Loop Rendering Demo".to_string())),
                    Metrics::new(18.0, 24.0),
                    theme.text_primary,
                ).with_padding(8.0),

                // Server list using for loop
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::column(12.0, 0.0) => {
                        Label::new(
                            Rc::new(RefCell::new("Available Servers:".to_string())),
                            Metrics::new(16.0, 22.0),
                            theme.text_primary,
                        ).with_padding(4.0),

                        // FOR LOOP - Render each server as a card
                        for (name, ip) in (&servers) {
                            Container::new()
                                .with_background([0.2, 0.25, 0.3])
                                .with_padding(12.0)
                                .with_border_radius(6.0) => {
                                Flex::column(6.0, 0.0) => {
                                    Label::new(
                                        Rc::new(RefCell::new(format!("🖥 {}", name))),
                                        metrics,
                                        [220, 240, 255],
                                    ).with_padding(4.0),

                                    Label::new(
                                        Rc::new(RefCell::new(format!("IP: {}", ip))),
                                        Metrics::new(12.0, 17.0),
                                        [180, 200, 220],
                                    ).with_padding(4.0),
                                }
                            }
                        },
                    }
                },

                // Footer
                Label::new(
                    Rc::new(RefCell::new(format!("Total servers: {}", servers.len()))),
                    metrics,
                    theme.text_secondary,
                ).with_padding(8.0),
            }
        }
    };

    App::new(root)
        .theme(theme)
        .title("BEXAUI - For Loop Example")
        .run();
}
