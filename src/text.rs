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
        let font = FontdueFont::from_bytes(bytes, FontSettings::default())
            .map_err(|_| TextError::ParseFont)?;
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

        let line_height = (px * 1.3) as i32;
        for (line_index, line) in lines.iter().enumerate() {
            let mut cursor_x = x;
            let cursor_y = y + line_index as i32 * line_height;

            for ch in line.chars() {
                let (metrics, bitmap) = self.font.rasterize(ch, px);
                for gy in 0..metrics.height {
                    for gx in 0..metrics.width {
                        let alpha = bitmap[gy * metrics.width + gx];
                        if alpha == 0 {
                            continue;
                        }
                        let blended = Color::rgba(color.r, color.g, color.b, alpha);
                        canvas.blend_pixel(
                            cursor_x + metrics.xmin + gx as i32,
                            cursor_y + metrics.ymin + gy as i32 + px as i32,
                            blended,
                        );
                    }
                }
                cursor_x += metrics.advance_width as i32;
            }
        }
    }

    pub fn draw_text_in_rect(&self, canvas: &mut Canvas, text: &str, rect: Rect, px: f32, color: Color) {
        self.draw_text(
            canvas,
            text,
            rect.x,
            rect.y,
            Some(rect.width),
            px,
            color,
        );
    }
}

fn load_handle_bytes(handle: Handle) -> Result<Vec<u8>, TextError> {
    match handle {
        Handle::Path { path, .. } => std::fs::read(path).map_err(|e| TextError::SystemFont(e.to_string())),
        Handle::Memory { bytes, .. } => Ok(bytes.to_vec()),
    }
}

