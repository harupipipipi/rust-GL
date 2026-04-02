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

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_widget_id() -> u64 {
    NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed)
}

pub trait Widget {
    fn id(&self) -> u64;
    fn layout(&mut self, constraints: BoxConstraints, x: i32, y: i32, fonts: &FontManager) -> LayoutNode;
    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager);
    fn handle_event(&mut self, event: &UiEvent, state: &mut EventState, layout: &LayoutNode) -> bool;
}
