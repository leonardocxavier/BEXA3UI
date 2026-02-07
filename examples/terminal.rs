use bexa_ui::prelude::*;

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(16.0, 22.0);
    let requests = App::window_requests();

    let mut btn = Button::new("Open Terminal", metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
        .with_text_colors(theme.text_primary, theme.text_primary, [255, 255, 255])
        .with_border_radius(6.0);
    btn.set_on_click({
        let reqs = requests.clone();
        move || {
            let term = Terminal::new(Metrics::new(14.0, 18.0));
            reqs.lock().unwrap().push(WindowRequest {
                title: "BexaUI Terminal".into(),
                width: 900,
                height: 600,
                root: ui!(term),
                theme: Theme::ocean(),
            });
        }
    });

    let (label_text, _) = create_signal("Click the button to open a terminal in a new window.".to_string());
    let label = Label::new(label_text, metrics, theme.text_secondary);

    let root = ui! {
        Container::new().with_padding(32.0) => {
            label,
            btn,
        }
    };

    App::new(root)
        .theme(theme)
        .title("BexaUI - Terminal Demo")
        .with_requests(requests)
        .run();
}
