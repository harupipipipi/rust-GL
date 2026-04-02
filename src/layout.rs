//! Layout primitives: constraints, sizes, insets, and the layout tree.

use crate::canvas::Rect;
use crate::widgets::WidgetId;

/// A two-dimensional size.
#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

/// Padding / margin expressed as four independent sides.
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeInsets {
    /// Top inset.
    pub top: f32,
    /// Right inset.
    pub right: f32,
    /// Bottom inset.
    pub bottom: f32,
    /// Left inset.
    pub left: f32,
}

impl EdgeInsets {
    /// Create insets where all four sides have the same value.
    pub fn all(v: f32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }

    /// Sum of left and right.
    #[inline]
    pub fn horizontal(&self) -> f32 { self.left + self.right }

    /// Sum of top and bottom.
    #[inline]
    pub fn vertical(&self) -> f32 { self.top + self.bottom }
}

/// Constraints passed down the widget tree during layout.
#[derive(Debug, Clone, Copy)]
pub struct BoxConstraints {
    /// Minimum allowed width.
    pub min_width: f32,
    /// Maximum allowed width.
    pub max_width: f32,
    /// Minimum allowed height.
    pub min_height: f32,
    /// Maximum allowed height.
    pub max_height: f32,
}

impl BoxConstraints {
    /// Both min and max are the same.
    pub fn tight(width: f32, height: f32) -> Self {
        Self { min_width: width, max_width: width, min_height: height, max_height: height }
    }

    /// Min is zero, max is the given size.
    pub fn loose(width: f32, height: f32) -> Self {
        Self { min_width: 0.0, max_width: width, min_height: 0.0, max_height: height }
    }

    /// Clamp `size` into these constraints.
    #[must_use]
    pub fn constrain(&self, mut size: Size) -> Size {
        size.width = size.width.clamp(self.min_width, self.max_width);
        size.height = size.height.clamp(self.min_height, self.max_height);
        size
    }
}

/// Flow direction for container children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    /// Stack children top-to-bottom.
    Vertical,
    /// Stack children left-to-right.
    Horizontal,
}

/// Style hints consumed during layout.
#[derive(Debug, Clone)]
pub struct LayoutStyle {
    /// Inner padding.
    pub padding: EdgeInsets,
    /// Outer margin.
    pub margin: EdgeInsets,
    /// Gap between children.
    pub gap: f32,
    /// Flow direction.
    pub direction: LayoutDirection,
    /// Whether to wrap text.
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

/// Round `f32` to `u32`, clamping negatives to 0.
#[inline]
pub fn f32_to_u32(v: f32) -> u32 {
    v.round().max(0.0) as u32
}

/// Round `f32` to `i32`.
#[inline]
pub fn f32_to_i32(v: f32) -> i32 {
    v.round() as i32
}

/// A node in the computed layout tree.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// The widget this node corresponds to.
    pub widget_id: WidgetId,
    /// Screen-space bounding rectangle.
    pub bounds: Rect,
    /// Child layout nodes.
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    /// Create a leaf node at the given position and size.
    pub fn new(widget_id: WidgetId, x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            widget_id,
            bounds: Rect::new(x, y, width, height),
            children: Vec::new(),
        }
    }

    /// Append a child node.
    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }

    /// Depth-first search for a node with the given id.
    pub fn find_by_id(&self, id: WidgetId) -> Option<&LayoutNode> {
        if self.widget_id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find_by_id(id))
    }
}
