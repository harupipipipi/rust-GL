//! A checkbox toggle widget with label.

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode, LayoutStyle, Size, f32_to_i32, f32_to_u32},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// Size of the checkbox square in pixels.
const BOX_SIZE: f32 = 18.0;
/// Gap between the checkbox square and the label text.
const BOX_LABEL_GAP: f32 = 6.0;
/// Corner radius for the checkbox outline.
const BOX_RADIUS: u32 = 3;
/// Inset of the inner check-mark fill relative to the box edge.
const CHECK_INSET: i32 = 4;

/// A checkbox control with a text label and toggle callback.
pub struct Checkbox {
    id: WidgetId,
    label: String,
    checked: bool,
    is_hovered: bool,
    font_size: f32,
    /// Layout hints used when measuring and placing the widget.
    pub style: LayoutStyle,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
}

impl Checkbox {
    /// Create a checkbox with an auto-generated widget ID.
    pub fn new_auto(label: impl Into<String>) -> Self {
        Self::new(next_widget_id(), label)
    }

    /// Create a checkbox with an explicit widget ID.
    pub fn new(id: WidgetId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            checked: false,
            is_hovered: false,
            font_size: 16.0,
            style: LayoutStyle::default(),
            on_toggle: None,
        }
    }

    // -- Builder methods --

    /// Set the initial checked state.
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }

    /// Attach a callback fired after the checked state changes.
    pub fn on_toggle(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Set the label font size in logical pixels.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_size = px;
        self
    }

    // -- Getters --

    /// Return whether the checkbox is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Update the checked state programmatically.
    pub fn set_checked(&mut self, value: bool) {
        self.checked = value;
    }
}

impl Widget for Checkbox {
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
        let total_w = BOX_SIZE + BOX_LABEL_GAP + text_w;
        let total_h = BOX_SIZE.max(text_h);
        let size = constraints.constrain(Size {
            width: total_w,
            height: total_h,
        });
        LayoutNode::new(self.id, x, y, f32_to_u32(size.width), f32_to_u32(size.height))
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        let bounds = layout.bounds;

        // Vertically centre the box within the widget bounds.
        let box_y = bounds.y + f32_to_i32((bounds.height as f32 - BOX_SIZE) * 0.5);
        let box_rect = Rect::new(bounds.x, box_y, BOX_SIZE as u32, BOX_SIZE as u32);

        // Outer outline -- colour depends on hover state.
        let outline_color = if self.is_hovered {
            Color::BLUE
        } else {
            Color::GRAY_600
        };
        canvas.draw_rounded_rect(box_rect, BOX_RADIUS, outline_color);

        // Inner fill -- slightly smaller so the outline is visible.
        let inner = Rect::new(
            box_rect.x + 2,
            box_rect.y + 2,
            box_rect.width.saturating_sub(4),
            box_rect.height.saturating_sub(4),
        );
        if self.checked {
            canvas.draw_rounded_rect(inner, 2, Color::BLUE);
        } else {
            canvas.draw_rounded_rect(inner, 2, Color::WHITE);
        }

        // Check mark -- when checked, draw a small filled rect inside.
        if self.checked {
            let mark = Rect::new(
                box_rect.x + CHECK_INSET,
                box_rect.y + CHECK_INSET,
                box_rect.width.saturating_sub(CHECK_INSET as u32 * 2),
                box_rect.height.saturating_sub(CHECK_INSET as u32 * 2),
            );
            canvas.fill_rect(mark, Color::WHITE);
        }

        // Label text.
        let text_x = bounds.x + f32_to_i32(BOX_SIZE + BOX_LABEL_GAP);
        let text_y = bounds.y + f32_to_i32((bounds.height as f32 - self.font_size) * 0.5);
        fonts.draw_text_in_rect(
            canvas,
            &self.label,
            Rect::new(
                text_x,
                text_y,
                bounds.width.saturating_sub(f32_to_u32(BOX_SIZE + BOX_LABEL_GAP)),
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
                if was_pressed && rect.contains(x, y) {
                    self.checked = !self.checked;
                    if let Some(cb) = self.on_toggle.as_mut() {
                        cb(self.checked);
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
    fn checkbox_default_unchecked() {
        let cb = Checkbox::new(WidgetId::manual(500), "Test");
        assert!(!cb.is_checked());
    }

    #[test]
    fn checkbox_builder_checked() {
        let cb = Checkbox::new(WidgetId::manual(501), "Test").checked(true);
        assert!(cb.is_checked());
    }

    #[test]
    fn checkbox_toggle_on_click() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        let toggled = Arc::new(AtomicBool::new(false));
        let t2 = toggled.clone();

        let mut cb = Checkbox::new(WidgetId::manual(502), "Toggle me").on_toggle(move |val| {
            t2.store(val, Ordering::SeqCst);
        });
        let layout = LayoutNode::new(WidgetId::manual(502), 0, 0, 100, 18);
        let mut es = EventState::default();

        cb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        assert_eq!(es.pressed, Some(WidgetId::manual(502)));

        cb.handle_event(&UiEvent::MouseUp { x: 5.0, y: 5.0 }, &mut es, &layout);
        assert!(cb.is_checked());
        assert!(toggled.load(Ordering::SeqCst));
    }

    #[test]
    fn checkbox_click_outside_does_not_toggle() {
        let mut cb = Checkbox::new(WidgetId::manual(503), "No toggle");
        let layout = LayoutNode::new(WidgetId::manual(503), 0, 0, 100, 18);
        let mut es = EventState::default();

        cb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        cb.handle_event(
            &UiEvent::MouseUp {
                x: 200.0,
                y: 200.0,
            },
            &mut es,
            &layout,
        );
        assert!(!cb.is_checked());
    }

    #[test]
    fn checkbox_hover_changes_state() {
        let mut cb = Checkbox::new(WidgetId::manual(504), "Hover");
        let layout = LayoutNode::new(WidgetId::manual(504), 0, 0, 100, 18);
        let mut es = EventState::default();

        let changed = cb.handle_event(
            &UiEvent::MouseMove { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        assert!(changed);

        let changed = cb.handle_event(
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
    fn checkbox_double_toggle() {
        let mut cb = Checkbox::new(WidgetId::manual(505), "Double");
        let layout = LayoutNode::new(WidgetId::manual(505), 0, 0, 100, 18);
        let mut es = EventState::default();

        cb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        cb.handle_event(&UiEvent::MouseUp { x: 5.0, y: 5.0 }, &mut es, &layout);
        assert!(cb.is_checked());

        cb.handle_event(
            &UiEvent::MouseDown { x: 5.0, y: 5.0 },
            &mut es,
            &layout,
        );
        cb.handle_event(&UiEvent::MouseUp { x: 5.0, y: 5.0 }, &mut es, &layout);
        assert!(!cb.is_checked());
    }
}
