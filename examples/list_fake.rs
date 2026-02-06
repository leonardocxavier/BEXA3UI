use bexa_ui::prelude::*;

fn list_item(text: &str, metrics: Metrics) -> WidgetNode {
    let label = Label::new(
        std::rc::Rc::new(std::cell::RefCell::new(text.to_string())),
        metrics,
        [230, 230, 230],
    )
    .with_align(Align::Left)
    .with_padding(6.0);

    WidgetNode::new(Container::new().with_background([0.20, 0.32, 0.44]).with_padding(4.0), vec![WidgetNode::new(label, vec![])])
}

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(18.0, 24.0);

    let mut items = Vec::new();
    for idx in 1..=8 {
        let label = format!("Item {} - example row", idx);
        items.push(list_item(&label, metrics));
    }

    let list = WidgetNode::new(
        Flex::column(8.0, 0.0),
        items,
    );

    let panel = WidgetNode::new(
        Container::new().with_background(theme.panel).with_padding(12.0),
        vec![list],
    );

    let root = WidgetNode::new(Container::new().with_padding(32.0), vec![panel]);

    App::new(root)
        .theme(theme)
        .title("BexaUI - List")
        .run();
}
