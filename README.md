# rust2d_ui

> Repository note: The repository is named rust-GL for historical reasons.
> This crate does not use OpenGL — it is a pure software rasteriser built on softbuffer.


A pure-Rust 2D UI experiment using:
- `winit` (window/event loop)
- `softbuffer` (software pixel presentation)
- `font-kit` + `fontdue` (font loading and rasterization)

> Note: This repository was initially bootstrapped via AI-assisted generation and is being iteratively corrected with human review.

## What this crate provides

- Basic rendering primitives (`Canvas`, `Color`, `Rect`)
- Simple widget system (`Container`, `Text`, `Button`, `Widget`)
- Layout tree (`LayoutNode`)
- `App` type for integration and a `run()` demo entrypoint

## Build & test

```bash
cargo check --lib
cargo test
```

## Usage (library)

```rust
use rust2d_ui::{App, Container, Text, Button};

fn build_app() -> Result<App, Box<dyn std::error::Error>> {
    let mut app = App::new(800, 600)?;

    let mut root = Container::new_auto();
    root.push(Text::new_auto("Hello"));
    root.push(Button::new_auto("Click"));

    app.root = root;
    app.request_layout();
    Ok(app)
}
```

## Demo run

`run()` creates a small demo UI and starts the event loop.

## License

MIT
