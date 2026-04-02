# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] – 2026-02-14

### Added
- Auto-generated `WidgetId` system (`next_widget_id`, `new_auto` constructors).
- `Button` hover and pressed visual states.
- CJK-aware word-wrap in `FontManager::wrap_text`.
- `App::demo()` convenience constructor for quick demos.
- `Canvas::draw_rounded_rect` filled rounded-rectangle primitive.
- `Canvas::blend_pixel` alpha-composite helper.
- `Rect::union` and `Rect::contains(f32, f32)`.
- `Canvas::resize` with content preservation.
- Comprehensive test suite (30+ tests).

## [0.1.0] – 2026-01-01

### Added
- `Canvas`, `Color`, `Rect` rendering primitives.
- `Container`, `Text`, `Button` widget types with `Widget` trait.
- `LayoutNode` tree and `BoxConstraints` layout system.
- `FontManager` with system font discovery (font-kit + fontdue).
- `App` runner with winit event loop and softbuffer presentation.
- `UiEvent` / `EventState` minimal event dispatching.
