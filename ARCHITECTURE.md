# Architecture

This document describes the internal design of **rust2d_ui**.

## Rendering Pipeline

```text
User interaction / resize
        │
        ▼
  ┌──────────────┐
  │ request_layout│   Triggered by resize or widget-tree mutation.
  └──────┬───────┘
         ▼
  ┌──────────────┐
  │  layout tree │   Each Widget::layout() produces a LayoutNode.
  │  (LayoutNode)│   Nodes form a tree mirroring the widget tree.
  └──────┬───────┘
         ▼
  ┌──────────────┐
  │   redraw     │   App::redraw() walks the layout tree,
  │              │   calling Widget::draw() on each node.
  └──────┬───────┘
         ▼
  ┌──────────────┐
  │   Canvas     │   A Vec<u32> pixel buffer (0x00RRGGBB).
  │              │   All drawing is clipped to bounds.
  └──────┬───────┘
         ▼
  ┌──────────────┐
  │  softbuffer  │   buffer.copy_from_slice(canvas.pixels())
  │   present    │   then buffer.present().
  └──────────────┘
```

## Module Structure

```text
rust2d_ui/
├── src/
│   ├── lib.rs          Crate root – re-exports public API
│   ├── app.rs          App struct, run() entry-point, event loop
│   ├── canvas.rs       Canvas, Color, Rect – software rasteriser
│   ├── event.rs        UiEvent enum, EventState
│   ├── layout.rs       BoxConstraints, LayoutNode, LayoutStyle, Size, EdgeInsets
│   ├── text.rs         FontManager – loading, measuring, wrapping, drawing
│   └── widgets/
│       ├── mod.rs      Widget trait, WidgetId, auto-ID generator
│       ├── button.rs   Button widget (hover/pressed states, on_click)
│       ├── container.rs Container widget (vertical/horizontal layout)
│       └── text.rs     Text widget (non-interactive label)
├── examples/
│   ├── demo.rs         Minimal demo (calls run())
│   └── hello.rs        Widget-tree construction example
└── src/main.rs         Binary entry-point (calls run())
```

## Design Decisions

### Why separate the Widget tree from the Layout tree?

Widgets own *behaviour* (state, event handlers, draw logic) while
`LayoutNode` owns *geometry* (position, size, children).  Separating
them gives two benefits:

1. **Immutable layout during draw/event dispatch.**  `draw()` and
   `handle_event()` receive an `&LayoutNode` — they can read geometry
   without being able to accidentally mutate it.  This avoids a class
   of bugs where event handling silently shifts widget positions
   mid-frame.

2. **Cheap relayout.**  A full layout pass builds a fresh `LayoutNode`
   tree.  Because the tree is a plain data structure (no `Rc`, no
   `RefCell`), allocation is a single `Vec::push` per node and the
   old tree is simply dropped.

### Why softbuffer?

The goal is **zero GPU dependencies**.  softbuffer gives us a
cross-platform way to present a `&[u32]` pixel buffer to the OS
compositor without linking to OpenGL, Vulkan, Metal, or DirectX.
This means:

- The build has far fewer native dependencies.
- CI can run on headless servers without GPU drivers.
- The rendering path is entirely in safe Rust (no shader code).

The trade-off is performance: every pixel is touched by the CPU.  For
a small UI toolkit used in tooling and prototyping this is acceptable.

## Known Constraints

- **Clone of layout tree on every event dispatch.**  `App::handle_ui_event`
  clones the entire `LayoutNode` tree before passing it to widgets.
  This is a workaround for the borrow checker — `app` must be mutably
  borrowed for event handling, but `layout_tree` is also part of `app`.
  A future refactor could move the layout tree into a separate
  allocation or use indices.

- **Only three event types.**  `UiEvent` currently supports `MouseMove`,
  `MouseDown`, and `MouseUp`.  Keyboard input, scroll, touch, and
  focus events are not yet implemented.

- **No partial relayout.**  Any mutation triggers a full layout pass
  from the root.  For large trees this will be slow.

- **System font only.**  `FontManager` loads the first available
  system font.  There is no API to load custom font files.

- **No accessibility.**  Screen readers, keyboard navigation, and
  ARIA-like roles are not supported.
