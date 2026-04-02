//! 2-D software canvas with safe integer arithmetic and optimised rendering.
//!
//! All public drawing methods clip to canvas bounds. Out-of-bounds writes are
//! silently ignored so callers never need to pre-clip.

// ── Colour ──────────────────────────────────────────────────────────

/// An RGBA colour with 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel (255 = fully opaque).
    pub a: u8,
}

impl Color {
    /// Pure white.
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);
    /// Pure black.
    pub const BLACK: Self = Self::rgba(0, 0, 0, 255);
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    /// Light gray (245).
    pub const GRAY_100: Self = Self::rgba(245, 245, 245, 255);
    /// Medium-light gray (220).
    pub const GRAY_300: Self = Self::rgba(220, 220, 220, 255);
    /// Medium-dark gray (95).
    pub const GRAY_600: Self = Self::rgba(95, 95, 95, 255);
    /// A medium blue.
    pub const BLUE: Self = Self::rgba(70, 120, 220, 255);

    /// Create a colour from RGBA components.
    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to `0x00RRGGBB` (softbuffer format). Alpha is dropped.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

// ── Rect ────────────────────────────────────────────────────────────

/// Axis-aligned rectangle: signed origin, unsigned extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// X coordinate of the top-left corner.
    pub x: i32,
    /// Y coordinate of the top-left corner.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Practical upper bound for dimensions — fits in `i32` after cast.
const MAX_EXTENT: u32 = i32::MAX as u32;

impl Rect {
    /// A zero-sized rect at the origin.
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    /// Create a new rectangle.
    #[inline]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Hit-test with `f32` coordinates (half-open: `[x, x+w)`).
    #[inline]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        let (x0, y0) = (self.x as f64, self.y as f64);
        let (x1, y1) = (x0 + self.width as f64, y0 + self.height as f64);
        let (px, py) = (px as f64, py as f64);
        px >= x0 && px < x1 && py >= y0 && py < y1
    }

    /// Smallest rect enclosing both. Zero-area rects are treated as empty.
    pub fn union(&self, other: &Rect) -> Rect {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let ax1 = self.x as i64 + self.width as i64;
        let ay1 = self.y as i64 + self.height as i64;
        let bx1 = other.x as i64 + other.width as i64;
        let by1 = other.y as i64 + other.height as i64;

        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = ax1.max(bx1).min(i32::MAX as i64) as i32;
        let y1 = ay1.max(by1).min(i32::MAX as i64) as i32;

        Rect::new(x0, y0, (x1 - x0).max(0) as u32, (y1 - y0).max(0) as u32)
    }

    /// Returns `true` if width or height is zero.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Right edge (exclusive) in i64 — avoids overflow.
    #[inline]
    fn right_i64(&self) -> i64 {
        self.x as i64 + self.width as i64
    }

    /// Bottom edge (exclusive) in i64.
    #[inline]
    fn bottom_i64(&self) -> i64 {
        self.y as i64 + self.height as i64
    }
}

// ── Canvas ──────────────────────────────────────────────────────────

/// Software pixel buffer. Pixels are `0x00RRGGBB` in row-major order.
pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
    clip_rect: Option<Rect>,
}

impl Canvas {
    /// Create a new canvas with the given dimensions, filled with black.
    pub fn new(width: u32, height: u32) -> Self {
        let w = width.min(MAX_EXTENT);
        let h = height.min(MAX_EXTENT);
        Self {
            width: w,
            height: h,
            pixels: vec![0u32; w as usize * h as usize],
            clip_rect: None,
        }
    }

    /// Resize, preserving existing rows/columns that fit. New pixels are zero.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let nw = new_width.min(MAX_EXTENT);
        let nh = new_height.min(MAX_EXTENT);
        if nw == self.width && nh == self.height {
            return;
        }

        let (ow, nw_us, nh_us) = (self.width as usize, nw as usize, nh as usize);
        let mut buf = vec![0u32; nw_us * nh_us];
        let copy_rows = (self.height as usize).min(nh_us);
        let copy_cols = ow.min(nw_us);

        for y in 0..copy_rows {
            buf[y * nw_us..y * nw_us + copy_cols]
                .copy_from_slice(&self.pixels[y * ow..y * ow + copy_cols]);
        }

        self.pixels = buf;
        self.width = nw;
        self.height = nh;
        self.clip_rect = None;
    }

    /// Width of the canvas in pixels.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the canvas in pixels.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Access the raw pixel data.
    #[inline]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Fill every pixel.
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.to_u32());
    }

    /// Fill a clipped axis-aligned rectangle.
    ///
    /// Opaque fills use `[u32]::fill` (one call per row) for throughput.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let (x0, y0, x1, y1) = self.clip(&rect);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        if color.a == 255 {
            let c = color.to_u32();
            let w = self.width as usize;
            for y in y0..y1 {
                let s = y * w + x0;
                self.pixels[s..s + (x1 - x0)].fill(c);
            }
        } else if color.a > 0 {
            for y in y0..y1 {
                for x in x0..x1 {
                    self.blend_unchecked(x, y, color);
                }
            }
        }
    }

    /// Bresenham line with per-pixel clipping.
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

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

    /// Filled rounded rectangle via scanline spans (no per-pixel distance
    /// check in the non-corner region).
    pub fn draw_rounded_rect(&mut self, rect: Rect, radius: u32, color: Color) {
        let (cx0, cy0, cx1, cy1) = self.clip(&rect);
        if cx0 >= cx1 || cy0 >= cy1 {
            return;
        }

        let max_r = (rect.width / 2).min(rect.height / 2);
        let r = radius.min(max_r);

        if r == 0 {
            self.fill_rect(rect, color);
            return;
        }

        let left = rect.x as i64;
        let top = rect.y as i64;
        let right = left + rect.width as i64;
        let bottom = top + rect.height as i64;
        let ri = r as i64;
        let r_sq = ri * ri;
        let c32 = color.to_u32();
        let opaque = color.a == 255;
        let w = self.width as usize;

        for y in cy0..cy1 {
            let yi = y as i64;

            let (row_l, row_r) = if yi < top + ri {
                let dy = top + ri - yi;
                let dsq = r_sq - dy * dy;
                if dsq < 0 {
                    continue;
                }
                let dx = (dsq as f64).sqrt() as i64;
                (left + ri - dx, right - ri + dx)
            } else if yi >= bottom - ri {
                let dy = yi - (bottom - ri) + 1;
                let dsq = r_sq - dy * dy;
                if dsq < 0 {
                    continue;
                }
                let dx = (dsq as f64).sqrt() as i64;
                (left + ri - dx, right - ri + dx)
            } else {
                (left, right)
            };

            let sx = (row_l.max(cx0 as i64)) as usize;
            let ex = (row_r.min(cx1 as i64)) as usize;
            if sx >= ex {
                continue;
            }

            if opaque {
                let s = y * w + sx;
                self.pixels[s..s + (ex - sx)].fill(c32);
            } else {
                for x in sx..ex {
                    self.blend_unchecked(x, y, color);
                }
            }
        }
    }

    /// Alpha-composite `src` onto `(x,y)`. Out-of-bounds → no-op.
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, src: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.blend_unchecked(x as usize, y as usize, src);
    }

    #[inline]
    fn blend_unchecked(&mut self, x: usize, y: usize, src: Color) {
        if src.a == 0 {
            return;
        }
        let idx = y * self.width as usize + x;
        if src.a == 255 {
            self.pixels[idx] = src.to_u32();
            return;
        }

        let a = src.a as u32;
        let inv = 255 - a;
        let dst = self.pixels[idx];

        let out_r = div255(src.r as u32 * a + ((dst >> 16) & 0xFF) * inv);
        let out_g = div255(src.g as u32 * a + ((dst >> 8) & 0xFF) * inv);
        let out_b = div255(src.b as u32 * a + (dst & 0xFF) * inv);

        self.pixels[idx] = (out_r << 16) | (out_g << 8) | out_b;
    }

    #[inline]
    fn clip(&self, r: &Rect) -> (usize, usize, usize, usize) {
        let mut x0 = r.x.max(0) as usize;
        let mut y0 = r.y.max(0) as usize;
        let mut x1 = r.right_i64().min(self.width as i64).max(0) as usize;
        let mut y1 = r.bottom_i64().min(self.height as i64).max(0) as usize;

        if let Some(ref cr) = self.clip_rect {
            x0 = x0.max(cr.x.max(0) as usize);
            y0 = y0.max(cr.y.max(0) as usize);
            x1 = x1.min(cr.right_i64().max(0) as usize);
            y1 = y1.min(cr.bottom_i64().max(0) as usize);
        }

        (x0, y0, x1, y1)
    }

    /// Set a clipping rectangle for subsequent drawing operations.
    pub fn set_clip(&mut self, rect: Rect) {
        self.clip_rect = Some(rect);
    }

    /// Clear the clipping rectangle.
    pub fn clear_clip(&mut self) {
        self.clip_rect = None;
    }
}

/// Integer division by 255: `(val + 128) / 255` without true division.
#[inline]
fn div255(val: u32) -> u32 {
    let t = val + 128;
    (t + (t >> 8)) >> 8
}
