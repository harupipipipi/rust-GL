//! Japanese text example — CJK rendering and word-wrap verification.
//!
//! Displays Japanese text, long sentences that exercise the word-wrap
//! algorithm, and mixed CJK / Latin strings such as「Hello世界！」.
//! Verifies that the font manager can load a CJK-capable system font
//! and that `wrap_text` correctly breaks lines for CJK characters.
//!
//! Run: `cargo run --example japanese`

use std::num::NonZeroU32;
use std::rc::Rc;

use rust2d_ui::{App, Button, Color, Text, UiEvent};

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
            .with_title("日本語テキスト表示テスト")
            .with_inner_size(LogicalSize::new(800.0_f64, 700.0))
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

    // ── Simple Japanese ──
    app.root.push(Text::new_auto("こんにちは、世界！"));

    // ── Mixed CJK / Latin ──
    app.root
        .push(Text::new_auto("Hello世界！ — Rust で GUI を作る"));

    // ── Long Japanese text to test word wrapping ──
    app.root.push(Text::new_auto(
        "これは長い日本語テキストの例です。ワードラップが正しく動作するかどうかを\
         確認するために、十分な長さの文章を用意しました。日本語のテキストでは、\
         漢字やひらがな、カタカナが混在しており、各文字の前で改行が可能です。\
         このテキストがウィンドウ幅に応じて適切に折り返されることを確認してください。",
    ));

    // ── CJK mixed with English — longer ──
    app.root.push(Text::new_auto(
        "Rustは安全性とパフォーマンスを両立するプログラミング言語です。\
         This sentence mixes English and 日本語 to verify that the \
         word-wrap algorithm handles transitions between Latin and CJK scripts correctly.",
    ));

    // ── Katakana ──
    app.root.push(Text::new_auto(
        "カタカナテスト：アイウエオ カキクケコ サシスセソ",
    ));

    // ── Various sizes ──
    let mut small_jp = Text::new_auto("小さいテキスト（12px）");
    small_jp.font_size = 12.0;
    app.root.push(small_jp);

    let mut large_jp = Text::new_auto("大きいテキスト（28px）");
    large_jp.font_size = 28.0;
    large_jp.color = Color::rgba(120, 40, 40, 255);
    app.root.push(large_jp);

    // ── Button with Japanese label ──
    app.root.push(
        Button::new_auto("押してね！").on_click(|| {
            println!("ボタンがクリックされました！");
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
