//! A non-interactive text label widget.

use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode, LayoutStyle, Size, f32_to_i32, f32_to_u32},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// A read-only text label.
pub struct Text {
    id: WidgetId,
    /// The text content to display.
    pub content: String,
    /// Text colour.
    pub color: Color,
    /// Font size in pixels.
    pub font_size: f32,
    /// Layout style.
    pub style: LayoutStyle,
}

impl Text {
    /// Create a new text widget with an auto-generated widget ID.
    pub fn new_auto(content: impl Into<String>) -> Self {
        Self::new(next_widget_id(), content)
    }

    /// Create a new text widget with a specific widget ID.
    pub fn new(id: WidgetId, content: impl Into<String>) -> Self {
        Self {
            id,
            content: content.into(),
            color: Color::BLACK,
            font_size: 20.0,
            style: LayoutStyle::default(),
        }
    }

    fn desired_size(&self, constraints: BoxConstraints, fonts: &FontManager) -> Size {
        let max_w = constraints.max_width - self.style.padding.horizontal();
        let lines = if self.style.wrap_text {
            fonts.wrap_text(&self.content, max_w, self.font_size)
        } else {
            vec![self.content.clone()]
        };

        let mut max_line: f32 = 0.0;
        for line in &lines {
            let (w, _) = fonts.measure_text(line, self.font_size);
            max_line = max_line.max(w);
        }

        let lh = fonts.line_height(self.font_size);
        Size {
            width: max_line + self.style.padding.horizontal(),
            height: lh * lines.len() as f32 + self.style.padding.vertical(),
        }
    }
}

impl Widget for Text {
    fn id(&self) -> WidgetId { self.id }

    fn debug_name(&self) -> &str { "Text" }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode {
        let desired = constraints.constrain(self.desired_size(constraints, fonts));
        LayoutNode::new(self.id, x, y, f32_to_u32(desired.width), f32_to_u32(desired.height))
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        let rect = layout.bounds;
        let tx = rect.x + f32_to_i32(self.style.padding.left);
        let ty = rect.y + f32_to_i32(self.style.padding.top);
        let max_w = f32_to_u32((rect.width as f32 - self.style.padding.horizontal()).max(0.0));

        fonts.draw_text(canvas, &self.content, tx, ty, Some(max_w), self.font_size, self.color);
    }

    fn handle_event(
        &mut self,
        _event: &UiEvent,
        _state: &mut EventState,
        _layout: &LayoutNode,
    ) -> bool {
        false
    }
}
