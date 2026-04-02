//! Layout demo — nested containers with different layout directions.
//!
//! Demonstrates `Container` nesting: an outer `Vertical` container holds
//! two inner `Horizontal` containers (rows). Each row uses different `gap`
//! and `padding` values so the effect of `LayoutDirection`, `gap`, and
//! `padding` is visually apparent.
//!
//! Run: `cargo run --example layout_demo`

use std::num::NonZeroU32;
use std::rc::Rc;

use rust2d_ui::{App, Button, Color, Container, EdgeInsets, LayoutDirection, Text, UiEvent};

use softbuffer::{Context, Surface};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Layout Demo")
            .with_inner_size(LogicalSize::new(800.0_f64, 600.0))
            .build(&event_loop)?,
    );

    let context = Context::new(window.clone())?;
    let mut surface = Surface::new(&context, window.clone())?;

    let size = window.inner_size();
    surface.resize(
        NonZeroU32::new(size.width.max(1)).unwrap(),
        NonZeroU32::new(size.height.max(1)).unwrap(),
    )?;

    let mut app = App::new(size.width, size.height)?;

    // Title
    app.root.push(Text::new_auto(
        "Layout Demo — Vertical > Horizontal nesting",
    ));

    // ── Row 1: Horizontal, gap=20, padding=12 ──────────────────
    let mut row1 = Container::new_auto();
    row1.style.direction = LayoutDirection::Horizontal;
    row1.style.gap = 20.0;
    row1.style.padding = EdgeInsets::all(12.0);
    row1.background = Some(Color::rgba(220, 235, 255, 255));

    row1.push(Text::new_auto("Row 1 (gap=20)"));
    row1.push(Button::new_auto("A"));
    row1.push(Button::new_auto("B"));
    row1.push(Button::new_auto("C"));

    // ── Row 2: Horizontal, gap=4, padding=24 ──────────────────
    let mut row2 = Container::new_auto();
    row2.style.direction = LayoutDirection::Horizontal;
    row2.style.gap = 4.0;
    row2.style.padding = EdgeInsets::all(24.0);
    row2.background = Some(Color::rgba(255, 235, 220, 255));

    row2.push(Text::new_auto("Row 2 (gap=4, pad=24)"));
    row2.push(Button::new_auto("X"));
    row2.push(Button::new_auto("Y"));
    row2.push(Button::new_auto("Z"));

    // ── Row 3: Horizontal with a nested Vertical container ────
    let mut row3 = Container::new_auto();
    row3.style.direction = LayoutDirection::Horizontal;
    row3.style.gap = 16.0;
    row3.style.padding = EdgeInsets::all(8.0);
    row3.background = Some(Color::rgba(220, 255, 220, 255));

    let mut inner_vert = Container::new_auto();
    inner_vert.style.direction = LayoutDirection::Vertical;
    inner_vert.style.gap = 4.0;
    inner_vert.style.padding = EdgeInsets::all(8.0);
    inner_vert.background = Some(Color::rgba(200, 240, 200, 255));
    inner_vert.push(Text::new_auto("Nested Vertical"));
    inner_vert.push(Button::new_auto("Top"));
    inner_vert.push(Button::new_auto("Bottom"));

    row3.push(Text::new_auto("Row 3 — mixed"));
    row3.push(inner_vert);
    row3.push(Button::new_auto("Beside"));

    // Add rows to the outer vertical container
    app.root.style.gap = 12.0;
    app.root.push(row1);
    app.root.push(row2);
    app.root.push(row3);

    app.request_layout();
    window.request_redraw();

    let mut cursor = (0.0_f32, 0.0_f32);

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),

                WindowEvent::Resized(new_size) => {
                    let _ = surface.resize(
                        NonZeroU32::new(new_size.width.max(1)).unwrap(),
                        NonZeroU32::new(new_size.height.max(1)).unwrap(),
                    );
                    app.resize(new_size.width, new_size.height);
                    window.request_redraw();
                }

                WindowEvent::CursorMoved { position, .. } => {
                    cursor = (position.x as f32, position.y as f32);
                    app.handle_ui_event(UiEvent::MouseMove {
                        x: cursor.0,
                        y: cursor.1,
                    });
                    window.request_redraw();
                }

                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Left,
                    ..
                } => {
                    let ui_event = if state == ElementState::Pressed {
                        UiEvent::MouseDown {
                            x: cursor.0,
                            y: cursor.1,
                        }
                    } else {
                        UiEvent::MouseUp {
                            x: cursor.0,
                            y: cursor.1,
                        }
                    };
                    app.handle_ui_event(ui_event);
                    window.request_redraw();
                }

                WindowEvent::RedrawRequested => {
                    if app.redraw() {
                        if let Ok(mut buffer) = surface.buffer_mut() {
                            let src = app.pixels();
                            if buffer.len() == src.len() {
                                buffer.copy_from_slice(src);
                            }
                            let _ = buffer.present();
                        }
                    }
                }

                _ => {}
            },
            _ => {}
        }
    })?;

    Ok(())
}
