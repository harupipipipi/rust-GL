//! Counter example — demonstrates shared mutable state with `AtomicU32`.
//!
//! A minimal pattern for manipulating state from a button callback:
//! an `AtomicU32` counter is incremented on each click and the current
//! value is printed to stdout via `println!`.
//!
//! Run: `cargo run --example counter`

use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use rust2d_ui::{App, Button, Text, UiEvent};

use softbuffer::{Context, Surface};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let counter = Arc::new(AtomicU32::new(0));

    let event_loop = EventLoop::new()?;

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Counter Example")
            .with_inner_size(LogicalSize::new(600.0_f64, 400.0))
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

    // Build widget tree
    app.root
        .push(Text::new_auto("Counter Example — Click the button to increment"));

    let c = counter.clone();
    app.root.push(
        Button::new_auto("+1").on_click(move || {
            let new_val = c.fetch_add(1, Ordering::SeqCst) + 1;
            println!("Counter: {new_val}");
        }),
    );

    let c2 = counter.clone();
    app.root.push(
        Button::new_auto("Reset").on_click(move || {
            c2.store(0, Ordering::SeqCst);
            println!("Counter reset to 0");
        }),
    );

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
