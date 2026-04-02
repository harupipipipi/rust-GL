//! Demonstrates how to build a widget tree with the rust2d_ui API.
//!
//! ```bash
//! cargo run --example hello
//! ```

use rust2d_ui::{App, Button, Container, Text};

/// Build a custom [`App`] with a hand-assembled widget tree.
///
/// This shows the fundamental pattern:
///   App::new() → Container::new_auto() → add children → app.root = root
fn build_app() -> App {
    let mut app = App::new(800, 600).expect("failed to create App");

    // ── widget tree ──────────────────────────────────────────────
    let mut root = Container::new_auto();
    root.push(Text::new_auto("Hello, rust2d_ui!"));
    root.push(Button::new_auto("Click me").on_click(|| {
        println!("🖱  button clicked!");
    }));

    app.root = root;
    app.request_layout();
    app
}

fn main() {
    // Build the app to demonstrate the API (printed to confirm it works).
    let app = build_app();
    println!("App built — root has {} children", app.root.children.len(),);

    // Launch the built-in demo window.
    // NOTE: `run()` currently creates its own App internally.
    //       A future version will accept a user-built App.
    if let Err(e) = rust2d_ui::run() {
        eprintln!("fatal: {e}");
        std::process::exit(1);
    }
}
