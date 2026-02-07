use bexa_ui::prelude::*;

// ── Helpers ──

fn stat_card(icon: &'static str, label: &str, value: &str, color: [f32; 3], metrics: Metrics) -> WidgetNode {
    let (val_sig, _) = create_signal(value.to_string());
    let (lbl_sig, _) = create_signal(label.to_string());
    ui! {
        Container::new()
            .with_background(color)
            .with_padding(12.0)
            .with_border_radius(8.0) => {
            Flex::row(8.0) => {
                Icon::new(icon, 20.0, [220, 220, 220]),
                Flex::column(2.0, 0.0) => {
                    Label::new(lbl_sig, Metrics::new(11.0, 14.0), [170, 180, 190])
                        .with_align(Align::Left),
                    Label::new(val_sig, metrics, [240, 240, 240])
                        .with_align(Align::Left),
                },
            },
        }
    }
}

fn log_line(level: &str, msg: &str, metrics: Metrics) -> WidgetNode {
    let color = match level {
        "INFO" => [80, 200, 140],
        "WARN" => [230, 180, 60],
        "ERROR" => [230, 80, 80],
        _ => [170, 170, 170],
    };
    let text = format!("[{}]  {}", level, msg);
    let (sig, _) = create_signal(text);
    ui! {
        Container::new()
            .with_background([0.10, 0.12, 0.16])
            .with_padding(6.0)
            .with_border_radius(3.0) => {
            Label::new(sig, metrics, color)
                .with_align(Align::Left)
                .with_font_family("Consolas"),
        }
    }
}

fn section_header(icon: &'static str, title: &str, metrics: Metrics, color: [u8; 3]) -> WidgetNode {
    let (sig, _) = create_signal(title.to_string());
    ui! {
        Flex::row(8.0) => {
            Icon::new(icon, 16.0, color),
            Label::new(sig, metrics, color)
                .with_align(Align::Left),
        }
    }
}

// ── Main ──

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(14.0, 18.0);
    let header_metrics = Metrics::new(13.0, 17.0);
    let title_metrics = Metrics::new(18.0, 24.0);

    // ── Status bar signal ──
    let (status, set_status) = create_signal("Ready".to_string());

    // ── Action buttons ──
    let mut btn_deploy = Button::new("Deploy", metrics)
        .with_colors([0.20, 0.65, 0.40], [0.30, 0.75, 0.50], [0.15, 0.55, 0.30], [0.25, 0.70, 0.45])
        .with_text_colors([240, 240, 240], [255, 255, 255], [240, 240, 240])
        .with_padding(10.0)
        .with_border_radius(6.0);

    let mut btn_restart = Button::new("Restart", metrics)
        .with_colors([0.90, 0.55, 0.20], [0.95, 0.65, 0.30], [0.80, 0.45, 0.15], [0.92, 0.60, 0.25])
        .with_text_colors([30, 30, 30], [15, 15, 15], [240, 240, 240])
        .with_padding(10.0)
        .with_border_radius(6.0);

    let mut btn_stop = Button::new("Stop", metrics)
        .with_colors([0.80, 0.25, 0.25], [0.88, 0.35, 0.35], [0.70, 0.18, 0.18], [0.84, 0.30, 0.30])
        .with_text_colors([240, 240, 240], [255, 255, 255], [240, 240, 240])
        .with_padding(10.0)
        .with_border_radius(6.0);

    let s = set_status.clone();
    btn_deploy.set_on_click(move || s.set("Deploying to production...".to_string()));
    let s = set_status.clone();
    btn_restart.set_on_click(move || s.set("Restarting services...".to_string()));
    let s = set_status.clone();
    btn_stop.set_on_click(move || s.set("Stopping all containers...".to_string()));

    // ── Stats row (built manually because children are WidgetNodes) ──
    let stats = WidgetNode::new(Flex::row(10.0), vec![
        stat_card(icons::COG, "CPU", "42%", [0.14, 0.38, 0.28], metrics),
        stat_card(icons::DATABASE, "RAM", "7.8 GB", [0.18, 0.28, 0.45], metrics),
        stat_card(icons::CLOUD, "DISK", "78%", [0.35, 0.24, 0.42], metrics),
        stat_card(icons::GLOBE, "NET", "1.2 MB/s", [0.40, 0.30, 0.18], metrics),
    ]);

    // ── Log entries ──
    let log_metrics = Metrics::new(12.0, 16.0);
    let logs: Vec<WidgetNode> = vec![
        log_line("INFO",  "Container web-api started (pid 2847)", log_metrics),
        log_line("INFO",  "Listening on 0.0.0.0:8080", log_metrics),
        log_line("INFO",  "Connected to postgres://db:5432/prod", log_metrics),
        log_line("WARN",  "Connection pool at 80% capacity", log_metrics),
        log_line("INFO",  "Health check: 200 OK (12ms)", log_metrics),
        log_line("ERROR", "Timeout on POST /api/deploy (30s)", log_metrics),
        log_line("WARN",  "Retrying request (attempt 2/3)...", log_metrics),
        log_line("INFO",  "Deploy succeeded: v2.4.1 -> v2.5.0", log_metrics),
        log_line("INFO",  "Rolling restart: node-1 done", log_metrics),
        log_line("INFO",  "Rolling restart: node-2 done", log_metrics),
        log_line("WARN",  "node-3: high memory usage (92%)", log_metrics),
        log_line("INFO",  "Scaling up: 3 -> 4 replicas", log_metrics),
        log_line("INFO",  "Certificate renewal: 45 days remaining", log_metrics),
        log_line("ERROR", "DNS resolution failed for metrics.internal", log_metrics),
        log_line("INFO",  "Fallback: using cached DNS entry", log_metrics),
    ];

    // ── Terminal (inline, same window) ──
    let term = Terminal::new(Metrics::new(13.0, 17.0))
        .with_background([0.06, 0.06, 0.09, 1.0]);

    // ── Status bar ──
    let status_label = Label::new(status, Metrics::new(12.0, 16.0), [120, 200, 160])
        .with_align(Align::Left)
        .with_padding(4.0);

    // ── Title ──
    let (title_sig, _) = create_signal("DevOps Console".to_string());
    let title_label = Label::new(title_sig, title_metrics, theme.text_primary)
        .with_align(Align::Left);

    // ── Terminal panel ──
    let terminal_panel = WidgetNode::new(
        Container::new()
            .with_background([0.10, 0.11, 0.14])
            .with_border_radius(8.0)
            .with_border(1.0, [0.2, 0.3, 0.5, 0.6])
            .with_padding(8.0)
            .with_height(400.0),
        vec![
            WidgetNode::new(Flex::column(6.0, 0.0), vec![
                section_header(icons::TERMINAL, "Terminal", header_metrics, [140, 180, 220]),
                ui!(term),
            ]),
        ],
    );

    // ── Logs panel ──
    let logs_panel = WidgetNode::new(
        Container::new()
            .with_background([0.10, 0.11, 0.14])
            .with_border_radius(8.0)
            .with_border(1.0, [0.2, 0.3, 0.5, 0.6])
            .with_padding(8.0)
            .with_height(400.0),
        vec![
            WidgetNode::new(Flex::column(6.0, 0.0), vec![
                section_header(icons::FILE_CODE, "Logs", header_metrics, [140, 180, 220]),
                WidgetNode::new(
                    Container::new()
                        .with_max_height(360.0)
                        .with_scroll(),
                    vec![WidgetNode::new(Flex::column(3.0, 0.0), logs)],
                ),
            ]),
        ],
    );

    // ── Layout ──
    let root = WidgetNode::new(
        Container::new()
            .with_background([0.08, 0.09, 0.12])
            .with_padding(16.0)
            .with_gap(12.0),
        vec![
            // Top bar: title + actions
            ui! {
                Flex::row(12.0) => {
                    Flex::row(8.0) => {
                        Icon::new(icons::ROCKET, 22.0, [100, 180, 255]),
                        title_label,
                    },
                    Flex::row(8.0) => {
                        btn_deploy,
                        btn_restart,
                        btn_stop,
                    },
                }
            },

            // Stats row
            stats,

            // Main content: terminal + logs side by side
            WidgetNode::new(Flex::row(12.0), vec![terminal_panel, logs_panel]),

            // Status bar
            ui! {
                Container::new()
                    .with_background([0.06, 0.07, 0.10])
                    .with_padding(6.0)
                    .with_border_radius(4.0) => {
                    Flex::row(8.0) => {
                        Icon::new(icons::CHECK_CIRCLE, 14.0, [80, 200, 140]),
                        status_label,
                    },
                }
            },
        ],
    );

    App::new(root)
        .theme(Theme {
            background: [0.08, 0.09, 0.12],
            ..theme
        })
        .title("BexaUI - DevOps Console")
        .run();
}
