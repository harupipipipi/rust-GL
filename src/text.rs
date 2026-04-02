use crate::canvas::{Canvas, Color, Rect};
use font_kit::{
    family_name::FamilyName,
    handle::Handle,
    properties::Properties,
    source::SystemSource,
};
use fontdue::{Font as FontdueFont, FontSettings};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextError {
    #[error("failed to load system font: {0}")]
    SystemFont(String),
    #[error("failed to parse font bytes")]
    ParseFont,
}

pub struct FontManager {
    font: FontdueFont,
}

impl FontManager {
    pub fn new() -> Result<Self, TextError> {
        let source = SystemSource::new();
        let handle = source
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .map_err(|e| TextError::SystemFont(e.to_string()))?;
        let bytes = load_handle_bytes(handle)?;
        let font =
            FontdueFont::from_bytes(bytes, FontSettings::default()).map_err(|_| TextError::ParseFont)?;
        Ok(Self { font })
    }

    pub fn measure_text(&self, text: &str, px: f32) -> (f32, f32) {
        let mut width: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for ch in text.chars() {
            let metrics = self.font.metrics(ch, px);
            width += metrics.advance_width;
            max_h = max_h.max(metrics.height as f32);
        }
        (width, max_h.max(px))
    }

    pub fn line_height(&self, px: f32) -> f32 {
        px * 1.3
    }

    pub fn wrap_text(&self, text: &str, max_width: f32, px: f32) -> Vec<String> {
        if max_width <= 0.0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current = String::new();
        let mut current_width = 0.0;

        for ch in text.chars() {
            if ch == '\n' {
                lines.push(std::mem::take(&mut current));
                current_width = 0.0;
                continue;
            }

            let w = self.font.metrics(ch, px).advance_width;
            if current_width + w > max_width && !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current_width = 0.0;
            }
            current.push(ch);
            current_width += w;
        }

        if !current.is_empty() {
            lines.push(current);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    /// Draw text at (x, y) which represents the top-left of the first
    /// line. fontdue metrics use a top-left coordinate system where ymin
    /// is the offset from the top of the glyph bounding box.
    /// We use ascent ≈ 0.8 * px to place glyphs on a visual baseline so
    /// that descenders (g, y, p …) hang below.
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
        let lines = if let Some(w) = max_width {
            self.wrap_text(text, w as f32, px)
        } else {
            vec![text.to_string()]
        };

        let line_height = self.line_height(px).round() as i32;
        // Approximate ascent — distance from top of em-box to baseline.
        let ascent = (px * 0.8).round() as i32;

        for (line_index, line) in lines.iter().enumerate() {
            let mut cursor_x = x;
            let baseline_y = y + (line_index as i32) * line_height + ascent;

            for ch in line.chars() {
                let (metrics, bitmap) = self.font.rasterize(ch, px);
                // metrics.ymin in fontdue = offset from bottom of bbox
                // glyph draw origin (top-left of bbox) relative to baseline:
                //   glyph_top = baseline_y - (metrics.height as i32 - (-metrics.ymin))
                //             = baseline_y - metrics.height as i32 - metrics.ymin
                // Simplified: fontdue ymin is bottom-relative.
                let glyph_top = baseline_y - (metrics.height as i32) + metrics.ymin;

                for gy in 0..metrics.height {
                    for gx in 0..metrics.width {
                        let alpha = bitmap[gy * metrics.width + gx];
                        if alpha == 0 {
                            continue;
                        }
                        let blended = Color::rgba(color.r, color.g, color.b, alpha);
                        canvas.blend_pixel(
                            cursor_x + metrics.xmin + gx as i32,
                            glyph_top + gy as i32,
                            blended,
                        );
                    }
                }
                cursor_x += metrics.advance_width.round() as i32;
            }
        }
    }

    pub fn draw_text_in_rect(&self, canvas: &mut Canvas, text: &str, rect: Rect, px: f32, color: Color) {
        self.draw_text(canvas, text, rect.x, rect.y, Some(rect.width), px, color);
    }
}

fn load_handle_bytes(handle: Handle) -> Result<Vec<u8>, TextError> {
    match handle {
        Handle::Path { path, .. } => std::fs::read(path).map_err(|e| TextError::SystemFont(e.to_string())),
        Handle::Memory { bytes, .. } => Ok(bytes.to_vec()),
    }
}
