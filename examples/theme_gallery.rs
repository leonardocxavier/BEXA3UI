use bexa_ui::prelude::*;

fn palette_card(title: &str, bg: [f32; 3], text: [u8; 3], metrics: Metrics) -> WidgetNode {
    let label = Label::new(
        std::rc::Rc::new(std::cell::RefCell::new(title.to_string())),
        metrics,
        text,
    )
    .with_align(Align::Left)
    .with_padding(8.0);

    ui! {
        Container::new().with_background(bg).with_padding(8.0) => {
            label,
        }
    }
}

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(18.0, 24.0);

    let root = ui! {
        Container::new().with_padding(32.0) => {
            Flex::row(16.0) => {
                palette_card("Ocean", [0.18, 0.30, 0.40], [230, 230, 230], metrics),
                palette_card("Light", [0.88, 0.90, 0.92], [20, 20, 20], metrics),
                palette_card("Dark", [0.12, 0.14, 0.18], [230, 230, 230], metrics),
            },
        }
    };

    App::new(root)
        .theme(theme)
        .title("BexaUI - Themes")
        .run();
}
