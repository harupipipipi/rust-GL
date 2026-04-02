/// 2-D software canvas with safe integer arithmetic and optimised alpha blending.

// ── Colour ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);
    pub const BLACK: Self = Self::rgba(0, 0, 0, 255);
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const GRAY_100: Self = Self::rgba(245, 245, 245, 255);
    pub const GRAY_300: Self = Self::rgba(220, 220, 220, 255);
    pub const GRAY_600: Self = Self::rgba(95, 95, 95, 255);
    pub const BLUE: Self = Self::rgba(70, 120, 220, 255);

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to the 0x00RRGGBB format that softbuffer expects.
    /// Alpha is intentionally dropped because the framebuffer has no alpha
    /// channel. Use [Canvas::blend_pixel] for compositing.
    #[inline]
    pub fn to_u32(self) -> u32 {
        (u32::from(self.r) << 16) | (u32::from(self.g) << 8) | u32::from(self.b)
    }
}

// ── Rect ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Hit-test using f32 coordinates. Comparison stays in f32 to avoid
    /// truncation artefacts at negative / fractional coordinates.
    #[inline]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        let x0 = self.x as f32;
        let y0 = self.y as f32;
        let x1 = x0 + self.width as f32;
        let y1 = y0 + self.height as f32;
        px >= x0 && px < x1 && py >= y0 && py < y1
    }

    pub fn union(&self, other: &Rect) -> Rect {
        if self.width == 0 && self.height == 0 {
            return *other;
        }
        if other.width == 0 && other.height == 0 {
            return *self;
        }

        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = self
            .x
            .saturating_add(self.width as i32)
            .max(other.x.saturating_add(other.width as i32));
        let y1 = self
            .y
            .saturating_add(self.height as i32)
            .max(other.y.saturating_add(other.height as i32));
        Rect::new(x0, y0, (x1 - x0).max(0) as u32, (y1 - y0).max(0) as u32)
    }

    /// True when the rect has no area.
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

// ── Canvas ──────────────────────────────────────────────────────────

pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width as usize) * (height as usize)],
        }
    }

    /// Resize the canvas. Existing rows that still fit are preserved;
    /// new/shortened rows are filled with zeroes.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width == self.width && new_height == self.height {
            return;
        }

        let nw = new_width as usize;
        let nh = new_height as usize;
        let ow = self.width as usize;
        let oh = self.height as usize;

        let mut buf = vec![0u32; nw * nh];
        let copy_rows = oh.min(nh);
        let copy_cols = ow.min(nw);

        for y in 0..copy_rows {
            let src_start = y * ow;
            let dst_start = y * nw;
            buf[dst_start..dst_start + copy_cols]
                .copy_from_slice(&self.pixels[src_start..src_start + copy_cols]);
        }

        self.pixels = buf;
        self.width = new_width;
        self.height = new_height;
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.to_u32());
    }

    /// Fill an axis-aligned rectangle, clipping to canvas bounds.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let (x0, y0, x1, y1) = self.clipped_bounds(&rect);
        if color.a == 255 {
            let c = color.to_u32();
            for y in y0..y1 {
                let row_start = y * (self.width as usize);
                for x in x0..x1 {
                    self.pixels[row_start + x] = c;
                }
            }
        } else if color.a > 0 {
            for y in y0..y1 {
                for x in x0..x1 {
                    self.blend_pixel_unchecked(x, y, color);
                }
            }
        }
    }

    /// Bresenham line. Clips per-pixel.
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err: i32 = dx + dy;

        loop {
            self.blend_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a filled rounded rectangle with radius in pixels.
    pub fn draw_rounded_rect(&mut self, rect: Rect, radius: u32, color: Color) {
        let (x0, y0, x1, y1) = self.clipped_bounds(&rect);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let max_r = (rect.width / 2).min(rect.height / 2);
        let r = radius.min(max_r) as i32;

        if r <= 0 {
            self.fill_rect(rect, color);
            return;
        }

        let left = rect.x;
        let top = rect.y;
        let right = rect.x + rect.width as i32 - 1;
        let bottom = rect.y + rect.height as i32 - 1;

        let corner_left = left + r;
        let corner_right = right - r;
        let corner_top = top + r;
        let corner_bottom = bottom - r;

        let r_sq = r * r;
        let c32 = color.to_u32();
        let opaque = color.a == 255;

        for y in y0..y1 {
            let yi = y as i32;
            for x in x0..x1 {
                let xi = x as i32;

                let in_corner = (xi < corner_left && yi < corner_top)
                    || (xi > corner_right && yi < corner_top)
                    || (xi < corner_left && yi > corner_bottom)
                    || (xi > corner_right && yi > corner_bottom);

                if in_corner {
                    let dx = if xi < corner_left {
                        corner_left - xi
                    } else {
                        xi - corner_right
                    };
                    let dy = if yi < corner_top {
                        corner_top - yi
                    } else {
                        yi - corner_bottom
                    };

                    if dx * dx + dy * dy > r_sq {
                        continue;
                    }
                }

                if opaque {
                    self.pixels[y * (self.width as usize) + x] = c32;
                } else {
                    self.blend_pixel_unchecked(x, y, color);
                }
            }
        }
    }

    /// Alpha-composite src onto the pixel at (x, y) using integer arithmetic only.
    /// Out-of-bounds writes are silently ignored.
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, src: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.blend_pixel_unchecked(x as usize, y as usize, src);
    }

    /// Same as blend_pixel but without bounds checking.
    /// Caller must guarantee `x < self.width` and `y < self.height`.
    #[inline]
    fn blend_pixel_unchecked(&mut self, x: usize, y: usize, src: Color) {
        if src.a == 0 {
            return;
        }

        let idx = y * (self.width as usize) + x;

        if src.a == 255 {
            self.pixels[idx] = src.to_u32();
            return;
        }

        let dst = self.pixels[idx];
        let a = u32::from(src.a);
        let inv = 255 - a;

        let dst_r = (dst >> 16) & 0xFF;
        let dst_g = (dst >> 8) & 0xFF;
        let dst_b = dst & 0xFF;

        let out_r = (u32::from(src.r) * a + dst_r * inv + 128) / 255;
        let out_g = (u32::from(src.g) * a + dst_g * inv + 128) / 255;
        let out_b = (u32::from(src.b) * a + dst_b * inv + 128) / 255;

        self.pixels[idx] = (out_r << 16) | (out_g << 8) | out_b;
    }

    #[inline]
    fn clipped_bounds(&self, rect: &Rect) -> (usize, usize, usize, usize) {
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let x1 = rect
            .x
            .saturating_add(rect.width as i32)
            .clamp(0, self.width as i32) as usize;
        let y1 = rect
            .y
            .saturating_add(rect.height as i32)
            .clamp(0, self.height as i32) as usize;
        (x0, y0, x1, y1)
    }
}
