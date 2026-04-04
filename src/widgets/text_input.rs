//! Single-line text input widget with cursor, selection, and basic editing.
//!
//! Supports keyboard input (character insertion, deletion, cursor movement),
//! mouse click for focus acquisition and cursor placement, placeholder text,
//! and horizontal scrolling when the text overflows the visible area.

use crate::{
    canvas::{Canvas, Color, Rect},
    event::{EventState, UiEvent},
    keyboard::{Key, KeyboardEvent, Modifiers},
    layout::{f32_to_i32, f32_to_u32, BoxConstraints, EdgeInsets, LayoutNode, LayoutStyle, Size},
    text::FontManager,
    widgets::{next_widget_id, Widget, WidgetId},
};

// ─────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────

const DEFAULT_FONT_SIZE: f32 = 18.0;
const PADDING_H: f32 = 8.0;
const PADDING_V: f32 = 6.0;
const CURSOR_WIDTH: f32 = 2.0;
const SELECTION_COLOR: Color = Color::rgba(70, 130, 220, 80);
const BORDER_COLOR: Color = Color::rgba(180, 180, 180, 255);
const FOCUSED_BORDER_COLOR: Color = Color::rgba(70, 120, 220, 255);
const BACKGROUND_COLOR: Color = Color::rgba(255, 255, 255, 255);
const PLACEHOLDER_COLOR: Color = Color::rgba(160, 160, 160, 255);
const CURSOR_COLOR: Color = Color::rgba(0, 0, 0, 255);

type TextInputCallback = Box<dyn FnMut(&str)>;

fn sanitize_single_line_text(text: &str) -> String {
    text.chars()
        .filter_map(|ch| match ch {
            '\r' | '\n' | '\t' => Some(' '),
            c if c.is_control() => None,
            c => Some(c),
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────
// TextInput
// ─────────────────────────────────────────────────────────────────────

/// A single-line text input widget.
pub struct TextInput {
    id: WidgetId,
    text: String,
    /// Cursor position in *character* (not byte) units.
    cursor: usize,
    /// Selected range `(start, end)` in character units, where `start <= end`.
    selection: Option<(usize, usize)>,
    placeholder: String,
    is_focused: bool,
    /// Horizontal scroll offset in pixels (for text wider than the widget).
    scroll_offset: f32,
    font_size: f32,
    style: LayoutStyle,
    on_change: Option<TextInputCallback>,
    on_submit: Option<TextInputCallback>,
}

impl TextInput {
    /// Create a new `TextInput` with an auto-generated widget ID.
    pub fn new_auto() -> Self {
        Self::new(next_widget_id())
    }

    /// Create a new `TextInput` with a specific widget ID.
    pub fn new(id: WidgetId) -> Self {
        let style = LayoutStyle {
            padding: EdgeInsets {
                top: PADDING_V,
                right: PADDING_H,
                bottom: PADDING_V,
                left: PADDING_H,
            },
            wrap_text: false,
            ..LayoutStyle::default()
        };

        Self {
            id,
            text: String::new(),
            cursor: 0,
            selection: None,
            placeholder: String::new(),
            is_focused: false,
            scroll_offset: 0.0,
            font_size: DEFAULT_FONT_SIZE,
            style,
            on_change: None,
            on_submit: None,
        }
    }

    // ── Builder methods ──────────────────────────────────────────

    /// Set placeholder text displayed when the input is empty and unfocused.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set a callback invoked whenever the text content changes.
    pub fn on_change(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Set a callback invoked when the user presses Enter.
    pub fn on_submit(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Set the font size in pixels.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_size = px;
        self
    }

    // ── Public accessors ─────────────────────────────────────────

    /// Get the current text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text content programmatically.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = sanitize_single_line_text(&text.into());
        let char_len = self.char_len();
        if self.cursor > char_len {
            self.cursor = char_len;
        }
        self.selection = None;
    }

    /// Whether this input currently has focus.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Programmatically set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    // ── Keyboard event handling ──────────────────────────────────

    /// Handle a keyboard event.  Returns `true` if the widget state changed.
    pub fn handle_keyboard_event(&mut self, event: &KeyboardEvent) -> bool {
        if !self.is_focused {
            return false;
        }

        match event {
            KeyboardEvent::KeyDown {
                key,
                modifiers,
                text,
                ..
            } => self.handle_key_down(key, modifiers, text.as_deref()),
            KeyboardEvent::ImeCommit(s) => {
                self.delete_selection();
                self.insert_str(s)
            }
            KeyboardEvent::ImePreedit(_, _) => {
                // Pre-edit display is not implemented in this version.
                false
            }
            KeyboardEvent::KeyUp { .. } => false,
        }
    }

    // ── Private helpers ──────────────────────────────────────────

    /// Number of characters (not bytes) in the text.
    fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    /// Convert character index to byte index.
    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Insert a string at the current cursor position.
    fn insert_str(&mut self, s: &str) -> bool {
        let sanitized = sanitize_single_line_text(s);
        if sanitized.is_empty() {
            return false;
        }
        let byte_pos = self.char_to_byte(self.cursor);
        self.text.insert_str(byte_pos, &sanitized);
        self.cursor += sanitized.chars().count();
        self.fire_on_change();
        true
    }

    /// Delete the currently selected text, if any.  Returns `true` if
    /// something was deleted.
    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection.take() {
            let byte_start = self.char_to_byte(start);
            let byte_end = self.char_to_byte(end);
            self.text.drain(byte_start..byte_end);
            self.cursor = start;
            self.fire_on_change();
            true
        } else {
            false
        }
    }

    fn fire_on_change(&mut self) {
        if let Some(cb) = self.on_change.as_mut() {
            let text = self.text.clone();
            cb(&text);
        }
    }

    fn handle_key_down(&mut self, key: &Key, modifiers: &Modifiers, _text: Option<&str>) -> bool {
        match key {
            Key::Backspace => {
                if self.delete_selection() {
                    return true;
                }
                if self.cursor > 0 {
                    self.cursor -= 1;
                    let byte_pos = self.char_to_byte(self.cursor);
                    let next_byte = self.char_to_byte(self.cursor + 1);
                    self.text.drain(byte_pos..next_byte);
                    self.fire_on_change();
                    return true;
                }
                false
            }
            Key::Delete => {
                if self.delete_selection() {
                    return true;
                }
                if self.cursor < self.char_len() {
                    let byte_start = self.char_to_byte(self.cursor);
                    let byte_end = self.char_to_byte(self.cursor + 1);
                    self.text.drain(byte_start..byte_end);
                    self.fire_on_change();
                    return true;
                }
                false
            }
            Key::Left => {
                self.selection = None;
                if self.cursor > 0 {
                    self.cursor -= 1;
                    return true;
                }
                false
            }
            Key::Right => {
                self.selection = None;
                if self.cursor < self.char_len() {
                    self.cursor += 1;
                    return true;
                }
                false
            }
            Key::Home => {
                self.selection = None;
                if self.cursor != 0 {
                    self.cursor = 0;
                    return true;
                }
                false
            }
            Key::End => {
                self.selection = None;
                let len = self.char_len();
                if self.cursor != len {
                    self.cursor = len;
                    return true;
                }
                false
            }
            Key::Enter => {
                if let Some(cb) = self.on_submit.as_mut() {
                    let text = self.text.clone();
                    cb(&text);
                }
                true
            }
            Key::Character(s) => {
                // If Ctrl or Meta is held, treat as a shortcut — do not insert.
                if modifiers.ctrl || modifiers.meta {
                    return false;
                }
                self.delete_selection();
                self.insert_str(s)
            }
            Key::Space => {
                if modifiers.ctrl || modifiers.meta {
                    return false;
                }
                self.delete_selection();
                self.insert_str(" ")
            }
            Key::Tab | Key::Escape | Key::Up | Key::Down | Key::Other(_) => false,
        }
    }

    /// Compute the pixel x-offset of the cursor (relative to text start).
    fn cursor_x_offset(&self, fonts: &FontManager) -> f32 {
        let text_before_cursor: String = self.text.chars().take(self.cursor).collect();
        fonts.aligned_text_width(&text_before_cursor, self.font_size) as f32
    }

    /// Compute the pixel x-offset for a given character index.
    fn char_x_offset(&self, char_idx: usize, fonts: &FontManager) -> f32 {
        let text_before: String = self.text.chars().take(char_idx).collect();
        fonts.aligned_text_width(&text_before, self.font_size) as f32
    }

    /// Ensure the cursor is visible by adjusting `scroll_offset`.
    fn ensure_cursor_visible(&mut self, visible_width: f32, fonts: &FontManager) {
        let cx = self.cursor_x_offset(fonts);
        if cx - self.scroll_offset < 0.0 {
            self.scroll_offset = cx;
        } else if cx - self.scroll_offset > visible_width {
            self.scroll_offset = cx - visible_width;
        }
        if self.scroll_offset < 0.0 {
            self.scroll_offset = 0.0;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Widget implementation
// ─────────────────────────────────────────────────────────────────────

impl Widget for TextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn outer_margin(&self) -> EdgeInsets {
        self.style.margin
    }

    fn debug_name(&self) -> &str {
        "TextInput"
    }

    fn layout(
        &mut self,
        constraints: BoxConstraints,
        x: i32,
        y: i32,
        fonts: &FontManager,
    ) -> LayoutNode {
        let height = self.font_size + self.style.padding.vertical() + 2.0; // +2 for border
        let width = constraints.max_width.max(constraints.min_width);

        let visible_width = (width - self.style.padding.horizontal() - 2.0).max(0.0);
        self.ensure_cursor_visible(visible_width, fonts);

        let size = constraints.constrain(Size { width, height });
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

        // Background
        canvas.fill_rect(rect, BACKGROUND_COLOR);

        // Border (1px each side)
        let border_color = if self.is_focused {
            FOCUSED_BORDER_COLOR
        } else {
            BORDER_COLOR
        };
        // Top
        canvas.fill_rect(Rect::new(rect.x, rect.y, rect.width, 1), border_color);
        // Bottom
        canvas.fill_rect(
            Rect::new(rect.x, rect.y + rect.height as i32 - 1, rect.width, 1),
            border_color,
        );
        // Left
        canvas.fill_rect(Rect::new(rect.x, rect.y, 1, rect.height), border_color);
        // Right
        canvas.fill_rect(
            Rect::new(rect.x + rect.width as i32 - 1, rect.y, 1, rect.height),
            border_color,
        );

        let text_x = rect.x + f32_to_i32(self.style.padding.left) + 1;
        let text_y = rect.y + f32_to_i32(self.style.padding.top) + 1;
        let visible_width = (rect.width as f32 - self.style.padding.horizontal() - 2.0).max(0.0);

        // Selection highlight (drawn before text so text sits on top)
        if let Some((sel_start, sel_end)) = self.selection {
            let x_start = self.char_x_offset(sel_start, fonts) - self.scroll_offset;
            let x_end = self.char_x_offset(sel_end, fonts) - self.scroll_offset;

            let vis_start = x_start.max(0.0);
            let vis_end = x_end.min(visible_width);

            if vis_start < vis_end {
                canvas.fill_rect(
                    Rect::new(
                        text_x + f32_to_i32(vis_start),
                        text_y,
                        f32_to_u32(vis_end - vis_start),
                        f32_to_u32(self.font_size),
                    ),
                    SELECTION_COLOR,
                );
            }
        }

        let text_clip = Rect::new(
            text_x,
            text_y,
            f32_to_u32(visible_width),
            f32_to_u32(self.font_size.max(0.0)),
        );

        // Text or placeholder
        if self.text.is_empty() && !self.is_focused {
            fonts.draw_text_in_rect(
                canvas,
                &self.placeholder,
                text_clip,
                self.font_size,
                PLACEHOLDER_COLOR,
            );
        } else {
            let draw_x = text_x - f32_to_i32(self.scroll_offset);
            fonts.draw_text_in_rect(
                canvas,
                &self.text,
                Rect::new(draw_x, text_y, rect.width, text_clip.height),
                self.font_size,
                Color::BLACK,
            );
        }

        // Cursor (vertical line, only when focused)
        if self.is_focused {
            let cursor_x = self.cursor_x_offset(fonts) - self.scroll_offset;
            if cursor_x >= 0.0 && cursor_x <= visible_width {
                canvas.fill_rect(
                    Rect::new(
                        text_x + f32_to_i32(cursor_x),
                        text_y,
                        f32_to_u32(CURSOR_WIDTH),
                        f32_to_u32(self.font_size),
                    ),
                    CURSOR_COLOR,
                );
            }
        }
    }

    fn handle_event(
        &mut self,
        event: &UiEvent,
        state: &mut EventState,
        layout: &LayoutNode,
    ) -> bool {
        let rect = layout.bounds;
        let mut changed = false;

        match *event {
            UiEvent::MouseDown { x, y } => {
                if rect.contains(x, y) {
                    if !self.is_focused {
                        self.is_focused = true;
                        changed = true;
                    }
                    // Cursor placement from mouse click.
                    // Widget::handle_event does not receive FontManager, so we
                    // estimate the character index from the click position using
                    // an average character width heuristic.
                    let text_area_x = rect.x as f32 + self.style.padding.left + 1.0;
                    let click_x_in_text = (x - text_area_x) + self.scroll_offset;
                    let avg_char_width = self.font_size * 0.6;
                    let estimated_idx = if avg_char_width > 0.0 {
                        (click_x_in_text / avg_char_width).round().max(0.0) as usize
                    } else {
                        0
                    };
                    let new_cursor = estimated_idx.min(self.char_len());
                    if new_cursor != self.cursor {
                        self.cursor = new_cursor;
                        changed = true;
                    }
                    self.selection = None;
                    state.set_pressed(Some(self.id));
                } else if self.is_focused {
                    self.is_focused = false;
                    changed = true;
                }
            }
            UiEvent::MouseMove { .. } | UiEvent::MouseUp { .. } => {}
        }

        if changed {
            state.request_redraw();
        }
        changed
    }

    fn handle_keyboard_event(
        &mut self,
        event: &KeyboardEvent,
        state: &mut EventState,
        _layout: &LayoutNode,
    ) -> bool {
        let changed = TextInput::handle_keyboard_event(self, event);
        if changed {
            state.request_redraw();
        }
        changed
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input() -> TextInput {
        TextInput::new(WidgetId::manual(500))
    }

    #[test]
    fn new_text_input_is_empty() {
        let ti = make_input();
        assert_eq!(ti.text(), "");
        assert_eq!(ti.cursor, 0);
        assert_eq!(ti.selection, None);
        assert!(!ti.is_focused());
    }

    #[test]
    fn set_text_updates_content() {
        let mut ti = make_input();
        ti.set_text("hello");
        assert_eq!(ti.text(), "hello");
    }

    #[test]
    fn set_text_clamps_cursor() {
        let mut ti = make_input();
        ti.set_text("hello");
        ti.cursor = 5;
        ti.set_text("hi");
        assert_eq!(ti.cursor, 2);
    }

    #[test]
    fn builder_pattern() {
        let ti = TextInput::new_auto()
            .placeholder("Enter name")
            .font_size(24.0);
        assert_eq!(ti.placeholder, "Enter name");
        assert_eq!(ti.font_size, 24.0);
    }

    #[test]
    fn insert_character() {
        let mut ti = make_input();
        ti.set_focused(true);

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("a".into()),
            modifiers: Modifiers::default(),
            text: Some("a".into()),
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "a");
        assert_eq!(ti.cursor, 1);
    }

    #[test]
    fn insert_multibyte_character() {
        let mut ti = make_input();
        ti.set_focused(true);

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("あ".into()),
            modifiers: Modifiers::default(),
            text: Some("あ".into()),
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "あ");
        assert_eq!(ti.cursor, 1);
    }

    #[test]
    fn backspace_deletes_before_cursor() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 3;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Backspace,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "ab");
        assert_eq!(ti.cursor, 2);
    }

    #[test]
    fn backspace_at_start_is_noop() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 0;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Backspace,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(!ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "abc");
    }

    #[test]
    fn delete_removes_after_cursor() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 1;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Delete,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "ac");
        assert_eq!(ti.cursor, 1);
    }

    #[test]
    fn delete_at_end_is_noop() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 3;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Delete,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(!ti.handle_keyboard_event(&ev));
    }

    #[test]
    fn left_arrow_moves_cursor() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 2;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Left,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.cursor, 1);
    }

    #[test]
    fn right_arrow_moves_cursor() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 1;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Right,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.cursor, 2);
    }

    #[test]
    fn home_moves_to_start() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 2;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Home,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.cursor, 0);
    }

    #[test]
    fn end_moves_to_end() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 0;

        let ev = KeyboardEvent::KeyDown {
            key: Key::End,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.cursor, 3);
    }

    #[test]
    fn enter_fires_on_submit() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        let submitted = Arc::new(AtomicBool::new(false));
        let s2 = submitted.clone();

        let mut ti = TextInput::new(WidgetId::manual(501)).on_submit(move |_text| {
            s2.store(true, Ordering::SeqCst);
        });
        ti.set_focused(true);
        ti.set_text("hello");

        let ev = KeyboardEvent::KeyDown {
            key: Key::Enter,
            modifiers: Modifiers::default(),
            text: None,
        };
        ti.handle_keyboard_event(&ev);
        assert!(submitted.load(Ordering::SeqCst));
    }

    #[test]
    fn on_change_fires_on_insert() {
        use std::sync::{Arc, Mutex};
        let log = Arc::new(Mutex::new(Vec::<String>::new()));
        let l2 = log.clone();

        let mut ti = TextInput::new(WidgetId::manual(502)).on_change(move |text| {
            l2.lock().unwrap().push(text.to_string());
        });
        ti.set_focused(true);

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("x".into()),
            modifiers: Modifiers::default(),
            text: Some("x".into()),
        };
        ti.handle_keyboard_event(&ev);
        assert_eq!(log.lock().unwrap().last().unwrap(), "x");
    }

    #[test]
    fn ctrl_key_does_not_insert() {
        let mut ti = make_input();
        ti.set_focused(true);

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("a".into()),
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
            text: Some("a".into()),
        };
        assert!(!ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "");
    }

    #[test]
    fn unfocused_ignores_keyboard() {
        let mut ti = make_input();
        assert!(!ti.is_focused());

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("a".into()),
            modifiers: Modifiers::default(),
            text: Some("a".into()),
        };
        assert!(!ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "");
    }

    #[test]
    fn selection_delete() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("hello world");
        ti.selection = Some((5, 11)); // " world"
        ti.cursor = 11;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Backspace,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "hello");
        assert_eq!(ti.cursor, 5);
    }

    #[test]
    fn selection_replaced_on_insert() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("hello");
        ti.selection = Some((0, 5));
        ti.cursor = 5;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("X".into()),
            modifiers: Modifiers::default(),
            text: Some("X".into()),
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "X");
        assert_eq!(ti.cursor, 1);
    }

    #[test]
    fn space_key_inserts_space() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("ab");
        ti.cursor = 1;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Space,
            modifiers: Modifiers::default(),
            text: Some(" ".into()),
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "a b");
        assert_eq!(ti.cursor, 2);
    }

    #[test]
    fn ime_commit_inserts_text() {
        let mut ti = make_input();
        ti.set_focused(true);

        let ev = KeyboardEvent::ImeCommit("日本語".into());
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "日本語");
        assert_eq!(ti.cursor, 3);
    }

    #[test]
    fn set_text_sanitizes_control_characters() {
        let mut ti = make_input();
        ti.set_text("A\r\nB\t\u{0007}C");
        assert_eq!(ti.text(), "A  B C");
    }

    #[test]
    fn ime_commit_sanitizes_newlines_and_controls() {
        let mut ti = make_input();
        ti.set_focused(true);

        let ev = KeyboardEvent::ImeCommit("日本\r\n語\u{0000}".into());
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "日本  語");
        assert_eq!(ti.cursor, 5);
    }

    #[test]
    fn control_character_key_input_is_ignored() {
        let mut ti = make_input();
        ti.set_focused(true);

        let ev = KeyboardEvent::KeyDown {
            key: Key::Character("\u{0007}".into()),
            modifiers: Modifiers::default(),
            text: Some("\u{0007}".into()),
        };
        assert!(!ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "");
        assert_eq!(ti.cursor, 0);
    }

    #[test]
    fn handle_event_mouse_click_focuses() {
        let mut ti = make_input();
        let layout = LayoutNode::new(WidgetId::manual(500), 10, 10, 200, 30);
        let mut es = EventState::default();

        assert!(!ti.is_focused());
        let ev = UiEvent::MouseDown { x: 20.0, y: 20.0 };
        let changed = ti.handle_event(&ev, &mut es, &layout);
        assert!(changed);
        assert!(ti.is_focused());
    }

    #[test]
    fn handle_event_click_outside_unfocuses() {
        let mut ti = make_input();
        ti.set_focused(true);
        let layout = LayoutNode::new(WidgetId::manual(500), 10, 10, 200, 30);
        let mut es = EventState::default();

        let ev = UiEvent::MouseDown { x: 300.0, y: 300.0 };
        let changed = ti.handle_event(&ev, &mut es, &layout);
        assert!(changed);
        assert!(!ti.is_focused());
    }

    #[test]
    fn left_at_start_is_noop() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 0;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Left,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(!ti.handle_keyboard_event(&ev));
    }

    #[test]
    fn right_at_end_is_noop() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("abc");
        ti.cursor = 3;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Right,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(!ti.handle_keyboard_event(&ev));
    }

    #[test]
    fn char_to_byte_multibyte() {
        let mut ti = make_input();
        ti.set_text("あいう");
        // 'あ' = 3 bytes, 'い' = 3 bytes, 'う' = 3 bytes
        assert_eq!(ti.char_to_byte(0), 0);
        assert_eq!(ti.char_to_byte(1), 3);
        assert_eq!(ti.char_to_byte(2), 6);
        assert_eq!(ti.char_to_byte(3), 9);
    }

    #[test]
    fn backspace_multibyte() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("あいう");
        ti.cursor = 3;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Backspace,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "あい");
        assert_eq!(ti.cursor, 2);
    }

    #[test]
    fn delete_multibyte() {
        let mut ti = make_input();
        ti.set_focused(true);
        ti.set_text("あいう");
        ti.cursor = 0;

        let ev = KeyboardEvent::KeyDown {
            key: Key::Delete,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert!(ti.handle_keyboard_event(&ev));
        assert_eq!(ti.text(), "いう");
        assert_eq!(ti.cursor, 0);
    }
}
