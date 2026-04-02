use std::{cell::RefCell, num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use thiserror::Error;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    canvas::{Canvas, Color},
    event::{EventState, UiEvent},
    layout::BoxConstraints,
    text::{FontManager, TextError},
    widgets::{button::Button, container::Container, text::Text, Widget},
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("window creation failed: {0}")]
    Window(String),
    #[error("rendering backend error: {0}")]
    Render(String),
    #[error(transparent)]
    Text(#[from] TextError),
}

pub struct App {
    pub root: Container,
    fonts: FontManager,
    canvas: Canvas,
    layout_tree: Option<crate::layout::LayoutNode>,
    event_state: EventState,
    background: Color,
}

impl App {
    pub fn new(width: u32, height: u32) -> Result<Self, AppError> {
        let fonts = FontManager::new()?;
        let mut root = Container::new(1);
        root.style.padding = crate::layout::EdgeInsets::all(16.0);
        root.push(Text::new(2, "純Rust 2D UIライブラリ (日本語対応)"));
        root.push(Button::new(3, "押してください").on_click(|| {
            println!("button clicked");
        }));

        Ok(Self {
            root,
            fonts,
            canvas: Canvas::new(width, height),
            layout_tree: None,
            event_state: EventState::default(),
            background: Color::WHITE,
        })
    }

    pub fn request_layout(&mut self) {
        let constraints = BoxConstraints::tight(self.canvas.width() as f32, self.canvas.height() as f32);
        self.layout_tree = Some(self.root.layout(constraints, 0, 0, &self.fonts));
        if let Some(layout) = &self.layout_tree {
            self.event_state.mark_dirty(layout.bounds);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.resize(width, height);
        self.request_layout();
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::MouseMove { x, y } => self.event_state.cursor = (x, y),
            UiEvent::MouseDown { x, y } => self.event_state.cursor = (x, y),
            UiEvent::MouseUp { x, y } => self.event_state.cursor = (x, y),
        }

        if let Some(layout) = &self.layout_tree {
            if self.root.handle_event(&event, &mut self.event_state, layout) {
                self.event_state.mark_dirty(layout.bounds);
            }
        }
    }

    pub fn redraw(&mut self) {
        let dirty = self.event_state.take_dirty_regions();
        if dirty.is_empty() {
            return;
        }

        if let Some(layout) = &self.layout_tree {
            self.canvas.clear(self.background);
            self.root.draw(&mut self.canvas, layout, &self.fonts);
        }
    }

    pub fn pixels(&self) -> &[u32] {
        self.canvas.pixels()
    }
}

pub fn run() -> Result<(), AppError> {
    let event_loop = EventLoop::new().map_err(|e| AppError::Window(e.to_string()))?;
    let window = Box::new(
        WindowBuilder::new()
            .with_title("Rust 2D UI")
            .with_inner_size(LogicalSize::new(960.0f64, 640.0f64))
            .build(&event_loop)
            .map_err(|e| AppError::Window(e.to_string()))?,
    );
    let window: &'static winit::window::Window = Box::leak(window);

    let context = Context::new(window).map_err(|e| AppError::Render(e.to_string()))?;
    let mut surface = Surface::new(&context, window).map_err(|e| AppError::Render(e.to_string()))?;

    let size = window.inner_size();
    surface
        .resize(
            NonZeroU32::new(size.width.max(1)).unwrap(),
            NonZeroU32::new(size.height.max(1)).unwrap(),
        )
        .map_err(|e| AppError::Render(e.to_string()))?;

    let app = Rc::new(RefCell::new(App::new(size.width, size.height)?));
    app.borrow_mut().request_layout();

    let app_ref = app.clone();

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(new_size) => {
                        let _ = surface.resize(
                            NonZeroU32::new(new_size.width.max(1)).unwrap(),
                            NonZeroU32::new(new_size.height.max(1)).unwrap(),
                        );
                        app_ref.borrow_mut().resize(new_size.width, new_size.height);
                        window.request_redraw();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        app_ref.borrow_mut().handle_ui_event(UiEvent::MouseMove {
                            x: position.x as f32,
                            y: position.y as f32,
                        });
                        window.request_redraw();
                    }
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    } => {
                        let pos = app_ref.borrow().event_state.cursor;
                        let ui_event = if state == ElementState::Pressed {
                            UiEvent::MouseDown { x: pos.0, y: pos.1 }
                        } else {
                            UiEvent::MouseUp { x: pos.0, y: pos.1 }
                        };
                        app_ref.borrow_mut().handle_ui_event(ui_event);
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        let mut app = app_ref.borrow_mut();
                        app.redraw();

                        if let Ok(mut buffer) = surface.buffer_mut() {
                            buffer.copy_from_slice(app.pixels());
                            let _ = buffer.present();
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => window.request_redraw(),
                _ => {}
            }
        })
        .map_err(|e| AppError::Window(e.to_string()))
}
