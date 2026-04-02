//! A container widget that lays children out vertically or horizontally.

use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{f32_to_i32, f32_to_u32, BoxConstraints, LayoutDirection, LayoutNode, LayoutStyle},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// A box that lays out child widgets in a given direction.
pub struct Container {
    id: WidgetId,
    /// Layout style (padding, gap, direction, etc.).
    pub style: LayoutStyle,
    /// Optional background colour.
    pub background: Option<Color>,
    /// Child widgets.
    pub children: Vec<Box<dyn Widget>>,
}

impl Container {
    /// Create a new container with an auto-generated widget ID.
    pub fn new_auto() -> Self {
        Self::new(next_widget_id())
    }

    /// Create a new container with a specific widget ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            style: LayoutStyle::default(),
            background: Some(Color::GRAY_100),
            children: Vec::new(),
        }
    }

    /// Builder method: add a child and return `self`.
    pub fn with_child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Add a child widget.
    pub fn push(&mut self, child: impl Widget + 'static) {
        self.children.push(Box::new(child));
    }
}

impl Widget for Container {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn debug_name(&self) -> &str {
        "Container"
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode {
        let width = constraints.max_width.max(constraints.min_width);
        // Accumulate in f32 to avoid rounding-error drift; convert to i32
        // only when passing the origin to child.layout().
        let mut cx_f = x as f32 + self.style.padding.left;
        let mut cy_f = y as f32 + self.style.padding.top;

        let mut node = LayoutNode::new(self.id, x, y, f32_to_u32(width), 0);
        let child_max_w = (width - self.style.padding.horizontal()).max(0.0);
        let gap = self.style.gap;
        let mut used_main: f32 = 0.0;
        let mut max_cross: f32 = 0.0;
        let n = self.children.len();

        for (i, child) in self.children.iter_mut().enumerate() {
            let cc = BoxConstraints {
                min_width: 0.0,
                max_width: child_max_w,
                min_height: 0.0,
                max_height: constraints.max_height.max(0.0),
            };

            let cn = child.layout(cc, f32_to_i32(cx_f), f32_to_i32(cy_f), fonts);
            let cw = cn.bounds.width as f32;
            let ch = cn.bounds.height as f32;

            let g = if i + 1 == n { 0.0 } else { gap };

            match self.style.direction {
                LayoutDirection::Vertical => {
                    cy_f += ch + g;
                    used_main += ch + g;
                    max_cross = max_cross.max(cw);
                }
                LayoutDirection::Horizontal => {
                    cx_f += cw + g;
                    used_main += cw + g;
                    max_cross = max_cross.max(ch);
                }
            }

            node.add_child(cn);
        }

        let total_h = match self.style.direction {
            LayoutDirection::Vertical => used_main + self.style.padding.vertical(),
            LayoutDirection::Horizontal => max_cross + self.style.padding.vertical(),
        };

        node.bounds.height =
            f32_to_u32(total_h.clamp(constraints.min_height, constraints.max_height));
        node
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        if let Some(bg) = self.background {
            canvas.fill_rect(layout.bounds, bg);
        }

        debug_assert_eq!(
            self.children.len(),
            layout.children.len(),
            "Container({:?}): child count mismatch in draw",
            self.id,
        );

        let len = self.children.len().min(layout.children.len());
        for i in 0..len {
            self.children[i].draw(canvas, &layout.children[i], fonts);
        }
    }

    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let mut changed = false;

        debug_assert_eq!(
            self.children.len(),
            layout.children.len(),
            "Container({:?}): child count mismatch in handle_event",
            self.id,
        );

        let len = self.children.len().min(layout.children.len());
        for i in 0..len {
            changed |= self.children[i].handle_event(event, state, &layout.children[i]);
        }
        changed
    }
}
