use bexa_ui::prelude::*;

// --- Helpers ---

fn section_title(text: &str, metrics: Metrics, color: [u8; 3]) -> WidgetNode {
    let (sig, _) = create_signal(text.to_string());
    WidgetNode::new(
        Label::new(sig, metrics, color)
            .with_align(Align::Left)
            .with_padding(4.0),
        vec![],
    )
}

fn info_card(label: &str, value: &str, bg: [f32; 3], metrics: Metrics) -> WidgetNode {
    let text = format!("{}: {}", label, value);
    let (sig, _) = create_signal(text);
    let label = Label::new(sig, metrics, [230, 230, 230])
        .with_align(Align::Left)
        .with_padding(8.0);

    WidgetNode::new(
        Container::new()
            .with_background(bg)
            .with_padding(10.0)
            .with_border_radius(8.0),
        vec![WidgetNode::new(label, vec![])],
    )
}

fn panel(theme: &Theme) -> Container {
    Container::new()
        .with_background(theme.panel)
        .with_padding(16.0)
        .with_border_radius(12.0)
}

// --- Sections ---

fn build_stats_section(theme: &Theme, metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let cards = WidgetNode::new(
        Flex::row(12.0),
        vec![
            info_card("CPU", "42%", [0.18, 0.50, 0.35], metrics),
            info_card("RAM", "7.8 GB", [0.22, 0.35, 0.55], metrics),
            info_card("Disk", "128 GB", [0.45, 0.30, 0.55], metrics),
            info_card("Net", "1.2 MB/s", [0.50, 0.38, 0.22], metrics),
        ],
    );

    WidgetNode::new(
        panel(theme),
        vec![
            section_title("System Stats", title_metrics, theme.text_primary),
            cards,
        ],
    )
}

fn build_buttons_section(
    theme: &Theme,
    button_metrics: Metrics,
    title_metrics: Metrics,
    set_status: &SetSignal<String>,
) -> WidgetNode {
    let mut btn_primary = Button::new("Sync", button_metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
        .with_padding(14.0)
        .with_border_radius(8.0)
        .with_text_colors([25, 25, 25], [15, 15, 15], [250, 250, 250]);

    let mut btn_warning = Button::new("Backup", button_metrics)
        .with_colors([0.90, 0.55, 0.20], [0.95, 0.70, 0.30], [0.85, 0.45, 0.10], [0.92, 0.62, 0.25])
        .with_padding(14.0)
        .with_border_radius(8.0)
        .with_text_colors([25, 25, 25], [15, 15, 15], [250, 250, 250]);

    let mut btn_danger = Button::new("Reset", button_metrics)
        .with_colors([0.85, 0.25, 0.25], [0.92, 0.35, 0.35], [0.75, 0.18, 0.18], [0.88, 0.30, 0.30])
        .with_padding(14.0)
        .with_border_radius(8.0)
        .with_text_colors([250, 250, 250], [255, 255, 255], [250, 250, 250]);

    let mut btn_success = Button::new("Deploy", button_metrics)
        .with_colors([0.20, 0.65, 0.40], [0.30, 0.75, 0.50], [0.15, 0.55, 0.30], [0.25, 0.70, 0.45])
        .with_padding(14.0)
        .with_border_radius(8.0)
        .with_text_colors([250, 250, 250], [255, 255, 255], [250, 250, 250]);

    let s = set_status.clone();
    btn_primary.set_on_click(move || s.set("Sync started...".to_string()));

    let s = set_status.clone();
    btn_warning.set_on_click(move || s.set("Backup running...".to_string()));

    let s = set_status.clone();
    btn_danger.set_on_click(move || s.set("System reset!".to_string()));

    let s = set_status.clone();
    btn_success.set_on_click(move || s.set("Deploying to production...".to_string()));

    let row = WidgetNode::new(
        Flex::row(12.0),
        vec![
            WidgetNode::new(btn_primary, vec![]),
            WidgetNode::new(btn_warning, vec![]),
            WidgetNode::new(btn_danger, vec![]),
            WidgetNode::new(btn_success, vec![]),
        ],
    );

    WidgetNode::new(
        panel(theme),
        vec![
            section_title("Buttons", title_metrics, theme.text_primary),
            row,
        ],
    )
}

fn build_borders_section(theme: &Theme, metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let card_no_border = WidgetNode::new(
        Container::new()
            .with_background([0.20, 0.32, 0.44])
            .with_padding(12.0)
            .with_border_radius(8.0),
        vec![section_title("No border", metrics, [200, 200, 200])],
    );

    let card_thin = WidgetNode::new(
        Container::new()
            .with_background([0.20, 0.32, 0.44])
            .with_padding(12.0)
            .with_border_radius(8.0)
            .with_border(1.0, [0.3, 0.6, 0.9, 1.0]),
        vec![section_title("Thin border", metrics, [200, 200, 200])],
    );

    let card_medium = WidgetNode::new(
        Container::new()
            .with_background([0.20, 0.32, 0.44])
            .with_padding(12.0)
            .with_border_radius(8.0)
            .with_border(2.0, [0.9, 0.6, 0.2, 1.0]),
        vec![section_title("Medium border", metrics, [200, 200, 200])],
    );

    let card_thick = WidgetNode::new(
        Container::new()
            .with_background([0.20, 0.32, 0.44])
            .with_padding(12.0)
            .with_border_radius(12.0)
            .with_border(3.0, [0.4, 0.85, 0.5, 1.0]),
        vec![section_title("Thick + radius", metrics, [200, 200, 200])],
    );

    let row = WidgetNode::new(
        Flex::row(12.0),
        vec![card_no_border, card_thin, card_medium, card_thick],
    );

    WidgetNode::new(
        panel(theme),
        vec![
            section_title("Borders & Radius", title_metrics, theme.text_primary),
            row,
        ],
    )
}

fn build_layout_section(theme: &Theme, metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let col_a = WidgetNode::new(
        Flex::column(8.0, 0.0),
        vec![
            info_card("A1", "Column", [0.25, 0.38, 0.52], metrics),
            info_card("A2", "Layout", [0.28, 0.42, 0.56], metrics),
        ],
    );

    let col_b = WidgetNode::new(
        Flex::column(8.0, 0.0),
        vec![
            info_card("B1", "Nested", [0.35, 0.28, 0.50], metrics),
            info_card("B2", "Flex", [0.38, 0.32, 0.54], metrics),
        ],
    );

    let grid = WidgetNode::new(Flex::row(12.0), vec![col_a, col_b]);

    WidgetNode::new(
        panel(theme),
        vec![
            section_title("Flex Layout", title_metrics, theme.text_primary),
            grid,
        ],
    )
}

fn build_list_section(theme: &Theme, metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let items: Vec<WidgetNode> = (1..=15)
        .map(|i| {
            let bg = if i % 2 == 0 {
                [0.20, 0.32, 0.44]
            } else {
                [0.18, 0.28, 0.40]
            };
            info_card(&format!("Job {}", i), &format!("Task description #{}", i), bg, metrics)
        })
        .collect();

    let scroll_list = WidgetNode::new(
        Container::new()
            .with_max_height(250.0)
            .with_scroll(),
        vec![WidgetNode::new(Flex::column(6.0, 0.0), items)],
    );

    WidgetNode::new(
        panel(theme)
            .with_border(2.0, [0.3, 0.5, 0.7, 1.0]),
        vec![
            section_title("List View (scroll me!)", title_metrics, theme.text_primary),
            scroll_list,
        ],
    )
}

fn build_colors_section(theme: &Theme, metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let colors = WidgetNode::new(
        Flex::row(8.0),
        vec![
            info_card("", "Ocean", [0.18, 0.30, 0.40], metrics),
            info_card("", "Forest", [0.15, 0.40, 0.25], metrics),
            info_card("", "Sunset", [0.55, 0.30, 0.18], metrics),
            info_card("", "Violet", [0.35, 0.22, 0.50], metrics),
            info_card("", "Steel", [0.30, 0.32, 0.35], metrics),
        ],
    );

    let alpha_cards = WidgetNode::new(
        Flex::row(8.0),
        vec![
            WidgetNode::new(
                Container::new()
                    .with_background_alpha([0.20, 0.65, 0.85, 1.0])
                    .with_padding(10.0)
                    .with_border_radius(8.0),
                vec![section_title("100%", metrics, [230, 230, 230])],
            ),
            WidgetNode::new(
                Container::new()
                    .with_background_alpha([0.20, 0.65, 0.85, 0.7])
                    .with_padding(10.0)
                    .with_border_radius(8.0),
                vec![section_title("70%", metrics, [230, 230, 230])],
            ),
            WidgetNode::new(
                Container::new()
                    .with_background_alpha([0.20, 0.65, 0.85, 0.4])
                    .with_padding(10.0)
                    .with_border_radius(8.0),
                vec![section_title("40%", metrics, [230, 230, 230])],
            ),
            WidgetNode::new(
                Container::new()
                    .with_background_alpha([0.20, 0.65, 0.85, 0.15])
                    .with_padding(10.0)
                    .with_border_radius(8.0),
                vec![section_title("15%", metrics, [230, 230, 230])],
            ),
        ],
    );

    WidgetNode::new(
        panel(theme),
        vec![
            section_title("Colors & Alpha", title_metrics, theme.text_primary),
            colors,
            alpha_cards,
        ],
    )
}

fn build_signals_section(
    theme: &Theme,
    metrics: Metrics,
    title_metrics: Metrics,
    status: Signal<String>,
) -> WidgetNode {
    let status_label = Label::new(status, metrics, [100, 220, 160])
        .with_align(Align::Left)
        .with_padding(8.0);

    WidgetNode::new(
        panel(theme).with_border(1.0, [0.3, 0.7, 0.5, 0.8]),
        vec![
            section_title("Reactive Signals", title_metrics, theme.text_primary),
            section_title("Click any button above to update this text:", metrics, theme.text_secondary),
            WidgetNode::new(status_label, vec![]),
        ],
    )
}

fn build_input_section(
    theme: &Theme,
    metrics: Metrics,
    title_metrics: Metrics,
) -> WidgetNode {
    let (name_val, name_set) = create_signal(String::new());
    let (email_val, email_set) = create_signal(String::new());

    let name_input = TextInput::new(name_set)
        .with_placeholder("Your name...")
        .with_metrics(metrics)
        .with_padding(10.0)
        .with_border_radius(6.0);

    let email_input = TextInput::new(email_set)
        .with_placeholder("email@example.com")
        .with_metrics(metrics)
        .with_padding(10.0)
        .with_border_radius(6.0);

    let name_preview = Label::new(name_val, metrics, [100, 220, 160])
        .with_align(Align::Left)
        .with_padding(4.0);

    let email_preview = Label::new(email_val, metrics, [100, 180, 220])
        .with_align(Align::Left)
        .with_padding(4.0);

    WidgetNode::new(
        panel(theme).with_border(1.0, [0.4, 0.55, 0.8, 0.8]),
        vec![
            section_title("Text Input", title_metrics, theme.text_primary),
            section_title("Name:", metrics, theme.text_secondary),
            WidgetNode::new(name_input, vec![]),
            section_title("Email:", metrics, theme.text_secondary),
            WidgetNode::new(email_input, vec![]),
            section_title("Live preview:", metrics, theme.text_secondary),
            WidgetNode::new(name_preview, vec![]),
            WidgetNode::new(email_preview, vec![]),
        ],
    )
}

fn icon_with_label(icon: &'static str, name: &str, metrics: Metrics) -> WidgetNode {
    let (sig, _) = create_signal(name.to_string());
    WidgetNode::new(
        Container::new()
            .with_background([0.18, 0.28, 0.40])
            .with_padding(8.0)
            .with_border_radius(6.0),
        vec![
            WidgetNode::new(
                Flex::row(6.0),
                vec![
                    WidgetNode::new(Icon::new(icon, 18.0, [120, 200, 255]), vec![]),
                    WidgetNode::new(
                        Label::new(sig, metrics, [200, 200, 200])
                            .with_align(Align::Left)
                            .with_padding(2.0),
                        vec![],
                    ),
                ],
            ),
        ],
    )
}

fn build_icons_section(theme: &Theme, metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let row1 = WidgetNode::new(
        Flex::row(8.0),
        vec![
            icon_with_label(icons::FOLDER, "Folder", metrics),
            icon_with_label(icons::FILE_CODE, "Code", metrics),
            icon_with_label(icons::SEARCH, "Search", metrics),
            icon_with_label(icons::COG, "Settings", metrics),
        ],
    );

    let row2 = WidgetNode::new(
        Flex::row(8.0),
        vec![
            icon_with_label(icons::CHECK_CIRCLE, "Done", metrics),
            icon_with_label(icons::WARNING, "Warning", metrics),
            icon_with_label(icons::ROCKET, "Deploy", metrics),
            icon_with_label(icons::GIT_BRANCH, "Branch", metrics),
        ],
    );

    let row3 = WidgetNode::new(
        Flex::row(8.0),
        vec![
            icon_with_label(icons::DATABASE, "Database", metrics),
            icon_with_label(icons::CLOUD, "Cloud", metrics),
            icon_with_label(icons::TERMINAL, "Terminal", metrics),
            icon_with_label(icons::LOCK, "Security", metrics),
        ],
    );

    let row4 = WidgetNode::new(
        Flex::row(8.0),
        vec![
            icon_with_label(icons::HEART, "Favorite", metrics),
            icon_with_label(icons::STAR, "Star", metrics),
            icon_with_label(icons::BELL, "Alerts", metrics),
            icon_with_label(icons::GLOBE, "Web", metrics),
        ],
    );

    WidgetNode::new(
        panel(theme).with_border(1.0, [0.3, 0.5, 0.8, 0.8]),
        vec![
            section_title("Icons (Nerd Font)", title_metrics, theme.text_primary),
            row1,
            row2,
            row3,
            row4,
        ],
    )
}

// --- Main ---

fn main() {
    let theme = Theme::ocean();
    let (status, set_status) = create_signal(String::from("System ready - click a button!"));

    let metrics = Metrics::new(16.0, 22.0);
    let title_metrics = Metrics::new(20.0, 28.0);
    let button_metrics = Metrics::new(18.0, 24.0);

    let left_column = WidgetNode::new(
        Flex::column(16.0, 0.0),
        vec![
            build_stats_section(&theme, metrics, title_metrics),
            build_buttons_section(&theme, button_metrics, title_metrics, &set_status),
            build_icons_section(&theme, metrics, title_metrics),
            build_signals_section(&theme, metrics, title_metrics, status),
        ],
    );

    let right_column = WidgetNode::new(
        Flex::column(16.0, 0.0),
        vec![
            build_input_section(&theme, metrics, title_metrics),
            build_borders_section(&theme, metrics, title_metrics),
            build_layout_section(&theme, metrics, title_metrics),
            build_colors_section(&theme, metrics, title_metrics),
            build_list_section(&theme, metrics, title_metrics),
        ],
    );

    let main_layout = WidgetNode::new(
        Flex::row(20.0),
        vec![left_column, right_column],
    );

    let root = WidgetNode::new(
        Container::new().with_padding(24.0).with_gap(16.0),
        vec![main_layout],
    );

    App::new(root)
        .theme(theme)
        .title("BexaUI - Feature Showcase")
        .run();
}
