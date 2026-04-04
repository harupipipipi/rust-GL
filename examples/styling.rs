#![allow(clippy::single_match)]

//! Styling example — customise widget colours and text appearance.
//!
//! Shows how to override `Button` fields (`normal_bg`, `hover_bg`,
//! `pressed_bg`, `text_color`) and `Text` fields (`font_size`, `color`)
//! to create visually distinct widgets side by side.
//!
//! Run: `cargo run --example styling`

use std::num::NonZeroU32;
use std::rc::Rc;

use rust2d_ui::{App, Button, Color, Container, LayoutDirection, Text, UiEvent};

use softbuffer::{Context, Surface};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn styled_button(label: &str, normal: Color, hover: Color, pressed: Color, text: Color) -> Button {
    let mut btn = Button::new_auto(label);
    btn.normal_bg = normal;
    btn.hover_bg = hover;
    btn.pressed_bg = pressed;
    btn.text_color = text;
    btn
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Styling Example")
            .with_inner_size(LogicalSize::new(800.0_f64, 500.0))
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

    // ── Title ──
    let mut title = Text::new_auto("Styling Example — Custom Colours & Font Sizes");
    title.font_size = 24.0;
    title.color = Color::rgba(50, 50, 120, 255);
    app.root.push(title);

    // ── Subtitle ──
    let mut subtitle =
        Text::new_auto("Each button below has different normal / hover / pressed colours.");
    subtitle.font_size = 14.0;
    subtitle.color = Color::GRAY_600;
    app.root.push(subtitle);

    // ── Button row ──
    let mut row = Container::new_auto();
    row.style.direction = LayoutDirection::Horizontal;
    row.style.gap = 12.0;
    row.background = None;

    // Blue theme
    row.push(styled_button(
        "Blue",
        Color::rgba(70, 130, 220, 255),
        Color::rgba(100, 160, 255, 255),
        Color::rgba(40, 80, 180, 255),
        Color::WHITE,
    ));

    // Green theme
    row.push(styled_button(
        "Green",
        Color::rgba(60, 180, 75, 255),
        Color::rgba(90, 210, 105, 255),
        Color::rgba(30, 130, 45, 255),
        Color::WHITE,
    ));

    // Red theme
    row.push(styled_button(
        "Red / Danger",
        Color::rgba(220, 60, 60, 255),
        Color::rgba(255, 90, 90, 255),
        Color::rgba(170, 30, 30, 255),
        Color::WHITE,
    ));

    // Dark theme
    row.push(styled_button(
        "Dark",
        Color::rgba(50, 50, 50, 255),
        Color::rgba(80, 80, 80, 255),
        Color::rgba(30, 30, 30, 255),
        Color::rgba(230, 230, 230, 255),
    ));

    app.root.push(row);

    // ── Text styling ──
    let mut big_text = Text::new_auto("Large text (32px)");
    big_text.font_size = 32.0;
    big_text.color = Color::rgba(180, 60, 60, 255);
    app.root.push(big_text);

    let mut small_text = Text::new_auto("Small coloured text (12px)");
    small_text.font_size = 12.0;
    small_text.color = Color::rgba(60, 60, 180, 255);
    app.root.push(small_text);

    let mut green_text = Text::new_auto("Green medium text (18px)");
    green_text.font_size = 18.0;
    green_text.color = Color::rgba(30, 140, 50, 255);
    app.root.push(green_text);

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
