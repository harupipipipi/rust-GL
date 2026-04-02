# rust2d_ui

[![CI](https://github.com/harupipipipi/rust-GL/actions/workflows/ci.yml/badge.svg)](https://github.com/harupipipipi/rust-GL/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Note:** The repository is named **rust-GL** for historical reasons.
> This crate does **not** use OpenGL — it is a pure software rasteriser
> built on [softbuffer](https://crates.io/crates/softbuffer).

A pure-Rust 2D UI toolkit experiment using:

- **winit** — cross-platform window and event loop
- **softbuffer** — software pixel buffer presentation (zero GPU deps)
- **font-kit + fontdue** — system font discovery and glyph rasterisation

<!-- TODO: screenshot -->

## Quick Start

```bash
git clone https://github.com/harupipipipi/rust-GL.git
cd rust-GL
cargo run
```

This launches a small demo window with a text label and a clickable button.

## Usage

The example below (`examples/hello.rs`) shows how to build a widget tree:

```rust
use rust2d_ui::{App, Container, Text, Button};

fn main() {
    // 1. Create the application.
    let mut app = App::new(800, 600).expect("failed to create App");

    // 2. Assemble a widget tree.
    let mut root = Container::new_auto();
    root.push(Text::new_auto("Hello, rust2d_ui!"));
    root.push(
        Button::new_auto("Click me").on_click(|| {
            println!("button clicked!");
        }),
    );

    // 3. Attach the tree and compute layout.
    app.root = root;
    app.request_layout();

    // 4. Run the event loop (opens a window).
    rust2d_ui::run().expect("event loop failed");
}
```

Run it with:

```bash
cargo run --example hello
```

## Examples

| Example | Description |
|---------|-------------|
| `demo`  | Minimal — calls `rust2d_ui::run()` |
| `hello` | Widget-tree construction pattern |

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for:

- Rendering pipeline diagram
- Module structure
- Design decisions (Widget / Layout tree separation, why softbuffer)
- Known constraints and future work

## Building & Testing

```bash
cargo check --lib        # type-check library only
cargo test               # run the test suite
cargo fmt --check        # verify formatting
cargo clippy -- -D warnings  # lint
```

## License

[MIT](LICENSE)
