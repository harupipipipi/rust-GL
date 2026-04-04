//! Font loading, measurement, word-wrapping, and text rasterisation.
//!
//! Uses `font-kit` for system font discovery and `fontdue` for glyph
//! rasterisation. Line metrics (ascent / descent) are read from the font
//! file via [`fontdue::Font::horizontal_line_metrics`].

use crate::canvas::{Canvas, Color, Rect};
use font_kit::{
    family_name::FamilyName,
    handle::Handle,
    properties::Properties,
    source::SystemSource,
};
use fontdue::{Font as FontdueFont, FontSettings};
use thiserror::Error;

/// Errors that can occur while loading fonts.
#[derive(Debug, Error)]
pub enum TextError {
    /// A system font could not be loaded.
    #[error("failed to load system font: {0}")]
    SystemFont(String),
    /// The raw font bytes could not be parsed.
    #[error("failed to parse font bytes")]
    ParseFont,
}

/// Manages a single font and exposes measurement / drawing helpers.
pub struct FontManager {
    font: FontdueFont,
}

impl FontManager {
    /// Load the best available system font with fallback.
    pub fn new() -> Result<Self, TextError> {
        let source = SystemSource::new();
        let props = Properties::new();

        let families: &[&[FamilyName]] = &[
            &[FamilyName::SansSerif],
            &[FamilyName::Title("Noto Sans CJK JP".into())],
            &[FamilyName::Title("Hiragino Sans".into())],
            &[FamilyName::Title("Yu Gothic".into())],
            &[FamilyName::Monospace],
        ];

        let mut last_err = String::new();
        for family in families {
            match source.select_best_match(family, &props) {
                Ok(handle) => {
                    let bytes = load_handle_bytes(handle)?;
                    let font = FontdueFont::from_bytes(bytes, FontSettings::default())
                        .map_err(|_| TextError::ParseFont)?;
                    return Ok(Self { font });
                }
                Err(e) => {
                    last_err = e.to_string();
                }
            }
        }

        Err(TextError::SystemFont(format!(
            "no suitable font found (last error: {last_err})"
        )))
    }

    // ── Metrics ──────────────────────────────────────────────────

    /// Measure the width and height of a single-line string.
    pub fn measure_text(&self, text: &str, px: f32) -> (f32, f32) {
        let mut width: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for ch in text.chars() {
            let m = self.font.metrics(ch, px);
            width += m.advance_width;
            max_h = max_h.max(m.height as f32);
        }
        (width, max_h.max(px))
    }

    /// Line height from font metrics, fallback `px * 1.3`.
    pub fn line_height(&self, px: f32) -> f32 {
        self.font
            .horizontal_line_metrics(px)
            .map(|lm| lm.new_line_size)
            .unwrap_or(px * 1.3)
    }

    /// Ascent from font metrics, fallback `px * 0.8`.
    fn ascent(&self, px: f32) -> f32 {
        self.font
            .horizontal_line_metrics(px)
            .map(|lm| lm.ascent)
            .unwrap_or(px * 0.8)
    }

    // ── Word wrap ────────────────────────────────────────────────

    /// Break `text` into lines that fit within `max_width` pixels.
    ///
    /// English text breaks at spaces; CJK allows a break before every char.
    pub fn wrap_text(&self, text: &str, max_width: f32, px: f32) -> Vec<String> {
        if max_width <= 0.0 {
            return vec![text.to_string()];
        }

        let mut lines: Vec<String> = Vec::new();
        for hard_line in text.split('\n') {
            let wrapped = self.wrap_hard_line(hard_line, max_width, px);
            if wrapped.is_empty() {
                lines.push(String::new());
            } else {
                lines.extend(wrapped);
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    fn wrap_hard_line(&self, line: &str, max_width: f32, px: f32) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut cur_w: f32 = 0.0;
        let mut word = String::new();
        let mut word_w: f32 = 0.0;

        for ch in line.chars() {
            let cw = self.font.metrics(ch, px).advance_width;

            if is_cjk(ch) || ch == ' ' {
                flush_word(&mut current, &mut cur_w, &mut word, &mut word_w,
                           max_width, &mut result);
                if cur_w + cw > max_width && !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                    cur_w = 0.0;
                }
                current.push(ch);
                cur_w += cw;
            } else {
                word.push(ch);
                word_w += cw;
            }
        }

        flush_word(&mut current, &mut cur_w, &mut word, &mut word_w,
                   max_width, &mut result);
        if !current.is_empty() {
            result.push(current);
        }
        result
    }

    // ── Drawing ──────────────────────────────────────────────────

    /// Draw text with top-left at `(x, y)`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: i32,
        y: i32,
        max_width: Option<u32>,
        px: f32,
        color: Color,
    ) {
        let lines = match max_width {
            Some(w) => self.wrap_text(text, w as f32, px),
            None => vec![text.to_string()],
        };

        let lh = self.line_height(px).round() as i32;
        let ascent = self.ascent(px).round() as i32;

        for (li, line) in lines.iter().enumerate() {
            let mut cx = x;
            let baseline_y = y + li as i32 * lh + ascent;

            for ch in line.chars() {
                let (metrics, bitmap) = self.font.rasterize(ch, px);
                let glyph_top = baseline_y - metrics.height as i32 - metrics.ymin;

                for gy in 0..metrics.height {
                    for gx in 0..metrics.width {
                        let alpha = bitmap[gy * metrics.width + gx];
                        if alpha == 0 { continue; }
                        canvas.blend_pixel(
                            cx + metrics.xmin + gx as i32,
                            glyph_top + gy as i32,
                            Color::rgba(color.r, color.g, color.b, alpha),
                        );
                    }
                }
                cx += metrics.advance_width.round() as i32;
            }
        }
    }

    /// Convenience: draw text clipped to `rect`.
    pub fn draw_text_in_rect(
        &self,
        canvas: &mut Canvas,
        text: &str,
        rect: Rect,
        px: f32,
        color: Color,
    ) {
        let clip = match canvas.clip_rect() {
            Some(current) => current.intersect(&rect),
            None => rect,
        };
        let previous_clip = canvas.replace_clip_rect(Some(clip));
        self.draw_text(canvas, text, rect.x, rect.y, Some(rect.width), px, color);
        canvas.replace_clip_rect(previous_clip);
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn load_handle_bytes(handle: Handle) -> Result<Vec<u8>, TextError> {
    match handle {
        Handle::Path { path, .. } => {
            std::fs::read(path).map_err(|e| TextError::SystemFont(e.to_string()))
        }
        Handle::Memory { bytes, .. } => Ok(bytes.to_vec()),
    }
}

fn flush_word(
    current: &mut String,
    cur_w: &mut f32,
    word: &mut String,
    word_w: &mut f32,
    max_width: f32,
    result: &mut Vec<String>,
) {
    if word.is_empty() { return; }

    if *cur_w + *word_w > max_width && !current.is_empty() {
        result.push(std::mem::take(current));
        *cur_w = 0.0;
    }
    current.push_str(word);
    *cur_w += *word_w;
    word.clear();
    *word_w = 0.0;
}

fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'
      | '\u{3400}'..='\u{4DBF}'
      | '\u{3000}'..='\u{303F}'
      | '\u{3040}'..='\u{309F}'
      | '\u{30A0}'..='\u{30FF}'
      | '\u{FF00}'..='\u{FFEF}'
      | '\u{20000}'..='\u{2A6DF}'
    )
}
