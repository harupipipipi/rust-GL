//! A clickable button widget with hover / pressed states.

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    layout::{f32_to_i32, f32_to_u32, BoxConstraints, EdgeInsets, LayoutNode, LayoutStyle, Size},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

/// A clickable button that displays a text label.
pub struct Button {
    id: WidgetId,
    /// The label displayed on the button.
    pub label: String,
    /// Layout style (padding, gap, etc.).
    pub style: LayoutStyle,
    /// Font size in pixels.
    pub font_size: f32,
    /// Background colour in normal state.
    pub normal_bg: Color,
    /// Background colour when hovered.
    pub hover_bg: Color,
    /// Background colour when pressed.
    pub pressed_bg: Color,
    /// Text colour.
    pub text_color: Color,
    is_hovered: bool,
    is_pressed: bool,
    on_click: Option<Box<dyn FnMut()>>,
}

impl Button {
    /// Create a new button with an auto-generated widget ID.
    pub fn new_auto(label: impl Into<String>) -> Self {
        Self::new(next_widget_id(), label)
    }

    /// Create a new button with a specific widget ID.
    pub fn new(id: WidgetId, label: impl Into<String>) -> Self {
        let style = LayoutStyle {
            padding: EdgeInsets::all(10.0),
            wrap_text: false,
            ..LayoutStyle::default()
        };
        Self {
            id,
            label: label.into(),
            style,
            font_size: 18.0,
            normal_bg: Color::GRAY_300,
            hover_bg: Color::rgba(190, 210, 255, 255),
            pressed_bg: Color::BLUE,
            text_color: Color::BLACK,
            is_hovered: false,
            is_pressed: false,
            on_click: None,
        }
    }

    /// Attach a click callback. Returns `self` for builder-style chaining.
    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    fn desired_size(&self, constraints: BoxConstraints, fonts: &FontManager) -> Size {
        let (w, h) = fonts.measure_text(&self.label, self.font_size);
        constraints.constrain(Size {
            width: w + self.style.padding.horizontal() + 10.0,
            height: h + self.style.padding.vertical() + 6.0,
        })
    }
}

impl Widget for Button {
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
        let size = self.desired_size(constraints, fonts);
        LayoutNode::new(
            self.id,
            x,
            y,
            f32_to_u32(size.width),
            f32_to_u32(size.height),
        )
    }

    fn draw(&self, canvas: &mut Canvas, layout: &LayoutNode, fonts: &FontManager) {
        let rect = layout.bounds;
        let bg = if self.is_pressed {
            self.pressed_bg
        } else if self.is_hovered {
            self.hover_bg
        } else {
            self.normal_bg
        };

        canvas.draw_rounded_rect(rect, 8, bg);
        if rect.width > 0 {
            canvas.draw_line(
                rect.x,
                rect.y,
                rect.x + rect.width as i32 - 1,
                rect.y,
                Color::GRAY_600,
            );
        }

        let text_y = rect.y + f32_to_i32(rect.height as f32 * 0.5 - self.font_size * 0.5);
        fonts.draw_text_in_rect(
            canvas,
            &self.label,
            Rect::new(
                rect.x + f32_to_i32(self.style.padding.left),
                text_y,
                f32_to_u32((rect.width as f32 - self.style.padding.horizontal()).max(0.0)),
                rect.height,
            ),
            self.font_size,
            self.text_color,
        );
    }

    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let mut changed = false;
        let rect = layout.bounds;

        match *event {
            UiEvent::MouseMove { x, y } => {
                let hovered = rect.contains(x, y);
                if hovered != self.is_hovered {
                    self.is_hovered = hovered;
                    changed = true;
                }
            }
            UiEvent::MouseDown { x, y } => {
                if rect.contains(x, y) {
                    self.is_pressed = true;
                    state.set_pressed(Some(self.id));
                    changed = true;
                }
            }
            UiEvent::MouseUp { x, y } => {
                let was = self.is_pressed;
                self.is_pressed = false;
                if was && rect.contains(x, y) {
                    if let Some(cb) = self.on_click.as_mut() {
                        cb();
                    }
                }
                if was {
                    changed = true;
                }
            }
        }

        if changed {
            state.request_redraw();
        }
        changed
    }
}
