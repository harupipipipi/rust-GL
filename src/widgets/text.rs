use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutNode, LayoutStyle, Size},
    text::FontManager,
    widgets::Widget,
};

pub struct Text {
    id: u64,
    pub content: String,
    pub color: Color,
    pub font_size: f32,
    pub style: LayoutStyle,
}

impl Text {
    pub fn new(id: u64, content: impl Into<String>) -> Self {
        Self {
            id,
            content: content.into(),
            color: Color::BLACK,
            font_size: 20.0,
            style: LayoutStyle::default(),
        }
    }

    fn desired_size(&self, constraints: BoxConstraints, fonts: &FontManager) -> Size {
        let max_width = constraints.max_width - self.style.padding.horizontal();
        let lines = if self.style.wrap_text {
            fonts.wrap_text(&self.content, max_width, self.font_size)
        } else {
            vec![self.content.clone()]
        };

        let mut max_line = 0.0;
        for line in &lines {
            let (w, _) = fonts.measure_text(line, self.font_size);
            max_line = max_line.max(w);
        }

        let line_h = self.font_size * 1.3;
        Size {
            width: max_line + self.style.padding.horizontal(),
            height: line_h * lines.len() as f32 + self.style.padding.vertical(),
        }
    }
}

impl Widget for Text {
    fn id(&self) -> u64 {
        self.id
    }

    fn layout(&mut self, constraints: BoxConstraints, x: i32, y: i32, fonts: &FontManager) -> LayoutNode {
        let desired = constraints.constrain(self.desired_size(constraints, fonts));
        LayoutNode::new(self.id, x, y, desired.width as u32, desired.height as u32)
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        let rect = layout.bounds;
        let tx = rect.x + self.style.padding.left as i32;
        let ty = rect.y + self.style.padding.top as i32;
        let max_w = (rect.width as f32 - self.style.padding.horizontal()).max(0.0) as u32;

        fonts.draw_text(
            canvas,
            &self.content,
            tx,
            ty,
            Some(max_w),
            self.font_size,
            self.color,
        );
    }

    fn handle_event(&mut self, _event: &UiEvent, _state: &mut EventState, _layout: &LayoutNode) -> bool {
        false
    }
}
