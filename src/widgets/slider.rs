//! A horizontal slider widget with optional step-snapping.

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode, LayoutStyle, Size, f32_to_i32, f32_to_u32},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// Total height of the slider widget.
const SLIDER_HEIGHT: f32 = 24.0;
/// Height of the track bar.
const TRACK_HEIGHT: f32 = 6.0;
/// Width and height of the thumb (square with rounded corners).
const THUMB_SIZE: f32 = 16.0;
/// Corner radius for the thumb.
const THUMB_RADIUS: u32 = 4;
/// Corner radius for the track.
const TRACK_RADIUS: u32 = 3;

pub struct Slider {
    id: WidgetId,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    is_dragging: bool,
    is_hovered: bool,
    pub style: LayoutStyle,
    on_change: Option<Box<dyn FnMut(f32)>>,
}

impl Slider {
    pub fn new_auto() -> Self {
        Self::new(next_widget_id())
    }

    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: None,
            is_dragging: false,
            is_hovered: false,
            style: LayoutStyle::default(),
            on_change: None,
        }
    }

    // -- Builder methods --

    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, s: f32) -> Self {
        self.step = Some(s);
        self
    }

    pub fn on_change(mut self, f: impl FnMut(f32) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    // -- Getters / Setters --

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, v: f32) {
        self.value = self.snap(v.clamp(self.min, self.max));
    }

    // -- Internal helpers --

    /// Normalised position of the thumb: 0.0 = leftmost, 1.0 = rightmost.
    fn ratio(&self) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON {
            return 0.0;
        }
        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// Snap `v` to the nearest step value, if `step` is set.
    fn snap(&self, v: f32) -> f32 {
        match self.step {
            Some(s) if s > 0.0 => {
                let steps = ((v - self.min) / s).round();
                (self.min + steps * s).clamp(self.min, self.max)
            }
            _ => v,
        }
    }

    /// Convert an x pixel coordinate (relative to the canvas) to a slider
    /// value, given the track bounds.
    fn x_to_value(&self, x: f32, track_x: f32, track_width: f32) -> f32 {
        if track_width <= 0.0 {
            return self.min;
        }
        let ratio = ((x - track_x) / track_width).clamp(0.0, 1.0);
        let raw = self.min + ratio * (self.max - self.min);
        self.snap(raw)
    }

    /// The usable track width (= widget width minus thumb width, so the
    /// thumb centre can reach both edges of the widget).
    fn track_width(bounds: &Rect) -> f32 {
        (bounds.width as f32 - THUMB_SIZE).max(0.0)
    }

    /// Left edge of the usable track in canvas coordinates.
    fn track_x(bounds: &Rect) -> f32 {
        bounds.x as f32 + THUMB_SIZE * 0.5
    }
}

impl Widget for Slider {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        _fonts: &FontManager,
    ) -> LayoutNode {
        let size = constraints.constrain(Size {
            width: constraints.max_width,
            height: SLIDER_HEIGHT,
        });
        LayoutNode::new(self.id, x, y, f32_to_u32(size.width), f32_to_u32(size.height))
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, _fonts: &FontManager) {
        let bounds = layout.bounds;
        let ratio = self.ratio();
        let tw = Self::track_width(&bounds);
        let tx = Self::track_x(&bounds);

        // Track vertical centre.
        let track_y = bounds.y + f32_to_i32((SLIDER_HEIGHT - TRACK_HEIGHT) * 0.5);

        // Full track (inactive part).
        let full_track = Rect::new(bounds.x, track_y, bounds.width, TRACK_HEIGHT as u32);
        canvas.draw_rounded_rect(full_track, TRACK_RADIUS, Color::GRAY_300);

        // Active part (left edge to thumb position).
        let active_width = f32_to_u32(ratio * tw + THUMB_SIZE * 0.5);
        if active_width > 0 {
            let active_track = Rect::new(bounds.x, track_y, active_width, TRACK_HEIGHT as u32);
            canvas.draw_rounded_rect(active_track, TRACK_RADIUS, Color::BLUE);
        }

        // Thumb.
        let thumb_centre_x = tx + ratio * tw;
        let thumb_x = f32_to_i32(thumb_centre_x - THUMB_SIZE * 0.5);
        let thumb_y = bounds.y + f32_to_i32((SLIDER_HEIGHT - THUMB_SIZE) * 0.5);
        let thumb_rect = Rect::new(thumb_x, thumb_y, THUMB_SIZE as u32, THUMB_SIZE as u32);

        let thumb_color = if self.is_dragging {
            Color::rgba(50, 100, 200, 255)
        } else if self.is_hovered {
            Color::rgba(80, 130, 230, 255)
        } else {
            Color::BLUE
        };
        canvas.draw_rounded_rect(thumb_rect, THUMB_RADIUS, thumb_color);
    }

    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let mut changed = false;
        let bounds = layout.bounds;
        let tw = Self::track_width(&bounds);
        let tx = Self::track_x(&bounds);

        match *event {
            UiEvent::MouseMove { x, y } => {
                let hovered = bounds.contains(x, y);
                if hovered != self.is_hovered {
                    self.is_hovered = hovered;
                    changed = true;
                }

                if self.is_dragging {
                    let new_val = self.x_to_value(x, tx, tw);
                    if (new_val - self.value).abs() > f32::EPSILON {
                        self.value = new_val;
                        if let Some(cb) = self.on_change.as_mut() {
                            cb(self.value);
                        }
                        changed = true;
                    }
                }
            }
            UiEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    self.is_dragging = true;
                    state.pressed = Some(self.id);
                    let new_val = self.x_to_value(x, tx, tw);
                    if (new_val - self.value).abs() > f32::EPSILON {
                        self.value = new_val;
                        if let Some(cb) = self.on_change.as_mut() {
                            cb(self.value);
                        }
                    }
                    changed = true;
                }
            }
            UiEvent::MouseUp { .. } => {
                if self.is_dragging {
                    self.is_dragging = false;
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
    fn slider_default_values() {
        let s = Slider::new(WidgetId::manual(700));
        assert!((s.get_value() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slider_builder() {
        let s = Slider::new(WidgetId::manual(701))
            .value(50.0)
            .range(0.0, 100.0)
            .step(10.0);
        assert!((s.get_value() - 50.0).abs() < f32::EPSILON);
        assert!((s.min - 0.0).abs() < f32::EPSILON);
        assert!((s.max - 100.0).abs() < f32::EPSILON);
        assert_eq!(s.step, Some(10.0));
    }

    #[test]
    fn slider_snap() {
        let s = Slider::new(WidgetId::manual(702))
            .range(0.0, 100.0)
            .step(25.0);
        assert!((s.snap(13.0) - 25.0).abs() < f32::EPSILON);
        assert!((s.snap(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((s.snap(99.0) - 100.0).abs() < f32::EPSILON);
        assert!((s.snap(37.0) - 25.0).abs() < f32::EPSILON);
        assert!((s.snap(38.0) - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slider_ratio() {
        let s = Slider::new(WidgetId::manual(703))
            .range(0.0, 100.0)
            .value(50.0);
        assert!((s.ratio() - 0.5).abs() < 0.001);

        let s2 = Slider::new(WidgetId::manual(704))
            .range(10.0, 10.0)
            .value(10.0);
        assert!((s2.ratio() - 0.0).abs() < 0.001);
    }

    #[test]
    fn slider_set_value_clamps() {
        let mut s = Slider::new(WidgetId::manual(705)).range(0.0, 100.0);
        s.set_value(150.0);
        assert!((s.get_value() - 100.0).abs() < f32::EPSILON);
        s.set_value(-50.0);
        assert!((s.get_value() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slider_set_value_snaps() {
        let mut s = Slider::new(WidgetId::manual(706))
            .range(0.0, 100.0)
            .step(25.0);
        s.set_value(30.0);
        assert!((s.get_value() - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slider_drag_updates_value() {
        use std::sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        };
        let call_count = Arc::new(AtomicU32::new(0));
        let c2 = call_count.clone();

        let mut s = Slider::new(WidgetId::manual(707))
            .range(0.0, 100.0)
            .on_change(move |_v| {
                c2.fetch_add(1, Ordering::SeqCst);
            });

        // Widget at (0, 0), width 116 (so track_width = 116 - 16 = 100).
        let layout = LayoutNode::new(WidgetId::manual(707), 0, 0, 116, 24);
        let mut es = EventState::default();

        // Click at centre of widget -> should set value near 50.
        s.handle_event(
            &UiEvent::MouseDown { x: 58.0, y: 12.0 },
            &mut es,
            &layout,
        );
        assert!(s.is_dragging);
        let v = s.get_value();
        assert!(v > 40.0 && v < 60.0, "value = {v}");

        // Drag right.
        s.handle_event(
            &UiEvent::MouseMove {
                x: 108.0,
                y: 12.0,
            },
            &mut es,
            &layout,
        );
        let v2 = s.get_value();
        assert!(v2 > v, "v2={v2} should be > v={v}");

        // Release.
        s.handle_event(
            &UiEvent::MouseUp {
                x: 108.0,
                y: 12.0,
            },
            &mut es,
            &layout,
        );
        assert!(!s.is_dragging);

        assert!(call_count.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn slider_click_outside_does_not_start_drag() {
        let mut s = Slider::new(WidgetId::manual(708)).range(0.0, 100.0);
        let layout = LayoutNode::new(WidgetId::manual(708), 0, 0, 116, 24);
        let mut es = EventState::default();

        s.handle_event(
            &UiEvent::MouseDown {
                x: 200.0,
                y: 200.0,
            },
            &mut es,
            &layout,
        );
        assert!(!s.is_dragging);
    }

    #[test]
    fn slider_hover_changes_state() {
        let mut s = Slider::new(WidgetId::manual(709));
        let layout = LayoutNode::new(WidgetId::manual(709), 0, 0, 116, 24);
        let mut es = EventState::default();

        let changed = s.handle_event(
            &UiEvent::MouseMove { x: 50.0, y: 12.0 },
            &mut es,
            &layout,
        );
        assert!(changed);

        let changed = s.handle_event(
            &UiEvent::MouseMove {
                x: 200.0,
                y: 200.0,
            },
            &mut es,
            &layout,
        );
        assert!(changed);
    }
}
