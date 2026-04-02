use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutDirection, LayoutNode, LayoutStyle, f32_to_i32, f32_to_u32},
    text::FontManager,
    widgets::{next_widget_id, Widget},
};

pub struct Container {
    id: u64,
    pub style: LayoutStyle,
    pub background: Option<Color>,
    pub children: Vec<Box<dyn Widget + Send>>,
}

impl Container {
    pub fn new_auto() -> Self {
        Self::new(next_widget_id())
    }

    pub fn new(id: u64) -> Self {
        Self {
            id,
            style: LayoutStyle::default(),
            background: Some(Color::GRAY_100),
            children: Vec::new(),
        }
    }

    pub fn with_child(mut self, child: impl Widget + Send + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn push(&mut self, child: impl Widget + Send + 'static) {
        self.children.push(Box::new(child));
    }
}

impl Widget for Container {
    fn id(&self) -> u64 {
        self.id
    }

    fn layout(&mut self, constraints: BoxConstraints, x: i32, y: i32, fonts: &FontManager) -> LayoutNode {
        let width = constraints.max_width.max(constraints.min_width);
        let mut cursor_x = x + f32_to_i32(self.style.padding.left);
        let mut cursor_y = y + f32_to_i32(self.style.padding.top);

        // Start with zero height — we will compute the real height from
        // children and then clamp to constraints.
        let mut node = LayoutNode::new(self.id, x, y, f32_to_u32(width), 0);
        let child_max_width = width - self.style.padding.horizontal();
        let mut used_main: f32 = 0.0;
        let mut max_cross: f32 = 0.0;

        for child in self.children.iter_mut() {
            let child_constraints = BoxConstraints {
                min_width: 0.0,
                max_width: child_max_width.max(0.0),
                min_height: 0.0,
                max_height: constraints.max_height.max(0.0),
            };

            let child_node = child.layout(child_constraints, cursor_x, cursor_y, fonts);
            let child_w = child_node.bounds.width as f32;
            let child_h = child_node.bounds.height as f32;

            match self.style.direction {
                LayoutDirection::Vertical => {
                    cursor_y += child_h.round() as i32 + self.style.gap.round() as i32;
                    used_main += child_h + self.style.gap;
                    max_cross = max_cross.max(child_w);
                }
                LayoutDirection::Horizontal => {
                    cursor_x += child_w.round() as i32 + self.style.gap.round() as i32;
                    used_main += child_w + self.style.gap;
                    max_cross = max_cross.max(child_h);
                }
            }

            node.add_child(child_node);
        }

        let content_main = (used_main - self.style.gap).max(0.0);
        let total_height = match self.style.direction {
            LayoutDirection::Vertical => content_main + self.style.padding.vertical(),
            LayoutDirection::Horizontal => max_cross + self.style.padding.vertical(),
        };

        node.bounds.height = f32_to_u32(total_height.clamp(constraints.min_height, constraints.max_height));
        node
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        if let Some(bg) = self.background {
            canvas.fill_rect(layout.bounds, bg);
        }

        for (child, layout_node) in self.children.iter().zip(layout.children.iter()) {
            child.draw(canvas, layout_node, fonts);
        }
    }

    fn handle_event(&mut self, event: &UiEvent, state: &mut EventState, layout: &LayoutNode) -> bool {
        let mut changed = false;
        for (child, layout_node) in self.children.iter_mut().zip(layout.children.iter()) {
            changed |= child.handle_event(event, state, layout_node);
        }
        changed
    }
}
