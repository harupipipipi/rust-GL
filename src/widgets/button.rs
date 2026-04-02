use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::{BoxConstraints, EdgeInsets, LayoutNode, LayoutStyle, Size},
    text::FontManager,
    widgets::{next_widget_id, Widget},
};

pub struct Button {
    id: u64,
    pub label: String,
    pub style: LayoutStyle,
    pub font_size: f32,
    pub normal_bg: Color,
    pub hover_bg: Color,
    pub pressed_bg: Color,
    pub text_color: Color,
    is_hovered: bool,
    is_pressed: bool,
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl Button {
    pub fn new_auto(label: impl Into<String>) -> Self {
        Self::new(next_widget_id(), label)
    }

    pub fn new(id: u64, label: impl Into<String>) -> Self {
        let mut style = LayoutStyle::default();
        style.padding = EdgeInsets::all(10.0);
        style.wrap_text = false;
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

    pub fn on_click(mut self, f: impl FnMut() + Send + 'static) -> Self {
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
    fn id(&self) -> u64 {
        self.id
    }

    fn layout(&mut self, constraints: BoxConstraints, x: i32, y: i32, fonts: &FontManager) -> LayoutNode {
        let size = self.desired_size(constraints, fonts);
        LayoutNode::new(self.id, x, y, size.width as u32, size.height as u32)
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
        canvas.draw_line(rect.x, rect.y, rect.x + rect.width as i32 - 1, rect.y, Color::GRAY_600);

        let text_y = rect.y + (rect.height as f32 * 0.5 - self.font_size * 0.5) as i32;
        fonts.draw_text(
            canvas,
            &self.label,
            rect.x + self.style.padding.left as i32,
            text_y,
            Some((rect.width as f32 - self.style.padding.horizontal()) as u32),
            self.font_size,
            self.text_color,
        );
    }

    fn handle_event(&mut self, event: &UiEvent, state: &mut EventState, layout: &LayoutNode) -> bool {
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
                    state.pressed = Some(self.id);
                    changed = true;
                }
            }
            UiEvent::MouseUp { x, y } => {
                let was_pressed = self.is_pressed;
                self.is_pressed = false;
                if was_pressed && rect.contains(x, y) {
                    if let Some(cb) = self.on_click.as_mut() {
                        cb();
                    }
                }
                if was_pressed {
                    changed = true;
                }
            }
        }

        if changed {
            state.mark_dirty(rect);
        }
        changed
    }
}
