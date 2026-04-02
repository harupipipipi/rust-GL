//! Minimal event types for the UI toolkit.

/// A high-level UI event produced from raw windowing events.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum UiEvent {
    /// The cursor moved to a new position.
    MouseMove { x: f32, y: f32 },
    /// A mouse button was pressed.
    MouseDown { x: f32, y: f32 },
    /// A mouse button was released.
    MouseUp   { x: f32, y: f32 },
}

/// Shared mutable state carried through the event-dispatch tree.
#[derive(Debug, Default)]
pub struct EventState {
    /// Widget currently under the cursor (if any).
    pub(crate) hovered: Option<crate::widgets::WidgetId>,
    /// Widget currently being pressed (if any).
    pub(crate) pressed: Option<crate::widgets::WidgetId>,
    /// Last known cursor position.
    pub(crate) cursor: (f32, f32),
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

    /// Returns the last known cursor position as `(x, y)`.
    pub fn cursor(&self) -> (f32, f32) {
        self.cursor
    }

    /// Returns the widget currently under the cursor, if any.
    pub fn hovered(&self) -> Option<crate::widgets::WidgetId> {
        self.hovered
    }

    /// Returns the widget currently being pressed, if any.
    pub fn pressed(&self) -> Option<crate::widgets::WidgetId> {
        self.pressed
    }

    /// Set the pressed widget (crate-internal).
    pub(crate) fn set_pressed(&mut self, id: Option<crate::widgets::WidgetId>) {
        self.pressed = id;
    }
}
