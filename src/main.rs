//! Minimal binary that launches the rust2d_ui demo window.

fn main() {
    if let Err(e) = rust2d_ui::run() {
        eprintln!("fatal: {e}");
        std::process::exit(1);
    }
}
