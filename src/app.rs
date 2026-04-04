//! Application scaffold: owns the widget tree, drives layout / draw / events.
//!
//! **No `Rc<RefCell<App>>`** — the `App` is moved directly into the event
//! loop closure. Only the `Window` is reference-counted (softbuffer needs it).

use std::{num::NonZeroU32, rc::Rc};

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
    widgets::{
        button::Button, container::Container, text::Text, Widget,
    },
};

/// Errors produced by [`App`] or [`run()`].
#[derive(Debug, Error)]
pub enum AppError {
    /// Window creation or event loop error.
    #[error("window creation failed: {0}")]
    Window(String),
    /// Rendering backend error.
    #[error("rendering backend error: {0}")]
    Render(String),
    /// Text/font subsystem error.
    #[error(transparent)]
    Text(#[from] TextError),
}

/// Root application state.
pub struct App {
    /// The root container widget.
    pub root: Container,
    fonts: FontManager,
    canvas: Canvas,
    layout_tree: Option<crate::layout::LayoutNode>,
    event_state: EventState,
    background: Color,
}

impl App {
    /// Create a new application with the given window dimensions.
    pub fn new(width: u32, height: u32) -> Result<Self, AppError> {
        let fonts = FontManager::new()?;
        let mut root = Container::new_auto();
        root.style.padding = crate::layout::EdgeInsets::all(16.0);

        Ok(Self {
            root,
            fonts,
            canvas: Canvas::new(width, height),
            layout_tree: None,
            event_state: EventState::default(),
            background: Color::WHITE,
        })
    }

    /// Create a demo application with sample widgets.
    pub fn demo(width: u32, height: u32) -> Result<Self, AppError> {
        let mut app = Self::new(width, height)?;
        app.root.push(Text::new_auto("純Rust 2D UIライブラリ (日本語対応)"));
        app.root.push(
            Button::new_auto("押してください").on_click(|| {
                println!("button clicked");
            }),
        );
        Ok(app)
    }

    /// Recompute the layout tree from the current root widget.
    pub fn request_layout(&mut self) {
        let c = BoxConstraints::tight(
            self.canvas.width() as f32,
            self.canvas.height() as f32,
        );
        self.layout_tree = Some(self.root.layout(c, 0, 0, &self.fonts));
        self.event_state.request_redraw();
    }

    /// Resize the canvas and recompute layout.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.resize(width, height);
        self.request_layout();
    }

    /// Dispatch a UI event to the widget tree.
    pub fn handle_ui_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::MouseMove { x, y }
            | UiEvent::MouseDown { x, y }
            | UiEvent::MouseUp { x, y } => {
                self.event_state.cursor = (x, y);
            }
        }

        // Take the layout tree out to avoid borrowing `self` while mutating
        // `self.root` and `self.event_state`. This eliminates the previous
        // `layout.clone()` on every event.
        if let Some(layout) = self.layout_tree.take() {
            self.root.handle_event(&event, &mut self.event_state, &layout);
            self.layout_tree = Some(layout);
        }
    }

    /// Redraw the canvas if a repaint was requested.
    /// Returns `true` if drawing actually occurred.
    pub fn redraw(&mut self) -> bool {
        if !self.event_state.take_needs_redraw() {
            return false;
        }
        if let Some(layout) = self.layout_tree.take() {
            self.canvas.clear(self.background);
            self.root.draw(&mut self.canvas, &layout, &self.fonts);
            self.layout_tree = Some(layout);
        }
        true
    }

    /// Access the raw pixel data for presentation.
    pub fn pixels(&self) -> &[u32] {
        self.canvas.pixels()
    }
}

/// Create a demo window and run the event loop (blocks until close).
pub fn run() -> Result<(), AppError> {
    let event_loop = EventLoop::new()
        .map_err(|e| AppError::Window(e.to_string()))?;

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Rust 2D UI")
            .with_inner_size(LogicalSize::new(960.0_f64, 640.0))
            .build(&event_loop)
            .map_err(|e| AppError::Window(e.to_string()))?,
    );

    let context =
        Context::new(window.clone()).map_err(|e| AppError::Render(e.to_string()))?;
    let mut surface =
        Surface::new(&context, window.clone()).map_err(|e| AppError::Render(e.to_string()))?;

    let size = window.inner_size();
    surface
        .resize(
            NonZeroU32::new(size.width.max(1)).unwrap(),
            NonZeroU32::new(size.height.max(1)).unwrap(),
        )
        .map_err(|e| AppError::Render(e.to_string()))?;

    let mut app = App::demo(size.width, size.height)?;
    app.request_layout();
    window.request_redraw();

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Wait);

            if let Event::WindowEvent { event, .. } = event { match event {
                    WindowEvent::CloseRequested => target.exit(),

                    WindowEvent::Resized(new_size) => {
                        if let Err(e) = surface.resize(
                            NonZeroU32::new(new_size.width.max(1)).unwrap(),
                            NonZeroU32::new(new_size.height.max(1)).unwrap(),
                        ) {
                            // TODO: replace with proper logging
                            eprintln!("surface resize failed: {e}");
                        }
                        app.resize(new_size.width, new_size.height);
                        window.request_redraw();
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        app.handle_ui_event(UiEvent::MouseMove {
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
                        let (cx, cy) = app.event_state.cursor;
                        let ui_event = if state == ElementState::Pressed {
                            UiEvent::MouseDown { x: cx, y: cy }
                        } else {
                            UiEvent::MouseUp { x: cx, y: cy }
                        };
                        app.handle_ui_event(ui_event);
                        window.request_redraw();
                    }

                    WindowEvent::RedrawRequested => {
                        if app.redraw() {
                            match surface.buffer_mut() {
                                Ok(mut buffer) => {
                                    let src = app.pixels();
                                    if buffer.len() == src.len() {
                                        buffer.copy_from_slice(src);
                                    }
                                    if let Err(e) = buffer.present() {
                                        eprintln!("buffer present failed: {e}");
                                    }
                                }
                                Err(e) => {
                                    eprintln!("buffer_mut failed: {e}");
                                }
                            }
                        }
                    }

                    _ => {}
                } }
        })
        .map_err(|e| AppError::Window(e.to_string()))
}
