use bexa_ui::prelude::*;

fn section_title(text: &str, metrics: Metrics, color: [u8; 3]) -> WidgetNode {
    let label = Label::new(
        std::rc::Rc::new(std::cell::RefCell::new(text.to_string())),
        metrics,
        color,
    )
    .with_align(Align::Left)
    .with_padding(4.0);
    ui!(label)
}

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(16.0, 22.0);
    let title_metrics = Metrics::new(20.0, 28.0);

    // Checkbox signals
    let (dark_mode, set_dark_mode) = create_signal(false);
    let (notifications, set_notifications) = create_signal(true);
    let (auto_save, set_auto_save) = create_signal(true);

    let cb1 = Checkbox::new("Dark mode", dark_mode, set_dark_mode, metrics)
        .with_colors(
            theme.checkbox_bg,
            theme.checkbox_checked_bg,
            theme.checkbox_border,
            theme.checkbox_check,
        );
    let cb2 = Checkbox::new("Enable notifications", notifications, set_notifications, metrics)
        .with_colors(
            theme.checkbox_bg,
            theme.checkbox_checked_bg,
            theme.checkbox_border,
            theme.checkbox_check,
        );
    let cb3 = Checkbox::new("Auto-save on exit", auto_save, set_auto_save, metrics)
        .with_colors(
            theme.checkbox_bg,
            theme.checkbox_checked_bg,
            theme.checkbox_border,
            theme.checkbox_check,
        );

    // Radio group signal
    let (protocol, set_protocol) = create_signal(0usize);
    let radios = radio_group(
        &["SSH", "FTP", "SFTP", "HTTP"],
        protocol,
        set_protocol,
        metrics,
    );

    // Radio group 2
    let (priority, set_priority) = create_signal(1usize);
    let priority_radios = radio_group(
        &["Low", "Medium", "High"],
        priority,
        set_priority,
        metrics,
    );

    // Select widget
    let (folder, set_folder) = create_signal(0usize);
    let folder_select = Select::new(
        vec![
            "Documents".into(),
            "Downloads".into(),
            "Pictures".into(),
            "Desktop".into(),
            "Music".into(),
        ],
        folder,
        set_folder,
        metrics,
    );

    let (env, set_env) = create_signal(0usize);
    let env_select = Select::new(
        vec![
            "Production".into(),
            "Staging".into(),
            "Development".into(),
            "Testing".into(),
        ],
        env,
        set_env,
        metrics,
    );

    let root = ui! {
        Container::new().with_padding(32.0).with_gap(20.0) => {
            Container::new()
                .with_background(theme.panel)
                .with_padding(16.0)
                .with_border_radius(8.0) => {
                Flex::column(10.0, 0.0) => {
                    section_title("Preferences", title_metrics, theme.text_primary),
                    cb1,
                    cb2,
                    cb3,
                },
            },
            Flex::row(20.0) => {
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::column(10.0, 0.0) => {
                        section_title("Folder", title_metrics, theme.text_primary),
                        folder_select,
                    },
                },
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::column(10.0, 0.0) => {
                        section_title("Environment", title_metrics, theme.text_primary),
                        env_select,
                    },
                },
            },
            Flex::row(20.0) => {
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::column(8.0, 0.0) => {
                        section_title("Protocol", title_metrics, theme.text_primary),
                        WidgetNode::new(Flex::column(6.0, 0.0), radios),
                    },
                },
                Container::new()
                    .with_background(theme.panel)
                    .with_padding(16.0)
                    .with_border_radius(8.0) => {
                    Flex::column(8.0, 0.0) => {
                        section_title("Priority", title_metrics, theme.text_primary),
                        WidgetNode::new(Flex::column(6.0, 0.0), priority_radios),
                    },
                },
            },
        }
    };

    App::new(root)
        .theme(theme)
        .title("BexaUI - Checkbox & Radio")
        .run();
}
