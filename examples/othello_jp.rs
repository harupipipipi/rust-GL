#![allow(clippy::single_match)]

//! 日本語 UI のオセロサンプル。
//!
//! 盤面描画、合法手の強調、石の反転、パス判定、勝敗表示、
//! そして再戦ボタンまでを `rust2d_ui` の `Canvas` と `FontManager`
//! を使ってまとめた完全日本語の例です。
//!
//! 実行:
//! `cargo run --example othello_jp`

use std::num::NonZeroU32;
use std::rc::Rc;

use rust2d_ui::{Canvas, Color, FontManager, Rect};
use softbuffer::{Context, Surface};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const BOARD_SIZE: usize = 8;
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stone {
    黒,
    白,
}

impl Stone {
    fn opposite(self) -> Self {
        match self {
            Self::黒 => Self::白,
            Self::白 => Self::黒,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::黒 => "黒",
            Self::白 => "白",
        }
    }

    fn fill_color(self) -> Color {
        match self {
            Self::黒 => Color::rgba(28, 30, 36, 255),
            Self::白 => Color::rgba(247, 247, 244, 255),
        }
    }

    fn edge_color(self) -> Color {
        match self {
            Self::黒 => Color::rgba(70, 76, 88, 255),
            Self::白 => Color::rgba(196, 201, 210, 255),
        }
    }
}

#[derive(Clone, Debug)]
struct OthelloGame {
    board: [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    turn: Stone,
    status: String,
    move_count: u32,
    last_move: Option<(usize, usize)>,
    game_over: bool,
}

impl OthelloGame {
    fn new() -> Self {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        board[3][3] = Some(Stone::白);
        board[4][4] = Some(Stone::白);
        board[3][4] = Some(Stone::黒);
        board[4][3] = Some(Stone::黒);

        let mut game = Self {
            board,
            turn: Stone::黒,
            status: String::new(),
            move_count: 0,
            last_move: None,
            game_over: false,
        };
        game.refresh_status();
        game
    }

    fn counts(&self) -> (u32, u32) {
        let mut black = 0;
        let mut white = 0;
        for row in &self.board {
            for cell in row {
                match cell {
                    Some(Stone::黒) => black += 1,
                    Some(Stone::白) => white += 1,
                    None => {}
                }
            }
        }
        (black, white)
    }

    fn legal_moves(&self, stone: Stone) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if !self.flips_for_move(x, y, stone).is_empty() {
                    moves.push((x, y));
                }
            }
        }
        moves
    }

    fn play(&mut self, x: usize, y: usize) -> bool {
        if self.game_over {
            self.status =
                "対局は終了しています。もう一度遊ぶには「最初から」を押してください。".into();
            return false;
        }

        let flips = self.flips_for_move(x, y, self.turn);
        if flips.is_empty() {
            self.status = format!(
                "そこには置けません。{}の合法手を選んでください。",
                self.turn.label()
            );
            return false;
        }

        self.board[y][x] = Some(self.turn);
        for (fx, fy) in flips {
            self.board[fy][fx] = Some(self.turn);
        }

        self.move_count += 1;
        self.last_move = Some((x, y));
        self.turn = self.turn.opposite();
        self.advance_turn_after_move();
        true
    }

    fn refresh_status(&mut self) {
        if self.game_over {
            let (black, white) = self.counts();
            self.status = if black > white {
                format!("対局終了。黒の勝ちです。最終結果は 黒 {black} - 白 {white} です。")
            } else if white > black {
                format!("対局終了。白の勝ちです。最終結果は 黒 {black} - 白 {white} です。")
            } else {
                format!("対局終了。引き分けです。最終結果は 黒 {black} - 白 {white} です。")
            };
            return;
        }

        let legal = self.legal_moves(self.turn);
        let (black, white) = self.counts();
        self.status = if legal.is_empty() {
            format!(
                "{}は置ける場所がありません。クリックすると手番を確認できます。 黒 {black} - 白 {white}",
                self.turn.label()
            )
        } else {
            format!(
                "{}の番です。光っているマスに置けます。 黒 {black} - 白 {white}",
                self.turn.label()
            )
        };
    }

    fn advance_turn_after_move(&mut self) {
        let current_legal = self.legal_moves(self.turn);
        if !current_legal.is_empty() {
            self.refresh_status();
            return;
        }

        let skipped = self.turn;
        let other = skipped.opposite();
        let other_legal = self.legal_moves(other);
        if other_legal.is_empty() {
            self.game_over = true;
            self.refresh_status();
            return;
        }

        self.turn = other;
        self.status = format!(
            "{}は置ける場所がないためパスです。続けて{}の番です。",
            skipped.label(),
            self.turn.label()
        );
    }

    fn flips_for_move(&self, x: usize, y: usize, stone: Stone) -> Vec<(usize, usize)> {
        if self.board[y][x].is_some() {
            return Vec::new();
        }

        let mut result = Vec::new();
        for (dx, dy) in DIRECTIONS {
            let mut line = Vec::new();
            let mut cx = x as i32 + dx;
            let mut cy = y as i32 + dy;

            while Self::in_bounds(cx, cy) {
                match self.board[cy as usize][cx as usize] {
                    Some(s) if s == stone.opposite() => line.push((cx as usize, cy as usize)),
                    Some(s) if s == stone => {
                        if !line.is_empty() {
                            result.extend(line);
                        }
                        break;
                    }
                    _ => break,
                }
                cx += dx;
                cy += dy;
            }
        }

        result
    }

    fn in_bounds(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < BOARD_SIZE as i32 && y < BOARD_SIZE as i32
    }
}

#[derive(Clone, Copy, Debug)]
struct UiLayout {
    board: Rect,
    cell: i32,
    reset_button: Rect,
}

impl UiLayout {
    fn new(width: u32, height: u32) -> Self {
        let margin = 28_i32;
        let top_panel = 164_i32;
        let available_w = width as i32 - margin * 2;
        let available_h = height as i32 - top_panel - margin * 2;
        let board_px = available_w.min(available_h).max(320);
        let board_px = board_px.min((BOARD_SIZE as i32) * 92);
        let cell = (board_px / BOARD_SIZE as i32).max(36);
        let board_side = cell * BOARD_SIZE as i32;
        let board_x = ((width as i32 - board_side) / 2).max(margin);
        let board_y = top_panel;

        let button_width = 190_u32;
        let button_height = 46_u32;
        let button_x = width as i32 - margin - button_width as i32;
        let button_y = 100_i32;

        Self {
            board: Rect::new(board_x, board_y, board_side as u32, board_side as u32),
            cell,
            reset_button: Rect::new(button_x, button_y, button_width, button_height),
        }
    }

    fn cell_rect(&self, x: usize, y: usize) -> Rect {
        Rect::new(
            self.board.x + x as i32 * self.cell,
            self.board.y + y as i32 * self.cell,
            self.cell as u32,
            self.cell as u32,
        )
    }

    fn cell_at(&self, px: f32, py: f32) -> Option<(usize, usize)> {
        if !self.board.contains(px, py) {
            return None;
        }

        let local_x = (px as i32 - self.board.x) / self.cell;
        let local_y = (py as i32 - self.board.y) / self.cell;
        if local_x < 0 || local_y < 0 {
            return None;
        }
        Some((local_x as usize, local_y as usize))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("日本語オセロ")
            .with_inner_size(LogicalSize::new(920.0_f64, 860.0))
            .build(&event_loop)?,
    );

    let context = Context::new(window.clone())?;
    let mut surface = Surface::new(&context, window.clone())?;
    let size = window.inner_size();
    surface.resize(
        NonZeroU32::new(size.width.max(1)).unwrap(),
        NonZeroU32::new(size.height.max(1)).unwrap(),
    )?;

    let fonts = FontManager::new()?;
    let mut canvas = Canvas::new(size.width, size.height);
    let mut game = OthelloGame::new();
    let mut layout = UiLayout::new(size.width, size.height);
    let mut cursor = (0.0_f32, 0.0_f32);

    window.request_redraw();

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
                    canvas.resize(new_size.width, new_size.height);
                    layout = UiLayout::new(new_size.width, new_size.height);
                    window.request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    cursor = (position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Left,
                    ..
                } => {
                    if layout.reset_button.contains(cursor.0, cursor.1) {
                        game = OthelloGame::new();
                        window.request_redraw();
                        return;
                    }

                    if let Some((x, y)) = layout.cell_at(cursor.0, cursor.1) {
                        if game.play(x, y) {
                            window.request_redraw();
                        } else {
                            window.request_redraw();
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    draw_scene(&mut canvas, &fonts, &game, layout, cursor);
                    if let Ok(mut buffer) = surface.buffer_mut() {
                        let src = canvas.pixels();
                        if src.len() == buffer.len() {
                            buffer.copy_from_slice(src);
                        }
                        let _ = buffer.present();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    })?;

    Ok(())
}

fn draw_scene(
    canvas: &mut Canvas,
    fonts: &FontManager,
    game: &OthelloGame,
    layout: UiLayout,
    cursor: (f32, f32),
) {
    canvas.clear(Color::rgba(244, 236, 214, 255));

    let legal_moves = game.legal_moves(game.turn);
    let hovered_cell = layout.cell_at(cursor.0, cursor.1);
    let button_hovered = layout.reset_button.contains(cursor.0, cursor.1);

    draw_header(
        canvas,
        fonts,
        game,
        layout,
        legal_moves.len() as u32,
        button_hovered,
    );
    draw_board_background(canvas, layout);

    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            let rect = layout.cell_rect(x, y);
            let is_legal = legal_moves.contains(&(x, y));
            let is_hovered = hovered_cell == Some((x, y));
            draw_cell(canvas, rect, is_legal, is_hovered);

            if let Some(stone) = game.board[y][x] {
                draw_stone(canvas, rect, stone);
            }

            if game.last_move == Some((x, y)) {
                draw_last_move_marker(canvas, rect);
            }
        }
    }

    draw_grid(canvas, layout);
    draw_coordinates(canvas, fonts, layout);
}

fn draw_header(
    canvas: &mut Canvas,
    fonts: &FontManager,
    game: &OthelloGame,
    layout: UiLayout,
    legal_count: u32,
    button_hovered: bool,
) {
    let panel = Rect::new(24, 24, canvas.width().saturating_sub(48), 116);
    canvas.draw_rounded_rect(panel, 18, Color::rgba(255, 251, 241, 255));

    draw_text(
        fonts,
        canvas,
        "日本語オセロ",
        40,
        38,
        30.0,
        Color::rgba(48, 40, 28, 255),
    );

    let sub = if game.game_over {
        "すべての判定と表示を日本語でまとめたローカル対戦です。"
    } else {
        "黒から開始します。光るマスが合法手です。置けない側は自動でパスされます。"
    };
    draw_text(
        fonts,
        canvas,
        sub,
        42,
        74,
        15.0,
        Color::rgba(96, 82, 62, 255),
    );

    let (black, white) = game.counts();
    let summary = format!(
        "手番: {}   合法手: {}   黒: {}   白: {}   手数: {}",
        game.turn.label(),
        legal_count,
        black,
        white,
        game.move_count
    );
    draw_text(
        fonts,
        canvas,
        &summary,
        42,
        102,
        17.0,
        Color::rgba(62, 67, 77, 255),
    );

    let status_panel = Rect::new(24, 148, canvas.width().saturating_sub(48), 74);
    canvas.draw_rounded_rect(status_panel, 18, Color::rgba(232, 243, 233, 255));
    fonts.draw_text_in_rect(
        canvas,
        &game.status,
        Rect::new(
            status_panel.x + 16,
            status_panel.y + 14,
            status_panel.width.saturating_sub(32),
            status_panel.height.saturating_sub(20),
        ),
        18.0,
        Color::rgba(36, 74, 44, 255),
    );

    let button_color = if button_hovered {
        Color::rgba(205, 91, 70, 255)
    } else {
        Color::rgba(182, 76, 58, 255)
    };
    canvas.draw_rounded_rect(layout.reset_button, 14, button_color);
    draw_centered_text(
        fonts,
        canvas,
        "最初から",
        layout.reset_button,
        18.0,
        Color::WHITE,
    );
}

fn draw_board_background(canvas: &mut Canvas, layout: UiLayout) {
    let frame = Rect::new(
        layout.board.x - 10,
        layout.board.y - 10,
        layout.board.width + 20,
        layout.board.height + 20,
    );
    canvas.draw_rounded_rect(frame, 18, Color::rgba(114, 72, 34, 255));
    canvas.fill_rect(layout.board, Color::rgba(34, 120, 71, 255));
}

fn draw_cell(canvas: &mut Canvas, rect: Rect, is_legal: bool, is_hovered: bool) {
    if is_hovered {
        canvas.fill_rect(rect, Color::rgba(66, 154, 96, 255));
    } else if is_legal {
        canvas.fill_rect(rect, Color::rgba(49, 138, 84, 255));
    }

    if is_legal {
        let cx = rect.x + rect.width as i32 / 2;
        let cy = rect.y + rect.height as i32 / 2;
        let radius = (rect.width.min(rect.height) as i32 / 8).max(4);
        draw_disc(canvas, cx, cy, radius, Color::rgba(239, 220, 118, 180));
    }
}

fn draw_stone(canvas: &mut Canvas, rect: Rect, stone: Stone) {
    let radius = (rect.width.min(rect.height) as i32 / 2) - 8;
    let cx = rect.x + rect.width as i32 / 2;
    let cy = rect.y + rect.height as i32 / 2;
    draw_disc(canvas, cx, cy, radius, stone.fill_color());
    draw_ring(canvas, cx, cy, radius, 2, stone.edge_color());
    draw_disc(
        canvas,
        cx - radius / 3,
        cy - radius / 3,
        (radius / 3).max(3),
        Color::rgba(255, 255, 255, if stone == Stone::白 { 70 } else { 38 }),
    );
}

fn draw_last_move_marker(canvas: &mut Canvas, rect: Rect) {
    let inset = 7;
    let x0 = rect.x + inset;
    let y0 = rect.y + inset;
    let x1 = rect.x + rect.width as i32 - inset - 1;
    let y1 = rect.y + rect.height as i32 - inset - 1;
    let color = Color::rgba(255, 213, 79, 255);

    canvas.draw_line(x0, y0, x1, y0, color);
    canvas.draw_line(x1, y0, x1, y1, color);
    canvas.draw_line(x1, y1, x0, y1, color);
    canvas.draw_line(x0, y1, x0, y0, color);
}

fn draw_grid(canvas: &mut Canvas, layout: UiLayout) {
    for index in 0..=BOARD_SIZE {
        let x = layout.board.x + index as i32 * layout.cell;
        canvas.draw_line(
            x,
            layout.board.y,
            x,
            layout.board.y + layout.board.height as i32,
            Color::BLACK,
        );
        let y = layout.board.y + index as i32 * layout.cell;
        canvas.draw_line(
            layout.board.x,
            y,
            layout.board.x + layout.board.width as i32,
            y,
            Color::BLACK,
        );
    }
}

fn draw_coordinates(canvas: &mut Canvas, fonts: &FontManager, layout: UiLayout) {
    for x in 0..BOARD_SIZE {
        let label = (b'A' + x as u8) as char;
        let rect = Rect::new(
            layout.cell_rect(x, 0).x,
            layout.board.y - 28,
            layout.cell as u32,
            22,
        );
        draw_centered_text(
            fonts,
            canvas,
            &label.to_string(),
            rect,
            15.0,
            Color::rgba(74, 54, 28, 255),
        );
    }

    for y in 0..BOARD_SIZE {
        let rect = Rect::new(
            layout.board.x - 24,
            layout.cell_rect(0, y).y,
            20,
            layout.cell as u32,
        );
        draw_centered_text(
            fonts,
            canvas,
            &(y + 1).to_string(),
            rect,
            15.0,
            Color::rgba(74, 54, 28, 255),
        );
    }
}

fn draw_text(
    fonts: &FontManager,
    canvas: &mut Canvas,
    text: &str,
    x: i32,
    y: i32,
    px: f32,
    color: Color,
) {
    fonts.draw_text(canvas, text, x, y, None, px, color);
}

fn draw_centered_text(
    fonts: &FontManager,
    canvas: &mut Canvas,
    text: &str,
    rect: Rect,
    px: f32,
    color: Color,
) {
    let (w, h) = fonts.measure_text(text, px);
    let x = rect.x + ((rect.width as i32 - w.round() as i32) / 2);
    let y = rect.y + ((rect.height as i32 - h.round() as i32) / 2);
    fonts.draw_text(canvas, text, x, y, None, px, color);
}

fn draw_disc(canvas: &mut Canvas, cx: i32, cy: i32, radius: i32, color: Color) {
    let r2 = radius * radius;
    for y in -radius..=radius {
        for x in -radius..=radius {
            if x * x + y * y <= r2 {
                canvas.blend_pixel(cx + x, cy + y, color);
            }
        }
    }
}

fn draw_ring(canvas: &mut Canvas, cx: i32, cy: i32, radius: i32, thickness: i32, color: Color) {
    let outer = radius * radius;
    let inner_radius = (radius - thickness).max(0);
    let inner = inner_radius * inner_radius;
    for y in -radius..=radius {
        for x in -radius..=radius {
            let d = x * x + y * y;
            if d <= outer && d >= inner {
                canvas.blend_pixel(cx + x, cy + y, color);
            }
        }
    }
}
