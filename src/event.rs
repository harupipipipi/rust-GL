//! Minimal event types for the UI toolkit.

/// A high-level UI event produced from raw windowing events.
#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32 },
    MouseUp   { x: f32, y: f32 },
}

/// Shared mutable state carried through the event-dispatch tree.
#[derive(Debug, Default)]
pub struct EventState {
    /// Widget currently under the cursor (if any).
    pub hovered: Option<crate::widgets::WidgetId>,
    /// Widget currently being pressed (if any).
    pub pressed: Option<crate::widgets::WidgetId>,
    /// Last known cursor position.
    pub cursor: (f32, f32),
    /// Whether any widget requested a repaint since the last frame.
    needs_redraw: bool,
}

impl EventState {
    /// Mark that a repaint is needed.
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Consume the redraw flag (returns `true` at most once per frame).
    pub fn take_needs_redraw(&mut self) -> bool {
        let v = self.needs_redraw;
        self.needs_redraw = false;
        v
    }
}
