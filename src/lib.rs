//! **rust2d_ui** — A pure-Rust 2D UI toolkit using software rasterisation.
//!
//! This crate provides basic rendering primitives ([`Canvas`], [`Color`],
//! [`Rect`]), a simple widget system ([`Container`], [`Text`], [`Button`]),
//! a layout tree ([`LayoutNode`]), and an [`App`] runner.

pub mod app;
pub mod canvas;
pub mod event;
pub mod layout;
pub mod text;
pub mod widgets;

pub use app::{run, App};
pub use canvas::{Canvas, Color, Rect};
pub use event::{EventState, UiEvent};
pub use layout::{BoxConstraints, EdgeInsets, LayoutDirection, LayoutNode, LayoutStyle, Size};
pub use text::FontManager;
pub use widgets::{button::Button, container::Container, text::Text, Widget, WidgetId};

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Canvas & Color ──────────────────────────────────────────────

    #[test]
    fn color_to_u32_ignores_alpha() {
        assert_eq!(Color::rgba(0xFF, 0x00, 0x00, 128).to_u32(), 0x00FF0000);
        assert_eq!(Color::WHITE.to_u32(), 0x00FFFFFF);
        assert_eq!(Color::BLACK.to_u32(), 0x00000000);
    }

    #[test]
    fn canvas_clear_fills_all_pixels() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        assert_eq!(c.pixels().len(), 16);
        assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
    }

    #[test]
    fn canvas_fill_rect_basic() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        c.fill_rect(Rect::new(1, 1, 2, 2), Color::BLACK);

        assert_eq!(c.pixels()[0], Color::WHITE.to_u32());
        assert_eq!(c.pixels()[5], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[6], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[9], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[10], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[3], Color::WHITE.to_u32());
        assert_eq!(c.pixels()[15], Color::WHITE.to_u32());
    }

    #[test]
    fn canvas_fill_rect_clips_negative_origin() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        c.fill_rect(Rect::new(-1, -1, 3, 3), Color::BLACK);
        assert_eq!(c.pixels()[0], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[1], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[4], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[5], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[2], Color::WHITE.to_u32());
    }

    #[test]
    fn canvas_fill_rect_clips_overflow() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        c.fill_rect(Rect::new(3, 3, 100, 100), Color::BLACK);
        assert_eq!(c.pixels()[15], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[14], Color::WHITE.to_u32());
    }

    #[test]
    fn canvas_resize_preserves_existing_rows() {
        let mut c = Canvas::new(2, 2);
        c.clear(Color::BLACK);
        c.fill_rect(Rect::new(0, 0, 1, 1), Color::WHITE);

        c.resize(4, 4);
        assert_eq!(c.width(), 4);
        assert_eq!(c.height(), 4);
        assert_eq!(c.pixels().len(), 16);
        assert_eq!(c.pixels()[0], Color::WHITE.to_u32());
        assert_eq!(c.pixels()[1], Color::BLACK.to_u32());
        assert_eq!(c.pixels()[2], 0);
    }

    #[test]
    fn canvas_resize_shrink() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        c.resize(2, 2);
        assert_eq!(c.pixels().len(), 4);
        assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
    }

    #[test]
    fn canvas_resize_noop() {
        let mut c = Canvas::new(3, 3);
        c.clear(Color::BLUE);
        c.resize(3, 3);
        assert!(c.pixels().iter().all(|&p| p == Color::BLUE.to_u32()));
    }

    #[test]
    fn blend_pixel_opaque_overwrites() {
        let mut c = Canvas::new(2, 2);
        c.clear(Color::WHITE);
        c.blend_pixel(0, 0, Color::BLACK);
        assert_eq!(c.pixels()[0], Color::BLACK.to_u32());
    }

    #[test]
    fn blend_pixel_transparent_noop() {
        let mut c = Canvas::new(2, 2);
        c.clear(Color::WHITE);
        c.blend_pixel(0, 0, Color::TRANSPARENT);
        assert_eq!(c.pixels()[0], Color::WHITE.to_u32());
    }

    #[test]
    fn blend_pixel_half_alpha() {
        let mut c = Canvas::new(1, 1);
        c.clear(Color::BLACK);
        c.blend_pixel(0, 0, Color::rgba(255, 255, 255, 128));
        let p = c.pixels()[0];
        let r = (p >> 16) & 0xFF;
        let g = (p >> 8) & 0xFF;
        let b = p & 0xFF;
        assert!((126..=129).contains(&r), "r = {r}");
        assert!((126..=129).contains(&g), "g = {g}");
        assert!((126..=129).contains(&b), "b = {b}");
    }

    #[test]
    fn blend_pixel_out_of_bounds_ignored() {
        let mut c = Canvas::new(2, 2);
        c.clear(Color::WHITE);
        c.blend_pixel(-1, 0, Color::BLACK);
        c.blend_pixel(0, -1, Color::BLACK);
        c.blend_pixel(2, 0, Color::BLACK);
        c.blend_pixel(0, 2, Color::BLACK);
        assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
    }

    #[test]
    fn draw_rounded_rect_fills_center() {
        let mut c = Canvas::new(20, 20);
        c.clear(Color::WHITE);
        c.draw_rounded_rect(Rect::new(0, 0, 20, 20), 4, Color::BLACK);
        assert_eq!(c.pixels()[10 * 20 + 10], Color::BLACK.to_u32());
    }

    #[test]
    fn draw_rounded_rect_skips_corners() {
        let mut c = Canvas::new(20, 20);
        c.clear(Color::WHITE);
        c.draw_rounded_rect(Rect::new(0, 0, 20, 20), 8, Color::BLACK);
        assert_eq!(c.pixels()[0], Color::WHITE.to_u32());
    }

    // ── Rect ────────────────────────────────────────────────────────

    #[test]
    fn rect_contains_f32_precision() {
        let r = Rect::new(0, 0, 100, 100);
        assert!(r.contains(0.0, 0.0));
        assert!(r.contains(99.9, 99.9));
        assert!(!r.contains(100.0, 0.0));
        assert!(!r.contains(-0.1, 0.0));
    }

    #[test]
    fn rect_contains_negative_origin() {
        let r = Rect::new(-10, -10, 20, 20);
        assert!(r.contains(-10.0, -10.0));
        assert!(r.contains(0.0, 0.0));
        assert!(r.contains(9.9, 9.9));
        assert!(!r.contains(10.0, 0.0));
    }

    #[test]
    fn rect_union_basic() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let u = a.union(&b);
        assert_eq!(u.x, 0);
        assert_eq!(u.y, 0);
        assert_eq!(u.width, 15);
        assert_eq!(u.height, 15);
    }

    #[test]
    fn rect_union_with_empty() {
        let a = Rect::new(5, 5, 10, 10);
        let empty = Rect::new(0, 0, 0, 0);
        assert_eq!(a.union(&empty), a);
        assert_eq!(empty.union(&a), a);
    }

    // ── Layout ──────────────────────────────────────────────────────

    #[test]
    fn layout_node_find_by_id_works() {
        let mut root = LayoutNode::new(WidgetId::manual(1), 0, 0, 100, 100);
        let mut child = LayoutNode::new(WidgetId::manual(2), 0, 0, 50, 50);
        child.add_child(LayoutNode::new(WidgetId::manual(3), 10, 10, 20, 20));
        root.add_child(child);

        assert!(root.find_by_id(WidgetId::manual(1)).is_some());
        assert!(root.find_by_id(WidgetId::manual(3)).is_some());
        assert!(root.find_by_id(WidgetId::manual(999)).is_none());
    }

    #[test]
    fn box_constraints_tight() {
        let c = BoxConstraints::tight(100.0, 50.0);
        assert_eq!(c.min_width, 100.0);
        assert_eq!(c.max_width, 100.0);
        let s = c.constrain(Size { width: 200.0, height: 200.0 });
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 50.0);
    }

    #[test]
    fn box_constraints_loose() {
        let c = BoxConstraints::loose(100.0, 50.0);
        assert_eq!(c.min_width, 0.0);
        let s = c.constrain(Size { width: 30.0, height: 10.0 });
        assert_eq!(s.width, 30.0);
        assert_eq!(s.height, 10.0);
    }

    // ── Widget IDs ──────────────────────────────────────────────────

    #[test]
    fn auto_generated_widget_ids_are_unique() {
        let a = widgets::next_widget_id();
        let b = widgets::next_widget_id();
        assert_ne!(a, b);
    }

    #[test]
    fn auto_ids_never_collide_with_small_manual_ids() {
        let auto = widgets::next_widget_id();
        let manual = WidgetId::manual(42);
        assert_ne!(auto, manual);
        assert!(auto.raw() >= (1u64 << 32));
    }

    #[test]
    fn auto_widget_constructors_produce_distinct_ids() {
        let text = Text::new_auto("a");
        let button = Button::new_auto("b");
        assert_ne!(text.id(), button.id());
    }

    #[test]
    fn manual_id_still_works() {
        let t = Text::new(WidgetId::manual(9999), "hi");
        assert_eq!(t.id(), WidgetId::manual(9999));
    }

    // ── Event state ─────────────────────────────────────────────────

    #[test]
    fn event_state_redraw_tracking() {
        let mut es = EventState::default();
        assert!(!es.take_needs_redraw());

        es.request_redraw();
        assert!(es.take_needs_redraw());
        assert!(!es.take_needs_redraw());
    }

    // ── Widget layout / events ──────────────────────────────────────

    #[test]
    fn button_handle_event_hover() {
        let mut btn = Button::new(WidgetId::manual(100), "test");
        let layout = LayoutNode::new(WidgetId::manual(100), 0, 0, 80, 30);
        let mut es = EventState::default();

        let changed = btn.handle_event(
            &UiEvent::MouseMove { x: 10.0, y: 10.0 }, &mut es, &layout,
        );
        assert!(changed);

        let changed = btn.handle_event(
            &UiEvent::MouseMove { x: 200.0, y: 200.0 }, &mut es, &layout,
        );
        assert!(changed);
    }

    #[test]
    fn button_handle_event_click() {
        use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
        let clicked = Arc::new(AtomicBool::new(false));
        let c2 = clicked.clone();

        let mut btn = Button::new(WidgetId::manual(101), "click me")
            .on_click(move || { c2.store(true, Ordering::SeqCst); });
        let layout = LayoutNode::new(WidgetId::manual(101), 0, 0, 80, 30);
        let mut es = EventState::default();

        btn.handle_event(&UiEvent::MouseDown { x: 10.0, y: 10.0 }, &mut es, &layout);
        assert!(!clicked.load(Ordering::SeqCst));

        btn.handle_event(&UiEvent::MouseUp { x: 10.0, y: 10.0 }, &mut es, &layout);
        assert!(clicked.load(Ordering::SeqCst));
    }

    #[test]
    fn button_click_outside_does_not_fire() {
        use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
        let clicked = Arc::new(AtomicBool::new(false));
        let c2 = clicked.clone();

        let mut btn = Button::new(WidgetId::manual(102), "nope")
            .on_click(move || { c2.store(true, Ordering::SeqCst); });
        let layout = LayoutNode::new(WidgetId::manual(102), 0, 0, 80, 30);
        let mut es = EventState::default();

        btn.handle_event(&UiEvent::MouseDown { x: 10.0, y: 10.0 }, &mut es, &layout);
        btn.handle_event(&UiEvent::MouseUp { x: 200.0, y: 200.0 }, &mut es, &layout);
        assert!(!clicked.load(Ordering::SeqCst));
    }

    #[test]
    fn text_widget_handle_event_always_false() {
        let mut t = Text::new(WidgetId::manual(200), "hello");
        let layout = LayoutNode::new(WidgetId::manual(200), 0, 0, 100, 30);
        let mut es = EventState::default();
        assert!(!t.handle_event(
            &UiEvent::MouseMove { x: 10.0, y: 10.0 }, &mut es, &layout,
        ));
    }

    #[test]
    fn container_propagates_events_to_children() {
        let mut c = Container::new(WidgetId::manual(300));
        c.push(Button::new(WidgetId::manual(301), "btn"));

        let mut root_layout = LayoutNode::new(WidgetId::manual(300), 0, 0, 200, 200);
        root_layout.add_child(LayoutNode::new(WidgetId::manual(301), 0, 0, 80, 30));

        let mut es = EventState::default();
        let changed = c.handle_event(
            &UiEvent::MouseMove { x: 10.0, y: 10.0 }, &mut es, &root_layout,
        );
        assert!(changed);
    }

    // ── Edge cases ──────────────────────────────────────────────────

    #[test]
    fn zero_size_canvas() {
        let c = Canvas::new(0, 0);
        assert_eq!(c.pixels().len(), 0);
    }

    #[test]
    fn canvas_1x1() {
        let mut c = Canvas::new(1, 1);
        c.clear(Color::BLUE);
        assert_eq!(c.pixels()[0], Color::BLUE.to_u32());
        c.blend_pixel(0, 0, Color::rgba(255, 0, 0, 128));
        let p = c.pixels()[0];
        let r = (p >> 16) & 0xFF;
        assert!(r > 100);
    }

    #[test]
    fn fill_rect_zero_width() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        c.fill_rect(Rect::new(0, 0, 0, 4), Color::BLACK);
        assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
    }

    #[test]
    fn fill_rect_completely_outside() {
        let mut c = Canvas::new(4, 4);
        c.clear(Color::WHITE);
        c.fill_rect(Rect::new(100, 100, 10, 10), Color::BLACK);
        assert!(c.pixels().iter().all(|&p| p == Color::WHITE.to_u32()));
    }
}
