//! Layout benchmarks (run with: cargo test --release -- --ignored bench_)
//!
//! These use `std::time::Instant` for manual benchmarking as `#[ignore]` tests.
//! They will be migrated to criterion once Cargo.toml is updated during integration.

use rust2d_ui::*;

/// Helper: skip the test if system fonts are unavailable (e.g. headless CI).
fn font_manager_or_skip() -> FontManager {
    match FontManager::new() {
        Ok(fm) => fm,
        Err(e) => {
            eprintln!("FontManager unavailable, skipping benchmark: {e}");
            panic!("SKIPPED: no system font available");
        }
    }
}

#[test]
#[ignore]
fn bench_layout_100_children_x100() {
    let fm = font_manager_or_skip();
    let constraints = BoxConstraints::loose(800.0, 10000.0);

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let mut root = Container::new_auto();
        for i in 0..100 {
            root.push(Text::new_auto(format!("Child item {i}")));
        }
        let _layout = root.layout(constraints, 0, 0, &fm);
    }
    let elapsed = start.elapsed();
    println!(
        "layout 100-children container x100: {:?} ({:.1} us/op)",
        elapsed,
        elapsed.as_micros() as f64 / 100.0
    );
}

#[test]
#[ignore]
fn bench_layout_nested_10_deep() {
    let fm = font_manager_or_skip();
    let constraints = BoxConstraints::loose(800.0, 10000.0);

    let start = std::time::Instant::now();
    for _ in 0..100 {
        // Build a 10-level deep nesting: Container > Container > ... > Text
        let mut innermost = Container::new_auto();
        innermost.push(Text::new_auto("Leaf node"));

        let mut current = innermost;
        for _ in 0..9 {
            let mut parent = Container::new_auto();
            parent.push(current);
            current = parent;
        }
        let _layout = current.layout(constraints, 0, 0, &fm);
    }
    let elapsed = start.elapsed();
    println!(
        "layout nested-10-deep x100: {:?} ({:.1} us/op)",
        elapsed,
        elapsed.as_micros() as f64 / 100.0
    );
}
