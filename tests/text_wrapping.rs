//! Integration tests for FontManager text wrapping and measurement.
//!
//! All tests gate on `FontManager::new()` being available.
//! If no system fonts exist, tests print a skip message and return.

#[macro_use]
#[path = "test_utils.rs"]
mod test_utils;

// ─────────────────────────────────────────────────────────────
// wrap_text — empty string
// ─────────────────────────────────────────────────────────────

#[test]
fn wrap_text_empty_string() {
    let fm = require_font_manager!();
    let lines = fm.wrap_text("", 200.0, 16.0);
    assert_eq!(lines.len(), 1, "empty string should produce one empty line");
    assert_eq!(lines[0], "");
}

// ─────────────────────────────────────────────────────────────
// wrap_text — all spaces
// ─────────────────────────────────────────────────────────────

#[test]
fn wrap_text_all_spaces_wide() {
    let fm = require_font_manager!();
    let lines = fm.wrap_text("     ", 500.0, 16.0);
    assert!(!lines.is_empty());
    let total_chars: usize = lines.iter().map(|l| l.len()).sum();
    assert_eq!(total_chars, 5, "all 5 spaces should be preserved");
}

#[test]
fn wrap_text_all_spaces_narrow() {
    let fm = require_font_manager!();
    let lines = fm.wrap_text("     ", 1.0, 16.0);
    assert!(!lines.is_empty());
}

// ─────────────────────────────────────────────────────────────
// wrap_text — newline only
// ─────────────────────────────────────────────────────────────

#[test]
fn wrap_text_newlines_only() {
    let fm = require_font_manager!();
    let lines = fm.wrap_text("\n\n", 200.0, 16.0);
    assert_eq!(lines.len(), 3, "two newlines produce 3 empty lines");
    for line in &lines {
        assert_eq!(line, "");
    }
}

#[test]
fn wrap_text_single_newline() {
    let fm = require_font_manager!();
    let lines = fm.wrap_text("\n", 200.0, 16.0);
    assert_eq!(lines.len(), 2);
}

// ─────────────────────────────────────────────────────────────
// wrap_text — single ASCII word longer than max_width
// ─────────────────────────────────────────────────────────────

#[test]
fn wrap_text_long_single_word() {
    let fm = require_font_manager!();
    let long_word = "Supercalifragilisticexpialidocious";
    let lines = fm.wrap_text(long_word, 20.0, 16.0);
    assert!(!lines.is_empty());
    let joined: String = lines.join("");
    assert_eq!(joined, long_word);
}

// ─────────────────────────────────────────────────────────────
// wrap_text — CJK + Latin mixed
// ─────────────────────────────────────────────────────────────

#[test]
fn wrap_text_cjk_latin_mixed() {
    let fm = require_font_manager!();
    let text = "Hello世界Test日本語OK";
    let lines = fm.wrap_text(text, 5000.0, 16.0);
    let joined: String = lines.join("");
    assert_eq!(joined, text, "all characters should be preserved");
}

#[test]
fn wrap_text_cjk_narrow_breaks_per_char() {
    let fm = require_font_manager!();
    let text = "あいう";
    let (single_w, _) = fm.measure_text("あ", 16.0);
    let lines = fm.wrap_text(text, single_w + 1.0, 16.0);
    assert!(
        lines.len() >= 2,
        "CJK narrow wrap should produce >= 2 lines, got {}",
        lines.len()
    );
    let joined: String = lines.join("");
    assert_eq!(joined, text, "no characters should be lost");
}

// ─────────────────────────────────────────────────────────────
// measure_text — empty string returns (0.0, _)
// ─────────────────────────────────────────────────────────────

#[test]
fn measure_text_empty_string() {
    let fm = require_font_manager!();
    let (w, h) = fm.measure_text("", 16.0);
    assert!(
        (w - 0.0).abs() < f32::EPSILON,
        "empty string width should be 0.0, got {}",
        w
    );
    assert!(
        (h - 16.0).abs() < 1.0,
        "empty string height should be ~16.0, got {}",
        h
    );
}

#[test]
fn measure_text_single_char_positive_width() {
    let fm = require_font_manager!();
    let (w, _h) = fm.measure_text("A", 16.0);
    assert!(w > 0.0, "single char should have positive width, got {}", w);
}
