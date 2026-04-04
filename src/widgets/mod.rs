//! Widget trait and ID management.
//!
//! Auto-generated IDs start at `1 << 32` so they never collide with
//! small manual IDs.

pub mod button;
pub mod checkbox;
pub mod container;
pub mod divider;
pub mod radio;
pub mod scroll;
pub mod slider;
pub mod spacer;
pub mod text;
pub mod text_input;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    canvas::Canvas,
    event::{EventState, UiEvent},
    keyboard::KeyboardEvent,
    layout::{BoxConstraints, EdgeInsets, LayoutNode},
    text::FontManager,
};

const AUTO_ID_BASE: u64 = 1 << 32;

static NEXT_AUTO_ID: AtomicU64 = AtomicU64::new(AUTO_ID_BASE);

/// Generate a new unique widget ID in the auto range.
pub fn next_widget_id() -> WidgetId {
    WidgetId(NEXT_AUTO_ID.fetch_add(1, Ordering::Relaxed))
}

/// Opaque, `Copy`-able identifier for a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a manually chosen ID. Keep `id < 1 << 32` by convention.
    pub const fn manual(id: u64) -> Self {
        Self(id)
    }

    /// The raw `u64` value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// The core trait implemented by every UI widget.
pub trait Widget {
    /// Return this widget's stable identifier.
    fn id(&self) -> WidgetId;

    /// Compute the widget layout for the given constraints and origin.
    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode;

    /// Paint the widget into the provided canvas.
    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager);

    /// Handle a pointer event. Returns `true` when widget state changed.
    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool;

    /// Handle a keyboard event. Returns `true` when widget state changed.
    fn handle_keyboard_event(
        &mut self,
        _event: &KeyboardEvent,
        _state: &mut EventState,
        _layout: &LayoutNode,
    ) -> bool {
        false
    }

    /// Return the widget flex factor used by containers for main-axis growth.
    fn flex_factor(&self) -> f32 {
        0.0
    }

    /// Outer margin used by parent containers during layout.
    fn outer_margin(&self) -> EdgeInsets {
        EdgeInsets::default()
    }

    /// Human-readable widget name for debugging output.
    fn debug_name(&self) -> &str {
        "Widget"
    }
}
