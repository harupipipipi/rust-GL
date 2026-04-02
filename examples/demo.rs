//! Run the built-in demo UI.
//!
//! ```bash
//! cargo run --example demo
//! ```

fn main() {
    if let Err(e) = rust2d_ui::run() {
        eprintln!("fatal: {e}");
        std::process::exit(1);
    }
}
