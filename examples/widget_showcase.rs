use bexa_ui::prelude::*;

// --- Helpers ---

fn label_node(text: &str, metrics: Metrics, color: [u8; 3]) -> WidgetNode {
    let (sig, _) = create_signal(text.to_string());
    WidgetNode::new(
        Label::new(sig, metrics, color)
            .with_align(Align::Left)
            .with_padding(4.0),
        vec![],
    )
}

fn panel(bg: [f32; 3]) -> Container {
    Container::new()
        .with_background(bg)
        .with_padding(16.0)
        .with_border_radius(10.0)
        .with_border(1.0, [0.25, 0.30, 0.40, 1.0])
}

// --- Sections ---

fn build_tabs_panel(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let (active_tab, set_active_tab) = create_signal(0_usize);

    let tabs = Tabs::new(
        vec![
            "Overview".into(),
            "Metrics".into(),
            "Logs".into(),
            "Settings".into(),
        ],
        active_tab,
        set_active_tab,
        metrics,
    );

    WidgetNode::new(
        panel([0.10, 0.12, 0.16]),
        vec![
            label_node("Tabs", title_metrics, [180, 190, 220]),
            WidgetNode::new(tabs, vec![]),
            label_node("Switch tabs with click or arrow keys when focused.", metrics, [140, 140, 160]),
        ],
    )
}

fn build_table_panel(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let columns = vec![
        Column::new("Service", 2.0),
        Column::new("Status", 1.0),
        Column::new("CPU %", 1.0),
        Column::new("Memory", 1.0),
        Column::new("Uptime", 1.5),
    ];

    let data = vec![
        vec!["api-gateway".into(), "Running".into(), "12.4".into(), "256 MB".into(), "14d 3h".into()],
        vec!["auth-service".into(), "Running".into(), "8.2".into(), "128 MB".into(), "14d 3h".into()],
        vec!["db-primary".into(), "Running".into(), "45.1".into(), "2.1 GB".into(), "30d 12h".into()],
        vec!["db-replica".into(), "Running".into(), "38.7".into(), "1.8 GB".into(), "30d 12h".into()],
        vec!["cache-redis".into(), "Running".into(), "3.1".into(), "512 MB".into(), "7d 8h".into()],
        vec!["worker-queue".into(), "Warning".into(), "67.8".into(), "384 MB".into(), "2d 5h".into()],
        vec!["scheduler".into(), "Running".into(), "2.0".into(), "64 MB".into(), "14d 3h".into()],
        vec!["logger".into(), "Running".into(), "5.5".into(), "96 MB".into(), "14d 3h".into()],
        vec!["metrics-agg".into(), "Stopped".into(), "0.0".into(), "0 MB".into(), "-".into()],
        vec!["cdn-edge".into(), "Running".into(), "15.3".into(), "192 MB".into(), "60d 1h".into()],
    ];

    let (rows, _set_rows) = create_signal(data);
    let (selected, set_selected) = create_signal(None);

    let table = Table::new(columns, rows, selected, set_selected, metrics);

    WidgetNode::new(
        panel([0.10, 0.12, 0.16]),
        vec![
            label_node("Table / DataGrid", title_metrics, [180, 190, 220]),
            WidgetNode::new(table, vec![]),
        ],
    )
}

fn build_tree_panel(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let tree = TreeView::new(
        vec![
            TreeNode::branch("src", vec![
                TreeNode::branch("components", vec![
                    TreeNode::leaf("Button.tsx").with_icon(icons::FILE_CODE),
                    TreeNode::leaf("Table.tsx").with_icon(icons::FILE_CODE),
                    TreeNode::leaf("TreeView.tsx").with_icon(icons::FILE_CODE),
                    TreeNode::leaf("BarChart.tsx").with_icon(icons::FILE_CODE),
                ]).with_icon(icons::FOLDER_OPEN),
                TreeNode::branch("utils", vec![
                    TreeNode::leaf("format.ts").with_icon(icons::FILE_CODE),
                    TreeNode::leaf("api.ts").with_icon(icons::FILE_CODE),
                ]).with_icon(icons::FOLDER),
                TreeNode::leaf("main.ts").with_icon(icons::FILE_CODE),
                TreeNode::leaf("App.tsx").with_icon(icons::FILE_CODE),
            ]).with_icon(icons::FOLDER_OPEN),
            TreeNode::branch("config", vec![
                TreeNode::leaf("database.toml").with_icon(icons::COG),
                TreeNode::leaf("deploy.yaml").with_icon(icons::ROCKET),
                TreeNode::leaf(".env").with_icon(icons::LOCK),
            ]).with_icon(icons::FOLDER).with_expanded(false),
            TreeNode::branch("docs", vec![
                TreeNode::leaf("README.md").with_icon(icons::FILE_TEXT),
                TreeNode::leaf("API.md").with_icon(icons::FILE_TEXT),
                TreeNode::leaf("CHANGELOG.md").with_icon(icons::FILE_TEXT),
            ]).with_icon(icons::FOLDER).with_expanded(false),
            TreeNode::leaf("Cargo.toml").with_icon(icons::RUST),
            TreeNode::leaf("LICENSE").with_icon(icons::FILE),
        ],
        metrics,
    );

    WidgetNode::new(
        panel([0.10, 0.12, 0.16]),
        vec![
            label_node("TreeView", title_metrics, [180, 190, 220]),
            WidgetNode::new(tree, vec![]),
        ],
    )
}

fn build_chart_panel(metrics: Metrics, title_metrics: Metrics) -> WidgetNode {
    let bars = vec![
        Bar::new("Mon", 320.0, [0.20, 0.65, 0.85, 1.0]),
        Bar::new("Tue", 480.0, [0.25, 0.70, 0.88, 1.0]),
        Bar::new("Wed", 720.0, [0.30, 0.75, 0.90, 1.0]),
        Bar::new("Thu", 560.0, [0.25, 0.70, 0.88, 1.0]),
        Bar::new("Fri", 890.0, [0.35, 0.80, 0.92, 1.0]),
        Bar::new("Sat", 240.0, [0.20, 0.60, 0.80, 1.0]),
        Bar::new("Sun", 180.0, [0.18, 0.55, 0.75, 1.0]),
    ];
    let (bars_sig, _set_bars) = create_signal(bars);

    let chart = BarChart::new(bars_sig.clone(), metrics, 260.0)
        .with_bar_radius(6.0)
        .with_bar_gap(10.0);

    // Second chart with different colors
    let bars2 = vec![
        Bar::new("API", 95.0, [0.20, 0.70, 0.40, 1.0]),
        Bar::new("Auth", 87.0, [0.25, 0.75, 0.45, 1.0]),
        Bar::new("DB", 72.0, [0.90, 0.55, 0.20, 1.0]),
        Bar::new("Cache", 99.0, [0.30, 0.80, 0.50, 1.0]),
        Bar::new("Queue", 45.0, [0.85, 0.25, 0.25, 1.0]),
    ];
    let (bars2_sig, _set_bars2) = create_signal(bars2);

    let chart2 = BarChart::new(bars2_sig, metrics, 200.0)
        .with_max_value(100.0)
        .with_bar_radius(4.0)
        .with_bar_gap(12.0);

    WidgetNode::new(
        panel([0.10, 0.12, 0.16]),
        vec![
            label_node("BarChart \u{2014} Requests/day", title_metrics, [180, 190, 220]),
            WidgetNode::new(chart, vec![]),
            label_node("Service Health (%)", title_metrics, [180, 190, 220]),
            WidgetNode::new(chart2, vec![]),
        ],
    )
}

// --- Main ---

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(14.0, 20.0);
    let title_metrics = Metrics::new(18.0, 26.0);

    let left_col = WidgetNode::new(
        Flex::column(16.0, 0.0),
        vec![
            build_tabs_panel(metrics, title_metrics),
            build_table_panel(metrics, title_metrics),
        ],
    );

    let right_col = WidgetNode::new(
        Flex::column(16.0, 0.0),
        vec![
            build_tree_panel(metrics, title_metrics),
            build_chart_panel(metrics, title_metrics),
        ],
    );

    let root = WidgetNode::new(
        Container::new()
            .with_background([0.06, 0.07, 0.10])
            .with_padding(20.0)
            .with_gap(16.0),
        vec![
            label_node("BexaUI \u{2014} Widget Showcase", Metrics::new(24.0, 34.0), [220, 225, 240]),
            WidgetNode::new(Flex::row(16.0), vec![left_col, right_col]),
        ],
    );

    App::new(root)
        .theme(theme)
        .title("BexaUI \u{2014} Widget Showcase")
        .run();
}
