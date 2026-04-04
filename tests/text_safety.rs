//! Integration tests for safe-by-default Unicode handling.

#[macro_use]
#[path = "test_utils.rs"]
mod test_utils;

use rust2d_ui::{Canvas, Color, FontManager, Rect, TextSafetyMode};

fn count_non_background_pixels(canvas: &Canvas, background: Color) -> usize {
    canvas
        .pixels()
        .iter()
        .filter(|&&pixel| pixel != background.to_u32())
        .count()
}

#[test]
fn safe_mode_is_the_default() {
    let fm = require_font_manager!();
    assert_eq!(fm.safety_mode(), TextSafetyMode::Safe);
}

#[test]
fn safe_mode_normalizes_common_mojibake_triggers() {
    let fm = require_font_manager!();

    let accidental = "Cafe\u{0301}\u{200D}\u{0007}\u{00A0}X";
    let safe = "Café\u{FFFD}\u{FFFD} X";

    assert_eq!(fm.measure_text(accidental, 18.0), fm.measure_text(safe, 18.0));
    assert_eq!(fm.wrap_text(accidental, 1000.0, 18.0), vec![safe.to_string()]);
}

#[test]
fn safe_mode_keeps_line_breaks_but_sanitizes_dangerous_chars() {
    let fm = require_font_manager!();

    let accidental = "1\r\n2\u{202E}3\n\n4";
    let lines = fm.wrap_text(accidental, 1000.0, 18.0);

    assert_eq!(lines, vec!["1".to_string(), "2\u{FFFD}3".to_string(), "".to_string(), "4".to_string()]);
}

#[test]
fn raw_mode_preserves_advanced_unicode_sequences() {
    let fm = match FontManager::with_safety_mode(TextSafetyMode::Raw) {
        Ok(fm) => fm,
        Err(e) => {
            eprintln!("SKIPPED: FontManager unavailable — {}", e);
            return;
        }
    };

    let raw = "A\u{200D}B";
    assert_eq!(fm.wrap_text(raw, 1000.0, 18.0), vec![raw.to_string()]);
}

#[test]
fn missing_glyphs_still_draw_visible_fallback_pixels() {
    let fm = require_font_manager!();
    let background = Color::WHITE;
    let mut canvas = Canvas::new(120, 48);
    canvas.clear(background);

    fm.draw_text_in_rect(
        &mut canvas,
        "\u{10FFFF}",
        Rect::new(4, 4, 80, 32),
        20.0,
        Color::BLACK,
    );

    assert!(
        count_non_background_pixels(&canvas, background) > 0,
        "missing glyph fallback should draw visible pixels"
    );
}
