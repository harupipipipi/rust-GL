//! A radio button widget. Mutual exclusion is managed by the parent via
//! callbacks (controlled-component pattern, like React).

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode, LayoutStyle, Size, f32_to_i32, f32_to_u32},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// Diameter of the radio circle in pixels.
const CIRCLE_SIZE: f32 = 18.0;
/// Gap between the circle and the label text.
const CIRCLE_LABEL_GAP: f32 = 6.0;
/// Corner radius for the outer circle (half of size = full circle).
const OUTER_RADIUS: u32 = 9;
/// Inner dot size when selected.
const DOT_SIZE: f32 = 10.0;
/// Corner radius for the inner dot (half of dot size = full circle).
const DOT_RADIUS: u32 = 5;

/// A radio button with a text label and caller-managed group selection.
pub struct RadioButton {
    id: WidgetId,
    label: String,
    selected: bool,
    group: u32,
    is_hovered: bool,
    font_size: f32,
    /// Layout hints used when measuring and placing the widget.
    pub style: LayoutStyle,
    on_select: Option<Box<dyn FnMut()>>,
}

impl RadioButton {
    /// Create a radio button with an auto-generated widget ID.
    pub fn new_auto(label: impl Into<String>, group: u32) -> Self {
        Self::new(next_widget_id(), label, group)
    }

    /// Create a radio button with an explicit widget ID and group.
    pub fn new(id: WidgetId, label: impl Into<String>, group: u32) -> Self {
        Self {
            id,
            label: label.into(),
            selected: false,
            group,
            is_hovered: false,
            font_size: 16.0,
            style: LayoutStyle::default(),
            on_select: None,
        }
    }

    // -- Builder methods --

    /// Set the initial selected state.
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    /// Attach a callback fired when this radio button becomes selected.
    pub fn on_select(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Set the label font size in logical pixels.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_size = px;
        self
    }

    // -- Getters / Setters --

    /// Return whether this radio button is currently selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Update the selected state programmatically.
    pub fn set_selected(&mut self, value: bool) {
        self.selected = value;
    }

    /// Return the logical radio group ID assigned by the caller.
    pub fn group(&self) -> u32 {
        self.group
    }
}

impl Widget for RadioButton {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode {
        let (text_w, text_h) = fonts.measure_text(&self.label, self.font_size);
        let total_w = CIRCLE_SIZE + CIRCLE_LABEL_GAP + text_w;
        let total_h = CIRCLE_SIZE.max(text_h);
        let size = constraints.constrain(Size {
            width: total_w,
            height: total_h,
        });
        LayoutNode::new(self.id, x, y, f32_to_u32(size.width), f32_to_u32(size.height))
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        let bounds = layout.bounds;

        let circle_y = bounds.y + f32_to_i32((bounds.height as f32 - CIRCLE_SIZE) * 0.5);
        let circle_rect = Rect::new(
            bounds.x,
            circle_y,
            CIRCLE_SIZE as u32,
            CIRCLE_SIZE as u32,
        );

        // Outer circle -- large radius to approximate a circle.
        let outline_color = if self.is_hovered {
            Color::BLUE
        } else {
            Color::GRAY_600
        };
        canvas.draw_rounded_rect(circle_rect, OUTER_RADIUS, outline_color);

        // White inner fill (to show outline ring).
        let inner = Rect::new(
            circle_rect.x + 2,
            circle_rect.y + 2,
            circle_rect.width.saturating_sub(4),
            circle_rect.height.saturating_sub(4),
        );
        canvas.draw_rounded_rect(inner, OUTER_RADIUS.saturating_sub(2), Color::WHITE);

        // Selected dot.
        if self.selected {
            let dot_offset = f32_to_i32((CIRCLE_SIZE - DOT_SIZE) * 0.5);
            let dot_rect = Rect::new(
                circle_rect.x + dot_offset,
                circle_rect.y + dot_offset,
                DOT_SIZE as u32,
                DOT_SIZE as u32,
            );
            canvas.draw_rounded_rect(dot_rect, DOT_RADIUS, Color::BLUE);
        }

        // Label text.
        let text_x = bounds.x + f32_to_i32(CIRCLE_SIZE + CIRCLE_LABEL_GAP);
        let text_y = bounds.y + f32_to_i32((bounds.height as f32 - self.font_size) * 0.5);
        fonts.draw_text_in_rect(
            canvas,
            &self.label,
            Rect::new(
                text_x,
                text_y,
                bounds.width.saturating_sub(f32_to_u32(CIRCLE_SIZE + CIRCLE_LABEL_GAP)),
                bounds.height,
            ),
            self.font_size,
            Color::BLACK,
        );
    }

    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let mut changed = false;
        let rect = layout.bounds;

        match *event {
            UiEvent::MouseMove { x, y } => {
                let hovered = rect.contains(x, y);
                if hovered != self.is_hovered {
                    self.is_hovered = hovered;
                    changed = true;
                }
            }
            UiEvent::MouseDown { x, y } => {
                if rect.contains(x, y) {
                    state.set_pressed(Some(self.id));
                }
            }
            UiEvent::MouseUp { x, y } => {
                let was_pressed = state.pressed == Some(self.id);
                if was_pressed && rect.contains(x, y) && !self.selected {
                    self.selected = true;
                    if let Some(cb) = self.on_select.as_mut() {
                        cb();
                    }
                    changed = true;
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
    use crate::event::EventState;
    use crate::layout::LayoutNode;
    use crate::widgets::WidgetId;

    #[test]
    fn radio_default_not_selected() {
        let rb = RadioButton::new(WidgetId::manual(600), "Option A", 1);
        assert!(!rb.is_selected());
        assert_eq!(rb.group(), 1);
    }

    #[test]
    fn radio_builder_selected() {
        let rb = RadioButton::new(WidgetId::manual(601), "Option B", 1).selected(true);
        assert!(rb.is_selected());
    }

    #[test]
    fn radio_select_on_click() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        let fired = Arc::new(AtomicBool::new(false));
        let f2 = fired.clone();

        let mut rb =
            RadioButton::new(WidgetId::manual(602), "Select me", 1).on_select(move || {
                f2.store(true, Ordering::SeqCst);
            });
        let layout = LayoutNode::new(WidgetId::manual(602), 0, 0, 100, 18);
        let mut es = EventState::default();

        rb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        rb.handle_event(&UiEvent::MouseUp { x: 5.0, y: 5.0 }, &mut es, &layout);
        assert!(rb.is_selected());
        assert!(fired.load(Ordering::SeqCst));
    }

    #[test]
    fn radio_already_selected_does_not_fire_again() {
        use std::sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        };
        let count = Arc::new(AtomicU32::new(0));
        let c2 = count.clone();

        let mut rb = RadioButton::new(WidgetId::manual(603), "Already", 1)
            .selected(true)
            .on_select(move || {
                c2.fetch_add(1, Ordering::SeqCst);
            });
        let layout = LayoutNode::new(WidgetId::manual(603), 0, 0, 100, 18);
        let mut es = EventState::default();

        rb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        rb.handle_event(&UiEvent::MouseUp { x: 5.0, y: 5.0 }, &mut es, &layout);
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn radio_click_outside_does_not_select() {
        let mut rb = RadioButton::new(WidgetId::manual(604), "No select", 2);
        let layout = LayoutNode::new(WidgetId::manual(604), 0, 0, 100, 18);
        let mut es = EventState::default();

        rb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        rb.handle_event(
            &UiEvent::MouseUp {
                x: 200.0,
                y: 200.0,
            },
            &mut es,
            &layout,
        );
        assert!(!rb.is_selected());
    }

    #[test]
    fn radio_hover_changes_state() {
        let mut rb = RadioButton::new(WidgetId::manual(605), "Hover", 1);
        let layout = LayoutNode::new(WidgetId::manual(605), 0, 0, 100, 18);
        let mut es = EventState::default();

        let changed = rb.handle_event(
            &UiEvent::MouseMove { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        assert!(changed);

        let changed = rb.handle_event(
            &UiEvent::MouseMove {
                x: 200.0,
                y: 200.0,
            },
            &mut es,
            &layout,
        );
        assert!(changed);
    }

    #[test]
    fn radio_set_selected_programmatic() {
        let mut rb = RadioButton::new(WidgetId::manual(606), "Prog", 1);
        assert!(!rb.is_selected());
        rb.set_selected(true);
        assert!(rb.is_selected());
        rb.set_selected(false);
        assert!(!rb.is_selected());
    }
}
