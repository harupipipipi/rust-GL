pub mod button;
pub mod container;
pub mod text;

use crate::{
    canvas::Canvas,
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode},
    text::FontManager,
};

pub trait Widget {
    fn id(&self) -> u64;
    fn layout(&mut self, constraints: BoxConstraints, x: i32, y: i32, fonts: &FontManager) -> LayoutNode;
    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager);
    fn handle_event(&mut self, event: &UiEvent, state: &mut EventState, layout: &LayoutNode) -> bool;
}
