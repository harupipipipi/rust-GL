use crate::canvas::Rect;

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    #[inline]
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    #[inline]
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoxConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl BoxConstraints {
    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    pub fn loose(width: f32, height: f32) -> Self {
        Self {
            min_width: 0.0,
            max_width: width,
            min_height: 0.0,
            max_height: height,
        }
    }

    pub fn constrain(&self, mut size: Size) -> Size {
        size.width = size.width.clamp(self.min_width, self.max_width);
        size.height = size.height.clamp(self.min_height, self.max_height);
        size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct LayoutStyle {
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub gap: f32,
    pub direction: LayoutDirection,
    pub wrap_text: bool,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            padding: EdgeInsets::all(0.0),
            margin: EdgeInsets::all(0.0),
            gap: 8.0,
            direction: LayoutDirection::Vertical,
            wrap_text: true,
        }
    }
}

/// Helper: convert f32 to u32 with rounding, clamping negative to 0.
#[inline]
pub fn f32_to_u32(v: f32) -> u32 {
    v.round().max(0.0) as u32
}

/// Helper: convert f32 to i32 with rounding.
#[inline]
pub fn f32_to_i32(v: f32) -> i32 {
    v.round() as i32
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub widget_id: u64,
    pub bounds: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(widget_id: u64, x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            widget_id,
            bounds: Rect::new(x, y, width, height),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }

    pub fn find_by_id(&self, id: u64) -> Option<&LayoutNode> {
        if self.widget_id == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find_by_id(id))
    }
}
