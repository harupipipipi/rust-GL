//! Rendering benchmarks (run with: cargo test --release -- --ignored bench_)
//!
//! These use `std::time::Instant` for manual benchmarking as `#[ignore]` tests.
//! They will be migrated to criterion once Cargo.toml is updated during integration.

use rust2d_ui::*;

#[test]
#[ignore]
fn bench_fill_rect_1000() {
    let mut c = Canvas::new(1920, 1080);
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        c.fill_rect(Rect::new(100, 100, 400, 300), Color::BLUE);
    }
    let elapsed = start.elapsed();
    println!(
        "fill_rect x1000: {:?} ({:.1} ns/op)",
        elapsed,
        elapsed.as_nanos() as f64 / 1000.0
    );
}

#[test]
#[ignore]
fn bench_draw_rounded_rect_1000() {
    let mut c = Canvas::new(1920, 1080);
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        c.draw_rounded_rect(Rect::new(100, 100, 400, 300), 16, Color::BLUE);
    }
    let elapsed = start.elapsed();
    println!(
        "draw_rounded_rect x1000: {:?} ({:.1} ns/op)",
        elapsed,
        elapsed.as_nanos() as f64 / 1000.0
    );
}

#[test]
#[ignore]
fn bench_blend_pixel_fullscreen() {
    let mut c = Canvas::new(1920, 1080);
    c.clear(Color::WHITE);
    let semi = Color::rgba(255, 0, 0, 128);
    let start = std::time::Instant::now();
    for y in 0..1080_i32 {
        for x in 0..1920_i32 {
            c.blend_pixel(x, y, semi);
        }
    }
    let elapsed = start.elapsed();
    let total_pixels = 1920u64 * 1080;
    println!(
        "blend_pixel fullscreen (1920x1080 = {} px): {:?} ({:.1} ns/px)",
        total_pixels,
        elapsed,
        elapsed.as_nanos() as f64 / total_pixels as f64
    );
}

#[test]
#[ignore]
fn bench_clear_100() {
    let mut c = Canvas::new(1920, 1080);
    let start = std::time::Instant::now();
    for _ in 0..100 {
        c.clear(Color::BLACK);
    }
    let elapsed = start.elapsed();
    println!(
        "clear x100: {:?} ({:.1} ns/op)",
        elapsed,
        elapsed.as_nanos() as f64 / 100.0
    );
}

#[test]
#[ignore]
fn bench_draw_line_diagonal_1000() {
    let mut c = Canvas::new(1920, 1080);
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        c.draw_line(0, 0, 1919, 1079, Color::BLUE);
    }
    let elapsed = start.elapsed();
    println!(
        "draw_line diagonal x1000: {:?} ({:.1} ns/op)",
        elapsed,
        elapsed.as_nanos() as f64 / 1000.0
    );
}
