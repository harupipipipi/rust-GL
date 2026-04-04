//! A container widget that lays children out vertically or horizontally.

use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{
        BoxConstraints, CrossAxisAlignment, LayoutDirection, LayoutNode, LayoutStyle,
        f32_to_i32, f32_to_u32,
    },
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
    fn id(&self) -> WidgetId { self.id }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode {
        let width = constraints.max_width.max(constraints.min_width);
        let inner_width = (width - self.style.padding.horizontal()).max(0.0);
        let inner_height_limit = (constraints.max_height - self.style.padding.vertical()).max(0.0);
        let gap = if self.children.is_empty() {
            0.0
        } else {
            self.style.gap * (self.children.len().saturating_sub(1) as f32)
        };

        let mut child_nodes: Vec<Option<LayoutNode>> = Vec::with_capacity(self.children.len());
        let mut total_flex = 0.0;
        let mut used_main = 0.0;
        let mut max_cross: f32 = 0.0;

        for child in &mut self.children {
            let flex = child.flex_factor().max(0.0);
            if flex > 0.0 {
                total_flex += flex;
                child_nodes.push(None);
                continue;
            }

            let cc = child_constraints(
                self.style.direction,
                self.style.align_items,
                inner_width,
                inner_height_limit,
                None,
            );
            let cn = child.layout(cc, 0, 0, fonts);
            used_main += main_extent(self.style.direction, &cn);
            max_cross = max_cross.max(cross_extent(self.style.direction, &cn));
            child_nodes.push(Some(cn));
        }

        let main_capacity = match self.style.direction {
            LayoutDirection::Vertical => inner_height_limit,
            LayoutDirection::Horizontal => inner_width,
        };
        let remaining_main = (main_capacity - used_main - gap).max(0.0);

        if total_flex > 0.0 {
            for (idx, child) in self.children.iter_mut().enumerate() {
                if child_nodes[idx].is_some() {
                    continue;
                }

                let share = remaining_main * (child.flex_factor().max(0.0) / total_flex);
                let cc = child_constraints(
                    self.style.direction,
                    self.style.align_items,
                    inner_width,
                    inner_height_limit,
                    Some(share),
                );
                let cn = child.layout(cc, 0, 0, fonts);
                used_main += main_extent(self.style.direction, &cn);
                max_cross = max_cross.max(cross_extent(self.style.direction, &cn));
                child_nodes[idx] = Some(cn);
            }
        }

        let children_main = used_main + gap;
        let unclamped_height = match self.style.direction {
            LayoutDirection::Vertical => children_main + self.style.padding.vertical(),
            LayoutDirection::Horizontal => max_cross + self.style.padding.vertical(),
        };
        let height = unclamped_height.clamp(constraints.min_height, constraints.max_height);

        let mut node = LayoutNode::new(self.id, x, y, f32_to_u32(width), f32_to_u32(height));
        let inner_height = (height - self.style.padding.vertical()).max(0.0);
        let mut cursor_main = 0.0;

        for mut child_node in child_nodes.into_iter().flatten() {
            let child_main = main_extent(self.style.direction, &child_node);
            let child_cross = cross_extent(self.style.direction, &child_node);
            let (child_x, child_y) = match self.style.direction {
                LayoutDirection::Vertical => {
                    let cross_x = aligned_cross_offset(
                        self.style.align_items,
                        inner_width,
                        child_cross,
                    );
                    (
                        x + f32_to_i32(self.style.padding.left + cross_x),
                        y + f32_to_i32(self.style.padding.top + cursor_main),
                    )
                }
                LayoutDirection::Horizontal => {
                    let cross_y = aligned_cross_offset(
                        self.style.align_items,
                        inner_height,
                        child_cross,
                    );
                    (
                        x + f32_to_i32(self.style.padding.left + cursor_main),
                        y + f32_to_i32(self.style.padding.top + cross_y),
                    )
                }
            };
            let dx = child_x - child_node.bounds.x;
            let dy = child_y - child_node.bounds.y;
            offset_layout_node(
                &mut child_node,
                dx,
                dy,
            );
            node.add_child(child_node);
            cursor_main += child_main + self.style.gap;
        }

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

fn child_constraints(
    direction: LayoutDirection,
    align_items: CrossAxisAlignment,
    inner_width: f32,
    inner_height_limit: f32,
    flex_main: Option<f32>,
) -> BoxConstraints {
    match direction {
        LayoutDirection::Vertical => BoxConstraints {
            min_width: if align_items == CrossAxisAlignment::Stretch {
                inner_width
            } else {
                0.0
            },
            max_width: inner_width,
            min_height: flex_main.unwrap_or(0.0),
            max_height: flex_main.unwrap_or(inner_height_limit),
        },
        LayoutDirection::Horizontal => BoxConstraints {
            min_width: flex_main.unwrap_or(0.0),
            max_width: flex_main.unwrap_or(inner_width),
            min_height: if align_items == CrossAxisAlignment::Stretch {
                inner_height_limit
            } else {
                0.0
            },
            max_height: inner_height_limit,
        },
    }
}

fn main_extent(direction: LayoutDirection, node: &LayoutNode) -> f32 {
    match direction {
        LayoutDirection::Vertical => node.bounds.height as f32,
        LayoutDirection::Horizontal => node.bounds.width as f32,
    }
}

fn cross_extent(direction: LayoutDirection, node: &LayoutNode) -> f32 {
    match direction {
        LayoutDirection::Vertical => node.bounds.width as f32,
        LayoutDirection::Horizontal => node.bounds.height as f32,
    }
}

fn aligned_cross_offset(align_items: CrossAxisAlignment, available: f32, child: f32) -> f32 {
    match align_items {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
        CrossAxisAlignment::Center => ((available - child) * 0.5).max(0.0),
        CrossAxisAlignment::End => (available - child).max(0.0),
    }
}

fn offset_layout_node(node: &mut LayoutNode, dx: i32, dy: i32) {
    node.bounds.x += dx;
    node.bounds.y += dy;
    for child in &mut node.children {
        offset_layout_node(child, dx, dy);
    }
}
