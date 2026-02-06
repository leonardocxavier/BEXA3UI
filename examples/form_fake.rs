use std::cell::RefCell;
use std::rc::Rc;

use bexa_ui::prelude::*;

fn input_like(label: &str, metrics: Metrics) -> WidgetNode {
    let label = Label::new(Rc::new(RefCell::new(label.to_string())), metrics, [230, 230, 230])
        .with_align(Align::Left)
        .with_padding(8.0);

    WidgetNode::new(
        Container::new().with_background([0.18, 0.30, 0.40]).with_padding(6.0),
        vec![WidgetNode::new(label, vec![])],
    )
}

fn main() {
    let theme = Theme::ocean();
    let status = Rc::new(RefCell::new(String::from("Idle")));

    let field_metrics = Metrics::new(18.0, 24.0);
    let button_metrics = Metrics::new(20.0, 28.0);

    let name_field = input_like("Name: example", field_metrics);
    let server_field = input_like("Server: 10.0.0.5", field_metrics);

    let mut submit = Button::new("Submit", button_metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
        .with_padding(14.0)
        .with_text_colors([25, 25, 25], [15, 15, 15], [250, 250, 250]);

    let status_submit = status.clone();
    submit.set_on_click(move || {
        *status_submit.borrow_mut() = "Form submitted".to_string();
    });

    let form = WidgetNode::new(
        Flex::column(12.0, 0.0),
        vec![
            name_field,
            server_field,
            WidgetNode::new(submit, vec![]),
        ],
    );

    let status_label = Label::new(status, field_metrics, theme.text_primary)
        .with_align(Align::Left)
        .with_padding(6.0);

    let root = WidgetNode::new(
        Container::new().with_padding(32.0),
        vec![
            WidgetNode::new(Container::new().with_background(theme.panel).with_padding(16.0), vec![form]),
            WidgetNode::new(status_label, vec![]),
        ],
    );

    App::new(root)
        .theme(theme)
        .title("BexaUI - Form")
        .run();
}
