use bexa_ui::prelude::*;

fn card(label: &str, color: [f32; 3], metrics: Metrics) -> WidgetNode {
    let label = Label::new(std::rc::Rc::new(std::cell::RefCell::new(label.to_string())), metrics, [230, 230, 230])
        .with_align(Align::Left)
        .with_padding(8.0);

    WidgetNode::new(
        Container::new().with_background(color).with_padding(8.0),
        vec![WidgetNode::new(label, vec![])],
    )
}

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(18.0, 24.0);

    let row1 = WidgetNode::new(
        Flex::row(16.0),
        vec![
            card("Card A", [0.22, 0.35, 0.48], metrics),
            card("Card B", [0.26, 0.40, 0.54], metrics),
        ],
    );

    let row2 = WidgetNode::new(
        Flex::row(16.0),
        vec![
            card("Card C", [0.30, 0.45, 0.60], metrics),
            card("Card D", [0.34, 0.50, 0.66], metrics),
        ],
    );

    let column = WidgetNode::new(
        Flex::column(16.0, 0.0),
        vec![row1, row2],
    );

    let root = WidgetNode::new(Container::new().with_padding(32.0), vec![column]);

    App::new(root)
        .theme(theme)
        .title("BexaUI - Grid")
        .run();
}
