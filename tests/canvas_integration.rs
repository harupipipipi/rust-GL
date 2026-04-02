//! Integration tests for Canvas drawing primitives.

#[macro_use]
#[path = "test_utils.rs"]
mod test_utils;

use rust2d_ui::*;

// ─────────────────────────────────────────────────────────────
// draw_line
// ─────────────────────────────────────────────────────────────

#[test]
fn draw_line_horizontal() {
    let mut c = test_utils::make_canvas(10, 5, Color::WHITE);
    c.draw_line(1, 2, 8, 2, Color::BLACK);

    for x in 1..=8 {
        assert_pixel!(c, x, 2, Color::BLACK);
    }
    assert_pixel!(c, 0, 2, Color::WHITE);
    assert_pixel!(c, 9, 2, Color::WHITE);
}

#[test]
fn draw_line_vertical() {
    let mut c = test_utils::make_canvas(5, 10, Color::WHITE);
    c.draw_line(2, 1, 2, 8, Color::BLACK);

    for y in 1..=8 {
        assert_pixel!(c, 2, y, Color::BLACK);
    }
    assert_pixel!(c, 2, 0, Color::WHITE);
    assert_pixel!(c, 2, 9, Color::WHITE);
}

#[test]
fn draw_line_diagonal() {
    let mut c = test_utils::make_canvas(10, 10, Color::WHITE);
    c.draw_line(0, 0, 9, 9, Color::BLACK);

    for i in 0..10 {
        assert_pixel!(c, i, i, Color::BLACK);
    }
    assert_pixel!(c, 0, 1, Color::WHITE);
}

#[test]
fn draw_line_reverse_direction() {
    let mut c = test_utils::make_canvas(10, 5, Color::WHITE);
    c.draw_line(8, 2, 1, 2, Color::BLACK);

    for x in 1..=8 {
        assert_pixel!(c, x, 2, Color::BLACK);
    }
}

#[test]
fn draw_line_entirely_out_of_bounds() {
    let mut c = test_utils::make_canvas(10, 10, Color::WHITE);
    c.draw_line(-50, -50, -10, -10, Color::BLACK);
    c.draw_line(100, 100, 200, 200, Color::BLACK);
    assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
}

#[test]
fn draw_line_partially_out_of_bounds() {
    let mut c = test_utils::make_canvas(10, 10, Color::WHITE);
    c.draw_line(-5, 5, 5, 5, Color::BLACK);
    for x in 0..=5 {
        assert_pixel!(c, x, 5, Color::BLACK);
    }
}

#[test]
fn draw_line_single_point() {
    let mut c = test_utils::make_canvas(5, 5, Color::WHITE);
    c.draw_line(2, 2, 2, 2, Color::BLACK);
    assert_pixel!(c, 2, 2, Color::BLACK);
    assert_pixel!(c, 1, 2, Color::WHITE);
    assert_pixel!(c, 3, 2, Color::WHITE);
}

// ─────────────────────────────────────────────────────────────
// draw_rounded_rect
// ─────────────────────────────────────────────────────────────

#[test]
fn draw_rounded_rect_radius_zero_fills_like_fill_rect() {
    let mut c1 = test_utils::make_canvas(20, 20, Color::WHITE);
    let mut c2 = test_utils::make_canvas(20, 20, Color::WHITE);

    let rect = Rect::new(2, 2, 16, 16);
    c1.draw_rounded_rect(rect, 0, Color::BLACK);
    c2.fill_rect(rect, Color::BLACK);

    assert_eq!(c1.pixels(), c2.pixels());
}

#[test]
fn draw_rounded_rect_radius_exceeds_half_size() {
    let mut c = test_utils::make_canvas(20, 20, Color::WHITE);
    c.draw_rounded_rect(Rect::new(0, 0, 20, 20), 1000, Color::BLACK);

    assert_pixel!(c, 10, 10, Color::BLACK);
    assert_pixel!(c, 0, 0, Color::WHITE);
}

#[test]
fn draw_rounded_rect_1x1() {
    let mut c = test_utils::make_canvas(5, 5, Color::WHITE);
    c.draw_rounded_rect(Rect::new(2, 2, 1, 1), 10, Color::BLACK);
    assert_pixel!(c, 2, 2, Color::BLACK);
    assert_pixel!(c, 1, 1, Color::WHITE);
}

// ─────────────────────────────────────────────────────────────
// fill_rect — extreme coordinates
// ─────────────────────────────────────────────────────────────

#[test]
fn fill_rect_huge_coords_no_panic() {
    let mut c = test_utils::make_canvas(100, 100, Color::WHITE);
    c.fill_rect(
        Rect::new(i32::MAX - 10, i32::MAX - 10, 20, 20),
        Color::BLACK,
    );
    assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
}

#[test]
fn fill_rect_negative_huge_coords_no_panic() {
    let mut c = test_utils::make_canvas(100, 100, Color::WHITE);
    c.fill_rect(Rect::new(i32::MIN, i32::MIN, 10, 10), Color::BLACK);
    assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
}

#[test]
fn fill_rect_partially_overlaps_huge_width() {
    let mut c = test_utils::make_canvas(100, 100, Color::WHITE);
    c.fill_rect(Rect::new(50, 50, u32::MAX, 10), Color::BLACK);
    assert_pixel!(c, 50, 50, Color::BLACK);
    assert_pixel!(c, 99, 50, Color::BLACK);
}

// ─────────────────────────────────────────────────────────────
// blend_pixel — mass call (no panic)
// ─────────────────────────────────────────────────────────────

#[test]
fn blend_pixel_1000x1000_no_panic() {
    let mut c = Canvas::new(1000, 1000);
    c.clear(Color::WHITE);
    let semi = Color::rgba(128, 64, 32, 128);
    for y in 0..1000i32 {
        for x in 0..1000i32 {
            c.blend_pixel(x, y, semi);
        }
    }
    let p = c.pixels()[500 * 1000 + 500];
    assert_ne!(p, Color::WHITE.to_u32());
    assert_ne!(p, 0);
}

// ─────────────────────────────────────────────────────────────
// Canvas::new(0,0) → resize → draw
// ─────────────────────────────────────────────────────────────

#[test]
fn canvas_zero_then_resize_then_draw() {
    let mut c = Canvas::new(0, 0);
    assert_eq!(c.pixels().len(), 0);
    assert_eq!(c.width(), 0);
    assert_eq!(c.height(), 0);

    c.resize(100, 100);
    assert_eq!(c.width(), 100);
    assert_eq!(c.height(), 100);
    assert_eq!(c.pixels().len(), 10_000);

    c.clear(Color::WHITE);
    c.fill_rect(Rect::new(10, 10, 20, 20), Color::BLACK);
    assert_pixel!(c, 15, 15, Color::BLACK);
    assert_pixel!(c, 0, 0, Color::WHITE);

    c.draw_line(0, 0, 99, 0, Color::BLUE);
    assert_pixel!(c, 50, 0, Color::BLUE);

    c.draw_rounded_rect(Rect::new(40, 40, 20, 20), 4, Color::BLUE);
    assert_pixel!(c, 50, 50, Color::BLUE);
}
