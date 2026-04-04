//! Font loading, measurement, word-wrapping, and text rasterisation.
//!
//! Uses `font-kit` for system font discovery and `fontdue` for glyph
//! rasterisation. Line metrics (ascent / descent) are read from the primary
//! font file via [`fontdue::Font::horizontal_line_metrics`].

use crate::canvas::{Canvas, Color, Rect};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use fontdue::{Font as FontdueFont, FontSettings};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};
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

/// Manages a stack of fonts and resolves glyph fallback per character.
pub struct FontManager {
    fonts: Vec<LoadedFont>,
    replacement: FallbackGlyph,
}

#[derive(Debug)]
struct LoadedFont {
    font: FontdueFont,
}

#[derive(Debug, Clone, Copy)]
struct FallbackGlyph {
    font_index: usize,
    ch: char,
}

#[derive(Debug, Clone, Copy)]
struct PositionedGlyph {
    ch: char,
    font_index: usize,
    x: i32,
}

impl FontManager {
    /// Load a set of system fonts and build per-glyph fallback.
    pub fn new() -> Result<Self, TextError> {
        let source = SystemSource::new();
        let props = Properties::new();
        let mut fonts = Vec::new();
        let mut seen = HashSet::new();
        let mut last_err = String::new();

        for family in family_candidates() {
            match source.select_best_match(&family, &props) {
                Ok(handle) => match load_font(handle, &mut seen) {
                    Ok(Some(font)) => fonts.push(font),
                    Ok(None) => {}
                    Err(e) => last_err = e.to_string(),
                },
                Err(e) if last_err.is_empty() => {
                    last_err = e.to_string();
                }
                Err(_) => {}
            }
        }

        if fonts.is_empty() {
            return Err(TextError::SystemFont(format!(
                "no suitable font found (last error: {last_err})"
            )));
        }

        let replacement = find_replacement(&fonts).ok_or_else(|| {
            TextError::SystemFont("loaded fonts, but none provided a replacement glyph".into())
        })?;

        Ok(Self { fonts, replacement })
    }

    // ── Metrics ──────────────────────────────────────────────────

    /// Measure the width and height of a single-line string.
    pub fn measure_text(&self, text: &str, px: f32) -> (f32, f32) {
        let mut width: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for ch in text.chars() {
            let glyph = self.resolve_glyph(ch);
            let m = self.font(glyph.font_index).metrics(glyph.ch, px);
            width += m.advance_width;
            max_h = max_h.max(m.height as f32);
        }
        (width, max_h.max(px))
    }

    /// Line height from font metrics, fallback `px * 1.3`.
    pub fn line_height(&self, px: f32) -> f32 {
        self.primary_font()
            .horizontal_line_metrics(px)
            .map(|lm| lm.new_line_size)
            .unwrap_or(px * 1.3)
    }

    /// Ascent from font metrics, fallback `px * 0.8`.
    fn ascent(&self, px: f32) -> f32 {
        self.primary_font()
            .horizontal_line_metrics(px)
            .map(|lm| lm.ascent)
            .unwrap_or(px * 0.8)
    }

    /// Returns true when any loaded font can render the character directly.
    pub fn has_display_glyph(&self, ch: char) -> bool {
        self.find_font_index_for_char(ch).is_some()
    }

    /// Convert floating-point advances into stable pixel-aligned glyph origins.
    fn layout_glyphs(&self, text: &str, px: f32) -> Vec<PositionedGlyph> {
        let mut glyphs = Vec::with_capacity(text.chars().count());
        let mut pen_x: f32 = 0.0;

        for ch in text.chars() {
            let glyph = self.resolve_glyph(ch);
            glyphs.push(PositionedGlyph {
                ch: glyph.ch,
                font_index: glyph.font_index,
                x: pen_x.round() as i32,
            });
            pen_x += self
                .font(glyph.font_index)
                .metrics(glyph.ch, px)
                .advance_width;
        }

        glyphs
    }

    /// Pixel-aligned width using the same accumulation rule as rasterised text.
    pub(crate) fn aligned_text_width(&self, text: &str, px: f32) -> i32 {
        let mut pen_x: f32 = 0.0;
        for ch in text.chars() {
            let glyph = self.resolve_glyph(ch);
            pen_x += self
                .font(glyph.font_index)
                .metrics(glyph.ch, px)
                .advance_width;
        }
        pen_x.round() as i32
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
            let glyph = self.resolve_glyph(ch);
            let cw = self
                .font(glyph.font_index)
                .metrics(glyph.ch, px)
                .advance_width;

            if is_cjk(ch) || ch == ' ' {
                flush_word(
                    &mut current,
                    &mut cur_w,
                    &mut word,
                    &mut word_w,
                    max_width,
                    &mut result,
                );
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

        flush_word(
            &mut current,
            &mut cur_w,
            &mut word,
            &mut word_w,
            max_width,
            &mut result,
        );
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
            let baseline_y = y + li as i32 * lh + ascent;

            for glyph in self.layout_glyphs(line, px) {
                let (metrics, bitmap) = self.font(glyph.font_index).rasterize(glyph.ch, px);
                let glyph_top = baseline_y - metrics.height as i32 - metrics.ymin;

                for gy in 0..metrics.height {
                    for gx in 0..metrics.width {
                        let alpha = bitmap[gy * metrics.width + gx];
                        if alpha == 0 {
                            continue;
                        }
                        canvas.blend_pixel(
                            x + glyph.x + metrics.xmin + gx as i32,
                            glyph_top + gy as i32,
                            Color::rgba(color.r, color.g, color.b, alpha),
                        );
                    }
                }
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

    fn primary_font(&self) -> &FontdueFont {
        self.font(0)
    }

    fn font(&self, index: usize) -> &FontdueFont {
        &self.fonts[index].font
    }

    fn find_font_index_for_char(&self, ch: char) -> Option<usize> {
        self.fonts.iter().position(|font| font.font.has_glyph(ch))
    }

    fn resolve_glyph(&self, ch: char) -> FallbackGlyph {
        self.find_font_index_for_char(ch)
            .map(|font_index| FallbackGlyph { font_index, ch })
            .unwrap_or(self.replacement)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn family_candidates() -> Vec<Vec<FamilyName>> {
    vec![
        vec![FamilyName::Title("Noto Sans CJK JP".into())],
        vec![FamilyName::Title("Hiragino Sans".into())],
        vec![FamilyName::Title("Yu Gothic".into())],
        vec![FamilyName::Title("Meiryo".into())],
        vec![FamilyName::Title("Segoe UI".into())],
        vec![FamilyName::Title("Arial Unicode MS".into())],
        vec![FamilyName::Title("Noto Sans Symbols 2".into())],
        vec![FamilyName::Title("Segoe UI Symbol".into())],
        vec![FamilyName::Title("Apple Symbols".into())],
        vec![FamilyName::Title("Apple Color Emoji".into())],
        vec![FamilyName::Title("Segoe UI Emoji".into())],
        vec![FamilyName::Title("Noto Color Emoji".into())],
        vec![FamilyName::SansSerif],
        vec![FamilyName::Serif],
        vec![FamilyName::Monospace],
    ]
}

fn load_font(handle: Handle, seen: &mut HashSet<u64>) -> Result<Option<LoadedFont>, TextError> {
    let bytes = load_handle_bytes(handle)?;
    let fingerprint = hash_bytes(&bytes);
    if !seen.insert(fingerprint) {
        return Ok(None);
    }

    let font = FontdueFont::from_bytes(bytes, FontSettings::default())
        .map_err(|_| TextError::ParseFont)?;
    Ok(Some(LoadedFont { font }))
}

fn load_handle_bytes(handle: Handle) -> Result<Vec<u8>, TextError> {
    match handle {
        Handle::Path { path, .. } => {
            std::fs::read(path).map_err(|e| TextError::SystemFont(e.to_string()))
        }
        Handle::Memory { bytes, .. } => Ok(bytes.to_vec()),
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn find_replacement(fonts: &[LoadedFont]) -> Option<FallbackGlyph> {
    for ch in ['\u{FFFD}', '?', ' '] {
        if let Some(font_index) = fonts.iter().position(|font| font.font.has_glyph(ch)) {
            return Some(FallbackGlyph { font_index, ch });
        }
    }
    None
}

fn flush_word(
    current: &mut String,
    cur_w: &mut f32,
    word: &mut String,
    word_w: &mut f32,
    max_width: f32,
    result: &mut Vec<String>,
) {
    if word.is_empty() {
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::FontManager;

    #[test]
    fn glyph_layout_matches_measured_width_when_rounded() {
        let fm = match FontManager::new() {
            Ok(fm) => fm,
            Err(e) => {
                eprintln!("skipping test: {e}");
                return;
            }
        };

        let samples = [
            "Hello, world!",
            "iiiiiiiiii",
            "純Rust 2D UIライブラリ",
            "WAVE wave",
        ];

        for sample in samples {
            let glyphs = fm.layout_glyphs(sample, 18.0);
            let (width, _) = fm.measure_text(sample, 18.0);
            let actual_end = fm.aligned_text_width(sample, 18.0);

            assert_eq!(
                actual_end,
                width.round() as i32,
                "glyph placement drifted for sample: {sample}"
            );

            let mut prefix = String::new();
            for glyph in glyphs {
                assert_eq!(
                    glyph.x,
                    fm.aligned_text_width(&prefix, 18.0),
                    "glyph origin should match the aligned width of its prefix for sample: {sample}"
                );
                prefix.push(glyph.ch);
            }
        }
    }

    #[test]
    fn unsupported_characters_fall_back_to_visible_replacement() {
        let fm = match FontManager::new() {
            Ok(fm) => fm,
            Err(e) => {
                eprintln!("skipping test: {e}");
                return;
            }
        };

        let missing = '\u{10FFFF}';
        let (width, height) = fm.measure_text(&missing.to_string(), 18.0);
        assert!(
            width > 0.0,
            "missing glyphs should still consume fallback width"
        );
        assert!(
            height > 0.0,
            "missing glyphs should still consume fallback height"
        );
    }

    #[test]
    fn manager_reports_ascii_as_displayable() {
        let fm = match FontManager::new() {
            Ok(fm) => fm,
            Err(e) => {
                eprintln!("skipping test: {e}");
                return;
            }
        };

        for ch in ['A', 'z', '0', '?'] {
            assert!(fm.has_display_glyph(ch), "expected a loaded font for {ch}");
        }
    }
}
