//! Widget trait and ID management.
//!
//! Auto-generated IDs start at `1 << 32` so they never collide with
//! small manual IDs.

pub mod button;
pub mod container;
pub mod text;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    canvas::Canvas,
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode},
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
    /// Returns the unique ID of this widget.
    fn id(&self) -> WidgetId;

    /// Human-readable name for debugging purposes.
    fn debug_name(&self) -> &str {
        "Widget"
    }

    /// Compute layout given constraints and an origin position.
    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode;

    /// Draw this widget onto the canvas using the computed layout.
    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager);

    /// Handle a UI event, returning `true` if state changed.
    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool;
}
