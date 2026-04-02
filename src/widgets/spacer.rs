//! A spacer widget that reserves blank space in a layout.
//!
//! `Spacer::fixed(size)` creates a rigid spacer of the given size.
//! `Spacer::flex()` creates a flexible spacer (future flex-grow support;
//! currently behaves as a zero-size spacer).

use crate::{
    canvas::Canvas,
    event::{EventState, UiEvent},
    layout::{f32_to_u32, BoxConstraints, LayoutNode},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// A transparent spacer widget.
///
/// In the current implementation `flex` is stored but not used by the
/// container layout algorithm.  `min_size` is always returned as the
/// widget's extent along the parent's main axis.
pub struct Spacer {
    id: WidgetId,
    min_size: f32,
    #[allow(dead_code)]
    flex: f32,
}

impl Spacer {
    /// Create a fixed-size spacer.
    pub fn fixed(size: f32) -> Self {
        Self {
            id: next_widget_id(),
            min_size: size.max(0.0),
            flex: 0.0,
        }
    }

    /// Create a flexible spacer (currently behaves as zero-size).
    pub fn flex() -> Self {
        Self {
            id: next_widget_id(),
            min_size: 0.0,
            flex: 1.0,
        }
    }
}

impl Widget for Spacer {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn debug_name(&self) -> &str {
        "Spacer"
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        _fonts: &FontManager,
    ) -> LayoutNode {
        let w = self
            .min_size
            .clamp(constraints.min_width, constraints.max_width);
        let h = self
            .min_size
            .clamp(constraints.min_height, constraints.max_height);
        LayoutNode::new(self.id, x, y, f32_to_u32(w), f32_to_u32(h))
    }

    fn draw(&self, _canvas: &mut Canvas, _layout: &LayoutNode, _fonts: &FontManager) {
        // Spacers are invisible — nothing to draw.
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
    fn fixed_spacer_returns_min_size() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut s = Spacer::fixed(20.0);
        let c = BoxConstraints::loose(200.0, 200.0);
        let node = s.layout(c, 0, 0, &fm);
        assert_eq!(node.bounds.width, 20);
        assert_eq!(node.bounds.height, 20);
    }

    #[test]
    fn fixed_spacer_clamped_to_constraints() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut s = Spacer::fixed(300.0);
        let c = BoxConstraints::loose(100.0, 100.0);
        let node = s.layout(c, 5, 10, &fm);
        assert_eq!(node.bounds.width, 100);
        assert_eq!(node.bounds.height, 100);
        assert_eq!(node.bounds.x, 5);
        assert_eq!(node.bounds.y, 10);
    }

    #[test]
    fn flex_spacer_returns_zero_size() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut s = Spacer::flex();
        let c = BoxConstraints::loose(200.0, 200.0);
        let node = s.layout(c, 0, 0, &fm);
        assert_eq!(node.bounds.width, 0);
        assert_eq!(node.bounds.height, 0);
    }

    #[test]
    fn spacer_handle_event_always_false() {
        let mut s = Spacer::fixed(10.0);
        let layout = LayoutNode::new(s.id(), 0, 0, 10, 10);
        let mut es = EventState::default();
        assert!(!s.handle_event(&UiEvent::MouseMove { x: 5.0, y: 5.0 }, &mut es, &layout,));
    }

    #[test]
    fn spacer_ids_are_unique() {
        let a = Spacer::fixed(10.0);
        let b = Spacer::flex();
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn negative_size_clamped_to_zero() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut s = Spacer::fixed(-5.0);
        let c = BoxConstraints::loose(100.0, 100.0);
        let node = s.layout(c, 0, 0, &fm);
        assert_eq!(node.bounds.width, 0);
        assert_eq!(node.bounds.height, 0);
    }
}
