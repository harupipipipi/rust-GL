//! A container widget that lays children out vertically or horizontally.

use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    keyboard::KeyboardEvent,
    layout::{
        f32_to_i32, f32_to_u32, BoxConstraints, CrossAxisAlignment, EdgeInsets, LayoutDirection,
        LayoutNode, LayoutStyle, OverflowBehavior,
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
    fn id(&self) -> WidgetId {
        self.id
    }

    fn outer_margin(&self) -> EdgeInsets {
        self.style.margin
    }

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
        let gap = match self.style.direction {
            LayoutDirection::Overlay => 0.0,
            _ if self.children.is_empty() => 0.0,
            _ => self.style.gap * (self.children.len().saturating_sub(1) as f32),
        };

        let mut child_nodes: Vec<Option<LayoutNode>> = Vec::with_capacity(self.children.len());
        let mut total_flex = 0.0;
        let mut used_main = 0.0;
        let mut max_cross: f32 = 0.0;

        for child in &mut self.children {
            let margin = child.outer_margin();
            let flex = child.flex_factor().max(0.0);
            if flex > 0.0 && self.style.direction != LayoutDirection::Overlay {
                total_flex += flex;
                child_nodes.push(None);
                continue;
            }

            let cc = child_constraints(
                self.style.direction,
                self.style.align_items,
                available_cross(
                    self.style.direction,
                    inner_width,
                    inner_height_limit,
                    margin,
                ),
                available_main(
                    self.style.direction,
                    inner_width,
                    inner_height_limit,
                    margin,
                ),
                None,
            );
            let cn = child.layout(cc, 0, 0, fonts);
            used_main += occupied_main(self.style.direction, &cn, margin);
            max_cross = max_cross.max(occupied_cross(self.style.direction, &cn, margin));
            child_nodes.push(Some(cn));
        }

        let main_capacity = match self.style.direction {
            LayoutDirection::Vertical => inner_height_limit,
            LayoutDirection::Horizontal => inner_width,
            LayoutDirection::Overlay => 0.0,
        };
        let remaining_main = (main_capacity - used_main - gap).max(0.0);

        if total_flex > 0.0 {
            for (idx, child) in self.children.iter_mut().enumerate() {
                if child_nodes[idx].is_some() {
                    continue;
                }
                let margin = child.outer_margin();

                let share = remaining_main * (child.flex_factor().max(0.0) / total_flex);
                let cc = child_constraints(
                    self.style.direction,
                    self.style.align_items,
                    available_cross(
                        self.style.direction,
                        inner_width,
                        inner_height_limit,
                        margin,
                    ),
                    available_main(
                        self.style.direction,
                        inner_width,
                        inner_height_limit,
                        margin,
                    ),
                    Some((share - main_margin(self.style.direction, margin)).max(0.0)),
                );
                let cn = child.layout(cc, 0, 0, fonts);
                used_main += occupied_main(self.style.direction, &cn, margin);
                max_cross = max_cross.max(occupied_cross(self.style.direction, &cn, margin));
                child_nodes[idx] = Some(cn);
            }
        }

        let children_main = match self.style.direction {
            LayoutDirection::Overlay => max_cross,
            _ => used_main + gap,
        };
        let unclamped_height = match self.style.direction {
            LayoutDirection::Vertical => children_main + self.style.padding.vertical(),
            LayoutDirection::Horizontal => max_cross + self.style.padding.vertical(),
            LayoutDirection::Overlay => children_main + self.style.padding.vertical(),
        };
        let height = unclamped_height.clamp(constraints.min_height, constraints.max_height);

        let mut node = LayoutNode::new(self.id, x, y, f32_to_u32(width), f32_to_u32(height));
        let inner_height = (height - self.style.padding.vertical()).max(0.0);
        let mut cursor_main = 0.0;

        for (child, maybe_child_node) in self.children.iter().zip(child_nodes.into_iter()) {
            let Some(mut child_node) = maybe_child_node else {
                continue;
            };
            let margin = child.outer_margin();
            let child_main = main_extent(self.style.direction, &child_node);
            let occupied_cross = occupied_cross(self.style.direction, &child_node, margin);
            let (child_x, child_y) = match self.style.direction {
                LayoutDirection::Vertical => {
                    let cross_x =
                        aligned_cross_offset(self.style.align_items, inner_width, occupied_cross);
                    (
                        x + f32_to_i32(self.style.padding.left + cross_x + margin.left),
                        y + f32_to_i32(self.style.padding.top + cursor_main + margin.top),
                    )
                }
                LayoutDirection::Horizontal => {
                    let cross_y =
                        aligned_cross_offset(self.style.align_items, inner_height, occupied_cross);
                    (
                        x + f32_to_i32(self.style.padding.left + cursor_main + margin.left),
                        y + f32_to_i32(self.style.padding.top + cross_y + margin.top),
                    )
                }
                LayoutDirection::Overlay => (
                    x + f32_to_i32(self.style.padding.left + margin.left),
                    y + f32_to_i32(self.style.padding.top + margin.top),
                ),
            };
            let dx = child_x - child_node.bounds.x;
            let dy = child_y - child_node.bounds.y;
            offset_layout_node(&mut child_node, dx, dy);
            node.add_child(child_node);
            if self.style.direction != LayoutDirection::Overlay {
                cursor_main +=
                    child_main + main_margin(self.style.direction, margin) + self.style.gap;
            }
        }

        node
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        if let Some(bg) = self.background {
            canvas.fill_rect(layout.bounds, bg);
        }

        let previous_clip = if self.style.overflow == OverflowBehavior::Clip {
            Some(canvas.replace_clip_rect(Some(match canvas.clip_rect() {
                Some(current) => current.intersect(&layout.bounds),
                None => layout.bounds,
            })))
        } else {
            None
        };

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

        if let Some(previous_clip) = previous_clip {
            canvas.replace_clip_rect(previous_clip);
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

        if self.style.overflow == OverflowBehavior::Clip
            && matches!(*event, UiEvent::MouseDown { x, y } if !layout.bounds.contains(x, y))
        {
            return false;
        }

        let len = self.children.len().min(layout.children.len());
        for i in 0..len {
            changed |= self.children[i].handle_event(event, state, &layout.children[i]);
        }
        changed
    }

    fn handle_keyboard_event(
        &mut self,
        event: &KeyboardEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let mut changed = false;

        let len = self.children.len().min(layout.children.len());
        for i in 0..len {
            changed |= self.children[i].handle_keyboard_event(event, state, &layout.children[i]);
        }

        if changed {
            state.request_redraw();
        }
        changed
    }
}

fn child_constraints(
    direction: LayoutDirection,
    align_items: CrossAxisAlignment,
    available_cross: f32,
    available_main: f32,
    flex_main: Option<f32>,
) -> BoxConstraints {
    match direction {
        LayoutDirection::Vertical => BoxConstraints {
            min_width: if align_items == CrossAxisAlignment::Stretch {
                available_cross
            } else {
                0.0
            },
            max_width: available_cross,
            min_height: flex_main.unwrap_or(0.0),
            max_height: flex_main.unwrap_or(available_main),
        },
        LayoutDirection::Horizontal => BoxConstraints {
            min_width: flex_main.unwrap_or(0.0),
            max_width: flex_main.unwrap_or(available_main),
            min_height: if align_items == CrossAxisAlignment::Stretch {
                available_cross
            } else {
                0.0
            },
            max_height: available_cross,
        },
        LayoutDirection::Overlay => BoxConstraints {
            min_width: 0.0,
            max_width: available_cross,
            min_height: 0.0,
            max_height: available_main,
        },
    }
}

fn main_extent(direction: LayoutDirection, node: &LayoutNode) -> f32 {
    match direction {
        LayoutDirection::Vertical => node.bounds.height as f32,
        LayoutDirection::Horizontal => node.bounds.width as f32,
        LayoutDirection::Overlay => node.bounds.height as f32,
    }
}

fn cross_extent(direction: LayoutDirection, node: &LayoutNode) -> f32 {
    match direction {
        LayoutDirection::Vertical => node.bounds.width as f32,
        LayoutDirection::Horizontal => node.bounds.height as f32,
        LayoutDirection::Overlay => node.bounds.height as f32,
    }
}

fn main_margin(direction: LayoutDirection, margin: EdgeInsets) -> f32 {
    match direction {
        LayoutDirection::Vertical | LayoutDirection::Overlay => margin.vertical(),
        LayoutDirection::Horizontal => margin.horizontal(),
    }
}

fn cross_margin(direction: LayoutDirection, margin: EdgeInsets) -> f32 {
    match direction {
        LayoutDirection::Vertical => margin.horizontal(),
        LayoutDirection::Horizontal | LayoutDirection::Overlay => margin.vertical(),
    }
}

fn occupied_main(direction: LayoutDirection, node: &LayoutNode, margin: EdgeInsets) -> f32 {
    main_extent(direction, node) + main_margin(direction, margin)
}

fn occupied_cross(direction: LayoutDirection, node: &LayoutNode, margin: EdgeInsets) -> f32 {
    cross_extent(direction, node) + cross_margin(direction, margin)
}

fn available_main(
    direction: LayoutDirection,
    inner_width: f32,
    inner_height_limit: f32,
    margin: EdgeInsets,
) -> f32 {
    match direction {
        LayoutDirection::Vertical => (inner_height_limit - margin.vertical()).max(0.0),
        LayoutDirection::Overlay => (inner_height_limit - margin.vertical()).max(0.0),
        LayoutDirection::Horizontal => (inner_width - margin.horizontal()).max(0.0),
    }
}

fn available_cross(
    direction: LayoutDirection,
    inner_width: f32,
    inner_height_limit: f32,
    margin: EdgeInsets,
) -> f32 {
    match direction {
        LayoutDirection::Vertical => (inner_width - margin.horizontal()).max(0.0),
        LayoutDirection::Horizontal => (inner_height_limit - margin.vertical()).max(0.0),
        LayoutDirection::Overlay => (inner_width - margin.horizontal()).max(0.0),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{canvas::Rect, widgets::Widget};

    struct OverflowPaint {
        id: WidgetId,
    }

    impl OverflowPaint {
        fn new(id: WidgetId) -> Self {
            Self { id }
        }
    }

    impl Widget for OverflowPaint {
        fn id(&self) -> WidgetId {
            self.id
        }

        fn layout(
            &mut self,
            _constraints: BoxConstraints,
            x: i32,
            y: i32,
            _fonts: &FontManager,
        ) -> LayoutNode {
            LayoutNode::new(self.id, x, y, 20, 20)
        }

        fn draw(&self, canvas: &mut Canvas, _layout: &LayoutNode, _fonts: &FontManager) {
            canvas.fill_rect(Rect::new(0, 0, 40, 40), Color::BLACK);
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

    fn try_fonts() -> Option<FontManager> {
        FontManager::new().ok()
    }

    #[test]
    fn container_clips_child_overflow_by_default() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut container = Container::new(WidgetId::manual(1));
        container.background = Some(Color::WHITE);
        container.push(OverflowPaint::new(WidgetId::manual(2)));
        let mut layout = LayoutNode::new(container.id(), 10, 10, 20, 20);
        layout.add_child(LayoutNode::new(WidgetId::manual(2), 10, 10, 20, 20));

        let mut canvas = Canvas::new(60, 60);
        canvas.clear(Color::WHITE);
        container.draw(&mut canvas, &layout, &fm);

        assert_eq!(canvas.pixels()[0], Color::WHITE.to_u32());
        assert_eq!(canvas.pixels()[10 * 60 + 10], Color::BLACK.to_u32());
    }

    #[test]
    fn container_can_allow_visible_overflow() {
        let fm = match try_fonts() {
            Some(f) => f,
            None => return,
        };
        let mut container = Container::new(WidgetId::manual(1));
        container.style.overflow = OverflowBehavior::Visible;
        container.background = None;
        container.push(OverflowPaint::new(WidgetId::manual(2)));
        let mut layout = LayoutNode::new(container.id(), 10, 10, 20, 20);
        layout.add_child(LayoutNode::new(WidgetId::manual(2), 10, 10, 20, 20));

        let mut canvas = Canvas::new(60, 60);
        canvas.clear(Color::WHITE);
        container.draw(&mut canvas, &layout, &fm);

        assert_eq!(canvas.pixels()[0], Color::BLACK.to_u32());
    }
}
