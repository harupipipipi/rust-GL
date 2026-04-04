//! Horizontal or vertical divider line.
//!
//! A `Divider` draws a thin rectangle in the given direction and does not
//! respond to any events.

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    layout::{f32_to_i32, f32_to_u32, BoxConstraints, EdgeInsets, LayoutDirection, LayoutNode},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// A simple separator line.
pub struct Divider {
    id: WidgetId,
    direction: LayoutDirection,
    thickness: f32,
    length: Option<f32>,
    color: Color,
    margin: EdgeInsets,
}

impl Divider {
    /// Create a horizontal divider (full width, `thickness` height).
    pub fn new_horizontal() -> Self {
        Self {
            id: next_widget_id(),
            direction: LayoutDirection::Horizontal,
            thickness: 1.0,
            length: None,
            color: Color::GRAY_300,
            margin: EdgeInsets::default(),
        }
    }

    /// Create a vertical divider (`thickness` width, full height).
    pub fn new_vertical() -> Self {
        Self {
            id: next_widget_id(),
            direction: LayoutDirection::Vertical,
            thickness: 1.0,
            length: None,
            color: Color::GRAY_300,
            margin: EdgeInsets::default(),
        }
    }

    /// Set the line colour.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the line thickness in pixels.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness.max(0.0);
        self
    }

    /// Set an explicit divider length along its main axis.
    ///
    /// For a horizontal divider this controls width.
    /// For a vertical divider this controls height.
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length.max(0.0));
        self
    }

    /// Set the margin around the divider.
    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }
}

impl Widget for Divider {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn debug_name(&self) -> &str {
        "Divider"
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        _fonts: &FontManager,
    ) -> LayoutNode {
        let (w, h) = match self.direction {
            LayoutDirection::Horizontal => {
                let total_w = self.length.unwrap_or(constraints.max_width);
                let total_h = self.thickness + self.margin.vertical();
                (total_w, total_h)
            }
            LayoutDirection::Vertical => {
                let total_w = self.thickness + self.margin.horizontal();
                let total_h = self.length.unwrap_or(constraints.max_height);
                (total_w, total_h)
            }
            LayoutDirection::Overlay => {
                let total_w = self.length.unwrap_or(constraints.max_width);
                let total_h = self.thickness + self.margin.vertical();
                (total_w, total_h)
            }
        };

        LayoutNode::new(
            self.id,
            x,
            y,
            f32_to_u32(w.clamp(constraints.min_width, constraints.max_width)),
            f32_to_u32(h.clamp(constraints.min_height, constraints.max_height)),
        )
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, _fonts: &FontManager) {
        let b = layout.bounds;
        let rect = match self.direction {
            LayoutDirection::Horizontal => Rect::new(
                b.x + f32_to_i32(self.margin.left),
                b.y + f32_to_i32(self.margin.top),
                (b.width as f32 - self.margin.horizontal()).max(0.0) as u32,
                f32_to_u32(self.thickness),
            ),
            LayoutDirection::Vertical => Rect::new(
                b.x + f32_to_i32(self.margin.left),
                b.y + f32_to_i32(self.margin.top),
                f32_to_u32(self.thickness),
                (b.height as f32 - self.margin.vertical()).max(0.0) as u32,
            ),
            LayoutDirection::Overlay => Rect::new(
                b.x + f32_to_i32(self.margin.left),
                b.y + f32_to_i32(self.margin.top),
                (b.width as f32 - self.margin.horizontal()).max(0.0) as u32,
                f32_to_u32(self.thickness),
            ),
        };
        canvas.fill_rect(rect, self.color);
    }

    fn handle_event(
        &mut self,
        _event: &UiEvent,
        _state: &mut EventState,
        _layout: &LayoutNode,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_fonts() -> Option<FontManager> {
        FontManager::new().ok()
    }

    #[test]
    fn horizontal_divider_layout() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut d = Divider::new_horizontal().thickness(2.0);
        let c = BoxConstraints::loose(400.0, 600.0);
        let node = d.layout(c, 0, 0, &fm);
        assert_eq!(node.bounds.width, 400);
        assert_eq!(node.bounds.height, 2);
    }

    #[test]
    fn vertical_divider_layout() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut d = Divider::new_vertical().thickness(3.0);
        let c = BoxConstraints::loose(400.0, 600.0);
        let node = d.layout(c, 10, 20, &fm);
        assert_eq!(node.bounds.width, 3);
        assert_eq!(node.bounds.height, 600);
        assert_eq!(node.bounds.x, 10);
        assert_eq!(node.bounds.y, 20);
    }

    #[test]
    fn vertical_divider_fixed_length() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut d = Divider::new_vertical().thickness(3.0).length(24.0);
        let c = BoxConstraints::loose(400.0, 600.0);
        let node = d.layout(c, 0, 0, &fm);
        assert_eq!(node.bounds.width, 3);
        assert_eq!(node.bounds.height, 24);
    }

    #[test]
    fn horizontal_divider_with_margin() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut d = Divider::new_horizontal().thickness(1.0).margin(EdgeInsets {
            top: 4.0,
            bottom: 4.0,
            left: 0.0,
            right: 0.0,
        });
        let c = BoxConstraints::loose(200.0, 200.0);
        let node = d.layout(c, 0, 0, &fm);
        assert_eq!(node.bounds.width, 200);
        assert_eq!(node.bounds.height, 9);
    }

    #[test]
    fn divider_draws_single_rect() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut canvas = Canvas::new(100, 100);
        canvas.clear(Color::WHITE);

        let d = Divider::new_horizontal().thickness(2.0).color(Color::BLACK);
        let layout = LayoutNode::new(d.id(), 0, 10, 100, 2);
        d.draw(&mut canvas, &layout, &fm);

        let row10_start = 10 * 100;
        assert_eq!(canvas.pixels()[row10_start], Color::BLACK.to_u32());
        assert_eq!(canvas.pixels()[row10_start + 50], Color::BLACK.to_u32());
        let row12_start = 12 * 100;
        assert_eq!(canvas.pixels()[row12_start], Color::WHITE.to_u32());
    }

    #[test]
    fn divider_handle_event_always_false() {
        let mut d = Divider::new_horizontal();
        let layout = LayoutNode::new(d.id(), 0, 0, 100, 1);
        let mut es = EventState::default();
        assert!(!d.handle_event(&UiEvent::MouseDown { x: 50.0, y: 0.0 }, &mut es, &layout,));
    }

    #[test]
    fn builder_chain_works() {
        let d = Divider::new_vertical()
            .color(Color::BLUE)
            .thickness(5.0)
            .margin(EdgeInsets::all(2.0));
        assert_eq!(d.color, Color::BLUE);
        assert!((d.thickness - 5.0).abs() < f32::EPSILON);
    }
}
