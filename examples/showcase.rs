#![allow(clippy::single_match)]

//! Widget showcase for `rust2d_ui`.
//!
//! Run with:
//! ```bash
//! cargo run --example showcase
//! ```

use std::num::NonZeroU32;
use std::rc::Rc;

use rust2d_ui::{
    App, Button, Checkbox, Color, Container, CrossAxisAlignment, Divider, EdgeInsets,
    LayoutDirection, RadioButton, ScrollView, Slider, Spacer, Text, TextInput, UiEvent,
};

use softbuffer::{Context, Surface};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn make_card(title: &str, subtitle: &str) -> Container {
    let mut card = Container::new_auto();
    card.style.padding = EdgeInsets::all(14.0);
    card.style.gap = 8.0;
    card.background = Some(Color::rgba(248, 249, 252, 255));

    let mut heading = Text::new_auto(title);
    heading.font_size = 20.0;
    heading.color = Color::rgba(35, 44, 72, 255);
    card.push(heading);

    let mut copy = Text::new_auto(subtitle);
    copy.font_size = 14.0;
    copy.color = Color::rgba(92, 101, 125, 255);
    card.push(copy);

    card
}

fn build_showcase(app: &mut App) {
    app.root.style.padding = EdgeInsets::all(20.0);
    app.root.style.gap = 16.0;
    app.root.background = Some(Color::rgba(239, 242, 247, 255));

    let mut hero = make_card(
        "rust2d_ui Showcase",
        "A small dashboard built entirely with the software-rendered widget set.",
    );

    let mut action_row = Container::new_auto();
    action_row.style.direction = LayoutDirection::Horizontal;
    action_row.style.align_items = CrossAxisAlignment::Center;
    action_row.style.gap = 12.0;
    action_row.background = None;

    let mut primary = Button::new_auto("Primary Action").on_click(|| {
        println!("primary action clicked");
    });
    primary.normal_bg = Color::rgba(31, 91, 214, 255);
    primary.hover_bg = Color::rgba(52, 116, 245, 255);
    primary.pressed_bg = Color::rgba(22, 65, 158, 255);
    primary.text_color = Color::WHITE;

    let mut secondary = Button::new_auto("Secondary").on_click(|| {
        println!("secondary action clicked");
    });
    secondary.normal_bg = Color::rgba(221, 227, 239, 255);
    secondary.hover_bg = Color::rgba(204, 214, 233, 255);
    secondary.pressed_bg = Color::rgba(181, 194, 219, 255);

    action_row.push(primary);
    action_row.push(secondary);
    action_row.push(Spacer::flex());
    action_row.push(Checkbox::new_auto("Enable live preview").checked(true));

    hero.push(Divider::new_horizontal());
    hero.push(action_row);
    app.root.push(hero);

    let mut controls = make_card(
        "Controls",
        "Mixed input widgets in a single horizontal flow container.",
    );

    let mut controls_row = Container::new_auto();
    controls_row.style.direction = LayoutDirection::Horizontal;
    controls_row.style.align_items = CrossAxisAlignment::Center;
    controls_row.style.gap = 18.0;
    controls_row.background = None;

    controls_row.push(RadioButton::new_auto("Design", 1).selected(true));
    controls_row.push(RadioButton::new_auto("Inspect", 1));
    controls_row.push(RadioButton::new_auto("Ship", 1));
    controls_row.push(Divider::new_vertical().thickness(2.0).length(24.0));

    let mut volume_col = Container::new_auto();
    volume_col.style.gap = 6.0;
    volume_col.background = None;
    volume_col.push(Text::new_auto("Intensity"));
    volume_col.push(Slider::new_auto().range(0.0, 100.0).value(72.0).step(1.0));
    controls_row.push(volume_col);

    controls.push(controls_row);
    app.root.push(controls);

    let mut form = make_card(
        "Input Surface",
        "TextInput is included too. Click it to focus; keyboard integration is the next layer to wire into App.",
    );

    let mut input = TextInput::new_auto()
        .placeholder("Project name")
        .font_size(18.0)
        .on_submit(|text| {
            println!("submitted: {text}");
        });
    input.set_text("Aurora");
    form.push(input);
    form.push(Checkbox::new_auto("Sync to cloud"));
    app.root.push(form);

    let mut scroll_content = Container::new_auto();
    scroll_content.style.gap = 10.0;
    scroll_content.background = Some(Color::rgba(255, 255, 255, 255));

    for index in 1..=8 {
        let mut row = Container::new_auto();
        row.style.direction = LayoutDirection::Horizontal;
        row.style.align_items = CrossAxisAlignment::Center;
        row.style.padding = EdgeInsets::all(10.0);
        row.style.gap = 12.0;
        row.background = Some(if index % 2 == 0 {
            Color::rgba(247, 249, 252, 255)
        } else {
            Color::rgba(239, 244, 250, 255)
        });

        let mut label = Text::new_auto(format!("Task {index}"));
        label.font_size = 16.0;
        row.push(label);
        row.push(Spacer::flex());
        row.push(Button::new_auto("Open").on_click(move || {
            println!("open task {index}");
        }));
        scroll_content.push(row);
    }

    let mut backlog = make_card(
        "Scrollable Backlog",
        "This section uses ScrollView with a stacked child container and a software-rendered scrollbar.",
    );
    backlog.push(ScrollView::new(scroll_content).max_height(220.0));
    app.root.push(backlog);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("rust2d_ui Showcase")
            .with_inner_size(LogicalSize::new(960.0_f64, 720.0))
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
    build_showcase(&mut app);
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
