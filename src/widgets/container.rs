use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, LayoutDirection, LayoutNode, LayoutStyle},
    text::FontManager,
    widgets::Widget,
};

pub struct Container {
    id: u64,
    pub style: LayoutStyle,
    pub background: Option<Color>,
    pub children: Vec<Box<dyn Widget + Send>>,
}

impl Container {
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
        let mut cursor_x = x + self.style.padding.left as i32;
        let mut cursor_y = y + self.style.padding.top as i32;

        let mut node = LayoutNode::new(self.id, x, y, width as u32, constraints.max_height as u32);
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
                    cursor_y += child_h as i32 + self.style.gap as i32;
                    used_main += child_h + self.style.gap;
                    max_cross = max_cross.max(child_w);
                }
                LayoutDirection::Horizontal => {
                    cursor_x += child_w as i32 + self.style.gap as i32;
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

        node.bounds.height = total_height.clamp(constraints.min_height, constraints.max_height) as u32;
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
