use std::cell::RefCell;
use std::rc::Rc;

use bexa_ui::prelude::*;

fn main() {
    let status = Rc::new(RefCell::new(String::from("Pronto")));
    let theme = Theme::ocean();

    let button_metrics = Metrics::new(22.0, 30.0);
    let status_metrics = Metrics::new(18.0, 24.0);

    let mut local_button = Button::new("Arquivos", button_metrics)
        .with_colors(
            theme.button,
            theme.button_hover,
            theme.button_active,
            theme.button_focus,
        )
        .with_padding(16.0)
        .with_text_colors([25, 25, 25], [15, 15, 15], [250, 250, 250]);

    let mut remote_button = Button::new("Conexao", button_metrics)
        .with_colors(
            [0.90, 0.55, 0.20],
            [0.95, 0.70, 0.30],
            [0.85, 0.45, 0.10],
            [0.92, 0.62, 0.25],
        )
        .with_padding(16.0)
        .with_text_colors([25, 25, 25], [15, 15, 15], [250, 250, 250]);

    let status_local = status.clone();
    local_button.set_on_click(move || {
        *status_local.borrow_mut() = "Abrindo painel local...".to_string();
    });

    let status_remote = status.clone();
    remote_button.set_on_click(move || {
        *status_remote.borrow_mut() = "Conectando ao remoto...".to_string();
    });

    let row = WidgetNode::new(
        Flex::row(24.0),
        vec![
            WidgetNode::new(local_button, vec![]),
            WidgetNode::new(remote_button, vec![]),
        ],
    );

    let panel = WidgetNode::new(
        Container::new()
            .with_background(theme.panel)
            .with_padding(12.0),
        vec![row],
    );

    let status_label = Label::new(status.clone(), status_metrics, theme.text_primary)
        .with_align(Align::Left)
        .with_padding(6.0);

    let column = WidgetNode::new(
        Flex::column(16.0, 0.0),
        vec![panel, WidgetNode::new(status_label, vec![])],
    );

    let root = WidgetNode::new(Container::new().with_padding(32.0), vec![column]);

    App::new(root)
        .theme(theme)
        .title("BexaUI - Hello")
        .run();
}
