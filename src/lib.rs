pub mod app;
pub mod canvas;
pub mod event;
pub mod layout;
pub mod text;
pub mod widgets;

pub use app::{run, App};
pub use canvas::{Canvas, Color, Rect};
pub use event::{EventState, UiEvent};
pub use layout::{BoxConstraints, EdgeInsets, LayoutDirection, LayoutNode, LayoutStyle, Size};
pub use text::FontManager;
pub use widgets::{button::Button, container::Container, text::Text, Widget};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_and_color_work_as_public_api() {
        let mut canvas = Canvas::new(4, 4);
        canvas.clear(Color::WHITE);
        assert_eq!(canvas.pixels().len(), 16);
        assert!(canvas.pixels().iter().all(|p| *p == Color::WHITE.to_u32()));

        let rect = Rect::new(1, 1, 2, 2);
        canvas.fill_rect(rect, Color::BLACK);

        assert_eq!(canvas.pixels()[0], Color::WHITE.to_u32());
        assert_eq!(canvas.pixels()[5], Color::BLACK.to_u32());
        assert_eq!(canvas.pixels()[10], Color::BLACK.to_u32());
    }

    #[test]
    fn layout_node_find_by_id_works() {
        let mut root = LayoutNode::new(1, 0, 0, 100, 100);
        let mut child = LayoutNode::new(2, 0, 0, 50, 50);
        child.add_child(LayoutNode::new(3, 10, 10, 20, 20));
        root.add_child(child);

        assert!(root.find_by_id(1).is_some());
        assert!(root.find_by_id(3).is_some());
        assert!(root.find_by_id(999).is_none());
    }

    #[test]
    fn auto_generated_widget_ids_are_unique() {
        let a = widgets::next_widget_id();
        let b = widgets::next_widget_id();
        assert_ne!(a, b);
    }

    #[test]
    fn auto_widget_constructors_produce_distinct_ids() {
        let text = Text::new_auto("a");
        let button = Button::new_auto("b");
        assert_ne!(text.id(), button.id());
    }
}
