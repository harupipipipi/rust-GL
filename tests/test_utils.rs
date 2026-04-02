//! Shared helpers for integration tests.

#![allow(dead_code)]

use rust2d_ui::*;

/// Create a `Canvas` of given size, pre-cleared to `color`.
pub fn make_canvas(w: u32, h: u32, color: Color) -> Canvas {
    let mut c = Canvas::new(w, h);
    c.clear(color);
    c
}

/// Extract the RGB components from a raw `0x00RRGGBB` pixel value.
pub fn pixel_rgb(pixel: u32) -> (u32, u32, u32) {
    let r = (pixel >> 16) & 0xFF;
    let g = (pixel >> 8) & 0xFF;
    let b = pixel & 0xFF;
    (r, g, b)
}

/// Assert that a specific pixel in the canvas matches the expected colour.
macro_rules! assert_pixel {
    ($canvas:expr, $x:expr, $y:expr, $color:expr) => {{
        let w = $canvas.width() as usize;
        let idx = ($y as usize) * w + ($x as usize);
        let actual = $canvas.pixels()[idx];
        let expected = $color.to_u32();
        assert_eq!(
            actual, expected,
            "pixel ({}, {}) = 0x{:08X}, expected 0x{:08X}",
            $x, $y, actual, expected
        );
    }};
}

/// Assert that a specific pixel does NOT match a colour.
macro_rules! assert_pixel_not {
    ($canvas:expr, $x:expr, $y:expr, $color:expr) => {{
        let w = $canvas.width() as usize;
        let idx = ($y as usize) * w + ($x as usize);
        let actual = $canvas.pixels()[idx];
        let rejected = $color.to_u32();
        assert_ne!(
            actual, rejected,
            "pixel ({}, {}) = 0x{:08X}, should NOT be 0x{:08X}",
            $x, $y, actual, rejected
        );
    }};
}

/// Try to create a FontManager; if it fails (no fonts available),
/// print a skip message and return from the calling function.
macro_rules! require_font_manager {
    () => {
        match rust2d_ui::FontManager::new() {
            Ok(fm) => fm,
            Err(e) => {
                eprintln!("SKIPPED: FontManager unavailable — {}", e);
                return;
            }
        }
    };
}
