use bexa_ui::prelude::*;

// ─── Helpers ─────────────────────────────────────────────────────────

fn label_node(text: &str, metrics: Metrics, color: [u8; 3]) -> WidgetNode {
    let (sig, _) = create_signal(text.to_string());
    WidgetNode::new(
        Label::new(sig, metrics, color).with_align(Align::Left).with_padding(4.0),
        vec![],
    )
}

fn panel(bg: [f32; 3]) -> Container {
    Container::new()
        .with_background(bg)
        .with_padding(14.0)
        .with_border_radius(10.0)
        .with_border(1.0, [0.20, 0.25, 0.35, 1.0])
}

fn stat_card(icon: &'static str, label: &str, value: &str, accent: [f32; 3], tooltip_text: &str) -> WidgetNode {
    let (val_sig, _) = create_signal(value.to_string());
    let (lbl_sig, _) = create_signal(label.to_string());

    let card = WidgetNode::new(
        Container::new()
            .with_background(accent)
            .with_padding(12.0)
            .with_border_radius(8.0),
        vec![
            WidgetNode::new(
                Flex::row(8.0),
                vec![
                    WidgetNode::new(Icon::new(icon, 22.0, [255, 255, 255]), vec![]),
                    WidgetNode::new(
                        Flex::column(2.0, 0.0),
                        vec![
                            WidgetNode::new(
                                Label::new(lbl_sig, Metrics::new(11.0, 14.0), [200, 210, 220])
                                    .with_align(Align::Left),
                                vec![],
                            ),
                            WidgetNode::new(
                                Label::new(val_sig, Metrics::new(20.0, 26.0), [255, 255, 255])
                                    .with_align(Align::Left),
                                vec![],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );

    // Wrap with Tooltip
    WidgetNode::new(
        Tooltip::new(tooltip_text).with_position(TooltipPosition::Bottom),
        vec![card],
    )
}

// ─── Tabs ────────────────────────────────────────────────────────────

fn build_tabs(metrics: Metrics) -> (WidgetNode, Signal<usize>) {
    let (active, set_active) = create_signal(0_usize);
    let tabs = Tabs::new(
        vec![
            "Overview".into(),
            "Services".into(),
            "Logs".into(),
            "Settings".into(),
        ],
        active.clone(),
        set_active,
        metrics,
    );
    (WidgetNode::new(tabs, vec![]), active)
}

// ─── Stat Cards Row ──────────────────────────────────────────────────

fn build_stats_row() -> WidgetNode {
    WidgetNode::new(
        Flex::row(10.0),
        vec![
            stat_card(icons::DASHBOARD, "CPU", "42%", [0.16, 0.42, 0.32], "Average CPU usage across all instances"),
            stat_card(icons::DATABASE, "RAM", "7.8 GB", [0.20, 0.32, 0.52], "Total memory used / 16 GB available"),
            stat_card(icons::CLOUD, "Network", "1.2 MB/s", [0.42, 0.28, 0.52], "Inbound + outbound bandwidth"),
            stat_card(icons::CLOCK, "Uptime", "14d 3h", [0.48, 0.35, 0.20], "Since last restart on Jan 23"),
            stat_card(icons::ROCKET, "Deploy", "#847", [0.22, 0.52, 0.42], "Latest deploy: 2h ago by CI/CD"),
        ],
    )
}

// ─── BarChart (requests/day) ─────────────────────────────────────────

fn build_requests_chart(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let bars = vec![
        Bar::new("Mon", 1240.0, [0.25, 0.60, 0.85, 1.0]),
        Bar::new("Tue", 1580.0, [0.28, 0.63, 0.87, 1.0]),
        Bar::new("Wed", 2120.0, [0.32, 0.68, 0.90, 1.0]),
        Bar::new("Thu", 1870.0, [0.28, 0.63, 0.87, 1.0]),
        Bar::new("Fri", 2450.0, [0.35, 0.72, 0.92, 1.0]),
        Bar::new("Sat", 980.0, [0.22, 0.55, 0.78, 1.0]),
        Bar::new("Sun", 720.0, [0.20, 0.50, 0.72, 1.0]),
    ];
    let (sig, _) = create_signal(bars);
    let chart = BarChart::new(sig, metrics, 220.0)
        .with_bar_radius(5.0)
        .with_bar_gap(8.0);

    WidgetNode::new(
        panel([0.08, 0.10, 0.14]),
        vec![
            label_node("Requests / Day", title_metrics, [170, 185, 210]),
            WidgetNode::new(chart, vec![]),
        ],
    )
}

// ─── Table (services) ────────────────────────────────────────────────

fn build_services_table(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let columns = vec![
        Column::new("Service", 2.5),
        Column::new("Status", 1.2),
        Column::new("CPU", 0.8),
        Column::new("Memory", 1.2),
        Column::new("Region", 1.2),
    ];

    let data = vec![
        vec!["api-gateway".into(), "Running".into(), "12.4%".into(), "256 MB".into(), "us-east-1".into()],
        vec!["auth-service".into(), "Running".into(), "8.2%".into(), "128 MB".into(), "us-east-1".into()],
        vec!["db-primary".into(), "Running".into(), "45.1%".into(), "2.1 GB".into(), "us-east-1".into()],
        vec!["db-replica".into(), "Running".into(), "38.7%".into(), "1.8 GB".into(), "eu-west-1".into()],
        vec!["cache-redis".into(), "Running".into(), "3.1%".into(), "512 MB".into(), "us-east-1".into()],
        vec!["worker-queue".into(), "Warning".into(), "67.8%".into(), "384 MB".into(), "us-east-1".into()],
        vec!["scheduler".into(), "Running".into(), "2.0%".into(), "64 MB".into(), "eu-west-1".into()],
        vec!["log-collector".into(), "Running".into(), "5.5%".into(), "96 MB".into(), "us-east-1".into()],
        vec!["metrics-agg".into(), "Stopped".into(), "0.0%".into(), "0 MB".into(), "us-east-1".into()],
        vec!["cdn-edge".into(), "Running".into(), "15.3%".into(), "192 MB".into(), "ap-south-1".into()],
        vec!["payment-svc".into(), "Running".into(), "6.7%".into(), "320 MB".into(), "us-east-1".into()],
        vec!["notification".into(), "Running".into(), "1.8%".into(), "48 MB".into(), "eu-west-1".into()],
    ];

    let (rows, _) = create_signal(data);
    let (selected, set_selected) = create_signal(None);
    let table = Table::new(columns, rows, selected, set_selected, metrics)
        .with_max_visible(8);

    WidgetNode::new(
        panel([0.08, 0.10, 0.14]),
        vec![
            label_node("Services", title_metrics, [170, 185, 210]),
            WidgetNode::new(table, vec![]),
        ],
    )
}

// ─── TreeView (infrastructure) ───────────────────────────────────────

fn build_infra_tree(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let tree = TreeView::new(
        vec![
            TreeNode::branch("us-east-1", vec![
                TreeNode::branch("Production", vec![
                    TreeNode::leaf("api-gateway (3 instances)").with_icon(icons::CLOUD),
                    TreeNode::leaf("auth-service (2 instances)").with_icon(icons::LOCK),
                    TreeNode::leaf("db-primary (master)").with_icon(icons::DATABASE),
                    TreeNode::leaf("cache-redis (cluster)").with_icon(icons::DATABASE),
                ]).with_icon(icons::ROCKET),
                TreeNode::branch("Staging", vec![
                    TreeNode::leaf("api-gateway (1 instance)").with_icon(icons::CLOUD),
                    TreeNode::leaf("db-staging").with_icon(icons::DATABASE),
                ]).with_icon(icons::COG).with_expanded(false),
            ]).with_icon(icons::GLOBE),
            TreeNode::branch("eu-west-1", vec![
                TreeNode::branch("Production", vec![
                    TreeNode::leaf("db-replica (read)").with_icon(icons::DATABASE),
                    TreeNode::leaf("scheduler").with_icon(icons::CLOCK),
                    TreeNode::leaf("notification-svc").with_icon(icons::BELL),
                ]).with_icon(icons::ROCKET),
            ]).with_icon(icons::GLOBE).with_expanded(false),
            TreeNode::branch("ap-south-1", vec![
                TreeNode::leaf("cdn-edge (12 nodes)").with_icon(icons::CLOUD),
            ]).with_icon(icons::GLOBE).with_expanded(false),
        ],
        metrics,
    );

    WidgetNode::new(
        panel([0.08, 0.10, 0.14]),
        vec![
            label_node("Infrastructure", title_metrics, [170, 185, 210]),
            WidgetNode::new(
                Container::new().with_max_height(250.0).with_scroll(),
                vec![WidgetNode::new(tree, vec![])],
            ),
        ],
    )
}

// ─── Health BarChart ─────────────────────────────────────────────────

fn build_health_chart(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let bars = vec![
        Bar::new("API", 98.0, [0.22, 0.72, 0.42, 1.0]),
        Bar::new("Auth", 95.0, [0.25, 0.75, 0.45, 1.0]),
        Bar::new("DB", 88.0, [0.90, 0.65, 0.20, 1.0]),
        Bar::new("Cache", 99.0, [0.22, 0.72, 0.42, 1.0]),
        Bar::new("Queue", 52.0, [0.88, 0.30, 0.28, 1.0]),
        Bar::new("CDN", 97.0, [0.22, 0.72, 0.42, 1.0]),
    ];
    let (sig, _) = create_signal(bars);
    let chart = BarChart::new(sig, metrics, 180.0)
        .with_max_value(100.0)
        .with_bar_radius(4.0)
        .with_bar_gap(10.0);

    WidgetNode::new(
        panel([0.08, 0.10, 0.14]),
        vec![
            label_node("Service Health (%)", title_metrics, [170, 185, 210]),
            WidgetNode::new(chart, vec![]),
        ],
    )
}

// ─── Controls Panel (TextInput, Checkbox, Radio, Select, Buttons) ────

fn build_controls_panel(
    metrics: Metrics,
    title_metrics: Metrics,
    set_status: &SetSignal<String>,
    set_modal: &SetSignal<bool>,
) -> WidgetNode {
    // TextInput - search
    let (_search_val, search_set) = create_signal(String::new());
    let search_input = TextInput::new(search_set)
        .with_placeholder("Search services...")
        .with_metrics(metrics)
        .with_padding(8.0)
        .with_border_radius(6.0);

    // Checkbox
    let (auto_refresh, set_auto_refresh) = create_signal(true);
    let cb = Checkbox::new("Auto-refresh (30s)", auto_refresh, set_auto_refresh, metrics);

    // Select - Region
    let (_region, set_region) = create_signal(0_usize);
    let region_select = Select::new(
        vec![
            "All Regions".into(),
            "us-east-1".into(),
            "eu-west-1".into(),
            "ap-south-1".into(),
        ],
        _region,
        set_region,
        metrics,
    );

    // Radio Group - View mode
    let (_view_mode, set_view_mode) = create_signal(0_usize);
    let radios = radio_group(
        &["Compact", "Detailed", "Cards"],
        _view_mode,
        set_view_mode,
        metrics,
    );

    // Buttons
    let mut btn_deploy = Button::new("Deploy", metrics)
        .with_colors([0.20, 0.65, 0.40], [0.28, 0.72, 0.48], [0.15, 0.55, 0.32], [0.24, 0.68, 0.44])
        .with_padding(12.0)
        .with_border_radius(6.0)
        .with_text_colors([255, 255, 255], [255, 255, 255], [255, 255, 255]);

    let mut btn_stop = Button::new("Stop", metrics)
        .with_colors([0.82, 0.25, 0.25], [0.90, 0.32, 0.32], [0.72, 0.18, 0.18], [0.86, 0.28, 0.28])
        .with_padding(12.0)
        .with_border_radius(6.0)
        .with_text_colors([255, 255, 255], [255, 255, 255], [255, 255, 255]);

    let mut btn_restart = Button::new("Restart", metrics)
        .with_colors([0.88, 0.55, 0.18], [0.92, 0.62, 0.25], [0.80, 0.48, 0.12], [0.90, 0.58, 0.20])
        .with_padding(12.0)
        .with_border_radius(6.0)
        .with_text_colors([255, 255, 255], [255, 255, 255], [255, 255, 255]);

    let m = set_modal.clone();
    btn_deploy.set_on_click(move || m.set(true));
    let s = set_status.clone();
    btn_stop.set_on_click(move || s.set("Stopping selected services...".into()));
    let s = set_status.clone();
    btn_restart.set_on_click(move || s.set("Restarting services...".into()));

    WidgetNode::new(
        panel([0.08, 0.10, 0.14]).with_gap(6.0),
        vec![
            label_node("Controls", title_metrics, [170, 185, 210]),
            // Search
            WidgetNode::new(search_input, vec![]),
            // Checkbox
            WidgetNode::new(cb, vec![]),
            // Region select
            label_node("Region:", Metrics::new(12.0, 16.0), [140, 150, 170]),
            WidgetNode::new(region_select, vec![]),
            // View mode
            label_node("View Mode:", Metrics::new(12.0, 16.0), [140, 150, 170]),
            WidgetNode::new(Flex::column(4.0, 0.0), radios),
            // Buttons row
            WidgetNode::new(
                Flex::row(8.0),
                vec![
                    WidgetNode::new(btn_deploy, vec![]),
                    WidgetNode::new(btn_stop, vec![]),
                    WidgetNode::new(btn_restart, vec![]),
                ],
            ),
        ],
    )
}

// ─── Status Bar ──────────────────────────────────────────────────────

fn build_status_bar(status: Signal<String>, metrics: Metrics) -> WidgetNode {
    WidgetNode::new(
        Container::new()
            .with_background([0.06, 0.08, 0.12])
            .with_padding(8.0)
            .with_border_radius(6.0)
            .with_border(1.0, [0.18, 0.22, 0.32, 1.0]),
        vec![
            WidgetNode::new(
                Flex::row(8.0),
                vec![
                    WidgetNode::new(Icon::new(icons::CHECK_CIRCLE, 14.0, [80, 200, 120]), vec![]),
                    WidgetNode::new(
                        Label::new(status, metrics, [80, 200, 120]).with_align(Align::Left),
                        vec![],
                    ),
                ],
            ),
        ],
    )
}

// ─── Main ────────────────────────────────────────────────────────────

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(13.0, 18.0);
    let title_metrics = Metrics::new(16.0, 22.0);
    let header_metrics = Metrics::new(22.0, 30.0);

    let (status, set_status) = create_signal("All systems operational".to_string());
    let (modal_open, set_modal_open) = create_signal(false);

    let (tabs_node, _active_tab) = build_tabs(metrics);

    // Left column: chart + table
    let left_col = WidgetNode::new(
        Flex::column(12.0, 0.0),
        vec![
            build_requests_chart(metrics, title_metrics),
            build_services_table(metrics, title_metrics),
        ],
    );

    // Right column: tree + health chart + controls
    let right_col = WidgetNode::new(
        Flex::column(12.0, 0.0),
        vec![
            build_infra_tree(metrics, title_metrics),
            build_health_chart(metrics, title_metrics),
            build_controls_panel(metrics, title_metrics, &set_status, &set_modal_open),
        ],
    );

    // Deploy confirmation modal
    let deploy_modal = Modal::new(modal_open, set_modal_open)
        .with_title("Confirm Deploy")
        .with_body(vec![
            "You are about to deploy to production.".into(),
            "".into(),
            "Target: us-east-1 (3 instances)".into(),
            "Branch: main (commit #a3f7b2c)".into(),
            "Pipeline: CI passed (all 247 tests)".into(),
            "".into(),
            "Click anywhere or press Escape to close.".into(),
        ])
        .with_width(420.0);

    let root = WidgetNode::new(
        Container::new()
            .with_background([0.05, 0.06, 0.09])
            .with_padding(16.0)
            .with_gap(12.0)
            .with_scroll(),
        vec![
            // Modal (takes no space, renders in overlay)
            WidgetNode::new(deploy_modal, vec![]),
            // Header
            label_node("Infrastructure Dashboard", header_metrics, [220, 225, 240]),
            // Tabs
            tabs_node,
            // Stat cards row
            build_stats_row(),
            // Main 2-column layout
            WidgetNode::new(Flex::row(12.0), vec![left_col, right_col]),
            // Status bar
            build_status_bar(status, metrics),
        ],
    );

    App::new(root)
        .theme(theme)
        .title("BexaUI - Infrastructure Dashboard")
        .run();
}
