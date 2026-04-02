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
    pub const GRAY_100: Self = Self::rgba(245, 245, 245, 255);
    pub const GRAY_300: Self = Self::rgba(220, 220, 220, 255);
    pub const GRAY_600: Self = Self::rgba(95, 95, 95, 255);
    pub const BLUE: Self = Self::rgba(70, 120, 220, 255);

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        let x2 = self.x + self.width as i32;
        let y2 = self.y + self.height as i32;
        (px as i32) >= self.x && (px as i32) < x2 && (py as i32) >= self.y && (py as i32) < y2
    }
}

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
            pixels: vec![0; (width * height) as usize],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.pixels.resize((width * height) as usize, 0);
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.to_u32());
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0) as u32;
        let y0 = rect.y.max(0) as u32;
        let x1 = (rect.x + rect.width as i32).min(self.width as i32).max(0) as u32;
        let y1 = (rect.y + rect.height as i32).min(self.height as i32).max(0) as u32;

        for y in y0..y1 {
            for x in x0..x1 {
                self.blend_pixel(x as i32, y as i32, color);
            }
        }
    }

    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
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

    pub fn draw_rounded_rect(&mut self, rect: Rect, radius: u32, color: Color) {
        let radius = radius.min(rect.width / 2).min(rect.height / 2) as i32;
        let cx_left = rect.x + radius;
        let cx_right = rect.x + rect.width as i32 - radius - 1;
        let cy_top = rect.y + radius;
        let cy_bottom = rect.y + rect.height as i32 - radius - 1;

        for y in rect.y..(rect.y + rect.height as i32) {
            for x in rect.x..(rect.x + rect.width as i32) {
                let mut inside = true;

                if x < cx_left && y < cy_top {
                    let dx = x - cx_left;
                    let dy = y - cy_top;
                    inside = dx * dx + dy * dy <= radius * radius;
                } else if x > cx_right && y < cy_top {
                    let dx = x - cx_right;
                    let dy = y - cy_top;
                    inside = dx * dx + dy * dy <= radius * radius;
                } else if x < cx_left && y > cy_bottom {
                    let dx = x - cx_left;
                    let dy = y - cy_bottom;
                    inside = dx * dx + dy * dy <= radius * radius;
                } else if x > cx_right && y > cy_bottom {
                    let dx = x - cx_right;
                    let dy = y - cy_bottom;
                    inside = dx * dx + dy * dy <= radius * radius;
                }

                if inside {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, src: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = (y as u32 * self.width + x as u32) as usize;
        let dst = self.pixels[idx];

        if src.a == 255 {
            self.pixels[idx] = src.to_u32();
            return;
        }

        let dst_r = ((dst >> 16) & 0xff) as f32;
        let dst_g = ((dst >> 8) & 0xff) as f32;
        let dst_b = (dst & 0xff) as f32;

        let a = src.a as f32 / 255.0;
        let inv = 1.0 - a;
        let out_r = src.r as f32 * a + dst_r * inv;
        let out_g = src.g as f32 * a + dst_g * inv;
        let out_b = src.b as f32 * a + dst_b * inv;

        self.pixels[idx] = ((out_r as u32) << 16) | ((out_g as u32) << 8) | (out_b as u32);
    }
}
