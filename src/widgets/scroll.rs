//! Scrollable container widget.
//!
//! `ScrollView` wraps a single child and constrains the visible area to a
//! viewport.  Scrolling is driven by mouse-drag on the scrollbar thumb or
//! by vertical mouse-move delta (as a stand-in for `MouseWheel`, which is
//! not yet in [`UiEvent`]).

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    layout::{f32_to_i32, f32_to_u32, BoxConstraints, LayoutNode, LayoutStyle},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// Width of the scrollbar track in pixels.
const SCROLLBAR_WIDTH: f32 = 8.0;
/// Colour of the scrollbar thumb.
const SCROLLBAR_COLOR: Color = Color::rgba(160, 160, 160, 180);
/// Colour of the scrollbar track background.
const SCROLLBAR_TRACK_COLOR: Color = Color::rgba(230, 230, 230, 100);
/// Scroll speed multiplier for mouse-move delta.
const SCROLL_SPEED: f32 = 3.0;

/// A vertically-scrollable container for a single child widget.
pub struct ScrollView {
    id: WidgetId,
    child: Box<dyn Widget>,
    scroll_offset_y: f32,
    max_height: Option<f32>,
    content_height: f32,
    viewport_height: f32,
    is_dragging: bool,
    drag_start_y: f32,
    drag_start_offset: f32,
    #[allow(dead_code)]
    style: LayoutStyle,
    /// Cached child layout produced by the most recent `layout()` call.
    child_layout: Option<LayoutNode>,
    /// Last known cursor y — used to compute move-delta for scrolling.
    last_cursor_y: Option<f32>,
}

impl ScrollView {
    /// Wrap a child widget in a scrollable container.
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            id: next_widget_id(),
            child: Box::new(child),
            scroll_offset_y: 0.0,
            max_height: None,
            content_height: 0.0,
            viewport_height: 0.0,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_offset: 0.0,
            style: LayoutStyle::default(),
            child_layout: None,
            last_cursor_y: None,
        }
    }

    /// Set a maximum viewport height.  If `None` the viewport uses the
    /// parent constraint's `max_height`.
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_height = Some(h);
        self
    }

    /// Maximum scrollable offset (clamped to >= 0).
    fn max_offset(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    /// Clamp `scroll_offset_y` into `0..=max_offset`.
    fn clamp_offset(&mut self) {
        self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, self.max_offset());
    }

    /// Whether the content overflows the viewport.
    fn is_scrollable(&self) -> bool {
        self.content_height > self.viewport_height
    }

    // -- Scrollbar geometry -----------------------------------------------

    fn scrollbar_track_rect(&self, viewport: &Rect) -> Rect {
        let bar_w = f32_to_u32(SCROLLBAR_WIDTH);
        Rect::new(
            viewport.x + viewport.width as i32 - bar_w as i32,
            viewport.y,
            bar_w,
            viewport.height,
        )
    }

    fn scrollbar_thumb_rect(&self, viewport: &Rect) -> Rect {
        if !self.is_scrollable() {
            return Rect::ZERO;
        }
        let track = self.scrollbar_track_rect(viewport);
        let ratio = self.viewport_height / self.content_height;
        let thumb_h = (track.height as f32 * ratio).max(20.0);
        let max_thumb_y = track.height as f32 - thumb_h;
        let thumb_y = if self.max_offset() > 0.0 {
            max_thumb_y * (self.scroll_offset_y / self.max_offset())
        } else {
            0.0
        };
        Rect::new(
            track.x,
            track.y + f32_to_i32(thumb_y),
            track.width,
            f32_to_u32(thumb_h),
        )
    }

    /// Recursively offset every layout node's y coordinate by `dy`.
    fn offset_children_y(node: &mut LayoutNode, dy: i32) {
        node.bounds.y += dy;
        for child in &mut node.children {
            Self::offset_children_y(child, dy);
        }
    }
}

impl Widget for ScrollView {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn debug_name(&self) -> &str {
        "ScrollView"
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode {
        let available_w = constraints.max_width.max(constraints.min_width);

        // Layout child without height constraint to measure natural height.
        let child_constraints = BoxConstraints {
            min_width: 0.0,
            max_width: available_w,
            min_height: 0.0,
            max_height: f32::INFINITY,
        };

        let child_node = self.child.layout(child_constraints, x, y, fonts);
        self.content_height = child_node.bounds.height as f32;

        // Viewport height = min(content, limit).
        let limit = match self.max_height {
            Some(mh) => mh.min(constraints.max_height),
            None => constraints.max_height,
        };
        self.viewport_height = self.content_height.min(limit);
        self.clamp_offset();

        self.child_layout = Some(child_node);

        LayoutNode::new(
            self.id,
            x,
            y,
            f32_to_u32(available_w),
            f32_to_u32(self.viewport_height),
        )
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        let vp = layout.bounds;
        if vp.is_empty() {
            return;
        }

        // Clip child drawing to viewport.
        canvas.set_clip(vp);

        if let Some(ref child_layout) = self.child_layout {
            let mut offset_layout = child_layout.clone();
            let dy = vp.y - f32_to_i32(self.scroll_offset_y) - child_layout.bounds.y;
            Self::offset_children_y(&mut offset_layout, dy);
            self.child.draw(canvas, &offset_layout, fonts);
        }

        canvas.clear_clip();

        // Draw scrollbar outside clip so it always appears.
        if self.is_scrollable() {
            let track = self.scrollbar_track_rect(&vp);
            canvas.fill_rect(track, SCROLLBAR_TRACK_COLOR);
            let thumb = self.scrollbar_thumb_rect(&vp);
            if !thumb.is_empty() {
                canvas.fill_rect(thumb, SCROLLBAR_COLOR);
            }
        }
    }

    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let vp = layout.bounds;
        let mut changed = false;

        match *event {
            UiEvent::MouseMove { x, y } => {
                if self.is_dragging {
                    // Scrollbar thumb drag.
                    let track = self.scrollbar_track_rect(&vp);
                    let ratio = self.viewport_height / self.content_height;
                    let thumb_h = (track.height as f32 * ratio).max(20.0);
                    let max_thumb_y = track.height as f32 - thumb_h;
                    if max_thumb_y > 0.0 {
                        let dy = y - self.drag_start_y;
                        let new_offset =
                            self.drag_start_offset + dy * self.max_offset() / max_thumb_y;
                        self.scroll_offset_y = new_offset;
                        self.clamp_offset();
                        changed = true;
                    }
                } else if vp.contains(x, y) && self.is_scrollable() {
                    // Mouse-move scroll (stand-in for MouseWheel).
                    if let Some(last_y) = self.last_cursor_y {
                        let dy = y - last_y;
                        if dy.abs() > 0.5 {
                            self.scroll_offset_y += dy * SCROLL_SPEED;
                            self.clamp_offset();
                            changed = true;
                        }
                    }
                }
                self.last_cursor_y = Some(y);

                // Forward to child.
                if let Some(ref child_layout) = self.child_layout {
                    let mut offset_layout = child_layout.clone();
                    let dy_offset = vp.y - f32_to_i32(self.scroll_offset_y) - child_layout.bounds.y;
                    Self::offset_children_y(&mut offset_layout, dy_offset);
                    changed |= self.child.handle_event(event, state, &offset_layout);
                }
            }
            UiEvent::MouseDown { x, y } => {
                let thumb = self.scrollbar_thumb_rect(&vp);
                if !thumb.is_empty() && thumb.contains(x, y) {
                    self.is_dragging = true;
                    self.drag_start_y = y;
                    self.drag_start_offset = self.scroll_offset_y;
                    changed = true;
                } else if vp.contains(x, y) {
                    if let Some(ref child_layout) = self.child_layout {
                        let mut offset_layout = child_layout.clone();
                        let dy_offset =
                            vp.y - f32_to_i32(self.scroll_offset_y) - child_layout.bounds.y;
                        Self::offset_children_y(&mut offset_layout, dy_offset);
                        changed |= self.child.handle_event(event, state, &offset_layout);
                    }
                }
            }
            UiEvent::MouseUp { x: _, y: _ } => {
                if self.is_dragging {
                    self.is_dragging = false;
                    changed = true;
                }
                if let Some(ref child_layout) = self.child_layout {
                    let mut offset_layout = child_layout.clone();
                    let dy_offset = vp.y - f32_to_i32(self.scroll_offset_y) - child_layout.bounds.y;
                    Self::offset_children_y(&mut offset_layout, dy_offset);
                    changed |= self.child.handle_event(event, state, &offset_layout);
                }
            }
        }

        if changed {
            state.request_redraw();
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::text::Text;

    fn try_fonts() -> Option<FontManager> {
        FontManager::new().ok()
    }

    #[test]
    fn scroll_view_viewport_clamps_to_max_height() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let child = Text::new_auto("line\n".repeat(50));
        let mut sv = ScrollView::new(child).max_height(100.0);
        let c = BoxConstraints::loose(300.0, 600.0);
        let node = sv.layout(c, 0, 0, &fm);
        assert!(sv.content_height > 100.0);
        assert_eq!(node.bounds.height, 100);
        assert!((sv.viewport_height - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scroll_view_no_overflow_no_scrollbar() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let child = Text::new_auto("short");
        let mut sv = ScrollView::new(child);
        let c = BoxConstraints::loose(300.0, 600.0);
        let _node = sv.layout(c, 0, 0, &fm);
        assert!(!sv.is_scrollable());
        assert!(sv.max_offset().abs() < f32::EPSILON);
    }

    #[test]
    fn scroll_offset_clamped() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let child = Text::new_auto("line\n".repeat(50));
        let mut sv = ScrollView::new(child).max_height(100.0);
        let c = BoxConstraints::loose(300.0, 600.0);
        sv.layout(c, 0, 0, &fm);

        sv.scroll_offset_y = 99999.0;
        sv.clamp_offset();
        assert!((sv.scroll_offset_y - sv.max_offset()).abs() < f32::EPSILON);

        sv.scroll_offset_y = -100.0;
        sv.clamp_offset();
        assert!(sv.scroll_offset_y.abs() < f32::EPSILON);
    }

    #[test]
    fn scrollbar_thumb_exists_when_scrollable() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let child = Text::new_auto("line\n".repeat(50));
        let mut sv = ScrollView::new(child).max_height(100.0);
        let c = BoxConstraints::loose(300.0, 600.0);
        let node = sv.layout(c, 0, 0, &fm);
        let thumb = sv.scrollbar_thumb_rect(&node.bounds);
        assert!(thumb.height > 0);
    }

    #[test]
    fn scrollbar_drag_changes_offset() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let child = Text::new_auto("line\n".repeat(50));
        let mut sv = ScrollView::new(child).max_height(100.0);
        let c = BoxConstraints::loose(300.0, 600.0);
        let node = sv.layout(c, 0, 0, &fm);

        let thumb = sv.scrollbar_thumb_rect(&node.bounds);
        let thumb_cx = thumb.x as f32 + thumb.width as f32 / 2.0;
        let thumb_cy = thumb.y as f32 + thumb.height as f32 / 2.0;

        let mut es = EventState::default();

        sv.handle_event(
            &UiEvent::MouseDown {
                x: thumb_cx,
                y: thumb_cy,
            },
            &mut es,
            &node,
        );
        assert!(sv.is_dragging);

        sv.handle_event(
            &UiEvent::MouseMove {
                x: thumb_cx,
                y: thumb_cy + 30.0,
            },
            &mut es,
            &node,
        );
        assert!(sv.scroll_offset_y > 0.0);

        sv.handle_event(
            &UiEvent::MouseUp {
                x: thumb_cx,
                y: thumb_cy + 30.0,
            },
            &mut es,
            &node,
        );
        assert!(!sv.is_dragging);
    }

    #[test]
    fn builder_max_height() {
        let child = Text::new_auto("test");
        let sv = ScrollView::new(child).max_height(200.0);
        assert_eq!(sv.max_height, Some(200.0));
    }
}
