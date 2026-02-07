use bexa_ui::prelude::*;

fn label_node(text: &str, metrics: Metrics) -> WidgetNode {
    let (sig, _) = create_signal(text.to_string());
    WidgetNode::new(
        Label::new(sig, metrics, [220, 225, 235]).with_align(Align::Left),
        vec![],
    )
}

fn image_card(title: &str, fit: ImageFit, path: &str, metrics: Metrics) -> WidgetNode {
    let image = Image::new(path)
        .with_size(260.0, 170.0)
        .with_fit(fit)
        .with_background([0.10, 0.12, 0.16, 1.0])
        .with_border_radius(8.0);

    let image_box = WidgetNode::new(
        Container::new()
            .with_background([0.08, 0.10, 0.14])
            .with_border_radius(10.0)
            .with_border(1.0, [0.18, 0.22, 0.32, 1.0]),
        vec![WidgetNode::new(image, vec![])],
    );

    WidgetNode::new(
        Flex::column(8.0, 0.0),
        vec![label_node(title, metrics), image_box],
    )
}

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(14.0, 20.0);
    let path = "examples/assets/placeholder.ppm";

    let row = WidgetNode::new(
        Flex::row(12.0),
        vec![
            image_card("Fill (stretch)", ImageFit::Fill, path, metrics),
            image_card("Contain", ImageFit::Contain, path, metrics),
            image_card("Cover", ImageFit::Cover, path, metrics),
        ],
    );

    let root = WidgetNode::new(
        Container::new()
            .with_background([0.05, 0.06, 0.09])
            .with_padding(24.0)
            .with_gap(16.0),
        vec![
            label_node("Image Fit Demo", Metrics::new(20.0, 28.0)),
            row,
        ],
    );

    App::new(root)
        .theme(theme)
        .title("BexaUI - Image Fit")
        .run();
}
