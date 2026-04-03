//! Text processing benchmarks (run with: cargo test --release -- --ignored bench_)
//!
//! These use `std::time::Instant` for manual benchmarking as `#[ignore]` tests.
//! They will be migrated to criterion once Cargo.toml is updated during integration.

use rust2d_ui::FontManager;

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
fn bench_measure_text_short_x1000() {
    let fm = font_manager_or_skip();
    let short = "Hello, world!";
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = fm.measure_text(short, 20.0);
    }
    let elapsed = start.elapsed();
    println!(
        "measure_text short ({} chars) x1000: {:?} ({:.1} ns/op)",
        short.len(),
        elapsed,
        elapsed.as_nanos() as f64 / 1000.0
    );
}

#[test]
#[ignore]
fn bench_measure_text_long_x1000() {
    let fm = font_manager_or_skip();
    let long: String = "The quick brown fox jumps over the lazy dog. ".repeat(20);
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = fm.measure_text(&long, 20.0);
    }
    let elapsed = start.elapsed();
    println!(
        "measure_text long ({} chars) x1000: {:?} ({:.1} ns/op)",
        long.len(),
        elapsed,
        elapsed.as_nanos() as f64 / 1000.0
    );
}

#[test]
#[ignore]
fn bench_wrap_text_cjk_mixed_x100() {
    let fm = font_manager_or_skip();
    // CJK-mixed long text: Japanese + English interleaved
    let cjk_mixed: String = "吾輩は猫である。名前はまだ無い。\
        Where I was born I have no idea. \
        ここで始めて人間というものを見た。\
        All I remember is that I was crying somewhere. \
        しかもあとで聞くとそれは書生という人間中で一番獰悪な種族であったそうだ。\
        This student was said to catch us and eat us. \
        吾輩はここで始めて人間というものを見た。\
        However, at the time, I did not feel particularly frightened. \
        ただ彼の掌に載せられてスーと持ち上げられた時何だかフワフワした感じがあったばかりである。"
        .repeat(5);
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = fm.wrap_text(&cjk_mixed, 600.0, 16.0);
    }
    let elapsed = start.elapsed();
    println!(
        "wrap_text CJK mixed ({} chars, max_width=600) x100: {:?} ({:.1} us/op)",
        cjk_mixed.chars().count(),
        elapsed,
        elapsed.as_micros() as f64 / 100.0
    );
}
