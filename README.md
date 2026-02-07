# BexaUI

**The Hacker's UI Toolkit for Rust.**

A GPU-accelerated, native GUI framework designed for building **developer tools**, **terminals**, and **infrastructure dashboards** in Rust.

Unlike generic GUI frameworks, BexaUI treats the **terminal** as a first-class citizen. No Electron. No web stack. Just Rust, GPU, and a real PTY.

## Why BexaUI?

- **Built-in Terminal** — Real PTY-backed VT100/ANSI terminal emulator with interactive shell, colors, and cursor navigation. No other Rust GUI toolkit ships this.
- **GPU Accelerated** — wgpu 28 with SDF rendering for crisp rounded corners, borders, and alpha blending at any scale.
- **Declarative `ui!` Macro** — Build widget trees like JSX, with compile-time Rust safety. No runtime overhead.
- **Flexbox Layout** — Powered by taffy with CSS-like semantics. No manual pixel math.
- **Reactive Signals** — Simple `Signal<T>` / `SetSignal<T>` state management inspired by modern reactive frameworks.
- **Multi-Window** — Spawn new windows from callbacks (like Delphi forms). Each window has its own widget tree.
- **Theming** — Centralized `Theme` struct with pre-built palettes (Ocean, Light, Dark).
- **Keyboard & Clipboard** — Tab/Shift+Tab focus, Enter/Space activation, Ctrl+C/V/X.

## Ideal Use Cases

BexaUI is **not** a general-purpose GUI framework. It is optimized for:

- **Custom Terminal Emulators** — SSH clients, multiplexers, serial monitors
- **DevTools** — Log viewers, debuggers, profilers, REPL interfaces
- **Admin Panels** — Docker/K8s dashboards, database managers, server monitoring
- **Infrastructure Tooling** — Embedded flash tools, deployment consoles, CI/CD dashboards
- **Internal Tools** — Company-specific ops tools that need a real terminal

## Widgets

| Widget | Description |
|--------|-------------|
| `Container` | Flexbox container with padding, background color, border radius |
| `Flex` | Row/column layout wrapper |
| `Label` | Text display with signal-based reactive content |
| `Button` | Interactive button with hover/active/focus states and click handlers |
| `TextInput` | Text field with cursor, selection, clipboard, and glyph-based positioning |
| `Checkbox` | Toggle checkbox with signal binding |
| `RadioButton` | Radio button groups with shared signal |
| `Select` | Dropdown select with overlay rendering |
| `Icon` | Nerd Font icon rendering |
| `Terminal` | PTY-backed terminal emulator (feature-gated) |

## Quick Start

```rust
use bexa_ui::prelude::*;

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(16.0, 22.0);

    let (text, set_text) = create_signal("Hello, BexaUI!".to_string());
    let label = Label::new(text, metrics, theme.text_primary);

    let mut btn = Button::new("Click me", metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
        .with_border_radius(6.0);
    btn.set_on_click(move || {
        set_text.set("Button clicked!".to_string());
    });

    let root = ui! {
        Container::new().with_padding(32.0) => {
            label,
            btn,
        }
    };

    App::new(root)
        .theme(theme)
        .title("My App")
        .run();
}
```

## Terminal Widget

The terminal widget spawns a real PTY process and renders it with monospace font and ANSI colors. It requires the `terminal` feature flag.

```rust
use bexa_ui::prelude::*;

fn main() {
    let theme = Theme::ocean();
    let metrics = Metrics::new(16.0, 22.0);
    let requests = App::window_requests();

    let mut btn = Button::new("Open Terminal", metrics)
        .with_colors(theme.button, theme.button_hover, theme.button_active, theme.button_focus)
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

    let root = ui! {
        Container::new().with_padding(32.0) => {
            btn,
        }
    };

    App::new(root)
        .theme(theme)
        .title("Terminal Demo")
        .with_requests(requests)
        .run();
}
```

## Architecture

BexaUI is organized as a Rust workspace with three crates:

```
bexa-ui/              # Facade crate — re-exports everything via prelude
bexa-ui-core/         # Widgets, layout, theme, signals, renderer commands
bexa-ui-render/       # wgpu rendering, glyphon text, event loop, multi-window
```

**Rendering pipeline** (4-pass per frame):
1. Main quad pass (SDF rounded rects with borders)
2. Main text pass (glyphon)
3. Overlay quad pass (dropdowns, tooltips)
4. Overlay text pass

## Examples

```sh
# Basic hello world
cargo run -p bexa-ui-examples --example hello

# Dashboard with all widgets
cargo run -p bexa-ui-examples --example dashboard

# Checkbox, radio buttons, and select dropdown
cargo run -p bexa-ui-examples --example checkbox_radio

# Terminal in a new window
cargo run -p bexa-ui-examples --example terminal --features terminal

# Theme gallery
cargo run -p bexa-ui-examples --example theme_gallery

# Grid layout
cargo run -p bexa-ui-examples --example layout_grid

#ShowCase
cargo run -p bexa-ui-examples --example widget_showcase
```
![alt text](image.png)

## Dependencies

| Crate | Purpose |
|-------|---------|
| [wgpu](https://crates.io/crates/wgpu) 28 | GPU rendering |
| [winit](https://crates.io/crates/winit) 0.29 | Window management and events |
| [glyphon](https://crates.io/crates/glyphon) 0.10 | Text rendering (cosmic-text) |
| [taffy](https://crates.io/crates/taffy) 0.4 | Flexbox layout engine |
| [arboard](https://crates.io/crates/arboard) 3 | Clipboard access |
| [portable-pty](https://crates.io/crates/portable-pty) 0.9 | PTY for terminal (optional) |
| [vte](https://crates.io/crates/vte) 0.15 | ANSI escape sequence parser (optional) |

## Roadmap

**Done:**
- [x] Text input with clipboard
- [x] Keyboard navigation
- [x] Checkbox / Radio / Select
- [x] Scrollbar
- [x] Multi-window support
- [x] Terminal widget (PTY + VT100)

**Next:**
- [ ] Table / DataGrid
- [ ] Slider widget
- [ ] Image widget
- [ ] Animations and transitions
- [ ] Accessibility (screen readers)
- [ ] Publish to crates.io
- [ ] CI/CD with GitHub Actions

## Contributing

Contributions are welcome! BexaUI is especially looking for help with:

- Terminal emulation improvements (mouse support, 24-bit color, scrollback)
- New widgets (Table, Slider, Tree view)
- Testing and CI setup
- Documentation and examples

Feel free to open issues, submit pull requests, or suggest new features.

## Support

If you find BexaUI useful and want to support its development:

<a href="https://www.buymeacoffee.com/leonardocx" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" ></a>

## License

MIT
