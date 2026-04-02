# Roadmap

This document tracks where **rust2d_ui** is today and where it is headed.
Items are roughly ordered by priority within each section.

> The project follows [Semantic Versioning](https://semver.org/).
> Breaking API changes bump the minor version until we reach 1.0.

---

## Current (v0.2.0)

These features are implemented and available on `main`:

- **Canvas / Color / Rect** — pixel-level software rasteriser (`canvas.rs`)
- **Container / Text / Button widgets** — vertical and horizontal layout with
  clickable buttons and non-interactive labels
- **softbuffer rendering** — zero-GPU-dependency pixel buffer presentation via
  [softbuffer](https://crates.io/crates/softbuffer)
- **CJK word-wrap** — line-breaking that respects CJK character boundaries
- **Widget auto-ID** — automatic unique ID generation for every widget instance
- **System font discovery** — first-available system font loaded via
  [font-kit](https://crates.io/crates/font-kit) and rasterised with
  [fontdue](https://crates.io/crates/fontdue)
- **CI** — GitHub Actions running `fmt`, `clippy`, and `test` on every push

---

## Next (v0.3.0)

Planned for the next release:

- **Remove `clone()` on layout tree** — eliminate the per-event full-tree clone
  identified in ARCHITECTURE.md by moving the layout tree into a separate
  allocation or using an index-based approach
- **Glyph cache** — cache rasterised glyphs to avoid redundant work on redraw
- **Font fallback fix** — improve the font fallback chain so missing glyphs
  fall back to a secondary font instead of rendering as `□`
- **CI hardening** — add `cargo deny`, MSRV check, and platform matrix
  (macOS / Windows / Linux)
- **More examples** — `counter`, `layout_demo`, `cjk_text` examples to
  showcase the API
- **Keyboard events** — extend `UiEvent` with `KeyDown` / `KeyUp` / `Char`
- **winit 0.30 migration** — update from winit 0.29 to the latest stable
  release

---

## Future

No timeline yet. Contributions welcome.

- **taffy integration** — replace the hand-rolled layout engine with
  [taffy](https://crates.io/crates/taffy) to gain Flexbox and CSS Grid support
- **ScrollView** — scrollable container widget with momentum and scrollbar
- **TextInput / IME** — editable text field with Input Method Editor support
  for CJK languages
- **wgpu backend** — optional GPU-accelerated rendering path via
  [wgpu](https://crates.io/crates/wgpu)
- **Animation** — property-based tweening and transition API
- **Accessibility** — screen reader support via
  [AccessKit](https://github.com/AccessKit/accesskit)
- **HiDPI / scale-factor support** — proper handling of display scaling and
  per-monitor DPI
- **Declarative View API** — higher-level, Elm-inspired `view()` → `Message`
  architecture

---

## Non-goals

The following are explicitly **not** goals of this project:

- **Wrapping native OS widgets** — rust2d_ui draws everything itself. It does
  not wrap platform-native controls (e.g. Win32 HWND, Cocoa NSView, GTK).
- **Browser / DOM rendering** — this is a native-only toolkit. There is no plan
  to target `<canvas>`, WebGL, or the DOM. For web UI in Rust, consider
  [Leptos](https://github.com/leptos-rs/leptos) or
  [Dioxus](https://github.com/DioxusLabs/dioxus).
